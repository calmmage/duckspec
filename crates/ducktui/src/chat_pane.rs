//! Terminal chat pane: segment collapse, autoscroll, numbered hints, composer.
//!
//! Segment *construction* semantics match `chat/transcript`; this module owns
//! terminal presentation and input bindings only.

use std::path::Path;

use duckcore::chat_store::{ChatMessage, ChatSession, ContentBlock, Role};
use duckcore::fast_response::{self, FastResponse};
use duckcore::meta_card::{NextAction, trailing_next_actions};
use duckcore::session_sharing::{self, DriveRole, PersistOutcome};
use duckcore::transcript::{
    self, TranscriptSeg, activity_collapsed_label, build_transcript_segments, thinking_collapsed_label,
};

use crate::md_render::{self, StyledLine};

/// Settled / live presentation segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SegKind {
    Thinking,
    Activity,
    Answer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentSeg {
    pub kind: SegKind,
    pub summary: String,
    pub body_lines: Vec<String>,
    /// Thinking/Activity start collapsed when settled; Answer always expanded.
    pub expanded: bool,
    pub live: bool,
}

impl PresentSeg {
    /// Plain lines for viewport / scroll math (markdown-rendered at `width`).
    pub fn rendered_lines(&self, width: usize) -> Vec<String> {
        self.styled_lines(width)
            .into_iter()
            .map(|l| l.as_plain())
            .collect()
    }

    /// Styled lines for the draw path (markdown + meta-card chrome).
    pub fn styled_lines(&self, width: usize) -> Vec<StyledLine> {
        let width = width.max(1);
        match self.kind {
            SegKind::Answer => {
                let md = self.body_lines.join("\n");
                md_render::render(&md, width)
            }
            SegKind::Thinking | SegKind::Activity => {
                if self.expanded {
                    let mut lines = vec![StyledLine::styled(
                        format!("▼ {}", self.summary),
                        md_render::SpanStyle {
                            bold: true,
                            ..Default::default()
                        },
                    )];
                    let md = self.body_lines.join("\n");
                    lines.extend(md_render::render(&md, width));
                    lines
                } else {
                    vec![StyledLine::plain(format!("▶ {}", self.summary))]
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HintKind {
    NextToken,
    FastResponse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberedHint {
    pub index: u8,
    pub label: String,
    pub kind: HintKind,
    pub payload: String,
}

/// Viewport over rendered transcript lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Viewport {
    pub stick_to_bottom: bool,
    /// First visible line index when not sticking to bottom.
    pub offset: usize,
    pub height: usize,
    /// Content width for markdown wrap / table fit (updated from draw).
    pub width: usize,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            stick_to_bottom: true,
            offset: 0,
            height: 20,
            width: 72,
        }
    }
}

impl Viewport {
    pub fn visible_window<'a>(&self, lines: &'a [String]) -> &'a [String] {
        if lines.is_empty() {
            return lines;
        }
        let len = lines.len();
        let height = self.height.max(1);
        let start = if self.stick_to_bottom {
            len.saturating_sub(height)
        } else {
            self.offset.min(len.saturating_sub(1))
        };
        let end = (start + height).min(len);
        &lines[start..end]
    }

    pub fn shows_latest(&self, lines: &[String]) -> bool {
        if lines.is_empty() {
            return true;
        }
        let window = self.visible_window(lines);
        window.last() == lines.last()
    }
}

/// Result of a composer key or activation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatAction {
    Submitted(String),
    ActivatedNext(String),
    ActivatedFastResponse(String),
    OpenSlashPalette,
    None,
}

