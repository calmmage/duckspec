//! Agent chat widget — per-message text editors in a scrollable column.

use iced::Task;
use iced::advanced::widget::{Id, Operation, operation};
use iced::widget::{Space, button, column, container, pick_list, row, rule, scrollable, text};
use iced::{Element, Length, Rectangle, Vector};

pub const CHAT_SCROLLABLE_ID: &str = "agent-chat-scroll";
pub const CHAT_INPUT_ID: &str = "agent-chat-input";
/// Cap the auto-growing chat input at this many visual rows; past it the
/// input scrolls internally to keep the caret visible instead of pushing the
/// chat history (and the caret) off the top of the window.
const CHAT_INPUT_MAX_ROWS: usize = 20;
/// Pixels of slack at the bottom edge that still count as "stuck to bottom".
/// Small enough that one wheel notch unsticks the view, large enough to
/// absorb sub-pixel layout rounding during streaming rebuilds.
pub const STICK_TO_BOTTOM_THRESHOLD: f32 = 16.0;

use duckchat::{ModelInfo, ModelRef};

use crate::agent::SlashCommand;
use crate::area::interaction::{self, SelectionContext};
use crate::chat_store::{ChatSession, ContentBlock, Role};
use crate::config::ViewerStyle;
use crate::slash_commands::{slash_kind_rank, slash_kind_row_tag};
use crate::theme;
use crate::widget::collapsible;
use crate::widget::find;
use crate::widget::streaming_indicator;
use crate::widget::text_edit::{self, Block, BlockKind, EditorState};

// ── Answer viewer presentation ───────────────────────────────────────────────

/// How an Answer body is painted for the effective viewer style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnswerBodyPresentation {
    /// Full-body classic TextEdit path (today's Answer viewer).
    ClassicFullBody,
    /// Focus chrome: foldable sections + always-open trailing meta region.
    FocusSectioned {
        sections: Vec<crate::focus_answer::FoldableSection>,
        open: crate::focus_answer::LineRange,
    },
}

/// Resolve Answer body presentation for the effective viewer style.
///
/// Non-Answer segments never call this — they keep their own paths.
/// Live Answers always Classic. Settled Focus without trailing `next` is Classic
/// passthrough.
pub fn answer_body_presentation(
    effective_style: ViewerStyle,
    answer_live: bool,
    source: &str,
) -> AnswerBodyPresentation {
    if effective_style != ViewerStyle::Focus || answer_live {
        return AnswerBodyPresentation::ClassicFullBody;
    }
    match crate::focus_answer::focus_layout(source) {
        crate::focus_answer::FocusLayout::PassthroughClassic => {
            AnswerBodyPresentation::ClassicFullBody
        }
        crate::focus_answer::FocusLayout::Sectioned { sections, open } => {
            AnswerBodyPresentation::FocusSectioned { sections, open }
        }
    }
}

/// Which Focus Answer slice is being presented (Hybrid C paint + band policy).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusSliceKind {
    /// Trailing meta open region — Classic TextEdit body paint.
    OpenRegion,
    /// Expanded foldable section body — plain content-font source text.
    ExpandedSection,
}

/// Hybrid C: only the open region uses classic Answer body paint.
pub fn focus_slice_uses_classic_body_paint(slice: FocusSliceKind) -> bool {
    matches!(slice, FocusSliceKind::OpenRegion)
}

/// Hybrid C: last-answer band applies only to the open-region widget.
pub fn focus_slice_uses_last_answer_band(slice: FocusSliceKind, is_last_answer: bool) -> bool {
    is_last_answer && matches!(slice, FocusSliceKind::OpenRegion)
}

/// Lines that the Answer's chat editor should hold for the effective style.
///
/// Classic / passthrough / live → full Answer body. Focus sectioned → open
/// region only (sections render as plain text from [`Block::lines`]).
pub fn answer_editor_desired_lines(
    block: &Block,
    effective_style: ViewerStyle,
) -> Vec<String> {
    if block.kind != BlockKind::Assistant {
        return block.lines.clone();
    }
    let source = block.lines.join("\n");
    match answer_body_presentation(effective_style, block.is_live, &source) {
        AnswerBodyPresentation::ClassicFullBody => block.lines.clone(),
        AnswerBodyPresentation::FocusSectioned { open, .. } => {
            slice_line_vec(&block.lines, open)
        }
    }
}

fn slice_line_vec(lines: &[String], range: crate::focus_answer::LineRange) -> Vec<String> {
    if lines.is_empty() {
        return vec![String::new()];
    }
    let start = range.start.min(lines.len().saturating_sub(1));
    let end = range.end.min(lines.len().saturating_sub(1));
    if start > end {
        return vec![String::new()];
    }
    lines[start..=end].to_vec()
}

/// Whether this block kind's presentation mode is selected by viewer style.
///
/// Only Answers branch on [`ViewerStyle`]; User / Thinking / Activity / System
/// keep fixed paths regardless of the setting.
pub fn block_presentation_uses_viewer_style(kind: BlockKind) -> bool {
    matches!(kind, BlockKind::Assistant)
}

// ── Messages ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Msg {
    /// Action from the chat input editor.
    InputAction(text_edit::EditorAction),
    SendPressed,
    /// Activate a fast-response chip (key or click). Payload is the pick id
    /// only (not the hotkey label).
    ActivateFastResponse(crate::fast_response::FastResponsePick),
    CancelPressed,
    CompletionAccept,
    CompletionNext,
    CompletionPrev,
    CompletionDismiss,
    /// Action from a per-block chat text editor (index, action).
    ChatAction(usize, text_edit::EditorAction),
    /// Toggle collapse state of a block.
    ToggleCollapse(usize),
    /// Toggle a Focus Answer section fold (`block_idx`, section key).
    ToggleFocusSection {
        block_idx: usize,
        key: String,
    },
    /// Action from the queued-message read-only editor.
    QueueAction(text_edit::EditorAction),
    /// Discard the queued message (from the pill's ✕ button).
    DiscardQueue,
    /// User scrolled the chat transcript. Drives the per-session
    /// `stick_to_bottom` flag — true while the viewport is within
    /// `STICK_TO_BOTTOM_THRESHOLD` pixels of the bottom.
    ChatScrolled(scrollable::Viewport),
    /// User picked a model from the meta-row selector.
    ModelSelected(ModelChoice),
    /// Cycle empty-input next actions (`+1` Tab, `-1` Shift-Tab).
    CycleNextAction(i8),
    /// Layout measure of the chat scrollable (viewport + content heights).
    /// Used to recompute the bottom-pin pad even when content fits the
    /// viewport and iced suppresses `on_scroll` notifications.
    ChromeLayout {
        viewport_h: f32,
        content_h: f32,
    },
    /// Phase-pill activation: empty-send next-stage text or `Commit`.
    PhasePillSend(String),
}

// ── Model picker ─────────────────────────────────────────────────────────────

/// One entry in the meta-row model `pick_list`. `id` is the `--model` value to
/// pin and `harness` the backend that owns it (`None`/`None` = "use project
/// default"). Equality is on `(harness, id)` so a picked model resolves to the
/// right harness even when two backends share a bare model id — while the
/// selected entry can still carry a richer label (e.g. the resolved model in
/// parens) and match its plain option in the dropdown.
#[derive(Debug, Clone)]
pub struct ModelChoice {
    /// The harness owning this model, e.g. `"claude-code"` | `"grok"`. `None`
    /// on the "use default" sentinel, which pins no specific model.
    pub harness: Option<String>,
    pub id: Option<String>,
    /// Menu / list label — harness-prefixed so multi-backend choices stay grouped.
    pub label: String,
    /// Short name for the closed control (model display only, no harness prefix).
    pub closed_label: String,
}

impl ModelChoice {
    /// The persisted model reference this choice selects, or `None` for the
    /// "use default" sentinel (which carries neither harness nor id).
    pub fn to_ref(&self) -> Option<ModelRef> {
        match (&self.harness, &self.id) {
            (Some(harness), Some(id)) => Some(ModelRef::new(harness.clone(), id.clone())),
            _ => None,
        }
    }
}

impl PartialEq for ModelChoice {
    fn eq(&self, other: &Self) -> bool {
        self.harness == other.harness && self.id == other.id
    }
}

impl Eq for ModelChoice {}

impl std::fmt::Display for ModelChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

/// Picker options for a chat's model selector. The first entry (`id: None`)
/// is the "use project default" sentinel; `project_default` lets its label
/// name the model that default currently resolves to.
pub fn chat_model_choices() -> Vec<ModelChoice> {
    // No "use default" sentinel: the chat selector always shows the concrete
    // model the next turn will run (the resolved cascade), never the word
    // "Default". `selected_model_choice` is fed that effective model.
    model_entries()
}

/// Picker options for the global main-chat default in Settings. Catalog models
/// only — no sentinel (the global default is always a concrete choice when set).
pub fn global_model_choices() -> Vec<ModelChoice> {
    model_entries()
}

/// Picker options for the project override in Settings. First entry clears the
/// override (`id: None` → use global default).
pub fn project_override_model_choices() -> Vec<ModelChoice> {
    let mut out = vec![ModelChoice {
        harness: None,
        id: None,
        label: "Use global default".to_string(),
        closed_label: "Use global default".to_string(),
    }];
    out.extend(model_entries());
    out
}

fn model_entries() -> Vec<ModelChoice> {
    group_choices(crate::agent::available_models())
}

/// Turn the aggregated model list into picker entries grouped by harness. Each
/// harness's models are kept contiguous (in first-seen order) and every label
/// is prefixed with its harness so the flat `pick_list` reads as harness
/// sections rather than an undifferentiated list.
fn group_choices(models: Vec<ModelInfo>) -> Vec<ModelChoice> {
    let mut harness_order: Vec<String> = Vec::new();
    for m in &models {
        if !harness_order.contains(&m.harness) {
            harness_order.push(m.harness.clone());
        }
    }
    let mut out = Vec::with_capacity(models.len());
    for harness in &harness_order {
        for m in models.iter().filter(|m| &m.harness == harness) {
            out.push(ModelChoice {
                harness: Some(m.harness.clone()),
                id: Some(m.id.clone()),
                label: format!("{} · {}", harness_display(&m.harness), m.display),
                closed_label: m.display.clone(),
            });
        }
    }
    out
}

/// Human-friendly name for a harness id, used to label and group picker
/// entries. Unknown ids fall back to the raw id.
fn harness_display(harness: &str) -> &str {
    match harness {
        "claude-code" => "Claude Code",
        "grok" => "Grok",
        "openai-codex" => "OpenAI Codex",
        other => other,
    }
}

/// Closed model control when the effective model is not available in the
/// process catalog. Optional `preferred` keeps harness/id for equality when a
/// cascade choice exists but is missing from the catalog.
pub fn missing_closed_model_choice(preferred: Option<&ModelRef>) -> ModelChoice {
    match preferred {
        Some(m) => ModelChoice {
            harness: Some(m.harness.clone()),
            id: Some(m.model.clone()),
            label: "Missing".to_string(),
            closed_label: "Missing".to_string(),
        },
        None => ModelChoice {
            harness: None,
            id: None,
            label: "Missing".to_string(),
            closed_label: "Missing".to_string(),
        },
    }
}

/// Resolve which option is selected for a pinned model reference. `None`
/// selects the first (sentinel) entry. A ref not in the offered list (e.g. a
/// full model name, or a harness that dropped out) yields a synthetic choice so
/// the picker still shows it rather than silently falling back to the sentinel.
pub fn selected_model_choice(choices: &[ModelChoice], selected: Option<&ModelRef>) -> ModelChoice {
    match selected {
        None => choices.first().cloned().unwrap_or(ModelChoice {
            harness: None,
            id: None,
            label: "Default".to_string(),
            closed_label: "Default".to_string(),
        }),
        Some(model_ref) => choices
            .iter()
            .find(|c| {
                c.harness.as_deref() == Some(model_ref.harness.as_str())
                    && c.id.as_deref() == Some(model_ref.model.as_str())
            })
            .cloned()
            .unwrap_or(ModelChoice {
                harness: Some(model_ref.harness.clone()),
                id: Some(model_ref.model.clone()),
                label: format!(
                    "{} · {}",
                    harness_display(&model_ref.harness),
                    model_ref.model
                ),
                closed_label: model_ref.model.clone(),
            }),
    }
}

/// The context window of a specific model, looked up by harness + id from the
/// process model catalog. `None` when the model is unknown or its harness
/// reports no window — the usage meter then shows no fill.
pub fn model_context_window(model: &ModelRef) -> Option<usize> {
    crate::agent::model_context_window(model)
}

/// Fraction of the selected model's context window consumed by `tokens`. A
/// `None` (or zero) window yields `None`: the usage meter shows no fill rather
/// than computing against a wrong or assumed window.
pub fn context_fill(tokens: usize, window: Option<usize>) -> Option<f32> {
    match window {
        Some(w) if w > 0 => Some(tokens as f32 / w as f32),
        _ => None,
    }
}

/// Fill fraction at which the usage readout expands from percentage-only to
/// full `used / max (%)`. Matches the existing warning color band.
pub const USAGE_HOT_FILL: f32 = 0.75;

/// Whether a stored agent session id is unresumable on the effective harness.
///
/// `has_stored_agent_id` / `will_resume` are pre-mapped booleans — the status
/// builder owns reading `agent_session_id` and `resumable_session_id()`; this
/// helper only combines them so the product rule stays unit-testable.
pub fn unresumable_stored_session(has_stored_agent_id: bool, will_resume: bool) -> bool {
    has_stored_agent_id && !will_resume
}

/// Whether the composer footer shows the resend-history hint. True only when
/// the transcript is non-empty *and* a stored agent session is not resumable
/// for the effective harness (typically after a harness switch).
pub fn show_resend_history_hint(has_messages: bool, unresumable_stored_session: bool) -> bool {
    has_messages && unresumable_stored_session
}

/// Progressive context-usage string for a **known** window. Cool fill (< 75%)
/// is percentage only; hot fill (≥ 75%) includes used, max, and percentage.
/// Callers that lack a window should not use this (model-picker owns "no fill").
pub fn format_usage_readout(tokens: usize, window: usize) -> String {
    let fill = tokens as f32 / window as f32;
    let pct = (fill * 100.0) as usize;
    if fill < USAGE_HOT_FILL {
        format!("{pct}%")
    } else {
        format!(
            "{} / {} ({}%)",
            format_number(tokens),
            format_number(window),
            pct
        )
    }
}

// ── Status bar info ────────────────────────────────────────────────────────

/// Data for the status bar below the chat input.
pub struct StatusInfo {
    pub is_streaming: bool,
    /// Mid-turn structured choice pending — chips stay visible while streaming.
    pub is_awaiting_user: bool,
    /// 0 = no esc pressed, 1 = one esc pressed (waiting for second).
    pub esc_count: u8,
    /// Picker options — one per provider model, grouped by harness.
    pub model_choices: Vec<ModelChoice>,
    /// The currently-selected picker entry (matched by `(harness, id)`).
    pub selected_model: ModelChoice,
    /// Stored agent session id exists but is not resumable for the effective
    /// harness (typically after a harness switch). False when unbound or when
    /// resume works. Combined with transcript emptiness via
    /// `show_resend_history_hint` for the meta-row resend indicator.
    pub unresumable_stored_session: bool,
    pub context_tokens: usize,
    /// The selected model's context window. `None` when the model reports no
    /// window — the meter then shows the token count with no fill.
    pub context_max: Option<usize>,
}

// ── Completion state ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct CompletionState {
    pub visible: bool,
    pub selected: usize,
}

// ── Transcript segments ────────────────────────────────────────────────────

/// One contiguous run of the calm transcript: user/system prose, thinking,
/// answer, or a grouped activity of tools.
// Shared segment model — construction and collapse live in duckcore.
pub use duckcore::transcript::{
    ActivityRowView, CollapseState, PRIMING_RECOLLAPSE_SECS, ToolRow, ToolRowStatus, TranscriptSeg,
    activity_collapsed_label, build_transcript_segments, expanded_activity_rows, format_tool_summary,
    is_host_choice_tool_name, priming_collapsed_label, recollapse_priming, strip_ansi_escapes,
    strip_tool_wrapper_tags, sync_collapse_states, text_lines, thinking_collapsed_label,
    toggle_collapse, tool_status_glyph, truncate_chars, truncate_output,
};

// ── Build blocks from session ──────────────────────────────────────────────