#[derive(Debug, Clone)]
pub struct ChatPane {
    pub segments: Vec<PresentSeg>,
    pub viewport: Viewport,
    pub composer: String,
    /// True when `/` at line start opened the slash catalog (not `//`).
    pub slash_palette_open: bool,
    pub next_hints: Vec<NumberedHint>,
    pub fast_hints: Vec<NumberedHint>,
    pub awaiting_user: bool,
    pub streaming: bool,
    pub drive_role: DriveRole,
    pub session: ChatSession,
    pub last_submit: Option<String>,
    pub last_activation: Option<String>,
    /// When true, a turn was handed to duckcore::agent (tests / host check).
    pub agent_drive_requested: bool,
    /// Live user-choice correlation id while awaiting (for `answer_user_choice`).
    pub pending_choice_correlation: Option<u64>,
}

impl Default for ChatPane {
    fn default() -> Self {
        Self::new("chat")
    }
}

impl ChatPane {
    pub fn new(scope: impl Into<String>) -> Self {
        Self {
            segments: Vec::new(),
            viewport: Viewport::default(),
            composer: String::new(),
            slash_palette_open: false,
            next_hints: Vec::new(),
            fast_hints: Vec::new(),
            awaiting_user: false,
            streaming: false,
            drive_role: DriveRole::Displayed,
            session: ChatSession::new(scope.into()),
            last_submit: None,
            last_activation: None,
            agent_drive_requested: false,
            pending_choice_correlation: None,
        }
    }

    /// Build presentation segments from simple settled pieces (tests / host).
    pub fn set_settled_segments(&mut self, segs: Vec<PresentSeg>) {
        self.segments = segs;
        for s in &mut self.segments {
            match s.kind {
                SegKind::Answer => s.expanded = true,
                SegKind::Thinking | SegKind::Activity if !s.live => s.expanded = false,
                _ => {}
            }
        }
        self.recompute_next_hints();
        self.pin_if_stick();
    }

    /// Rebuild presentation from a shared session via duckcore transcript model.
    pub fn load_from_session(&mut self, session: ChatSession) {
        self.streaming = session.is_streaming;
        let segs = build_transcript_segments(&session);
        let mut collapse = Vec::new();
        transcript::sync_collapse_states(&mut collapse, &segs);
        self.segments = segs
            .iter()
            .enumerate()
            .filter_map(|(i, seg)| present_from_transcript(seg, collapse.get(i).map(|c| c.collapsed)))
            .collect();
        self.session = session;
        self.recompute_next_hints();
        self.pin_if_stick();
    }

    pub fn expand_segment(&mut self, index: usize) {
        if let Some(s) = self.segments.get_mut(index)
            && matches!(s.kind, SegKind::Thinking | SegKind::Activity)
        {
            s.expanded = true;
        }
    }

    /// Expand the first collapsed Thinking/Activity, or collapse the last expanded one.
    pub fn toggle_next_collapsible(&mut self) {
        if let Some(i) = self.segments.iter().position(|s| {
            matches!(s.kind, SegKind::Thinking | SegKind::Activity) && !s.expanded && !s.live
        }) {
            if let Some(s) = self.segments.get_mut(i) {
                s.expanded = true;
            }
            return;
        }
        if let Some(i) = self.segments.iter().rposition(|s| {
            matches!(s.kind, SegKind::Thinking | SegKind::Activity) && s.expanded && !s.live
        }) && let Some(s) = self.segments.get_mut(i)
        {
            s.expanded = false;
        }
    }

    pub fn all_rendered_lines(&self) -> Vec<String> {
        let w = self.viewport.width.max(1);
        let mut out = Vec::new();
        for s in &self.segments {
            out.extend(s.rendered_lines(w));
        }
        out
    }

    pub fn all_styled_lines(&self) -> Vec<StyledLine> {
        let w = self.viewport.width.max(1);
        let mut out = Vec::new();
        for s in &self.segments {
            out.extend(s.styled_lines(w));
        }
        out
    }

    pub fn visible_lines(&self) -> Vec<String> {
        let lines = self.all_rendered_lines();
        self.viewport.visible_window(&lines).to_vec()
    }

    pub fn visible_styled_lines(&self) -> Vec<StyledLine> {
        let lines = self.all_styled_lines();
        if lines.is_empty() {
            return Vec::new();
        }
        let height = self.viewport.height.max(1);
        let start = if self.viewport.stick_to_bottom {
            lines.len().saturating_sub(height)
        } else {
            self.viewport.offset.min(lines.len().saturating_sub(1))
        };
        let end = (start + height).min(lines.len());
        lines[start..end].to_vec()
    }

    /// New stream content arrived; pin viewport when stick is engaged.
    pub fn on_stream_content(&mut self, line: impl Into<String>) {
        self.streaming = true;
        let line = line.into();
        if let Some(PresentSeg {
            kind: SegKind::Answer,
            body_lines,
            ..
        }) = self.segments.last_mut()
        {
            body_lines.push(line);
        } else {
            self.segments.push(PresentSeg {
                kind: SegKind::Answer,
                summary: "answer".into(),
                body_lines: vec![line],
                expanded: true,
                live: true,
            });
        }
        self.pin_if_stick();
    }

    pub fn pin_if_stick(&mut self) {
        if self.viewport.stick_to_bottom {
            let len = self.all_rendered_lines().len();
            let height = self.viewport.height.max(1);
            self.viewport.offset = len.saturating_sub(height);
        }
    }

    /// Manual scroll away from bottom releases stick.
    pub fn scroll_up(&mut self, lines: usize) {
        let total = self.all_rendered_lines().len();
        let height = self.viewport.height.max(1);
        let at_bottom_start = total.saturating_sub(height);
        if self.viewport.stick_to_bottom {
            self.viewport.offset = at_bottom_start;
        }
        self.viewport.stick_to_bottom = false;
        self.viewport.offset = self.viewport.offset.saturating_sub(lines);
    }

    /// Scroll toward latest content; re-engages stick when the bottom is reached.
    pub fn scroll_down(&mut self, lines: usize) {
        let total = self.all_rendered_lines().len();
        let height = self.viewport.height.max(1);
        let max_offset = total.saturating_sub(height);
        if self.viewport.stick_to_bottom {
            return;
        }
        self.viewport.offset = (self.viewport.offset + lines).min(max_offset);
        if self.viewport.offset >= max_offset {
            self.scroll_to_bottom();
        }
    }

    pub fn scroll_to_bottom(&mut self) {
        self.viewport.stick_to_bottom = true;
        self.pin_if_stick();
    }

    /// Default page size for PgUp/PgDn (viewport height, at least 1).
    pub fn page_lines(&self) -> usize {
        self.viewport.height.max(1)
    }

    fn recompute_next_hints(&mut self) {
        let answer_text: String = self
            .segments
            .iter()
            .filter(|s| s.kind == SegKind::Answer)
            .flat_map(|s| s.body_lines.iter().cloned())
            .collect::<Vec<_>>()
            .join("\n");
        let actions = trailing_next_actions(&answer_text);
        self.next_hints = actions
            .into_iter()
            .take(9)
            .enumerate()
            .map(|(i, a): (usize, NextAction)| NumberedHint {
                index: (i + 1) as u8,
                label: a.send.clone(),
                kind: HintKind::NextToken,
                payload: a.send,
            })
            .collect();
    }

    /// Set fast-response options while awaiting (or clear).
    pub fn set_fast_response(&mut self, fr: FastResponse, awaiting: bool) {
        self.awaiting_user = awaiting;
        let show = fast_response::visible(
            self.streaming,
            self.awaiting_user,
            self.composer.is_empty(),
            &fr,
        );
        self.fast_hints = if show {
            fr.options
                .into_iter()
                .take(9)
                .enumerate()
                .map(|(i, o)| NumberedHint {
                    index: (i + 1) as u8,
                    label: o.label,
                    kind: HintKind::FastResponse,
                    payload: o.id,
                })
                .collect()
        } else {
            Vec::new()
        };
    }

    pub fn numbered_hints_active(&self) -> Vec<&NumberedHint> {
        // Prefer fast-response while awaiting; otherwise next tokens.
        if !self.fast_hints.is_empty() {
            self.fast_hints.iter().collect()
        } else {
            self.next_hints.iter().collect()
        }
    }