/// Map transcript segments 1:1 into editor blocks (index-aligned with
/// [`sync_collapse_states`]).
///
/// Contiguous tools form one Activity block; reasoning becomes Reasoning
/// (Thinking); orphan results are named done rows inside Activity — never a
/// bare "✓ done" block. Call after `build_transcript_segments`.
pub fn blocks_from_segments(segs: &[TranscriptSeg]) -> Vec<Block> {
    segs.iter()
        .map(|seg| match seg {
            TranscriptSeg::User {
                lines,
                is_priming,
            } => Block {
                kind: BlockKind::User,
                label: if *is_priming {
                    "Setup".to_string()
                } else {
                    "User".to_string()
                },
                lines: lines.clone(),
                is_priming: *is_priming,
                is_live: false,
            },
            TranscriptSeg::System { lines } => Block {
                kind: BlockKind::System,
                label: "System".to_string(),
                lines: lines.clone(),
                is_priming: false,
                is_live: false,
            },
            TranscriptSeg::Thinking { lines, live } => Block {
                kind: BlockKind::Reasoning,
                label: if *live {
                    "Thinking ···".to_string()
                } else {
                    "Thinking".to_string()
                },
                lines: lines.clone(),
                is_priming: false,
                is_live: *live,
            },
            TranscriptSeg::Answer { lines, live } => Block {
                kind: BlockKind::Assistant,
                label: if *live {
                    "Assistant ···".to_string()
                } else {
                    "Assistant".to_string()
                },
                lines: lines.clone(),
                is_priming: false,
                is_live: *live,
            },
            TranscriptSeg::Activity { tools, .. } => Block {
                kind: BlockKind::Activity,
                label: activity_collapsed_label(tools),
                lines: activity_body_lines(tools),
                is_priming: false,
                is_live: false,
            },
            TranscriptSeg::UserChoiceQuestion { text } => Block {
                kind: BlockKind::UserChoiceQuestion,
                label: "Question".to_string(),
                lines: text_lines(text),
                is_priming: false,
                is_live: false,
            },
            TranscriptSeg::UserChoiceAnswer { text } => Block {
                kind: BlockKind::UserChoiceAnswer,
                label: "Answer".to_string(),
                lines: text_lines(text),
                is_priming: false,
                is_live: false,
            },
        })
        .collect()
}

/// Quiet-row dump for an expanded Activity body (status + summary + indented
/// truncated output). Group-level expand only — no per-tool expand state.
fn activity_body_lines(tools: &[ToolRow]) -> Vec<String> {
    let mut lines = Vec::new();
    for row in expanded_activity_rows(tools) {
        lines.push(format!("{} {}", row.status_glyph, row.summary));
        for out in &row.output_lines {
            lines.push(format!("  {out}"));
        }
    }
    lines
}

/// Truncate tool output to a reasonable number of lines, filtering
/// non-printable characters that cause rendering artifacts.
// ── Last-Answer band ────────────────────────────────────────────────────────

/// Index of the latest non-empty Answer (`BlockKind::Assistant`) block, if any.
/// Empty Answer bodies are not band targets.
pub fn last_answer_band_target(blocks: &[Block]) -> Option<usize> {
    blocks
        .iter()
        .rposition(|b| b.kind == BlockKind::Assistant && b.lines.iter().any(|l| !l.is_empty()))
}

// ── Answer reply landmarks ──────────────────────────────────────────────────

/// Block indices of every Answer (`BlockKind::Assistant`) in transcript order.
pub fn answer_block_indices(blocks: &[Block]) -> Vec<usize> {
    blocks
        .iter()
        .enumerate()
        .filter(|(_, b)| b.kind == BlockKind::Assistant)
        .map(|(i, _)| i)
        .collect()
}

/// Previous Answer anchor before `from` (a block index). No wrap.
pub fn prev_answer_idx(anchors: &[usize], from: Option<usize>) -> Option<usize> {
    let from = from?;
    let pos = anchors.iter().position(|&i| i == from)?;
    pos.checked_sub(1).map(|p| anchors[p])
}

/// Next Answer anchor after `from` (a block index). No wrap.
pub fn next_answer_idx(anchors: &[usize], from: Option<usize>) -> Option<usize> {
    let from = from?;
    let pos = anchors.iter().position(|&i| i == from)?;
    anchors.get(pos + 1).copied()
}

/// Float slack when comparing Answer tops to the viewport scroll offset.
const VIEWPORT_TOP_EPS: f32 = 1.0;

/// Resolve the current Answer for ⌘←/→ from stick-to-bottom or scroll position.
///
/// `answer_tops` is `(block_idx, content_y)` for Answer blocks (layout coords
/// relative to the scrollable content origin). When not stuck: last Answer
/// whose top ≤ `offset_y`; if none, the first Answer.
pub fn current_answer_for_reply_jumps(
    anchors: &[usize],
    answer_tops: &[(usize, f32)],
    offset_y: f32,
    stick_to_bottom: bool,
) -> Option<usize> {
    if anchors.is_empty() {
        return None;
    }
    if stick_to_bottom {
        return anchors.last().copied();
    }
    let mut current = None;
    for &idx in anchors {
        let Some(&(_, top)) = answer_tops.iter().find(|(i, _)| *i == idx) else {
            continue;
        };
        if top <= offset_y + VIEWPORT_TOP_EPS {
            current = Some(idx);
        }
    }
    current.or_else(|| anchors.first().copied())
}

/// Prev jump: re-align to `current` when the viewport is below its top; else prior Answer.
/// Next jump: adjacent next only (no re-align-first).
///
/// `answer_tops` is `(block_idx, content_y)` relative to the scrollable content origin.
/// Alignment slack matches `current_answer_for_reply_jumps` (`VIEWPORT_TOP_EPS`).
pub fn target_answer_for_reply_jump(
    anchors: &[usize],
    answer_tops: &[(usize, f32)],
    current: Option<usize>,
    go_prev: bool,
    offset_y: f32,
) -> Option<usize> {
    if go_prev {
        if let Some(cur) = current
            && let Some(&(_, top)) = answer_tops.iter().find(|(i, _)| *i == cur)
            && offset_y > top + VIEWPORT_TOP_EPS
        {
            return Some(cur);
        }
        return prev_answer_idx(anchors, current);
    }
    next_answer_idx(anchors, current)
}

/// Measure Answer block tops, resolve prev/next from viewport, scroll target
/// to the top of the chat scrollable. No-op when there is no target.
///
/// `offset_y` / `stick_to_bottom` describe the viewport *before* the jump.
/// When the layout Operation measures the scrollable translation, that measured
/// offset is preferred over the passed `offset_y`.
pub fn scroll_to_adjacent_answer<M: Send + 'static>(
    anchors: &[usize],
    go_prev: bool,
    offset_y: f32,
    stick_to_bottom: bool,
) -> Task<M> {
    if anchors.is_empty() {
        return Task::none();
    }
    let answer_blocks: Vec<(usize, Id)> = anchors
        .iter()
        .map(|&i| (i, find::chat_block_widget_id(i)))
        .collect();
    let op = ScrollToAdjacentAnswer {
        scrollable_id: Id::from(CHAT_SCROLLABLE_ID),
        answer_blocks,
        go_prev,
        offset_y,
        stick_to_bottom,
        scrollable_y: None,
        measured_offset_y: None,
        collected_ys: Vec::new(),
    };
    iced::advanced::widget::operate(op).discard()
}

struct ScrollToAdjacentAnswer {
    scrollable_id: Id,
    answer_blocks: Vec<(usize, Id)>,
    go_prev: bool,
    offset_y: f32,
    stick_to_bottom: bool,
    scrollable_y: Option<f32>,
    /// Layout translation.y of the chat scrollable, when measured.
    measured_offset_y: Option<f32>,
    /// Absolute layout `bounds.y` per answer block idx.
    collected_ys: Vec<(usize, f32)>,
}

impl Operation<()> for ScrollToAdjacentAnswer {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<()>)) {
        operate(self);
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        let Some(id) = id else {
            return;
        };
        for (idx, block_id) in &self.answer_blocks {
            if id == block_id {
                self.collected_ys.push((*idx, bounds.y));
            }
        }
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        _content_bounds: Rectangle,
        translation: Vector,
        _state: &mut dyn operation::Scrollable,
    ) {
        if id == Some(&self.scrollable_id) {
            self.scrollable_y = Some(bounds.y);
            self.measured_offset_y = Some(translation.y);
        }
    }

    fn finish(&self) -> operation::Outcome<()> {
        let Some(sy) = self.scrollable_y else {
            return operation::Outcome::None;
        };
        let anchors: Vec<usize> = self.answer_blocks.iter().map(|(i, _)| *i).collect();
        let tops: Vec<(usize, f32)> = self
            .collected_ys
            .iter()
            .map(|(i, y)| (*i, (y - sy).max(0.0)))
            .collect();
        let offset_y = self.measured_offset_y.unwrap_or(self.offset_y);
        let current =
            current_answer_for_reply_jumps(&anchors, &tops, offset_y, self.stick_to_bottom);
        let target = target_answer_for_reply_jump(&anchors, &tops, current, self.go_prev, offset_y);
        let Some(target_idx) = target else {
            return operation::Outcome::None;
        };
        let Some(&(_, by)) = self.collected_ys.iter().find(|(i, _)| *i == target_idx) else {
            return operation::Outcome::None;
        };
        let target_y = (by - sy).max(0.0);
        operation::Outcome::Chain(Box::new(operation::scrollable::scroll_to(
            self.scrollable_id.clone(),
            operation::scrollable::AbsoluteOffset {
                x: 0.0,
                y: target_y,
            }
            .into(),
        )))
    }
}