    pub fn activate_number(&mut self, digit: u8) -> ChatAction {
        let hints: Vec<NumberedHint> = self.numbered_hints_active().into_iter().cloned().collect();
        let Some(h) = hints.into_iter().find(|h| h.index == digit) else {
            return ChatAction::None;
        };
        self.last_activation = Some(h.payload.clone());
        match h.kind {
            HintKind::NextToken => ChatAction::ActivatedNext(h.payload),
            HintKind::FastResponse => ChatAction::ActivatedFastResponse(h.payload),
        }
    }

    /// Composer typing / keys. `alt` marks Alt+Enter style newline.
    pub fn handle_composer_key(&mut self, key: ComposerKey) -> ChatAction {
        match key {
            ComposerKey::Enter => {
                if self.composer.trim().is_empty() {
                    return ChatAction::None;
                }
                self.submit_composer()
            }
            ComposerKey::AltEnter => {
                self.composer.push('\n');
                ChatAction::None
            }
            ComposerKey::Char(c) => {
                self.composer.push(c);
                self.on_composer_changed();
                if self.slash_palette_open {
                    ChatAction::OpenSlashPalette
                } else {
                    ChatAction::None
                }
            }
            ComposerKey::Backspace => {
                self.composer.pop();
                self.on_composer_changed();
                ChatAction::None
            }
        }
    }

    fn on_composer_changed(&mut self) {
        // Slash palette: open for `/…` at line start; not for `//` pass-through.
        let line_start = self.composer.rsplit('\n').next().unwrap_or(&self.composer);
        self.slash_palette_open =
            line_start.starts_with('/') && !line_start.starts_with("//");
    }

    /// Clear the composer and return the submitted text. Session mutation and
    /// agent drive are owned by the host submit router (`dispatch_user_submit`).
    pub fn submit_composer(&mut self) -> ChatAction {
        let text = self.composer.trim().to_string();
        if text.is_empty() {
            return ChatAction::None;
        }
        self.composer.clear();
        self.slash_palette_open = false;
        self.last_submit = Some(text.clone());
        ChatAction::Submitted(text)
    }

    /// Append a user text message (display form) to the session.
    pub fn push_user_text(&mut self, text: impl Into<String>) {
        self.session.messages.push(ChatMessage {
            role: Role::User,
            content: vec![ContentBlock::Text(text.into())],
            timestamp: String::new(),
            is_priming: false,
        });
    }

    /// Append a system text message to the session.
    pub fn push_system_text(&mut self, text: impl Into<String>) {
        self.session.messages.push(ChatMessage {
            role: Role::System,
            content: vec![ContentBlock::Text(text.into())],
            timestamp: String::new(),
            is_priming: false,
        });
    }

    /// Persist the session when driven (session-sharing policy).
    pub fn persist_if_driven(&self, project_root: Option<&Path>) -> PersistOutcome {
        session_sharing::persist_driven(&self.session, self.drive_role, project_root)
    }