// ── View ────────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub fn view<'a>(
    session: &'a ChatSession,
    blocks: &'a [Block],
    editors: &'a [EditorState],
    collapse: &'a [CollapseState],
    input_value: &'a EditorState,
    queue_editor: Option<&'a EditorState>,
    commands: &'a [SlashCommand],
    completion: &CompletionState,
    status: StatusInfo,
    next_actions: &'a [crate::meta_card::NextAction],
    next_action_idx: usize,
    // Multi-option fast-response shell (send form derived in view).
    fast_response: &'a crate::fast_response::FastResponse,
    // Spacer above chips when history is shorter than the viewport.
    fast_response_top_pad: f32,
    pinned_selections: &'a [SelectionContext],
    tentative_selection: Option<&'a SelectionContext>,
    block_highlights: Vec<(
        Vec<text_edit::HighlightRange>,
        Option<text_edit::HighlightRange>,
    )>,
    // Above-composer phase pills when the setting is on and scope has a display.
    phase_display: Option<crate::area::change::PhaseDisplay>,
    // Build pilot mode plaque (`Build auto` / `Build fast`) when armed.
    pilot_plaque: Option<&'static str>,
    // Effective chat Answer viewer style (from config).
    viewer_style: ViewerStyle,
    // Ephemeral Focus section folds keyed by block index.
    focus_folds: &'a std::collections::HashMap<
        usize,
        std::collections::HashMap<String, crate::focus_answer::SectionFoldState>,
    >,
) -> Element<'a, Msg> {
    // Chat content — scrollable column of full-width sections.
    let mut chat_col = column![]
        .spacing(theme::SPACING_XS)
        .padding([theme::SPACING_SM, 0.0]);

    let mut block_highlights = block_highlights;
    let last_answer_band = last_answer_band_target(blocks);
    for (i, block) in blocks.iter().enumerate() {
        let is_collapsed = collapse.get(i).map(|s| s.collapsed).unwrap_or(false);
        let (ranges, current) = if i < block_highlights.len() {
            std::mem::take(&mut block_highlights[i])
        } else {
            (Vec::new(), None)
        };
        let is_last_answer = last_answer_band == Some(i);
        let block_el = view_block(
            i,
            block,
            editors.get(i),
            is_collapsed,
            ranges,
            current,
            is_last_answer,
            viewer_style,
            focus_folds.get(&i),
        );
        // Tag each block with a stable widget id so `widget::find` can read
        // the laid-out bounds during an Operation pass and scroll the
        // matching block to the top of the viewport — bypasses all the
        // per-kind padding / wrap / collapse pixel math.
        let tagged = container(block_el)
            .id(crate::widget::find::chat_block_widget_id(i))
            .width(Length::Fill);
        chat_col = chat_col.push(tagged);
    }

    // Streaming indicator: animated pulsing dots + inline cancel hint at
    // the bottom of the transcript, visible only while the agent is
    // producing a response. The left padding (`SPACING_MD + SPACING_SM`)
    // mirrors the block-container padding + `TextEdit`'s internal
    // `CONTENT_PAD`, so the dots land at the same x as message body text.
    if status.is_streaming {
        chat_col = chat_col.push(
            container(streaming_indicator::view(status.esc_count))
                .padding([theme::SPACING_SM, theme::SPACING_MD + theme::SPACING_SM])
                .width(Length::Fill),
        );
    }

    // Fast response after transcript content, inside the scroll column.
    // Optional top pad pins chips to the bottom of the viewport when history
    // is short; when history already fills the viewport, pad is 0 and chips
    // sit naturally after the last message. Keeping chrome inside the scroll
    // (not between scroll and composer) preserves a stable outer widget tree
    // so the input keeps focus when chrome shows/hides.
    let input_empty = input_value.text().trim().is_empty();
    if crate::fast_response::visible(
        status.is_streaming,
        status.is_awaiting_user,
        input_empty,
        fast_response,
    ) {
        if fast_response_top_pad > 0.0 {
            chat_col = chat_col.push(
                Space::new()
                    .width(Length::Fill)
                    .height(fast_response_top_pad),
            );
        }
        chat_col = chat_col.push(view_fast_response(fast_response));
    }

    let chat_scroll = scrollable(chat_col)
        .direction(theme::thin_scrollbar_direction())
        .style(theme::thin_scrollbar)
        .width(Length::Fill)
        .height(Length::Fill)
        .on_scroll(Msg::ChatScrolled)
        .id(CHAT_SCROLLABLE_ID);
    let chat_area = container(chat_scroll)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::chat_area);

    // Completion popup — always rendered with the same widget type so iced's
    // tree diff preserves input focus. When hidden, the inner column is
    // empty and the background is suppressed so the popup collapses cleanly.
    // When shown it shares the chat input's "paper" bg so the popup reads as
    // a continuation of the input field with a top hairline separating it
    // from the chat transcript.
    let has_completion = completion.visible && {
        let input_text = input_value.text();
        let query = input_text.trim_start_matches('/');
        !filter_commands(commands, query).is_empty()
    };
    let completion_col = if has_completion {
        let input_text = input_value.text();
        let query = input_text.trim_start_matches('/');
        let filtered = filter_commands(commands, query);
        let mut col = column![].spacing(0.0);
        col = col.push(completion_divider());
        col = col.push(view_completion_col(
            commands,
            &filtered,
            completion.selected,
        ));
        col
    } else {
        column![].spacing(0.0)
    };
    let completion_el: Element<'a, Msg> = container(completion_col)
        .width(Length::Fill)
        .style(move |_theme: &iced::Theme| {
            if has_completion {
                container::Style {
                    background: Some(iced::Background::Color(theme::bg_base())),
                    ..Default::default()
                }
            } else {
                container::Style::default()
            }
        })
        .into();

    // Input area — promoted to the custom TextEdit widget so prompts get
    // markdown syntax highlighting and the full editor toolkit (undo,
    // word-nav, selection). Plain Enter sends via `on_submit`; Shift+Enter
    // inserts a newline. Grows via `fit_content`. Display-only key prefixes on
    // ghost text; send paths use next_actions.send only.
    let show_tab_marker = crate::default_prompts::next_tab_marker_visible(
        input_empty,
        status.is_streaming,
        next_actions.len(),
    );
    let ghost_body =
        crate::default_prompts::next_ghost_text(status.is_streaming, next_actions, next_action_idx)
            .unwrap_or("");
    let ghost = if ghost_body.is_empty() {
        String::new()
    } else if show_tab_marker {
        format!("⇥  {ghost_body}")
    } else {
        ghost_body.to_string()
    };

    let mut input = text_edit::TextEdit::new(input_value, Msg::InputAction)
        .id(CHAT_INPUT_ID)
        .show_gutter(false)
        .word_wrap(true)
        .fit_content(true)
        .max_rows(CHAT_INPUT_MAX_ROWS)
        .transparent_bg(true)
        .on_submit(Msg::SendPressed);
    if !ghost.is_empty() {
        input = input.placeholder(ghost);
    }

    let input_divider = rule::horizontal(1).style(|_theme: &iced::Theme| rule::Style {
        color: theme::border_color(),
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: true,
    });

    // Meta row — model + context tokens — sits inside the input container
    // below the editor, blending into the "paper" surface (à la Zed). The
    // extra `SPACING_SM` horizontal padding lines the meta text up with the
    // input's own text (container XS + TextEdit CONTENT_PAD = 12px).
    // Fill is measured against the *selected* model's window (`context_max`).
    // An unknown window yields no fill — raw token count only, no percentage.
    let ctx_pct =
        context_fill(status.context_tokens, status.context_max).map(|fill| (fill * 100.0) as usize);
    let ctx_color = match ctx_pct {
        Some(pct) if pct >= 90 => theme::error(),
        Some(pct) if pct >= 75 => theme::warning(),
        _ => theme::text_muted(),
    };
    let has_attachments = !pinned_selections.is_empty() || tentative_selection.is_some();
    let mut meta_inner = row![]
        .spacing(theme::SPACING_SM)
        .align_y(iced::Alignment::Center);
    if has_attachments {
        meta_inner = meta_inner.push(
            text("⌘R reset")
                .size(theme::font_sm())
                .color(theme::text_muted()),
        );
    }
    meta_inner = meta_inner.push(Space::new().width(Length::Fill));
    // Resend-history hint: non-empty transcript + stored but unresumable session.
    if show_resend_history_hint(
        !session.messages.is_empty(),
        status.unresumable_stored_session,
    ) {
        meta_inner = meta_inner.push(
            text("⟳ resends full history")
                .size(theme::font_sm())
                .color(theme::accent()),
        );
    }
    // Closed control shows the short display name; menu options keep
    // harness-prefixed `label` via Display. Equality is on (harness, id).
    let mut selected_closed = status.selected_model.clone();
    selected_closed.label = selected_closed.closed_label.clone();
    let model_pick_style =
        if crate::fast_response::awaiting_composer_chrome(status.is_awaiting_user) {
            theme::pick_list_ghost_awaiting_style
        } else {
            theme::pick_list_ghost_style
        };
    meta_inner = meta_inner.push(
        pick_list(
            status.model_choices,
            Some(selected_closed),
            Msg::ModelSelected,
        )
        .text_size(theme::font_sm())
        .padding([0.0, theme::SPACING_XS])
        .style(model_pick_style)
        .menu_style(theme::pick_list_menu),
    );
    let ctx_label = match status.context_max {
        // Progressive readout when the window is known and positive.
        Some(max) if max > 0 => format_usage_readout(status.context_tokens, max),
        // No known window → raw token count with no fill.
        _ => format_number(status.context_tokens),
    };
    meta_inner = meta_inner.push(text(ctx_label).size(theme::font_sm()).color(ctx_color));
    // Extra top padding separates the prompt from the meta strip so the
    // toolbar doesn't crowd the last input line.
    let meta_row = container(meta_inner)
        .padding(iced::Padding {
            top: theme::SPACING_SM,
            right: theme::SPACING_SM,
            bottom: 0.0,
            left: theme::SPACING_SM,
        })
        .width(Length::Fill);

    // Build the composer column without zero-height placeholders — empty
    // `Space` siblings still consume `column` spacing and inflated the gap
    // above the default-prompt strip.
    let mut composer_col = column![].spacing(theme::SPACING_XS);

    if let Some(display) = phase_display.as_ref() {
        let on_lifecycle = display
            .lifecycle_send
            .as_ref()
            .map(|t| Msg::PhasePillSend(t.clone()));
        let on_vcs = display
            .vcs_send
            .map(|t| Msg::PhasePillSend(t.to_string()));
        composer_col = composer_col.push(
            container(crate::widget::phase_pill::view_pair(
                display,
                on_lifecycle,
                on_vcs,
            ))
            .padding([0.0, theme::SPACING_SM])
            .width(Length::Fill),
        );
    }

    if let Some(label) = pilot_plaque {
        composer_col = composer_col.push(
            container(
                text(label)
                    .size(theme::font_sm())
                    .color(theme::text_muted()),
            )
            .padding([0.0, theme::SPACING_SM])
            .width(Length::Fill),
        );
    }

    // Queue pill — renders above the input when a message is staged while the
    // agent is still streaming. Uses a read-only TextEdit so it matches the
    // shape of regular chat messages.
    if let Some(ed) = queue_editor {
        let editor = text_edit::TextEdit::new(ed, Msg::QueueAction)
            .show_gutter(false)
            .word_wrap(true)
            .read_only(true)
            .fit_content(true)
            .transparent_bg(true);
        let close_btn = button(
            text("×")
                .size(theme::content_size())
                .color(theme::text_muted()),
        )
        .on_press(Msg::DiscardQueue)
        .padding([0.0, theme::SPACING_XS])
        .style(|_theme, _status| iced::widget::button::Style {
            background: None,
            ..Default::default()
        });
        let label = text("Queued (enter to interrupt and send, backspace to cancel)")
            .size(theme::font_sm())
            .color(theme::text_muted());
        let header_row = row![
            container(label).width(Length::Fill),
            container(close_btn).align_y(iced::Alignment::Start),
        ]
        .spacing(theme::SPACING_XS)
        .align_y(iced::Alignment::Center);
        let pill_col =
            column![header_row, container(editor).width(Length::Fill)].spacing(theme::SPACING_XS);
        composer_col = composer_col.push(
            container(pill_col)
                .padding([theme::SPACING_SM, theme::SPACING_MD])
                .width(Length::Fill)
                .style(theme::chat_queued_card),
        );
    }

    // Selection-context chips: pinned first, then the live tentative slot.
    // Iced 0.14's `Row::wrap()` lays children out across multiple lines
    // when they overflow horizontally, so a long chip set grows upward
    // above the input rather than forcing a horizontal scroll. Tab-source
    // labels are abbreviated to filename + minimal disambiguating
    // parents — long paths would otherwise get truncated by ellipsis.
    if has_attachments {
        let mut all: Vec<&SelectionContext> = pinned_selections.iter().collect();
        if let Some(t) = tentative_selection {
            all.push(t);
        }
        let labels = interaction::chip_labels_abbreviated(&all);
        let pinned_count = pinned_selections.len();
        let mut chips: Vec<Element<'a, Msg>> = Vec::with_capacity(all.len());
        for (i, label) in labels.into_iter().enumerate() {
            let tentative = i >= pinned_count;
            chips.push(view_selection_chip(label, tentative));
        }
        let wrapped = iced::widget::Row::with_children(chips)
            .spacing(theme::SPACING_XS)
            .align_y(iced::Alignment::Center)
            .wrap()
            .vertical_spacing(theme::SPACING_XS);
        composer_col = composer_col.push(
            container(wrapped)
                .padding([0.0, theme::SPACING_SM])
                .width(Length::Fill),
        );
    }

    composer_col = composer_col.push(input);
    composer_col = composer_col.push(meta_row);

    // Horizontal padding here sums with TextEdit's internal CONTENT_PAD (8px)
    // to land the input's text at the same 12px the chat headers use.
    // Awaiting a user choice: quiet accent tint on the whole composer section.
    let composer_style = if crate::fast_response::awaiting_composer_chrome(status.is_awaiting_user)
    {
        theme::chat_composer_awaiting
    } else {
        theme::chat_input
    };
    let input_row = container(composer_col)
        .padding([theme::SPACING_SM, theme::SPACING_XS])
        .width(Length::Fill)
        .style(composer_style);

    // Stable outer column (scroll → completion → divider → input) so showing
    // or hiding in-scroll chrome never remounts the input and steals focus.
    column![chat_area, completion_el, input_divider, input_row]
        .height(Length::Fill)
        .into()
}

/// Measure the chat scrollable's viewport and content heights via a widget
/// operation. Unlike `on_scroll`, this runs even when content fits the
/// viewport (iced suppresses scroll notifications in that case).
pub fn measure_scroll_bounds() -> iced::Task<(f32, f32)> {
    use iced::Rectangle;
    use iced::Vector;
    use iced::advanced::widget::Id;
    use iced::advanced::widget::operation::{self, Operation, Outcome};

    struct Measure {
        viewport_h: Option<f32>,
        content_h: Option<f32>,
    }

    impl Operation<(f32, f32)> for Measure {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<(f32, f32)>)) {
            operate(self);
        }

        fn scrollable(
            &mut self,
            id: Option<&Id>,
            bounds: Rectangle,
            content_bounds: Rectangle,
            _translation: Vector,
            _state: &mut dyn operation::Scrollable,
        ) {
            if id == Some(&Id::new(CHAT_SCROLLABLE_ID)) {
                self.viewport_h = Some(bounds.height);
                self.content_h = Some(content_bounds.height);
            }
        }

        fn finish(&self) -> Outcome<(f32, f32)> {
            match (self.viewport_h, self.content_h) {
                (Some(v), Some(c)) if v > 0.0 => Outcome::Some((v, c)),
                _ => Outcome::None,
            }
        }
    }

    iced::advanced::widget::operate(Measure {
        viewport_h: None,
        content_h: None,
    })
}

/// Render a single chat block, Zed-style calm transcript:
///
/// - **User** (normal): bordered card on the "paper" surface (no label, no chevron).
/// - **User** (priming Setup): collapsible muted header; user-card body when open.
/// - **Answer / System**: plain text flowing on the chat background.
/// - **Thinking**: muted collapsible header; body when expanded.
/// - **Activity**: same flat secondary chrome + quiet tool rows when expanded.
fn view_block<'a>(
    idx: usize,
    block: &'a Block,
    editor: Option<&'a EditorState>,
    collapsed: bool,
    hl_ranges: Vec<text_edit::HighlightRange>,
    hl_current: Option<text_edit::HighlightRange>,
    is_last_answer: bool,
    viewer_style: ViewerStyle,
    focus_folds: Option<&'a std::collections::HashMap<String, crate::focus_answer::SectionFoldState>>,
) -> Element<'a, Msg> {
    match block.kind {
        BlockKind::Reasoning => {
            view_thinking_block(idx, block, editor, collapsed, hl_ranges, hl_current)
        }
        BlockKind::Activity | BlockKind::ToolUse | BlockKind::ToolResult => {
            view_activity_block(idx, block, editor, collapsed, hl_ranges, hl_current)
        }
        BlockKind::User if block.is_priming => {
            view_priming_user_block(idx, block, editor, collapsed, hl_ranges, hl_current)
        }
        BlockKind::Assistant => {
            let source = block.lines.join("\n");
            match answer_body_presentation(viewer_style, block.is_live, &source) {
                AnswerBodyPresentation::ClassicFullBody => {
                    view_prose_block(idx, block, editor, hl_ranges, hl_current, is_last_answer)
                }
                AnswerBodyPresentation::FocusSectioned { sections, open: _ } => {
                    view_focus_answer(
                        idx,
                        block,
                        editor,
                        &sections,
                        focus_folds,
                        hl_ranges,
                        hl_current,
                        is_last_answer,
                    )
                }
            }
        }
        BlockKind::User | BlockKind::System => {
            // Non-Answer prose: viewer style does not select the path.
            view_prose_block(idx, block, editor, hl_ranges, hl_current, is_last_answer)
        }
        BlockKind::UserChoiceQuestion => {
            view_transcript_choice_chip(block, theme::chat_fast_response_chip_question)
        }
        BlockKind::UserChoiceAnswer => {
            view_transcript_choice_chip(block, theme::chat_fast_response_chip_numbered)
        }
    }
}

/// Focus Answer chrome: collapsible section headers + always-open meta region.
///
/// Hybrid C: expanded sections are plain content-font text (no band); the open
/// region uses the Classic TextEdit body recipe, with last-answer band only there.
fn view_focus_answer<'a>(
    block_idx: usize,
    block: &'a Block,
    editor: Option<&'a EditorState>,
    sections: &[crate::focus_answer::FoldableSection],
    focus_folds: Option<&'a std::collections::HashMap<String, crate::focus_answer::SectionFoldState>>,
    hl_ranges: Vec<text_edit::HighlightRange>,
    hl_current: Option<text_edit::HighlightRange>,
    is_last_answer: bool,
) -> Element<'a, Msg> {
    use crate::focus_answer::{range_line_count, section_collapsed_label, section_key};

    debug_assert!(focus_slice_uses_classic_body_paint(FocusSliceKind::OpenRegion));
    debug_assert!(!focus_slice_uses_classic_body_paint(
        FocusSliceKind::ExpandedSection
    ));

    let mut col = column![].spacing(theme::SPACING_XS).width(Length::Fill);

    for section in sections {
        let key = section_key(&section.kind);
        let line_count = range_line_count(section.lines);
        let collapsed = focus_folds
            .and_then(|m| m.get(&key))
            .map(|s| s.collapsed)
            .unwrap_or(true);
        let label = if collapsed {
            section_collapsed_label(&section.kind, line_count)
        } else {
            match &section.kind {
                crate::focus_answer::SectionKind::Preamble => {
                    section_collapsed_label(&section.kind, line_count)
                }
                crate::focus_answer::SectionKind::Heading { text, .. } => text.clone(),
            }
        };
        let key_for_msg = key.clone();
        let header_label = text(label)
            .size(theme::content_size())
            .font(theme::content_font())
            .color(theme::text_muted());
        let header_row = row![collapsible::chevron(!collapsed), header_label]
            .spacing(theme::SPACING_XS)
            .align_y(iced::Alignment::Center);
        let header = button(header_row)
            .on_press(Msg::ToggleFocusSection {
                block_idx,
                key: key_for_msg,
            })
            .padding(0.0)
            .style(|_theme, _status| iced::widget::button::Style {
                background: None,
                ..Default::default()
            });
        col = col.push(
            container(header)
                .padding([theme::SPACING_XS, theme::SPACING_MD])
                .width(Length::Fill),
        );
        if !collapsed {
            // Expanded section: plain source text; never last-answer band.
            let body = slice_lines(&block.lines, section.lines);
            col = col.push(plain_section_body_view(body));
        }
    }

    // Open region: Classic TextEdit recipe; band only when last Answer.
    col = col.push(focus_open_region_view(
        block_idx,
        editor,
        hl_ranges,
        hl_current,
        is_last_answer,
    ));

    col.into()
}

fn slice_lines(lines: &[String], range: crate::focus_answer::LineRange) -> String {
    if lines.is_empty() {
        return String::new();
    }
    let start = range.start.min(lines.len().saturating_sub(1));
    let end = range.end.min(lines.len().saturating_sub(1));
    if start > end {
        return String::new();
    }
    lines[start..=end].join("\n")
}

/// Expanded Focus section body: plain content-font source (not classic paint).
fn plain_section_body_view(body: String) -> Element<'static, Msg> {
    let body_el = text(body)
        .size(theme::content_size())
        .font(theme::content_font())
        .color(theme::text_primary());
    container(body_el)
        .padding([theme::SPACING_SM, theme::SPACING_MD])
        .width(Length::Fill)
        .into()
}

/// Open region: same Classic body recipe as [`view_prose_block`] for Answers.
fn focus_open_region_view<'a>(
    idx: usize,
    editor: Option<&'a EditorState>,
    hl_ranges: Vec<text_edit::HighlightRange>,
    hl_current: Option<text_edit::HighlightRange>,
    is_last_answer: bool,
) -> Element<'a, Msg> {
    let Some(ed) = editor else {
        return Space::new().into();
    };
    if ed.lines.is_empty() || (ed.lines.len() == 1 && ed.lines[0].is_empty()) {
        return Space::new().into();
    }

    let content = text_edit::TextEdit::new(ed, move |action| Msg::ChatAction(idx, action))
        .show_gutter(false)
        .word_wrap(true)
        .md_tables(true)
        .read_only(true)
        .fit_content(true)
        .transparent_bg(true)
        .highlights(hl_ranges, hl_current);

    let padded = container(content)
        .padding([theme::SPACING_SM, theme::SPACING_MD])
        .width(Length::Fill);

    if focus_slice_uses_last_answer_band(FocusSliceKind::OpenRegion, is_last_answer) {
        padded.style(theme::chat_last_answer_band).into()
    } else {
        padded.into()
    }
}

/// Settled question/answer chip in the transcript (same chrome as live shell).
fn view_transcript_choice_chip<'a>(
    block: &'a Block,
    style: fn(&iced::Theme) -> container::Style,
) -> Element<'a, Msg> {
    let label = block.lines.join("\n");
    if label.is_empty() {
        return Space::new().into();
    }
    let body = text(label)
        .size(theme::content_size())
        .color(theme::text_secondary())
        .font(theme::content_font());
    let card = container(body)
        .padding([theme::SPACING_SM, theme::SPACING_MD])
        .width(Length::Fill)
        .style(style);
    container(card)
        .padding([0.0, theme::SPACING_SM])
        .width(Length::Fill)
        .into()
}