    /// After submit, host may call this to start the duckcore agent turn.
    pub async fn drive_agent_turn(
        &self,
        project_root: std::path::PathBuf,
        harness: String,
        oneshot_model: Option<String>,
        tx: tokio::sync::mpsc::Sender<duckcore::agent::AgentEvent>,
    ) {
        if !self.agent_drive_requested {
            return;
        }
        duckcore::agent::drive_harness(project_root, harness, oneshot_model, tx).await;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComposerKey {
    Enter,
    AltEnter,
    Char(char),
    Backspace,
}

fn present_from_transcript(seg: &TranscriptSeg, collapsed: Option<bool>) -> Option<PresentSeg> {
    let collapsed = collapsed.unwrap_or(false);
    match seg {
        TranscriptSeg::Thinking { lines, live } => Some(PresentSeg {
            kind: SegKind::Thinking,
            summary: thinking_collapsed_label(lines),
            body_lines: lines.clone(),
            expanded: !collapsed,
            live: *live,
        }),
        TranscriptSeg::Activity { tools, live } => {
            let body: Vec<String> = tools
                .iter()
                .flat_map(|t| {
                    let mut v = vec![t.summary.clone()];
                    v.extend(t.output_lines.iter().cloned());
                    v
                })
                .collect();
            Some(PresentSeg {
                kind: SegKind::Activity,
                summary: activity_collapsed_label(tools),
                body_lines: body,
                expanded: !collapsed,
                live: *live,
            })
        }
        TranscriptSeg::Answer { lines, live } => Some(PresentSeg {
            kind: SegKind::Answer,
            summary: "answer".into(),
            body_lines: lines.clone(),
            expanded: true,
            live: *live,
        }),
        // User/system/choice chips: show as expanded answer-like lines for TUI list.
        TranscriptSeg::User { lines, .. } | TranscriptSeg::System { lines } => Some(PresentSeg {
            kind: SegKind::Answer,
            summary: "message".into(),
            body_lines: lines.clone(),
            expanded: true,
            live: false,
        }),
        TranscriptSeg::UserChoiceQuestion { text }
        | TranscriptSeg::UserChoiceAnswer { text } => Some(PresentSeg {
            kind: SegKind::Answer,
            summary: "message".into(),
            body_lines: vec![text.clone()],
            expanded: true,
            live: false,
        }),
    }
}

/// Helper for tests: settled thinking segment.
pub fn settled_thinking(summary: &str, body: &str) -> PresentSeg {
    PresentSeg {
        kind: SegKind::Thinking,
        summary: summary.into(),
        body_lines: body.lines().map(str::to_string).collect(),
        expanded: false,
        live: false,
    }
}

pub fn settled_activity(summary: &str, body: &str) -> PresentSeg {
    PresentSeg {
        kind: SegKind::Activity,
        summary: summary.into(),
        body_lines: body.lines().map(str::to_string).collect(),
        expanded: false,
        live: false,
    }
}

pub fn answer_seg(body: &str) -> PresentSeg {
    PresentSeg {
        kind: SegKind::Answer,
        summary: "answer".into(),
        body_lines: body.lines().map(str::to_string).collect(),
        expanded: true,
        live: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use duckcore::fast_response::from_user_choice;
    use duckcore::test_support::{FsTmp, with_home};

    // @spec tui/chat Settled segment collapse: Settled Thinking shows one summary line until expanded
    #[test]
    fn settled_thinking_shows_one_summary_line_until_expanded() {
        let mut pane = ChatPane::new("s");
        pane.set_settled_segments(vec![settled_thinking(
            "thought",
            "line one\nline two\nline three",
        )]);
        let lines = pane.segments[0].rendered_lines(72);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with('▶'));
        pane.expand_segment(0);
        let lines = pane.segments[0].rendered_lines(72);
        assert!(lines.len() > 1);
        assert!(lines[0].starts_with('▼'));
        assert!(lines.iter().any(|l| l.contains("line two")));
    }

    // @spec tui/chat Settled segment collapse: Settled Activity shows one summary line until expanded
    #[test]
    fn settled_activity_shows_one_summary_line_until_expanded() {
        let mut pane = ChatPane::new("s");
        pane.set_settled_segments(vec![settled_activity("tools", "read foo\nwrite bar")]);
        let lines = pane.segments[0].rendered_lines(72);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("tools"));
        pane.expand_segment(0);
        assert!(pane.segments[0].rendered_lines(72).len() > 1);
    }

    // @spec tui/chat Settled segment collapse: Answer body is fully visible without expand
    #[test]
    fn answer_body_is_fully_visible_without_expand() {
        let mut pane = ChatPane::new("s");
        pane.set_settled_segments(vec![answer_seg("hello\nworld")]);
        let lines = pane.segments[0].rendered_lines(72);
        assert_eq!(lines, vec!["hello".to_string(), "world".to_string()]);
        assert!(pane.segments[0].expanded);
    }

    // @spec tui/chat Autoscroll pin during stream: Streaming with stick engaged keeps latest lines in view
    #[test]
    fn streaming_with_stick_engaged_keeps_latest_lines_in_view() {
        let mut pane = ChatPane::new("s");
        pane.viewport.height = 3;
        pane.viewport.stick_to_bottom = true;
        for i in 0..10 {
            pane.on_stream_content(format!("line {i}"));
        }
        let visible = pane.visible_lines();
        assert!(pane.viewport.shows_latest(&pane.all_rendered_lines()));
        assert_eq!(visible.last().map(String::as_str), Some("line 9"));
    }

    // @spec tui/chat Autoscroll pin during stream: Manual scroll up releases the pin
    #[test]
    fn manual_scroll_up_releases_the_pin() {
        let mut pane = ChatPane::new("s");
        pane.viewport.height = 3;
        pane.viewport.stick_to_bottom = true;
        for i in 0..10 {
            pane.on_stream_content(format!("line {i}"));
        }
        pane.scroll_up(5);
        assert!(!pane.viewport.stick_to_bottom);
        let before = pane.visible_lines();
        pane.on_stream_content("line 10");
        let after = pane.visible_lines();
        // Further stream output does not force viewport back to bottom
        assert!(!pane.viewport.stick_to_bottom);
        assert_eq!(before.first(), after.first());
        assert!(!pane.viewport.shows_latest(&pane.all_rendered_lines()));
    }

    // @spec tui/chat Numbered action hints: Trailing next tokens appear as numbered hints
    #[test]
    fn trailing_next_tokens_appear_as_numbered_hints() {
        let mut pane = ChatPane::new("s");
        let body = "done\n\n> **next**\n>\n> `/ds-step`\n> `/ds-apply`\n";
        pane.set_settled_segments(vec![answer_seg(body)]);
        assert_eq!(pane.next_hints.len(), 2);
        assert_eq!(pane.next_hints[0].index, 1);
        assert_eq!(pane.next_hints[0].label, "/ds-step");
        assert_eq!(pane.next_hints[1].index, 2);
        assert_eq!(pane.next_hints[1].label, "/ds-apply");
    }

    // @spec tui/chat Numbered action hints: Number key activates the matching next token
    #[test]
    fn number_key_activates_the_matching_next_token() {
        let mut pane = ChatPane::new("s");
        let body = "x\n\n> **next**\n>\n> `confirm map`\n> `/ds-step`\n";
        pane.set_settled_segments(vec![answer_seg(body)]);
        let action = pane.activate_number(1);
        assert_eq!(action, ChatAction::ActivatedNext("confirm map".into()));
        assert_eq!(pane.last_activation.as_deref(), Some("confirm map"));
    }

    // @spec tui/chat Numbered action hints: Fast-response options appear as numbered hints while awaiting
    #[test]
    fn fast_response_options_appear_as_numbered_hints_while_awaiting() {
        let mut pane = ChatPane::new("s");
        pane.streaming = true;
        let fr = from_user_choice(
            1,
            Some("Pick?".into()),
            [
                ("a".into(), "Alpha".into()),
                ("b".into(), "Beta".into()),
            ],
        );
        pane.set_fast_response(fr, true);
        assert_eq!(pane.fast_hints.len(), 2);
        assert_eq!(pane.fast_hints[0].index, 1);
        assert_eq!(pane.fast_hints[0].label, "Alpha");
        assert_eq!(pane.fast_hints[1].label, "Beta");
    }

    // @spec tui/chat Numbered action hints: Number key activates the matching fast-response option
    #[test]
    fn number_key_activates_the_matching_fast_response_option() {
        let mut pane = ChatPane::new("s");
        pane.streaming = true;
        let fr = from_user_choice(7, None, [("opt-a".into(), "Alpha".into())]);
        pane.set_fast_response(fr, true);
        // Prefer fast hints over next
        let action = pane.activate_number(1);
        assert_eq!(
            action,
            ChatAction::ActivatedFastResponse("opt-a".into())
        );
        // Semantics align with duckcore resolve
        let fr = from_user_choice(7, None, [("opt-a".into(), "Alpha".into())]);
        let pick = fast_response::resolve_cmd_digit(&fr, 1);
        assert!(matches!(
            pick,
            Some(fast_response::FastResponsePick::Option { id }) if id == "opt-a"
        ));
    }

    // @spec tui/chat Composer send and newline: Enter with non-empty composer submits
    #[test]
    fn enter_with_non_empty_composer_submits() {
        let mut pane = ChatPane::new("s");
        pane.composer = "hello agent".into();
        let action = pane.handle_composer_key(ComposerKey::Enter);
        assert_eq!(action, ChatAction::Submitted("hello agent".into()));
        assert!(pane.composer.is_empty());
        assert_eq!(pane.last_submit.as_deref(), Some("hello agent"));
        // Agent drive and session mutation are owned by the host submit router.
        assert!(!pane.agent_drive_requested);
    }

    // @spec tui/chat Composer send and newline: Alt+Enter inserts a newline without submitting
    #[test]
    fn alt_enter_inserts_a_newline_without_submitting() {
        let mut pane = ChatPane::new("s");
        pane.composer = "line1".into();
        let action = pane.handle_composer_key(ComposerKey::AltEnter);
        assert_eq!(action, ChatAction::None);
        assert_eq!(pane.composer, "line1\n");
        assert!(pane.last_submit.is_none());
    }

    // @spec tui/chat Slash palette open: Slash at line start opens the palette
    #[test]
    fn slash_at_line_start_opens_the_palette() {
        let mut pane = ChatPane::new("s");
        assert!(!pane.slash_palette_open);
        let action = pane.handle_composer_key(ComposerKey::Char('/'));
        assert!(pane.slash_palette_open);
        assert_eq!(action, ChatAction::OpenSlashPalette);
    }

    // @spec tui/chat Slash palette open: Double-slash does not open as a single-slash catalog
    #[test]
    fn double_slash_does_not_open_as_a_single_slash_catalog() {
        let mut pane = ChatPane::new("s");
        pane.handle_composer_key(ComposerKey::Char('/'));
        assert!(pane.slash_palette_open);
        pane.handle_composer_key(ComposerKey::Char('/'));
        // `//` is pass-through — not a single-slash catalog open
        assert!(!pane.slash_palette_open);
        assert_eq!(pane.composer, "//");
    }

    #[test]
    fn submit_persists_when_driven() {
        let tmp = FsTmp::new();
        with_home(tmp.path(), || {
            let root = tmp.path().join("proj");
            std::fs::create_dir_all(&root).unwrap();
            let mut pane = ChatPane::new("drive-scope");
            pane.drive_role = DriveRole::Driven;
            pane.push_user_text("hi");
            let outcome = pane.persist_if_driven(Some(&root));
            assert!(matches!(outcome, PersistOutcome::Written));
            let loaded = duckcore::chat_store::load_sessions_for("drive-scope", Some(&root));
            assert_eq!(loaded.len(), 1);
        });
    }

    #[test]
    fn load_from_session_uses_shared_transcript_builder() {
        let mut session = ChatSession::new("s".into());
        session.messages.push(ChatMessage {
            role: Role::Assistant,
            content: vec![
                ContentBlock::Reasoning("think".into()),
                ContentBlock::Text("hello answer".into()),
            ],
            timestamp: String::new(),
            is_priming: false,
        });
        let mut pane = ChatPane::new("s");
        pane.load_from_session(session);
        assert!(
            pane.segments
                .iter()
                .any(|s| s.kind == SegKind::Thinking && !s.expanded)
        );
        assert!(
            pane.segments
                .iter()
                .any(|s| s.kind == SegKind::Answer
                    && s.body_lines.iter().any(|l| l.contains("hello answer")))
        );
    }
}