/// User / Answer / System: no header, no chevron.
fn view_prose_block<'a>(
    idx: usize,
    block: &'a Block,
    editor: Option<&'a EditorState>,
    hl_ranges: Vec<text_edit::HighlightRange>,
    hl_current: Option<text_edit::HighlightRange>,
    is_last_answer: bool,
) -> Element<'a, Msg> {
    let has_content = !block.lines.is_empty();
    if !has_content {
        return Space::new().into();
    }
    let Some(ed) = editor else {
        return Space::new().into();
    };

    let content = text_edit::TextEdit::new(ed, move |action| Msg::ChatAction(idx, action))
        .show_gutter(false)
        .word_wrap(true)
        .md_tables(true)
        .read_only(true)
        .fit_content(true)
        .transparent_bg(true)
        .highlights(hl_ranges, hl_current);

    let padded = container(content)
        .padding([theme::SPACING_SM, theme::SPACING_MD])
        .width(Length::Fill);

    match block.kind {
        BlockKind::User => container(padded.style(theme::chat_user_card))
            .padding([0.0, theme::SPACING_SM])
            .width(Length::Fill)
            .into(),
        BlockKind::Assistant if is_last_answer => padded
            .style(theme::chat_last_answer_band)
            .width(Length::Fill)
            .into(),
        _ => padded.into(),
    }
}

/// Flat collapsible header: chevron + muted label (shared by Thinking and Activity).
fn secondary_segment_header<'a>(
    expanded: bool,
    label: impl Into<String>,
    on_toggle: Msg,
) -> Element<'a, Msg> {
    let label = text(label.into())
        .size(theme::content_size())
        .font(theme::content_font())
        .color(theme::text_muted());
    let header_row = row![collapsible::chevron(expanded), label]
        .spacing(theme::SPACING_XS)
        .align_y(iced::Alignment::Center);
    let header_content: Element<'a, Msg> = button(header_row)
        .on_press(on_toggle)
        .padding(0.0)
        .style(|_theme, _status| iced::widget::button::Style {
            background: None,
            ..Default::default()
        })
        .into();
    container(header_content)
        .padding([theme::SPACING_XS, theme::SPACING_MD])
        .width(Length::Fill)
        .into()
}

/// Priming Setup user message: collapsible so scroll-to-top skips the
/// AGENTS.md / orientation inject. Starts collapsed; expand on click;
/// auto-hides again after [`PRIMING_RECOLLAPSE_SECS`].
fn view_priming_user_block<'a>(
    idx: usize,
    block: &'a Block,
    editor: Option<&'a EditorState>,
    collapsed: bool,
    hl_ranges: Vec<text_edit::HighlightRange>,
    hl_current: Option<text_edit::HighlightRange>,
) -> Element<'a, Msg> {
    let has_content = !block.lines.is_empty();
    let body_shown = has_content && !collapsed && editor.is_some();
    let header_label = if collapsed {
        priming_collapsed_label(&block.lines)
    } else {
        block.label.clone()
    };
    let label = text(header_label)
        .size(theme::content_size())
        .font(theme::content_font())
        .color(theme::text_muted());
    let header_row = row![collapsible::chevron(!collapsed), label]
        .spacing(theme::SPACING_XS)
        .align_y(iced::Alignment::Center);
    let header_content: Element<'a, Msg> = button(header_row)
        .on_press(Msg::ToggleCollapse(idx))
        .padding(0.0)
        .style(|_theme, _status| iced::widget::button::Style {
            background: None,
            ..Default::default()
        })
        .into();
    let header = container(header_content)
        .padding([theme::SPACING_XS, theme::SPACING_MD])
        .width(Length::Fill);

    let mut col = column![header].width(Length::Fill);
    if body_shown && let Some(ed) = editor {
        let content = text_edit::TextEdit::new(ed, move |action| Msg::ChatAction(idx, action))
            .show_gutter(false)
            .word_wrap(true)
            .md_tables(true)
            .read_only(true)
            .fit_content(true)
            .transparent_bg(true)
            .highlights(hl_ranges, hl_current);
        let padded = container(content)
            .padding([theme::SPACING_SM, theme::SPACING_MD])
            .width(Length::Fill)
            .style(theme::chat_user_card);
        col = col.push(
            container(padded)
                .padding([0.0, theme::SPACING_SM])
                .width(Length::Fill),
        );
    }

    container(col)
        .padding([0.0, theme::SPACING_SM])
        .width(Length::Fill)
        .into()
}

/// Thinking: collapsible muted header; expanded body is the thought text.
fn view_thinking_block<'a>(
    idx: usize,
    block: &'a Block,
    editor: Option<&'a EditorState>,
    collapsed: bool,
    hl_ranges: Vec<text_edit::HighlightRange>,
    hl_current: Option<text_edit::HighlightRange>,
) -> Element<'a, Msg> {
    let has_content = !block.lines.is_empty();
    let body_shown = has_content && !collapsed && editor.is_some();
    let header_label = if collapsed {
        thinking_collapsed_label(&block.lines)
    } else {
        block.label.clone()
    };
    let header = secondary_segment_header(!collapsed, header_label, Msg::ToggleCollapse(idx));

    let mut col = column![header].width(Length::Fill);
    if body_shown && let Some(ed) = editor {
        let body = container(
            text_edit::TextEdit::new(ed, move |action| Msg::ChatAction(idx, action))
                .show_gutter(false)
                .word_wrap(true)
                .md_tables(true)
                .read_only(true)
                .fit_content(true)
                .transparent_bg(true)
                .base_color(theme::text_secondary())
                .highlights(hl_ranges, hl_current),
        )
        .padding(iced::Padding {
            top: 0.0,
            right: theme::SPACING_MD,
            bottom: theme::SPACING_SM,
            left: theme::SPACING_MD,
        })
        .width(Length::Fill);
        col = col.push(body);
    }

    container(col)
        .padding([0.0, theme::SPACING_SM])
        .width(Length::Fill)
        .into()
}

/// Activity group: flat secondary header with quiet tool rows (no card chrome).
fn view_activity_block<'a>(
    idx: usize,
    block: &'a Block,
    editor: Option<&'a EditorState>,
    collapsed: bool,
    hl_ranges: Vec<text_edit::HighlightRange>,
    hl_current: Option<text_edit::HighlightRange>,
) -> Element<'a, Msg> {
    let has_content = !block.lines.is_empty();
    let body_shown = has_content && !collapsed && editor.is_some();
    let header =
        secondary_segment_header(!collapsed, block.label.clone(), Msg::ToggleCollapse(idx));

    let mut col = column![header].width(Length::Fill);
    if body_shown && let Some(ed) = editor {
        let body = container(
            text_edit::TextEdit::new(ed, move |action| Msg::ChatAction(idx, action))
                .show_gutter(false)
                .word_wrap(true)
                .md_tables(true)
                .read_only(true)
                .fit_content(true)
                .transparent_bg(true)
                .base_color(theme::text_secondary())
                .highlights(hl_ranges, hl_current),
        )
        .padding(iced::Padding {
            top: 0.0,
            right: theme::SPACING_MD,
            bottom: theme::SPACING_SM,
            left: theme::SPACING_MD,
        })
        .width(Length::Fill);
        col = col.push(body);
    }

    container(col)
        .padding([0.0, theme::SPACING_SM])
        .width(Length::Fill)
        .into()
}

/// One selection-context chip — a small bordered label sitting above the
/// chat input. `tentative` chips use a muted border to signal "not yet
/// pinned (Cmd-K to keep)"; pinned chips use the primary border color.
fn view_selection_chip<'a>(label: String, tentative: bool) -> Element<'a, Msg> {
    let style = if tentative {
        theme::selection_chip_tentative
    } else {
        theme::selection_chip_pinned
    };
    let color = if tentative {
        theme::text_secondary()
    } else {
        theme::text_primary()
    };
    container(
        text(label)
            .size(theme::font_sm())
            .color(color)
            .wrapping(iced::widget::text::Wrapping::None),
    )
    .padding([2.0, theme::SPACING_SM])
    .style(style)
    .into()
}

fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(ch);
    }
    result
}

/// Option chrome: optional question chip, then numbered option chips.
/// View chrome only until activation / settle commits host blocks.
fn view_fast_response<'a>(fr: &'a crate::fast_response::FastResponse) -> Element<'a, Msg> {
    use crate::fast_response::{FastResponsePick, live_question_prompt, option_chip_label};

    let mut col = column![].spacing(theme::SPACING_XS);

    if let Some(prompt) = live_question_prompt(fr) {
        col = col.push(view_fast_response_question_chip(&prompt));
    }

    for (i, opt) in fr.options.iter().enumerate() {
        col = col.push(view_fast_response_chip(
            option_chip_label(i + 1, &opt.label),
            FastResponsePick::Option { id: opt.id.clone() },
        ));
    }

    container(col)
        .padding([0.0, theme::SPACING_SM])
        .width(Length::Fill)
        .into()
}

/// Non-selectable question chip above live option chips (chat-area fill).
fn view_fast_response_question_chip<'a>(prompt: &str) -> Element<'a, Msg> {
    let body = text(prompt.to_string())
        .size(theme::content_size())
        .color(theme::text_secondary())
        .font(theme::content_font());
    container(body)
        .padding([theme::SPACING_SM, theme::SPACING_MD])
        .width(Length::Fill)
        .style(theme::chat_fast_response_chip_question)
        .into()
}

/// One action chip: hotkey-first label; click activates the pick only.
/// Label ink uses secondary text (full alpha) for readable contrast on quiet
/// tinted fills in both light and dark themes.
fn view_fast_response_chip<'a>(
    label: String,
    pick: crate::fast_response::FastResponsePick,
) -> Element<'a, Msg> {
    let body = text(label)
        .size(theme::content_size())
        .color(theme::text_secondary())
        .font(theme::content_font());

    let card = container(body)
        .padding([theme::SPACING_SM, theme::SPACING_MD])
        .width(Length::Fill)
        .style(theme::chat_fast_response_chip_numbered);

    button(card)
        .on_press(Msg::ActivateFastResponse(pick))
        .padding(0.0)
        .width(Length::Fill)
        .style(|_theme, status| {
            let base = iced::widget::button::Style {
                background: None,
                ..Default::default()
            };
            match status {
                iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
                    base
                }
                _ => base,
            }
        })
        .into()
}

// view_status_bar removed: model + context now blend into the input area
// (see `view`), and stream state is conveyed by the streaming indicator.

// ── Completion popup ────────────────────────────────────────────────────────

fn view_completion_col<'a>(
    commands: &'a [SlashCommand],
    filtered: &[(usize, i32)],
    selected: usize,
) -> iced::widget::Column<'a, Msg> {
    let mut items = column![].spacing(0.0);
    for (i, &(cmd_idx, _score)) in filtered.iter().enumerate() {
        let cmd = &commands[cmd_idx];
        let is_selected = i == selected;
        let name_color = theme::slash_command_name_color(cmd.kind);
        let mut label = row![
            text(format!("/{}", cmd.name))
                .size(theme::font_sm())
                .color(name_color),
        ];
        if let Some(tag) = slash_kind_row_tag(cmd.kind) {
            label = label.push(Space::new().width(theme::SPACING_SM)).push(
                text(tag)
                    .size(theme::font_sm())
                    .color(theme::text_muted()),
            );
        }
        label = label
            .push(Space::new().width(theme::SPACING_SM))
            .push(
                text(&cmd.description)
                    .size(theme::font_sm())
                    .color(theme::text_muted()),
            )
            .align_y(iced::Alignment::Center);
        items = items.push(
            container(label)
                .width(Length::Fill)
                .padding([theme::SPACING_XS, theme::SPACING_MD])
                .style(move |_theme: &iced::Theme| {
                    if is_selected {
                        container::Style {
                            background: Some(iced::Background::Color(theme::bg_list_hover())),
                            ..Default::default()
                        }
                    } else {
                        container::Style::default()
                    }
                }),
        );
    }
    items
}

/// Hairline separator used at the top of the completion popup so it reads
/// as a distinct surface sitting above the chat transcript.
fn completion_divider<'a>() -> Element<'a, Msg> {
    rule::horizontal(1)
        .style(|_theme: &iced::Theme| rule::Style {
            color: theme::border_color(),
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        })
        .into()
}

// ── Fuzzy matching ──────────────────────────────────────────────────────────

/// Filter and score commands by fuzzy-matching `query` against command names.
/// Returns `(index_into_commands, score)` sorted by descending score, then
/// System → Workflow → Agent, then Workflow order_key (None last), then name.
pub fn filter_commands(commands: &[SlashCommand], query: &str) -> Vec<(usize, i32)> {
    let mut matches: Vec<(usize, i32)> = commands
        .iter()
        .enumerate()
        .filter_map(|(i, cmd)| fuzzy_score(query, &cmd.name).map(|s| (i, s)))
        .collect();
    matches.sort_by(|a, b| {
        let ca = &commands[a.0];
        let cb = &commands[b.0];
        b.1.cmp(&a.1)
            .then_with(|| slash_kind_rank(ca.kind).cmp(&slash_kind_rank(cb.kind)))
            .then_with(|| {
                crate::slash_commands::slash_order_rank(ca.order_key)
                    .cmp(&crate::slash_commands::slash_order_rank(cb.order_key))
            })
            .then_with(|| ca.name.cmp(&cb.name))
    });
    matches
}

/// Subsequence fuzzy match. Returns `None` if `query` is not a subsequence of
/// `target`, otherwise a score (higher = better).
fn fuzzy_score(query: &str, target: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }
    let query_lower: Vec<char> = query.to_lowercase().chars().collect();
    let target_lower: Vec<char> = target.to_lowercase().chars().collect();
    let mut qi = 0;
    let mut score = 0i32;
    let mut prev_match = false;

    for (i, &ch) in target_lower.iter().enumerate() {
        if qi < query_lower.len() && ch == query_lower[qi] {
            qi += 1;
            score += 1;
            if i == 0 {
                score += 3; // bonus for matching start
            }
            if prev_match {
                score += 2; // bonus for consecutive
            }
            prev_match = true;
        } else {
            prev_match = false;
        }
    }

    if qi == query_lower.len() {
        Some(score)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use duckchat::SlashCommandKind;

    fn model(harness: &str, id: &str, window: Option<usize>) -> ModelInfo {
        ModelInfo {
            harness: harness.to_string(),
            id: id.to_string(),
            display: id.to_string(),
            context_window: window,
        }
    }

    fn cmd(name: &str, kind: SlashCommandKind) -> SlashCommand {
        SlashCommand {
            name: name.into(),
            description: name.into(),
            kind,
            order_key: None,
        }
    }

    /// @spec chat/slash-commands Kind cues in completion: Name token color maps by kind
    #[test]
    fn name_token_color_maps_by_kind() {
        // GIVEN completion rows for System, Workflow, and Agent entries
        // WHEN name-token colors are resolved
        let sys = theme::slash_command_name_color(SlashCommandKind::System);
        let wf = theme::slash_command_name_color(SlashCommandKind::Workflow);
        let agent = theme::slash_command_name_color(SlashCommandKind::Agent);
        // THEN the three kinds resolve to three different colors
        assert_ne!(sys, wf);
        assert_ne!(sys, agent);
        assert_ne!(wf, agent);
    }

    /// @spec chat/slash-commands Kind cues in completion: System rows include a sys tag
    #[test]
    fn system_rows_include_a_sys_tag() {
        // GIVEN a System completion entry
        // WHEN the completion row tag is resolved
        // THEN the row includes a `sys` tag
        assert_eq!(
            slash_kind_row_tag(SlashCommandKind::System),
            Some("sys")
        );
        assert_eq!(slash_kind_row_tag(SlashCommandKind::Workflow), None);
        assert_eq!(slash_kind_row_tag(SlashCommandKind::Agent), None);
    }

    /// @spec chat/slash-commands Kind cues in completion: Equal fuzzy scores order System, Workflow, Agent
    #[test]
    fn equal_fuzzy_scores_order_system_workflow_agent() {
        // GIVEN three catalog entries of kinds System, Workflow, Agent that all score equally
        // (empty query scores every name 0)
        let commands = vec![
            cmd("agent-cmd", SlashCommandKind::Agent),
            cmd("sys-cmd", SlashCommandKind::System),
            cmd("ds-cmd", SlashCommandKind::Workflow),
        ];
        // WHEN the filtered completion list is built
        let filtered = filter_commands(&commands, "");
        // THEN those three appear in order System, then Workflow, then Agent
        assert_eq!(filtered.len(), 3);
        assert_eq!(commands[filtered[0].0].kind, SlashCommandKind::System);
        assert_eq!(commands[filtered[1].0].kind, SlashCommandKind::Workflow);
        assert_eq!(commands[filtered[2].0].kind, SlashCommandKind::Agent);
    }

    fn workflow(name: &str, order_key: Option<u32>) -> SlashCommand {
        SlashCommand {
            name: name.into(),
            description: name.into(),
            kind: SlashCommandKind::Workflow,
            order_key,
        }
    }

    // @spec chat/slash-commands Kind cues in completion: Equal scores order Workflow by order key then name
    #[test]
    fn equal_scores_order_workflow_by_order_key_then_name() {
        // GIVEN two Workflow catalog entries with equal fuzzy scores for the current query
        // AND the first has a higher order key than the second
        let commands = vec![
            workflow("ds-spec", Some(40)),
            workflow("ds-explore", Some(10)),
        ];
        // WHEN the filtered completion list is built
        let filtered = filter_commands(&commands, "");
        // THEN the entry with the lower order key appears before the entry with the higher order key
        assert_eq!(filtered.len(), 2);
        assert_eq!(commands[filtered[0].0].name, "ds-explore");
        assert_eq!(commands[filtered[1].0].name, "ds-spec");
    }

    // @spec chat/slash-commands Kind cues in completion: Workflow without order key sorts after ordered Workflow
    #[test]
    fn workflow_without_order_key_sorts_after_ordered_workflow() {
        // GIVEN two Workflow catalog entries with equal fuzzy scores for the current query
        // AND one entry has an order key and the other has no order key
        let commands = vec![
            workflow("ds-custom", None),
            workflow("ds-explore", Some(10)),
        ];
        // WHEN the filtered completion list is built
        let filtered = filter_commands(&commands, "");
        // THEN the entry with an order key appears before the entry without an order key
        assert_eq!(filtered.len(), 2);
        assert_eq!(commands[filtered[0].0].name, "ds-explore");
        assert_eq!(commands[filtered[1].0].name, "ds-custom");
    }

    /// @spec harness/model-picker Harness-grouped choices: Choices present each model under its harness
    #[test]
    fn choices_present_each_model_under_its_harness() {
        // GIVEN selectable models drawn from more than one harness.
        let models = vec![
            model("claude-code", "opus", None),
            model("grok", "grok-4.5", Some(256_000)),
            model("claude-code", "sonnet", None),
        ];
        // WHEN the picker choices are built.
        let choices = group_choices(models);
        // THEN each model appears under its owning harness — the choice carries
        // its model's harness and its label is presented under that harness.
        let opus = choices
            .iter()
            .find(|c| c.id.as_deref() == Some("opus"))
            .unwrap();
        assert_eq!(opus.harness.as_deref(), Some("claude-code"));
        assert!(opus.label.starts_with("Claude Code · "));
        let grok = choices
            .iter()
            .find(|c| c.id.as_deref() == Some("grok-4.5"))
            .unwrap();
        assert_eq!(grok.harness.as_deref(), Some("grok"));
        assert!(grok.label.starts_with("Grok · "));
        // AND a harness's models stay contiguous rather than interleaved.
        let harnesses: Vec<&str> = choices
            .iter()
            .map(|c| c.harness.as_deref().unwrap())
            .collect();
        assert_eq!(harnesses, ["claude-code", "claude-code", "grok"]);
    }

    /// @spec harness/model-picker Context fill from the active model's window: Fill is measured against the selected model's window
    #[test]
    fn fill_is_measured_against_the_selected_models_window() {
        // GIVEN a selected model with a known context window AND a used-token count.
        let window = Some(200_000);
        let used = 50_000;
        // WHEN the usage meter fill is computed.
        let fill = context_fill(used, window);
        // THEN the fill is the used tokens relative to that model's window.
        assert_eq!(fill, Some(0.25));
    }

    /// @spec harness/model-picker Context fill from the active model's window: A model with no known window shows no fill
    #[test]
    fn a_model_with_no_known_window_shows_no_fill() {
        // GIVEN a selected model with no known context window.
        // WHEN the usage meter fill is computed.
        let fill = context_fill(50_000, None);
        // THEN the meter shows no fill.
        assert_eq!(fill, None);
    }

    /// @spec chat/composer-footer Resend hint only for unresumable stored session: Hint shown when stored session is unresumable
    #[test]
    fn hint_shown_when_stored_session_is_unresumable() {
        // GIVEN a non-empty transcript AND a stored agent session id that is
        // not resumable for the effective harness.
        let has_messages = true;
        let has_stored_agent_id = true;
        let will_resume = false;
        // WHEN the composer footer is rendered (hint visibility is computed).
        let show = show_resend_history_hint(
            has_messages,
            unresumable_stored_session(has_stored_agent_id, will_resume),
        );
        // THEN the resend-history hint is shown.
        assert!(show);
    }

    /// @spec chat/composer-footer Resend hint only for unresumable stored session: Hint hidden when stored session is resumable
    #[test]
    fn hint_hidden_when_stored_session_is_resumable() {
        // GIVEN a non-empty transcript AND a stored agent session id that is
        // resumable for the effective harness.
        let has_messages = true;
        let has_stored_agent_id = true;
        let will_resume = true;
        // WHEN the composer footer is rendered.
        let show = show_resend_history_hint(
            has_messages,
            unresumable_stored_session(has_stored_agent_id, will_resume),
        );
        // THEN the resend-history hint is not shown.
        assert!(!show);
    }

    /// @spec chat/composer-footer Resend hint only for unresumable stored session: Hint hidden when transcript is empty
    #[test]
    fn hint_hidden_when_transcript_is_empty() {
        // GIVEN an empty transcript.
        let has_messages = false;
        let has_stored_agent_id = true;
        let will_resume = false;
        // WHEN the composer footer is rendered.
        let show = show_resend_history_hint(
            has_messages,
            unresumable_stored_session(has_stored_agent_id, will_resume),
        );
        // THEN the resend-history hint is not shown.
        assert!(!show);
    }

    /// @spec chat/composer-footer Resend hint only for unresumable stored session: Hint hidden when no stored agent session id
    #[test]
    fn hint_hidden_when_no_stored_agent_session_id() {
        // GIVEN a non-empty transcript AND no stored agent session id.
        let has_messages = true;
        let has_stored_agent_id = false;
        let will_resume = false;
        // WHEN the composer footer is rendered.
        let show = show_resend_history_hint(
            has_messages,
            unresumable_stored_session(has_stored_agent_id, will_resume),
        );
        // THEN the resend-history hint is not shown.
        assert!(!show);
    }

    /// @spec chat/composer-footer Progressive usage readout: Cool fill shows percentage only
    #[test]
    fn cool_fill_shows_percentage_only() {
        // GIVEN a known context window AND used tokens such that fill is below 75%.
        let window = 200_000;
        let used = 50_000; // 25%
        // WHEN the usage readout is formatted.
        let readout = format_usage_readout(used, window);
        // THEN the readout shows the fill percentage AND does not include absolute used or max.
        assert_eq!(readout, "25%");
        assert!(!readout.contains('/'));
        assert!(!readout.contains(','));
    }

    /// @spec chat/composer-footer Progressive usage readout: Hot fill shows used, max, and percentage
    #[test]
    fn hot_fill_shows_used_max_and_percentage() {
        // GIVEN a known context window AND used tokens such that fill is at least 75%.
        let window = 200_000;
        let used = 150_000; // 75%
        // WHEN the usage readout is formatted.
        let readout = format_usage_readout(used, window);
        // THEN the readout includes used tokens, the window max, and the fill percentage.
        assert_eq!(readout, "150,000 / 200,000 (75%)");
    }

    /// @spec chat/composer-footer Short closed model label: Closed label is the model display name
    #[test]
    fn closed_label_is_the_model_display_name() {
        // GIVEN a selectable model with a harness name and a short display name.
        let models = vec![ModelInfo {
            harness: "grok".to_string(),
            id: "grok-4.5".to_string(),
            display: "Grok 4.5".to_string(),
            context_window: Some(500_000),
        }];
        // WHEN the closed model control label is built (with menu choices).
        let choices = group_choices(models);
        let choice = choices.first().expect("one choice");
        // THEN the closed label is the short display name AND does not include a harness prefix.
        assert_eq!(choice.closed_label, "Grok 4.5");
        assert!(!choice.closed_label.contains('·'));
        // Menu label remains harness-prefixed for grouped choices.
        assert!(choice.label.starts_with("Grok · "));
    }

    /// @spec chat/composer-footer Missing closed model label: Closed label is Missing when the effective model is not available
    #[test]
    fn closed_label_is_missing_when_the_effective_model_is_not_available() {
        // GIVEN an effective model that is not available
        let preferred = ModelRef::new("grok", "grok-4.5");
        // WHEN the closed model control label is built
        let with_preferred = missing_closed_model_choice(Some(&preferred));
        let unconfigured = missing_closed_model_choice(None);
        // THEN the label is Missing
        assert_eq!(with_preferred.closed_label, "Missing");
        assert_eq!(with_preferred.label, "Missing");
        assert_eq!(unconfigured.closed_label, "Missing");
        assert_eq!(unconfigured.label, "Missing");
    }

    #[test]
    fn strips_csi_color_codes() {
        let input = "\x1B[32mcreated \x1B[39m changes/foo/proposal.md";
        assert_eq!(
            strip_ansi_escapes(input),
            "created  changes/foo/proposal.md"
        );
    }

    #[test]
    fn strips_tool_use_error_tags() {
        let input = "<tool_use_error>File has not been read yet.</tool_use_error>";
        assert_eq!(
            strip_tool_wrapper_tags(input),
            "File has not been read yet.",
        );
    }

    #[test]
    fn truncate_output_cleans_color_and_tags() {
        let raw = "\x1B[32mcreated \x1B[39m changes/foo/proposal.md";
        assert_eq!(
            truncate_output(raw),
            vec!["created  changes/foo/proposal.md".to_string()],
        );

        let raw = "<tool_use_error>File has not been read yet.</tool_use_error>";
        assert_eq!(
            truncate_output(raw),
            vec!["File has not been read yet.".to_string()],
        );
    }

    #[test]
    fn truncate_chars_keeps_short_strings() {
        assert_eq!(truncate_chars("short", 40), "short");
        assert_eq!(truncate_chars("", 40), "");
    }

    #[test]
    fn truncate_chars_never_splits_multibyte() {
        // A run of multibyte chars whose byte length exceeds the limit: a byte
        // slice at `max` would land mid-character and panic. We must cut on a
        // char boundary and keep exactly `max` chars.
        let s = "日".repeat(60); // 60 three-byte chars
        let out = truncate_chars(&s, 40);
        assert_eq!(out.chars().count(), 40);
        assert!(s.starts_with(out));
    }

    #[test]
    fn tool_summary_handles_multibyte_pattern_and_command() {
        // Regression: these previously byte-sliced at 40/50 and aborted the
        // app when a multibyte char straddled the boundary.
        let long_pat = "—".repeat(60); // em-dash is 3 bytes each
        let input = format!(r#"{{"pattern":"{long_pat}"}}"#);
        let summary = format_tool_summary("Grep", &input);
        assert!(
            summary.starts_with("Grep · \"—"),
            "expected calm Grep label, got {summary}"
        );

        let long_cmd = "é".repeat(60); // 2 bytes each
        let input = format!(r#"{{"command":"{long_cmd}"}}"#);
        let summary = format_tool_summary("Bash", &input);
        assert!(
            summary.starts_with("Shell · é"),
            "Bash should map to Shell with command detail: {summary}"
        );
    }

    #[test]
    fn known_claude_and_grok_tools_share_calm_labels() {
        // Claude-style names
        assert_eq!(
            format_tool_summary(
                "Read",
                r#"{"path":"crates/duckboard/src/widget/agent_chat.rs"}"#
            ),
            "Read · src/widget/agent_chat.rs"
        );
        assert_eq!(
            format_tool_summary("Bash", r#"{"command":"cargo test -p duckboard"}"#),
            "Shell · cargo test -p duckboard"
        );
        assert_eq!(
            format_tool_summary("Grep", r#"{"pattern":"format_tool_summary"}"#),
            "Grep · \"format_tool_summary\""
        );
        assert_eq!(
            format_tool_summary(
                "Edit",
                r#"{"file_path":"src/state.rs","old_string":"a","new_string":"b"}"#
            ),
            "Edit · src/state.rs"
        );

        // Grok-style names — same verbs, same calm shape
        assert_eq!(
            format_tool_summary("read_file", r#"{"path":"foo.rs"}"#),
            "Read · foo.rs"
        );
        assert_eq!(
            format_tool_summary("run_terminal_command", r#"{"command":"ds status"}"#),
            "Shell · ds status"
        );
        assert_eq!(
            format_tool_summary(
                "search_replace",
                r#"{"path":"crates/duckboard/src/main.rs","old_string":"x","new_string":"y"}"#
            ),
            "Edit · duckboard/src/main.rs"
        );
    }

    #[test]
    fn unknown_tools_look_intentional_not_raw_json() {
        // Humanized name + one short detail; no JSON blob.
        let summary = format_tool_summary(
            "some_obscure_tool",
            r#"{"target":"widget","payload":{"nested":true},"contents":"a huge body\nwith lines"}"#,
        );
        assert_eq!(summary, "Some obscure tool · widget");
        assert!(!summary.contains('{'), "must not dump JSON: {summary}");
        assert!(
            !summary.contains("nested"),
            "must not dump nested objects: {summary}"
        );

        // Empty / minimal input: name alone, still clean.
        assert_eq!(
            format_tool_summary("camelCaseThing", ""),
            "Camel case thing"
        );
        assert_eq!(format_tool_summary("  ", r#"{}"#), "Tool");
        assert_eq!(
            format_tool_summary(
                "run_mystery",
                r#"{"old_string":"a\nb","new_string":"c\nd"}"#
            ),
            "Run mystery"
        );
    }

    #[test]
    fn collapsed_activity_uses_humanized_verbs() {
        let session = assistant_blocks(vec![
            tool_use("1", "read_file", r#"{"path":"a.rs"}"#),
            tool_result("1", "read_file", "ok"),
            tool_use("2", "run_terminal_command", r#"{"command":"ls"}"#),
            tool_result("2", "run_terminal_command", "file"),
            tool_use("3", "grep", r#"{"pattern":"x"}"#),
            tool_result("3", "grep", "hit"),
        ]);
        let segs = build_transcript_segments(&session);
        let tools = activity_tools(&segs);
        let label = activity_collapsed_label(tools);
        assert!(
            label.contains("Read") && label.contains("Shell") && label.contains("Grep"),
            "collapsed samples should use human verbs, not harness ids: {label}"
        );
        assert!(
            !label.contains("run_terminal") && !label.contains("read_file"),
            "raw harness ids must not appear: {label}"
        );
    }

    // @spec chat/transcript Host-choice tools omitted from Activity: AskUserQuestion tool content is omitted from Activity
    #[test]
    fn ask_user_question_tools_are_omitted_from_activity() {
        assert!(is_host_choice_tool_name("AskUserQuestion"));
        assert!(is_host_choice_tool_name("Ask user question"));
        assert!(!is_host_choice_tool_name("Read"));

        // GIVEN ToolUse/ToolResult for AskUserQuestion plus a real Read tool
        let session = assistant_blocks(vec![
            tool_use("q1", "AskUserQuestion", r#"{"questions":[]}"#),
            tool_result("q1", "AskUserQuestion", "ok"),
            tool_use("q2", "Ask user question", r#"{}"#),
            tool_result("q2", "Ask user question", "ok"),
            tool_use("r1", "Read", r#"{"path":"a.rs"}"#),
            tool_result("r1", "Read", "file"),
        ]);
        // WHEN transcript segments are built
        let segs = build_transcript_segments(&session);
        // THEN no Ask user activity rows; Read still appears
        let tools = activity_tools(&segs);
        assert_eq!(tools.len(), 1, "tools={tools:?}");
        assert!(
            tools[0].summary.contains("Read"),
            "expected Read only, got {:?}",
            tools[0].summary
        );
        assert!(
            segs.iter().all(|s| match s {
                TranscriptSeg::Activity { tools, .. } => tools
                    .iter()
                    .all(|t| !t.summary.to_lowercase().contains("ask user")),
                _ => true,
            }),
            "Ask user question must not appear in Activity: {segs:?}"
        );
    }

    // ── Transcript segment builder ──────────────────────────────────────

    fn assistant_blocks(blocks: Vec<ContentBlock>) -> ChatSession {
        let mut s = ChatSession::new("test".into());
        s.messages.push(crate::chat_store::ChatMessage {
            role: Role::Assistant,
            content: blocks,
            timestamp: String::new(),
            is_priming: false,
        });
        s
    }

    fn tool_use(id: &str, name: &str, input: &str) -> ContentBlock {
        ContentBlock::ToolUse {
            id: id.into(),
            name: name.into(),
            input: input.into(),
        }
    }

    fn tool_result(id: &str, name: &str, output: &str) -> ContentBlock {
        ContentBlock::ToolResult {
            id: id.into(),
            name: name.into(),
            output: output.into(),
        }
    }

    fn activity_tools(segs: &[TranscriptSeg]) -> &[ToolRow] {
        segs.iter()
            .find_map(|s| match s {
                TranscriptSeg::Activity { tools, .. } => Some(tools.as_slice()),
                _ => None,
            })
            .expect("expected an Activity segment")
    }

    /// @spec chat/transcript Segment construction: Reasoning then answer yields Thinking then Answer
    #[test]
    fn reasoning_then_answer_yields_thinking_then_answer() {
        // GIVEN a session whose assistant content is a reasoning block
        // followed by a text block.
        let session = assistant_blocks(vec![
            ContentBlock::Reasoning("ponder the options".into()),
            ContentBlock::Text("here is the answer".into()),
        ]);

        // WHEN the transcript segments are built.
        let segs = build_transcript_segments(&session);

        // THEN the segments are a Thinking segment then an Answer segment
        // AND the reasoning body is not part of the Answer segment.
        assert_eq!(segs.len(), 2);
        match &segs[0] {
            TranscriptSeg::Thinking { lines, live } => {
                assert_eq!(lines, &["ponder the options".to_string()]);
                assert!(!*live);
            }
            other => panic!("expected Thinking, got {other:?}"),
        }
        match &segs[1] {
            TranscriptSeg::Answer { lines, live } => {
                assert_eq!(lines, &["here is the answer".to_string()]);
                assert!(!*live);
                assert!(!lines.iter().any(|l| l.contains("ponder")));
            }
            other => panic!("expected Answer, got {other:?}"),
        }
    }

    /// @spec chat/transcript Segment construction: Contiguous tools yield one Activity with multiple rows
    #[test]
    fn contiguous_tools_yield_one_activity_with_multiple_rows() {
        // GIVEN a session whose assistant content is several consecutive
        // tool uses and their results.
        let session = assistant_blocks(vec![
            tool_use("1", "Read", r#"{"path":"a.rs"}"#),
            tool_result("1", "Read", "fn a() {}"),
            tool_use("2", "grep", r#"{"pattern":"foo"}"#),
            tool_result("2", "grep", "match"),
            tool_use("3", "shell", r#"{"command":"ls"}"#),
            tool_result("3", "shell", "file"),
        ]);

        // WHEN the transcript segments are built.
        let segs = build_transcript_segments(&session);

        // THEN those tools form a single Activity segment AND the segment
        // has one row per tool call.
        assert_eq!(segs.len(), 1);
        let tools = activity_tools(&segs);
        assert_eq!(tools.len(), 3);
        assert_eq!(tools[0].id, "1");
        assert_eq!(tools[1].id, "2");
        assert_eq!(tools[2].id, "3");
    }

    /// @spec chat/transcript Segment construction: Thought, tools, thought, answer yields four segments in order
    #[test]
    fn thought_tools_thought_answer_yields_four_segments() {
        // GIVEN a session whose assistant content is reasoning, then tools,
        // then reasoning, then text.
        let session = assistant_blocks(vec![
            ContentBlock::Reasoning("first thought".into()),
            tool_use("t1", "Read", r#"{"path":"x"}"#),
            tool_result("t1", "Read", "ok"),
            ContentBlock::Reasoning("second thought".into()),
            ContentBlock::Text("final answer".into()),
        ]);

        // WHEN the transcript segments are built.
        let segs = build_transcript_segments(&session);

        // THEN the segments are Thinking, Activity, Thinking, Answer in order.
        assert_eq!(segs.len(), 4);
        assert!(
            matches!(&segs[0], TranscriptSeg::Thinking { lines, .. } if lines == &["first thought".to_string()])
        );
        assert!(matches!(&segs[1], TranscriptSeg::Activity { tools, .. } if tools.len() == 1));
        assert!(
            matches!(&segs[2], TranscriptSeg::Thinking { lines, .. } if lines == &["second thought".to_string()])
        );
        assert!(
            matches!(&segs[3], TranscriptSeg::Answer { lines, .. } if lines == &["final answer".to_string()])
        );
    }

    /// @spec chat/transcript Segment construction: Live pending reasoning appears on an open Thinking segment
    #[test]
    fn live_pending_reasoning_appears_on_open_thinking() {
        // GIVEN a streaming session with non-empty pending reasoning and no
        // committed reasoning for that run yet.
        let mut session = ChatSession::new("test".into());
        session.is_streaming = true;
        session.pending_reasoning = "still thinking…".into();

        // WHEN the transcript segments are built.
        let segs = build_transcript_segments(&session);

        // THEN a live Thinking segment includes that pending reasoning text.
        assert_eq!(segs.len(), 1);
        match &segs[0] {
            TranscriptSeg::Thinking { lines, live } => {
                assert!(*live);
                assert_eq!(lines, &["still thinking…".to_string()]);
            }
            other => panic!("expected live Thinking, got {other:?}"),
        }
    }

    /// @spec chat/transcript Segment construction: Live reasoning with an open answer draft yields Thinking then one Answer
    #[test]
    fn live_reasoning_with_open_answer_draft_yields_thinking_then_one_answer() {
        // GIVEN a streaming session with both pending reasoning and pending answer.
        let mut session = ChatSession::new("test".into());
        session.is_streaming = true;
        session.pending_reasoning = "rethinking the outline".into();
        session.pending_text = "first draft of the write-gate".into();

        // WHEN the transcript segments are built.
        let segs = build_transcript_segments(&session);

        // THEN Thinking then one Answer for that open draft (not stacked answers).
        // Thinking may auto-collapse (`live=false`) when Answer follows — that is
        // collapse policy, not multi-answer thrash.
        assert_eq!(segs.len(), 2);
        match &segs[0] {
            TranscriptSeg::Thinking { lines, .. } => {
                assert_eq!(lines, &["rethinking the outline".to_string()]);
            }
            other => panic!("expected Thinking first, got {other:?}"),
        }
        match &segs[1] {
            TranscriptSeg::Answer { lines, live } => {
                assert!(*live, "open draft Answer should be live while streaming");
                assert_eq!(lines, &["first draft of the write-gate".to_string()]);
            }
            other => panic!("expected Answer second, got {other:?}"),
        }
        let answer_count = segs
            .iter()
            .filter(|s| matches!(s, TranscriptSeg::Answer { .. }))
            .count();
        assert_eq!(answer_count, 1, "exactly one Answer for the open draft");
    }

    /// @spec chat/transcript Activity pairing: Matching use and result become one done row
    #[test]
    fn matching_use_and_result_become_one_done_row() {
        // GIVEN a tool use and a tool result that share the same call id.
        let session = assistant_blocks(vec![
            tool_use("call-1", "Read", r#"{"path":"src/main.rs"}"#),
            tool_result("call-1", "Read", "fn main() {}"),
        ]);

        // WHEN the transcript segments are built.
        let segs = build_transcript_segments(&session);

        // THEN the Activity segment has one done row for that id AND the
        // row carries the tool summary and the result body.
        let tools = activity_tools(&segs);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].id, "call-1");
        assert_eq!(tools[0].status, ToolRowStatus::Done);
        assert!(tools[0].summary.contains("Read"));
        assert!(tools[0].summary.contains("main.rs"));
        assert_eq!(tools[0].output_lines, vec!["fn main() {}".to_string()]);
    }

    /// @spec chat/transcript Activity pairing: Non-adjacent use and result still pair by id
    #[test]
    fn non_adjacent_use_and_result_still_pair_by_id() {
        // GIVEN two tool uses and two results ordered so each result is not
        // immediately after its matching use, AND each result shares a call
        // id with exactly one of the uses.
        let session = assistant_blocks(vec![
            tool_use("a", "Read", r#"{"path":"a.rs"}"#),
            tool_use("b", "grep", r#"{"pattern":"x"}"#),
            tool_result("a", "Read", "contents of a"),
            tool_result("b", "grep", "match in b"),
        ]);

        // WHEN the transcript segments are built.
        let segs = build_transcript_segments(&session);

        // THEN each use is paired with its matching result into one done row
        // AND no row is labeled only as a generic done placeholder.
        let tools = activity_tools(&segs);
        assert_eq!(tools.len(), 2);
        let row_a = tools.iter().find(|t| t.id == "a").unwrap();
        let row_b = tools.iter().find(|t| t.id == "b").unwrap();
        assert_eq!(row_a.status, ToolRowStatus::Done);
        assert_eq!(row_b.status, ToolRowStatus::Done);
        assert_eq!(row_a.output_lines, vec!["contents of a".to_string()]);
        assert_eq!(row_b.output_lines, vec!["match in b".to_string()]);
        for row in tools {
            assert_ne!(row.summary, "✓ done");
            assert!(!row.summary.eq_ignore_ascii_case("done"));
        }
    }

    /// @spec chat/transcript Activity pairing: Orphan result is a named done row
    #[test]
    fn orphan_result_is_a_named_done_row() {
        // GIVEN a tool result with no preceding tool use for the same call id.
        let session = assistant_blocks(vec![tool_result("orphan-1", "Read", "file body")]);

        // WHEN the transcript segments are built.
        let segs = build_transcript_segments(&session);

        // THEN the Activity segment includes a done row labeled from the
        // result's tool name AND the row is not labeled only as a generic
        // done placeholder.
        let tools = activity_tools(&segs);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].status, ToolRowStatus::Done);
        assert_eq!(tools[0].summary, "Read");
        assert_ne!(tools[0].summary, "✓ done");
        assert_eq!(tools[0].output_lines, vec!["file body".to_string()]);
    }

    // ── Segment presentation ────────────────────────────────────────────

    /// @spec chat/transcript Segment presentation: Thinking collapsed label includes line count
    #[test]
    fn thinking_collapsed_label_includes_line_count() {
        // GIVEN a Thinking segment whose body has a known number of lines.
        let lines: Vec<String> = vec!["line one".into(), "line two".into(), "line three".into()];

        // WHEN the collapsed label for that segment is produced.
        let label = thinking_collapsed_label(&lines);

        // THEN the label includes that line count AND does not include a duration.
        assert!(
            label.contains('3'),
            "label should include line count 3: {label}"
        );
        assert!(
            label.contains("line"),
            "label should mention lines: {label}"
        );
        let lower = label.to_ascii_lowercase();
        for duration_token in ["ms", "sec", "min", "hour", "duration"] {
            assert!(
                !lower.contains(duration_token),
                "label must not include duration ({duration_token}): {label}"
            );
        }
        // "s" alone is too ambiguous (matches "lines"); require no time units.
        assert!(!lower.contains("seconds") && !lower.contains("minutes"));
    }

    /// @spec chat/transcript Segment presentation: Activity collapsed label includes count and sample names
    #[test]
    fn activity_collapsed_label_includes_count_and_sample_names() {
        // GIVEN an Activity segment with multiple completed tools.
        let session = assistant_blocks(vec![
            tool_use("1", "Read", r#"{"path":"a.rs"}"#),
            tool_result("1", "Read", "ok"),
            tool_use("2", "grep", r#"{"pattern":"foo"}"#),
            tool_result("2", "grep", "match"),
            tool_use("3", "shell", r#"{"command":"ls"}"#),
            tool_result("3", "shell", "file"),
            tool_use("4", "Write", r#"{"path":"b.rs"}"#),
            tool_result("4", "Write", "written"),
        ]);
        let segs = build_transcript_segments(&session);
        let tools = activity_tools(&segs);
        assert_eq!(tools.len(), 4);

        // WHEN the collapsed label for that segment is produced.
        let label = activity_collapsed_label(tools);

        // THEN the label includes the tool count AND sample tool names.
        assert!(
            label.contains('4') && label.contains("tool"),
            "label should include tool count: {label}"
        );
        assert!(
            label.contains("Read") && label.contains("Grep") && label.contains("Shell"),
            "label should include sample humanized tool names: {label}"
        );
    }

    /// @spec chat/transcript Segment presentation: Expanded activity exposes status, summary, and truncated output
    #[test]
    fn expanded_activity_exposes_status_summary_and_truncated_output() {
        // GIVEN an expanded Activity segment with a completed tool that
        // produced multi-line output.
        let multi_line: String = (0..15).map(|i| format!("out line {i}\n")).collect();
        let session = assistant_blocks(vec![
            tool_use("1", "Read", r#"{"path":"big.txt"}"#),
            tool_result("1", "Read", &multi_line),
            tool_use("2", "grep", r#"{"pattern":"x"}"#),
            tool_result("2", "grep", "hit"),
        ]);
        let segs = build_transcript_segments(&session);
        let tools = activity_tools(&segs);

        // WHEN the segment's rows are presented.
        let rows = expanded_activity_rows(tools);

        // THEN each tool has one row showing its status and summary
        // AND truncated output is available under the row for that tool
        // AND no separate per-tool expand state is required to show it.
        assert_eq!(rows.len(), tools.len());
        for (row, tool) in rows.iter().zip(tools.iter()) {
            assert_eq!(row.status, tool.status);
            assert_eq!(row.summary, tool.summary);
            assert!(!row.status_glyph.is_empty());
            // Output is on the row itself (no nested expand flag to consult).
            assert_eq!(row.output_lines, tool.output_lines);
        }
        let first = &rows[0];
        assert_eq!(first.status, ToolRowStatus::Done);
        assert!(first.summary.contains("Read"));
        assert!(
            first.output_lines.len() > 1,
            "multi-line result should surface truncated output under the row"
        );
        assert!(
            first.output_lines.len() <= 11,
            "output should be truncated (max 10 lines + ellipsis)"
        );
    }

    // ── Segment → editor blocks ─────────────────────────────────────────

    fn answer_block(text: &str) -> Block {
        Block {
            kind: BlockKind::Assistant,
            label: "Answer".into(),
            lines: if text.is_empty() {
                vec![]
            } else {
                text.lines().map(String::from).collect()
            },
            is_priming: false,
            is_live: false,
        }
    }

    fn user_block(text: &str) -> Block {
        Block {
            kind: BlockKind::User,
            label: "User".into(),
            lines: vec![text.into()],
            is_priming: false,
            is_live: false,
        }
    }

    // ── Viewer style / Classic Answer path ──────────────────────────────

    /// @spec chat/viewer-style Classic Answer identity path: Effective classic presents Answer as one full body
    #[test]
    fn effective_classic_presents_answer_as_one_full_body() {
        // GIVEN the effective viewer style is classic
        // AND a settled Answer with body text
        let style = ViewerStyle::Classic;
        let block = answer_block("settled reply body");
        assert_eq!(block.kind, BlockKind::Assistant);
        assert!(!block.lines.is_empty());

        // WHEN the Answer is presented
        let source = block.lines.join("\n");
        let mode = answer_body_presentation(style, false, &source);

        // THEN the Answer uses the full-body classic presentation
        assert_eq!(mode, AnswerBodyPresentation::ClassicFullBody);
        assert!(block_presentation_uses_viewer_style(block.kind));
        // Identity: Classic editor content is the full Answer body, not Focus slices.
        assert_eq!(
            answer_editor_desired_lines(&block, style),
            block.lines.clone()
        );
    }

    /// @spec chat/viewer-style Classic Answer identity path: Non-Answer segments ignore viewer style for presentation mode
    #[test]
    fn non_answer_segments_ignore_viewer_style_for_presentation_mode() {
        // GIVEN the effective viewer style is classic or focus
        // AND a transcript that includes User, Thinking, or Activity segments
        let styles = [ViewerStyle::Classic, ViewerStyle::Focus];
        let non_answer = [
            BlockKind::User,
            BlockKind::System,
            BlockKind::Reasoning,
            BlockKind::Activity,
            BlockKind::ToolUse,
            BlockKind::ToolResult,
        ];

        for style in styles {
            // WHEN those non-Answer segments are presented
            // THEN their presentation mode is not selected by the viewer style
            for kind in non_answer {
                assert!(
                    !block_presentation_uses_viewer_style(kind),
                    "{kind:?} must ignore viewer style (effective {style:?})"
                );
            }
            // Answer still selects presentation via viewer style (for contrast).
            assert!(block_presentation_uses_viewer_style(BlockKind::Assistant));
            // Plain prose without trailing next stays Classic under Focus (passthrough).
            assert_eq!(
                answer_body_presentation(style, false, "hello"),
                AnswerBodyPresentation::ClassicFullBody
            );
        }
    }

    fn sample_focus_answer_source() -> &'static str {
        "\
## Motivation

Why.

## Intent

What.

> **write**
>
> path

# Preview

> **next**
>
> `confirm`
"
    }

    /// @spec chat/focus-answer When Focus applies: Live Answer under focus style uses classic presentation
    #[test]
    fn live_answer_under_focus_style_uses_classic_presentation() {
        // GIVEN the effective viewer style is focus
        // AND a live Answer segment
        let src = sample_focus_answer_source();
        // WHEN the Answer is presented
        let mode = answer_body_presentation(ViewerStyle::Focus, true, src);
        // THEN the Answer uses classic presentation
        assert_eq!(mode, AnswerBodyPresentation::ClassicFullBody);
    }

    /// @spec chat/focus-answer When Focus applies: Settled Answer without trailing next uses classic passthrough
    #[test]
    fn settled_answer_without_trailing_next_uses_classic_passthrough() {
        // GIVEN the effective viewer style is focus
        // AND a settled Answer with no trailing `next` meta card
        let src = "## Only\n\nbody without gate.\n";
        // WHEN the Answer is presented
        let mode = answer_body_presentation(ViewerStyle::Focus, false, src);
        // THEN the Answer uses classic presentation
        assert_eq!(mode, AnswerBodyPresentation::ClassicFullBody);
    }

    /// @spec chat/focus-answer When Focus applies: Settled Answer with trailing next uses Focus layout
    #[test]
    fn settled_answer_with_trailing_next_uses_focus_layout() {
        // GIVEN the effective viewer style is focus
        // AND a settled Answer that ends with a trailing `next` meta card
        let src = sample_focus_answer_source();
        // WHEN the Answer is presented
        let mode = answer_body_presentation(ViewerStyle::Focus, false, src);
        // THEN the Answer uses Focus layout
        assert!(matches!(mode, AnswerBodyPresentation::FocusSectioned { .. }));
    }

    /// @spec chat/focus-answer Fold defaults and toggles: First sight collapses foldable sections and shows open region
    #[test]
    fn first_sight_collapses_foldable_sections_and_shows_open_region() {
        use crate::focus_answer::{
            focus_layout, range_line_count, section_key, sync_section_folds, FocusLayout,
            SectionFoldState,
        };
        use std::collections::HashMap;

        // GIVEN a settled Answer under Focus layout with at least one foldable section and an
        // open region
        // AND no user fold overrides for that Answer
        let src = sample_focus_answer_source();
        let FocusLayout::Sectioned { sections, open } = focus_layout(src) else {
            panic!("expected sectioned");
        };
        assert!(!sections.is_empty());
        let mut folds = HashMap::new();
        // WHEN the Answer is first presented under Focus
        sync_section_folds(&mut folds, &sections);
        // THEN every foldable section is collapsed
        // AND the open region is shown
        for s in &sections {
            let st = folds.get(&section_key(&s.kind)).expect("fold state");
            assert!(st.collapsed);
            assert!(!st.user_set);
        }
        assert!(range_line_count(open) > 0);
        let _ = SectionFoldState {
            collapsed: true,
            user_set: false,
        };
    }

    /// @spec chat/focus-answer Fold defaults and toggles: User expand survives rematerialize for the same section key
    #[test]
    fn user_expand_survives_rematerialize_for_the_same_section_key() {
        use crate::focus_answer::{
            focus_layout, section_key, sync_section_folds, toggle_section_fold, FocusLayout,
        };
        use std::collections::HashMap;

        // GIVEN a Focus layout Answer with a foldable section the user has expanded
        let src = sample_focus_answer_source();
        let FocusLayout::Sectioned { sections, .. } = focus_layout(src) else {
            panic!("expected sectioned");
        };
        let mut folds = HashMap::new();
        sync_section_folds(&mut folds, &sections);
        let key = section_key(&sections[0].kind);
        toggle_section_fold(&mut folds, &key);
        assert!(!folds[&key].collapsed);
        assert!(folds[&key].user_set);

        // WHEN the Answer is rematerialized with that section key still present
        sync_section_folds(&mut folds, &sections);

        // THEN that section remains expanded
        assert!(!folds[&key].collapsed);
        assert!(folds[&key].user_set);
    }

    /// @spec chat/focus-answer Fold defaults and toggles: Leaving Focus clears section fold state
    #[test]
    fn leaving_focus_clears_section_fold_state() {
        use std::collections::HashMap;

        // GIVEN Focus section fold state for one or more Answers
        let mut folds: HashMap<usize, HashMap<String, crate::focus_answer::SectionFoldState>> =
            HashMap::new();
        folds.insert(
            0,
            HashMap::from([(
                "2:Motivation".into(),
                crate::focus_answer::SectionFoldState {
                    collapsed: false,
                    user_set: true,
                },
            )]),
        );
        // WHEN the effective viewer style is no longer focus
        let style = ViewerStyle::Classic;
        if style != ViewerStyle::Focus {
            folds.clear();
        }
        // THEN all Focus section fold state is cleared
        assert!(folds.is_empty());
    }

    /// @spec chat/focus-answer Focus slice presentation (Hybrid C): Open region uses classic Answer body paint
    #[test]
    fn open_region_uses_classic_answer_body_paint() {
        // GIVEN a Focus layout Answer with an open region
        let src = sample_focus_answer_source();
        let mode = answer_body_presentation(ViewerStyle::Focus, false, src);
        assert!(matches!(mode, AnswerBodyPresentation::FocusSectioned { .. }));
        // WHEN the open region is presented
        // THEN the open region uses classic Answer body paint
        assert!(focus_slice_uses_classic_body_paint(FocusSliceKind::OpenRegion));
        // Editor content for Focus is open-region lines only (TextEdit recipe surface).
        let block = answer_block(src);
        let desired = answer_editor_desired_lines(&block, ViewerStyle::Focus);
        assert!(
            !desired.is_empty(),
            "open-region editor should hold open-region lines"
        );
        assert!(
            desired.iter().any(|l| l.contains("**next**")),
            "open-region editor should include trailing next: {desired:?}"
        );
        assert!(
            desired.iter().all(|l| !l.starts_with("## ")),
            "open-region editor should not include foldable H2 sections: {desired:?}"
        );
    }

    /// @spec chat/focus-answer Focus slice presentation (Hybrid C): Expanded foldable section uses plain content presentation
    #[test]
    fn expanded_foldable_section_uses_plain_content_presentation() {
        // GIVEN a Focus layout Answer with an expanded foldable section
        let src = sample_focus_answer_source();
        assert!(matches!(
            answer_body_presentation(ViewerStyle::Focus, false, src),
            AnswerBodyPresentation::FocusSectioned { .. }
        ));
        // WHEN that section body is presented
        // THEN the section body uses plain content-font source presentation
        // AND the section body does not use classic Answer body paint
        assert!(!focus_slice_uses_classic_body_paint(
            FocusSliceKind::ExpandedSection
        ));
        assert!(!focus_slice_uses_last_answer_band(
            FocusSliceKind::ExpandedSection,
            true
        ));
    }

    /// @spec chat/focus-answer Focus slice presentation (Hybrid C): Collapsed preamble label uses line count form
    #[test]
    fn collapsed_preamble_label_uses_line_count_form() {
        use crate::focus_answer::{section_collapsed_label, SectionKind};

        // GIVEN a Focus layout Answer with a collapsed preamble section of known line count
        let kind = SectionKind::Preamble;
        let line_count = 4;
        // WHEN the collapsed preamble header is presented
        let label = section_collapsed_label(&kind, line_count);
        // THEN the label includes that line count
        assert!(
            label.contains("4"),
            "preamble label should include line count: {label}"
        );
        assert!(
            label.to_lowercase().contains("preamble"),
            "label: {label}"
        );
    }

    /// @spec chat/focus-answer Last-answer band under Focus: Last Focus Answer bands only the open region
    #[test]
    fn last_focus_answer_bands_only_the_open_region() {
        // GIVEN a Focus layout Answer that is the last-answer band target
        // AND that Answer has an open region
        assert!(matches!(
            answer_body_presentation(ViewerStyle::Focus, false, sample_focus_answer_source()),
            AnswerBodyPresentation::FocusSectioned { .. }
        ));
        // WHEN the Answer is presented
        // THEN the open region has last-answer band styling
        assert!(focus_slice_uses_last_answer_band(
            FocusSliceKind::OpenRegion,
            true
        ));
        // Non-last Focus Answer: no band on open region either.
        assert!(!focus_slice_uses_last_answer_band(
            FocusSliceKind::OpenRegion,
            false
        ));
    }

    /// @spec chat/focus-answer Last-answer band under Focus: Expanded section of last Focus Answer is not banded
    #[test]
    fn expanded_section_of_last_focus_answer_is_not_banded() {
        // GIVEN a Focus layout Answer that is the last-answer band target
        // AND an expanded foldable section on that Answer
        assert!(matches!(
            answer_body_presentation(ViewerStyle::Focus, false, sample_focus_answer_source()),
            AnswerBodyPresentation::FocusSectioned { .. }
        ));
        // WHEN that section body is presented
        // THEN the section body does not have last-answer band styling
        assert!(!focus_slice_uses_last_answer_band(
            FocusSliceKind::ExpandedSection,
            true
        ));
    }

    /// Smoke: switching stored style does not rewrite session messages.
    #[test]
    fn switching_viewer_style_does_not_rewrite_session_messages() {
        let mut session = ChatSession::new("smoke".into());
        session.messages.push(crate::chat_store::ChatMessage {
            role: Role::User,
            content: vec![ContentBlock::Text("hello".into())],
            timestamp: String::new(),
            is_priming: false,
        });
        session.messages.push(crate::chat_store::ChatMessage {
            role: Role::Assistant,
            content: vec![ContentBlock::Text("world".into())],
            timestamp: String::new(),
            is_priming: false,
        });
        let before = format!("{:?}", session.messages);

        // Style lives on config; presentation helpers must not mutate the session.
        let _classic = answer_body_presentation(ViewerStyle::Classic, false, "world");
        let _focus = answer_body_presentation(ViewerStyle::Focus, false, "world");
        let _doc = answer_body_presentation(ViewerStyle::Document, false, "world");

        assert_eq!(format!("{:?}", session.messages), before);
        assert_eq!(session.messages.len(), 2);
    }

    // ── Last Answer band target ─────────────────────────────────────────

    /// @spec chat/answer-landmarks Last Answer contrast band: Sole latest non-empty Answer is the band target
    #[test]
    fn sole_latest_non_empty_answer_is_the_band_target() {
        // GIVEN a transcript with more than one Answer segment that has non-empty body text
        let blocks = vec![
            user_block("q1"),
            answer_block("first reply"),
            user_block("q2"),
            answer_block("second reply"),
        ];

        // WHEN the last-Answer band target is resolved
        let target = last_answer_band_target(&blocks);

        // THEN only the latest non-empty Answer is the band target
        // AND every earlier Answer is not a band target
        assert_eq!(target, Some(3));
        assert_ne!(target, Some(1));
    }

    /// @spec chat/answer-landmarks Last Answer contrast band: Empty latest Answer is not a band target
    #[test]
    fn empty_latest_answer_is_not_a_band_target() {
        // GIVEN a transcript whose latest Answer segment has empty body text
        // AND an earlier Answer segment has non-empty body text
        let blocks = vec![
            answer_block("settled reply"),
            answer_block(""), // empty latest
        ];

        // WHEN the last-Answer band target is resolved
        let target = last_answer_band_target(&blocks);

        // THEN the empty latest Answer is not the band target
        // AND the latest non-empty Answer is the band target
        assert_eq!(target, Some(0));
        assert_ne!(target, Some(1));
    }

    fn thinking_block(text: &str) -> Block {
        Block {
            kind: BlockKind::Reasoning,
            label: "Thinking".into(),
            lines: vec![text.into()],
            is_priming: false,
            is_live: false,
        }
    }

    fn activity_block() -> Block {
        Block {
            kind: BlockKind::Activity,
            label: "1 tool".into(),
            lines: vec!["· Read a.rs".into()],
            is_priming: false,
            is_live: false,
        }
    }

    // ── Answer reply anchors ────────────────────────────────────────────

    /// @spec chat/answer-landmarks Answer reply anchors: Only Answer blocks are reply anchors
    #[test]
    fn only_answer_blocks_are_reply_anchors() {
        // GIVEN a transcript that mixes Answer segments with Thinking, Activity, or User
        let blocks = vec![
            user_block("q"),
            thinking_block("why"),
            activity_block(),
            answer_block("a1"),
            thinking_block("more"),
            answer_block("a2"),
        ];

        // WHEN the reply-anchor list is built
        let anchors = answer_block_indices(&blocks);

        // THEN the anchors are exactly the Answer segments in transcript order
        // AND no Thinking, Activity, or User segment is an anchor
        assert_eq!(anchors, vec![3, 5]);
    }

    /// @spec chat/answer-landmarks Answer reply anchors: Prev and next step to adjacent Answer anchors
    #[test]
    fn prev_and_next_step_to_adjacent_answer_anchors() {
        // GIVEN a transcript with at least three Answer anchors
        // AND the current Answer is the middle of those three
        let blocks = vec![
            answer_block("a0"),
            thinking_block("t"),
            answer_block("a1"),
            answer_block("a2"),
        ];
        let anchors = answer_block_indices(&blocks);
        assert_eq!(anchors, vec![0, 2, 3]);
        let current = Some(2); // middle Answer block index

        // WHEN previous and next reply targets are resolved
        let prev = prev_answer_idx(&anchors, current);
        let next = next_answer_idx(&anchors, current);

        // THEN previous is the Answer immediately before the current one
        // AND next is the Answer immediately after the current one
        assert_eq!(prev, Some(0));
        assert_eq!(next, Some(3));
    }

    /// @spec chat/answer-landmarks Answer reply anchors: Prev at first and next at last yield no target
    #[test]
    fn prev_at_first_and_next_at_last_yield_no_target() {
        // GIVEN a transcript with at least one Answer anchor
        let blocks = vec![answer_block("only"), answer_block("last")];
        let anchors = answer_block_indices(&blocks);
        let first = anchors.first().copied();
        let last = anchors.last().copied();

        // WHEN previous is resolved from the first Answer and next is resolved from the last Answer
        let prev = prev_answer_idx(&anchors, first);
        let next = next_answer_idx(&anchors, last);

        // THEN there is no previous target
        // AND there is no next target
        assert_eq!(prev, None);
        assert_eq!(next, None);
    }

    // ── Viewport current for reply jumps ────────────────────────────────

    /// @spec chat/answer-landmarks Viewport current for reply jumps: Stick-to-bottom treats the last Answer as current
    #[test]
    fn stick_to_bottom_treats_the_last_answer_as_current() {
        // GIVEN a transcript with more than one Answer anchor
        // AND the chat is stuck to the bottom
        let anchors = vec![0, 2, 4];
        let tops = [(0, 0.0), (2, 100.0), (4, 200.0)];

        // WHEN the current Answer for reply jumps is resolved
        let current = current_answer_for_reply_jumps(&anchors, &tops, 0.0, true);

        // THEN the current Answer is the last Answer anchor
        assert_eq!(current, Some(4));
    }

    /// @spec chat/answer-landmarks Viewport current for reply jumps: Scroll offset selects the Answer at or above the viewport top
    #[test]
    fn scroll_offset_selects_the_answer_at_or_above_the_viewport_top() {
        // GIVEN a transcript with more than one Answer anchor with known tops
        // AND the chat is not stuck to the bottom
        // AND the viewport top lies at or below one Answer top and above the next
        let anchors = vec![0, 2, 4];
        let tops = [(0, 0.0), (2, 100.0), (4, 200.0)];
        let offset_y = 150.0; // past Answer 2's top, before Answer 4's top

        // WHEN the current Answer for reply jumps is resolved
        let current = current_answer_for_reply_jumps(&anchors, &tops, offset_y, false);

        // THEN the current Answer is the last Answer whose top is at or above the viewport top
        assert_eq!(current, Some(2));
    }

    // ── Previous reply re-align ─────────────────────────────────────────

    /// @spec chat/answer-landmarks Previous reply re-align: Viewport below current top targets current Answer
    #[test]
    fn viewport_below_current_top_targets_current_answer() {
        // GIVEN a transcript with more than one Answer anchor with known tops
        // AND a resolved current Answer
        // AND the viewport top is strictly below that Answer's top
        let anchors = vec![0, 2, 4];
        let tops = [(0, 0.0), (2, 100.0), (4, 200.0)];
        let current = Some(2);
        let offset_y = 150.0; // below Answer 2's top (100)

        // WHEN the previous reply target is resolved
        let target = target_answer_for_reply_jump(&anchors, &tops, current, true, offset_y);

        // THEN the target is the current Answer
        assert_eq!(target, Some(2));
    }

    /// @spec chat/answer-landmarks Previous reply re-align: At current top previous targets prior Answer
    #[test]
    fn at_current_top_previous_targets_prior_answer() {
        // GIVEN a transcript with more than one Answer anchor with known tops
        // AND a resolved current Answer that is not the first
        // AND the viewport top is at that Answer's top
        let anchors = vec![0, 2, 4];
        let tops = [(0, 0.0), (2, 100.0), (4, 200.0)];
        let current = Some(2);
        let offset_y = 100.0; // at Answer 2's top

        // WHEN the previous reply target is resolved
        let target = target_answer_for_reply_jump(&anchors, &tops, current, true, offset_y);

        // THEN the target is the Answer immediately before the current one
        assert_eq!(target, Some(0));
    }

    /// @spec chat/answer-landmarks Previous reply re-align: Next ignores re-align when below current top
    #[test]
    fn next_ignores_re_align_when_below_current_top() {
        // GIVEN a transcript with more than one Answer anchor with known tops
        // AND a resolved current Answer that is not the last
        // AND the viewport top is strictly below that Answer's top
        let anchors = vec![0, 2, 4];
        let tops = [(0, 0.0), (2, 100.0), (4, 200.0)];
        let current = Some(2);
        let offset_y = 150.0; // below Answer 2's top

        // WHEN the next reply target is resolved
        let target = target_answer_for_reply_jump(&anchors, &tops, current, false, offset_y);

        // THEN the target is the Answer immediately after the current one
        assert_eq!(target, Some(4));
    }

    #[test]
    fn blocks_from_segments_maps_calm_transcript_not_adjacency() {
        // Reasoning + tools + orphan-style pairing + answer become
        // Thinking / Activity / Answer — not one card per tool or "✓ done".
        let session = assistant_blocks(vec![
            ContentBlock::Reasoning("why".into()),
            tool_use("1", "Read", r#"{"path":"a.rs"}"#),
            tool_result("1", "Read", "body"),
            tool_result("orphan", "grep", "hit"),
            ContentBlock::Text("answer".into()),
        ]);
        let blocks = blocks_from_segments(&build_transcript_segments(&session));
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[0].kind, BlockKind::Reasoning);
        assert_eq!(blocks[0].lines, vec!["why".to_string()]);
        assert_eq!(blocks[1].kind, BlockKind::Activity);
        assert!(blocks[1].label.contains("tool"));
        assert!(
            blocks[1].lines.iter().any(|l| l.contains("grep")),
            "orphan result should be a named activity row, not a bare done block: {:?}",
            blocks[1]
        );
        assert!(!blocks.iter().any(|b| b.label == "✓ done"));
        assert_eq!(blocks[2].kind, BlockKind::Assistant);
        assert_eq!(blocks[2].lines, vec!["answer".to_string()]);
    }

    // ── Collapse defaults ───────────────────────────────────────────────

    /// @spec chat/transcript Collapse defaults: Thinking collapses when answer follows
    #[test]
    fn thinking_collapses_when_answer_follows() {
        // GIVEN a live Thinking segment that the user has not toggled.
        let mut session = ChatSession::new("test".into());
        session.is_streaming = true;
        session.pending_reasoning = "still thinking".into();
        let segs_live = build_transcript_segments(&session);
        let mut states = Vec::new();
        sync_collapse_states(&mut states, &segs_live);
        assert_eq!(states.len(), 1);
        assert!(!states[0].collapsed, "live Thinking should start expanded");
        assert!(!states[0].user_set);

        // Intermediate: reasoning committed, still streaming, NO answer yet.
        // Collapse must not fire merely because reasoning stopped receiving
        // deltas — only when Answer follows (or the turn settles).
        session.pending_reasoning.clear();
        session.messages.push(crate::chat_store::ChatMessage {
            role: Role::Assistant,
            content: vec![ContentBlock::Reasoning("still thinking".into())],
            timestamp: String::new(),
            is_priming: false,
        });
        let segs_committed = build_transcript_segments(&session);
        assert_eq!(segs_committed.len(), 1);
        assert!(
            matches!(
                &segs_committed[0],
                TranscriptSeg::Thinking { live: true, .. }
            ),
            "committed Thinking mid-stream with no Answer should stay live: {segs_committed:?}"
        );
        sync_collapse_states(&mut states, &segs_committed);
        assert!(
            !states[0].collapsed,
            "committing Reasoning alone must not auto-collapse Thinking"
        );

        // WHEN a following Answer segment appears for the same turn.
        session.pending_text = "here is the answer".into();
        let segs = build_transcript_segments(&session);
        assert!(
            matches!(&segs[0], TranscriptSeg::Thinking { live: false, .. })
                && matches!(&segs[1], TranscriptSeg::Answer { .. }),
            "expected settled Thinking then Answer, got {segs:?}"
        );
        sync_collapse_states(&mut states, &segs);

        // THEN the Thinking segment is collapsed.
        assert!(
            states[0].collapsed,
            "Thinking should auto-collapse when Answer follows"
        );
        assert!(!states[0].user_set);
    }

    /// @spec chat/transcript Collapse defaults: User-expanded Thinking is not auto-collapsed
    #[test]
    fn user_expanded_thinking_is_not_auto_collapsed() {
        // GIVEN a Thinking segment the user has expanded.
        let mut session = ChatSession::new("test".into());
        session.is_streaming = true;
        session.pending_reasoning = "draft thought".into();
        let segs_live = build_transcript_segments(&session);
        let mut states = Vec::new();
        sync_collapse_states(&mut states, &segs_live);
        // Simulate user expanding (or re-expanding) and locking the choice.
        toggle_collapse(&mut states, 0);
        // If first-sight was expanded, toggle collapses; expand again to match
        // "user has expanded".
        if states[0].collapsed {
            toggle_collapse(&mut states, 0);
        }
        assert!(!states[0].collapsed);
        assert!(states[0].user_set);

        // WHEN a following Answer segment appears for the same turn.
        session.pending_reasoning.clear();
        session.messages.push(crate::chat_store::ChatMessage {
            role: Role::Assistant,
            content: vec![ContentBlock::Reasoning("draft thought".into())],
            timestamp: String::new(),
            is_priming: false,
        });
        session.pending_text = "answer body".into();
        let segs = build_transcript_segments(&session);
        sync_collapse_states(&mut states, &segs);

        // THEN the Thinking segment remains expanded.
        assert!(
            !states[0].collapsed,
            "user-expanded Thinking must not auto-collapse"
        );
        assert!(states[0].user_set);
    }

    /// @spec chat/transcript Collapse defaults: Settled Activity starts collapsed
    #[test]
    fn settled_activity_starts_collapsed() {
        // GIVEN a finished turn whose transcript includes an Activity segment.
        let session = assistant_blocks(vec![
            tool_use("1", "Read", r#"{"path":"a.rs"}"#),
            tool_result("1", "Read", "ok"),
            tool_use("2", "grep", r#"{"pattern":"x"}"#),
            tool_result("2", "grep", "hit"),
            ContentBlock::Text("done".into()),
        ]);
        assert!(!session.is_streaming);
        let segs = build_transcript_segments(&session);
        let activity_idx = segs
            .iter()
            .position(|s| matches!(s, TranscriptSeg::Activity { .. }))
            .expect("expected Activity segment");

        // WHEN the transcript is presented for that settled turn.
        let mut states = Vec::new();
        sync_collapse_states(&mut states, &segs);

        // THEN the Activity segment is collapsed.
        assert!(
            states[activity_idx].collapsed,
            "settled Activity should start collapsed"
        );
        assert!(!states[activity_idx].user_set);
    }

    #[test]
    fn thinking_stays_expanded_during_live_activity() {
        // GIVEN committed Reasoning + live Activity, streaming, no Answer yet.
        let mut session = assistant_blocks(vec![
            ContentBlock::Reasoning("plan the approach".into()),
            tool_use("1", "Read", r#"{"path":"a.rs"}"#),
            tool_result("1", "Read", "ok"),
            tool_use("2", "grep", r#"{"pattern":"x"}"#),
        ]);
        session.is_streaming = true;

        let segs = build_transcript_segments(&session);
        assert!(
            matches!(&segs[0], TranscriptSeg::Thinking { live: true, .. }),
            "Thinking should stay open-in-turn during tools: {segs:?}"
        );
        assert!(
            matches!(&segs[1], TranscriptSeg::Activity { live: true, .. }),
            "Activity should be live during tools: {segs:?}"
        );

        let mut states = Vec::new();
        sync_collapse_states(&mut states, &segs);

        // THEN Thinking stays expanded unless user-set.
        assert!(
            !states[0].collapsed,
            "Thinking must stay expanded while following Activity is live"
        );
        assert!(!states[0].user_set);
        assert!(!states[1].collapsed, "live Activity should start expanded");
    }

    #[test]
    fn think_tools_answer_settles_thinking_collapsed() {
        // GIVEN a think → tools stream that the user has not toggled.
        let mut session = ChatSession::new("test".into());
        session.is_streaming = true;
        session.pending_reasoning = "first thought".into();
        let mut states = Vec::new();
        sync_collapse_states(&mut states, &build_transcript_segments(&session));
        assert!(!states[0].collapsed);

        // Tools flush reasoning; still no answer.
        session.pending_reasoning.clear();
        session.messages.push(crate::chat_store::ChatMessage {
            role: Role::Assistant,
            content: vec![
                ContentBlock::Reasoning("first thought".into()),
                tool_use("1", "Read", r#"{"path":"a.rs"}"#),
                tool_result("1", "Read", "body"),
            ],
            timestamp: String::new(),
            is_priming: false,
        });
        let segs_tools = build_transcript_segments(&session);
        sync_collapse_states(&mut states, &segs_tools);
        assert!(
            !states[0].collapsed,
            "Thinking should stay open through the tool phase"
        );

        // WHEN answer arrives (or the turn would complete after).
        session.pending_text = "final answer".into();
        let segs_answer = build_transcript_segments(&session);
        assert!(
            matches!(&segs_answer[0], TranscriptSeg::Thinking { live: false, .. })
                && matches!(&segs_answer[1], TranscriptSeg::Activity { .. })
                && matches!(&segs_answer[2], TranscriptSeg::Answer { .. }),
            "expected Thinking, Activity, Answer: {segs_answer:?}"
        );
        sync_collapse_states(&mut states, &segs_answer);
        assert!(
            states[0].collapsed,
            "Thinking should collapse when Answer follows"
        );

        // TurnComplete / settled: still collapsed, Activity settles too.
        session.pending_text.clear();
        session.messages[0]
            .content
            .push(ContentBlock::Text("final answer".into()));
        session.is_streaming = false;
        let segs_settled = build_transcript_segments(&session);
        sync_collapse_states(&mut states, &segs_settled);
        assert!(states[0].collapsed, "settled Thinking stays collapsed");
        assert!(states[1].collapsed, "settled Activity should be collapsed");
        assert!(!states[0].user_set);
    }

    /// @spec chat/transcript Collapse defaults: Priming Setup starts collapsed
    #[test]
    fn priming_setup_starts_collapsed() {
        // GIVEN a session whose first user message is the synthetic priming inject.
        let mut session = ChatSession::new("test".into());
        session.messages.push(crate::chat_store::ChatMessage {
            role: Role::User,
            content: vec![ContentBlock::Text(
                "Project conventions from AGENTS.md…\n\nDo not respond — reply with a single dot (.)"
                    .into(),
            )],
            timestamp: String::new(),
            is_priming: true,
        });
        session.messages.push(crate::chat_store::ChatMessage {
            role: Role::Assistant,
            content: vec![ContentBlock::Text(".".into())],
            timestamp: String::new(),
            is_priming: false,
        });
        session.messages.push(crate::chat_store::ChatMessage {
            role: Role::User,
            content: vec![ContentBlock::Text("real first message".into())],
            timestamp: String::new(),
            is_priming: false,
        });

        let segs = build_transcript_segments(&session);
        assert!(
            matches!(
                &segs[0],
                TranscriptSeg::User {
                    is_priming: true,
                    ..
                }
            ),
            "first segment should be priming user: {segs:?}"
        );
        assert!(
            matches!(
                &segs[2],
                TranscriptSeg::User {
                    is_priming: false,
                    ..
                }
            ),
            "real user message is not priming: {segs:?}"
        );

        // WHEN collapse state is synced for that transcript.
        let mut states = Vec::new();
        sync_collapse_states(&mut states, &segs);

        // THEN the priming Setup block is collapsed and the real user is not.
        assert!(
            states[0].collapsed,
            "priming Setup should start collapsed"
        );
        assert!(!states[0].user_set);
        assert!(
            !states[2].collapsed,
            "real user message must stay expanded"
        );

        let blocks = blocks_from_segments(&segs);
        assert!(blocks[0].is_priming);
        assert_eq!(blocks[0].label, "Setup");
        assert!(!blocks[2].is_priming);
    }

    /// @spec chat/transcript Collapse defaults: User-expanded priming is not force-collapsed by sync
    #[test]
    fn user_expanded_priming_is_not_force_collapsed_by_sync() {
        // GIVEN a priming User segment the user has expanded.
        let mut session = ChatSession::new("test".into());
        session.messages.push(crate::chat_store::ChatMessage {
            role: Role::User,
            content: vec![ContentBlock::Text("priming body".into())],
            timestamp: String::new(),
            is_priming: true,
        });
        let segs = build_transcript_segments(&session);
        let mut states = Vec::new();
        sync_collapse_states(&mut states, &segs);
        assert!(states[0].collapsed);
        toggle_collapse(&mut states, 0);
        assert!(!states[0].collapsed);
        assert!(states[0].user_set);

        // WHEN collapse state is synced again without a timed re-collapse.
        sync_collapse_states(&mut states, &segs);

        // THEN the priming User segment remains expanded.
        assert!(
            !states[0].collapsed,
            "user-expanded priming must not be force-collapsed by sync"
        );
    }

    /// @spec chat/transcript Collapse defaults: Timed re-collapse forces priming collapsed
    #[test]
    fn timed_recollapse_forces_priming_collapsed() {
        // GIVEN a priming User segment that is currently expanded.
        let mut session = ChatSession::new("test".into());
        session.messages.push(crate::chat_store::ChatMessage {
            role: Role::User,
            content: vec![ContentBlock::Text("priming body".into())],
            timestamp: String::new(),
            is_priming: true,
        });
        let segs = build_transcript_segments(&session);
        let mut states = Vec::new();
        sync_collapse_states(&mut states, &segs);
        toggle_collapse(&mut states, 0);
        assert!(!states[0].collapsed);

        // WHEN the priming re-collapse path runs for that segment.
        recollapse_priming(&mut states, 0);

        // THEN the priming User segment is collapsed.
        assert!(states[0].collapsed);
    }

    /// @spec chat/transcript Segment presentation: Priming collapsed label uses Setup and line count
    #[test]
    fn priming_collapsed_label_uses_setup_and_line_count() {
        // GIVEN a priming User segment whose body has a known number of lines.
        let lines = vec![
            "Project conventions…".to_string(),
            String::new(),
            "reply with a single dot (.)".to_string(),
        ];

        // WHEN the collapsed label for that segment is produced.
        let label = priming_collapsed_label(&lines);

        // THEN the label includes Setup and that line count.
        assert!(
            label.contains("Setup"),
            "collapsed priming label should name Setup: {label}"
        );
        assert!(
            label.contains("3 lines"),
            "collapsed priming label should include line count: {label}"
        );
    }
}
