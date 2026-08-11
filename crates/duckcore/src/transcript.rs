//! Shared transcript segment model and collapse policy.
//!
//! Builds harness-neutral Thinking / Activity / Answer segments from
//! [`crate::chat_store::ChatSession`] content. UI crates only render.

use crate::chat_store::{ChatSession, ContentBlock, Role};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranscriptSeg {
    User {
        lines: Vec<String>,
        /// Synthetic first-turn AGENTS.md / orientation inject. Starts
        /// collapsed so scroll-to-top lands on the real first user message.
        is_priming: bool,
    },
    System {
        lines: Vec<String>,
    },
    Thinking {
        lines: Vec<String>,
        /// True while this segment is still open in the turn (streaming and
        /// no following Answer yet) — not merely "still receiving deltas".
        live: bool,
    },
    Answer {
        lines: Vec<String>,
        live: bool,
    },
    Activity {
        tools: Vec<ToolRow>,
        /// True while the activity group is still open in the turn.
        live: bool,
    },
    /// Settled mid-turn question chip (host display).
    UserChoiceQuestion {
        text: String,
    },
    /// Settled mid-turn answer chip (host display).
    UserChoiceAnswer {
        text: String,
    },
}

/// One tool call inside an [`TranscriptSeg::Activity`] group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolRow {
    pub id: String,
    /// Human-readable tool summary (`format_tool_summary`), or the tool name
    /// alone for orphan results.
    pub summary: String,
    /// Truncated result output; empty while still running.
    pub output_lines: Vec<String>,
    pub status: ToolRowStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolRowStatus {
    Running,
    Done,
    /// Reserved for later error-shaped output detection.
    #[allow(dead_code)]
    Error,
}

/// Build calm transcript segments from committed messages and live stream
/// buffers. Contiguous same-kind assistant content coalesces; tools pair by
/// call id within an activity run.
pub fn build_transcript_segments(session: &ChatSession) -> Vec<TranscriptSeg> {
    use std::collections::HashMap;

    let mut segs: Vec<TranscriptSeg> = Vec::new();
    // Within the open Activity: row order + id → index for pairing.
    let mut activity_index: HashMap<String, usize> = HashMap::new();

    for msg in &session.messages {
        for cb in &msg.content {
            match (msg.role, cb) {
                (Role::User, ContentBlock::Text(t)) => {
                    activity_index.clear();
                    segs.push(TranscriptSeg::User {
                        lines: text_lines(t),
                        is_priming: msg.is_priming,
                    });
                }
                (Role::System, ContentBlock::Text(t)) => {
                    activity_index.clear();
                    segs.push(TranscriptSeg::System {
                        lines: text_lines(t),
                    });
                }
                (Role::Assistant, ContentBlock::Reasoning(t)) => {
                    activity_index.clear();
                    append_thinking(&mut segs, t, false);
                }
                (Role::Assistant, ContentBlock::Text(t)) => {
                    activity_index.clear();
                    append_answer(&mut segs, t, false);
                }
                (Role::Assistant, ContentBlock::ToolUse { id, name, input }) => {
                    // Structured questions use host chips, not Activity rows.
                    if is_host_choice_tool_name(name) {
                        continue;
                    }
                    ensure_activity(&mut segs, &mut activity_index);
                    let tools = activity_tools_mut(&mut segs);
                    if let Some(&idx) = activity_index.get(id) {
                        // Duplicate id: refresh summary, leave status/output.
                        tools[idx].summary = format_tool_summary(name, input);
                    } else {
                        let idx = tools.len();
                        activity_index.insert(id.clone(), idx);
                        tools.push(ToolRow {
                            id: id.clone(),
                            summary: format_tool_summary(name, input),
                            output_lines: Vec::new(),
                            status: ToolRowStatus::Running,
                        });
                    }
                }
                (Role::Assistant, ContentBlock::ToolResult { id, name, output }) => {
                    if is_host_choice_tool_name(name) {
                        continue;
                    }
                    ensure_activity(&mut segs, &mut activity_index);
                    let tools = activity_tools_mut(&mut segs);
                    if let Some(&idx) = activity_index.get(id) {
                        tools[idx].output_lines = truncate_output(output);
                        tools[idx].status = ToolRowStatus::Done;
                    } else {
                        // Orphan result: named done row from the tool name,
                        // never a bare "✓ done" placeholder.
                        let idx = tools.len();
                        activity_index.insert(id.clone(), idx);
                        tools.push(ToolRow {
                            id: id.clone(),
                            summary: name.clone(),
                            output_lines: truncate_output(output),
                            status: ToolRowStatus::Done,
                        });
                    }
                }
                (_, ContentBlock::UserChoiceQuestion { text }) => {
                    activity_index.clear();
                    segs.push(TranscriptSeg::UserChoiceQuestion { text: text.clone() });
                }
                (_, ContentBlock::UserChoiceAnswer { text }) => {
                    activity_index.clear();
                    segs.push(TranscriptSeg::UserChoiceAnswer { text: text.clone() });
                }
                // Non-text user/system content (e.g. tools) is not expected
                // on those roles — skip rather than invent a segment.
                (Role::User | Role::System, _) => {}
            }
        }
    }

    // Live stream buffers: append to or open Thinking / Answer segments.
    if session.is_streaming {
        if !session.pending_reasoning.is_empty() {
            activity_index.clear();
            append_thinking(&mut segs, &session.pending_reasoning, true);
        }
        if !session.pending_text.is_empty() {
            activity_index.clear();
            append_answer(&mut segs, &session.pending_text, true);
        }
    }

    // Settle tool status and turn-open live flags.
    //
    // `live` means "still open in the turn" — not "still receiving deltas of
    // this kind". Committed reasoning is built with live=false above; while
    // streaming and no following Answer, Thinking stays open so collapse
    // policy does not snap it shut when tools start.
    let streaming = session.is_streaming;
    let answer_after: Vec<bool> = (0..segs.len())
        .map(|i| {
            segs[i + 1..]
                .iter()
                .any(|s| matches!(s, TranscriptSeg::Answer { .. }))
        })
        .collect();
    for (i, seg) in segs.iter_mut().enumerate() {
        match seg {
            TranscriptSeg::Activity { tools, live } => {
                if !streaming {
                    for row in tools.iter_mut() {
                        if row.status == ToolRowStatus::Running {
                            row.status = ToolRowStatus::Done;
                        }
                    }
                    *live = false;
                } else {
                    *live = !answer_after[i]
                        || tools.iter().any(|t| t.status == ToolRowStatus::Running);
                }
            }
            TranscriptSeg::Thinking { live, .. } => {
                if !streaming {
                    *live = false;
                } else {
                    // Open until a following Answer appears or the turn ends.
                    *live = !answer_after[i];
                }
            }
            _ => {}
        }
    }

    segs
}

pub fn text_lines(t: &str) -> Vec<String> {
    t.lines().map(String::from).collect()
}

fn append_thinking(segs: &mut Vec<TranscriptSeg>, text: &str, live: bool) {
    let mut lines = text_lines(text);
    if let Some(TranscriptSeg::Thinking {
        lines: existing,
        live: existing_live,
    }) = segs.last_mut()
    {
        existing.append(&mut lines);
        *existing_live = *existing_live || live;
    } else {
        segs.push(TranscriptSeg::Thinking { lines, live });
    }
}

fn append_answer(segs: &mut Vec<TranscriptSeg>, text: &str, live: bool) {
    let mut lines = text_lines(text);
    if let Some(TranscriptSeg::Answer {
        lines: existing,
        live: existing_live,
    }) = segs.last_mut()
    {
        existing.append(&mut lines);
        *existing_live = *existing_live || live;
    } else {
        segs.push(TranscriptSeg::Answer { lines, live });
    }
}

fn ensure_activity(
    segs: &mut Vec<TranscriptSeg>,
    activity_index: &mut std::collections::HashMap<String, usize>,
) {
    if !matches!(segs.last(), Some(TranscriptSeg::Activity { .. })) {
        activity_index.clear();
        segs.push(TranscriptSeg::Activity {
            tools: Vec::new(),
            live: false,
        });
    }
}

fn activity_tools_mut(segs: &mut [TranscriptSeg]) -> &mut Vec<ToolRow> {
    match segs.last_mut() {
        Some(TranscriptSeg::Activity { tools, .. }) => tools,
        _ => unreachable!("ensure_activity must open an Activity first"),
    }
}

// ── Collapse policy ────────────────────────────────────────────────────────

/// Per-segment collapse flag plus whether the user has manually toggled it.
/// Index-aligned with the transcript segment list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CollapseState {
    pub collapsed: bool,
    /// Once true, auto-collapse must not force this segment shut again.
    pub user_set: bool,
}

/// Seconds after a manual expand of the priming Setup block before it
/// auto-hides again. Mid of the 10–20s product range.
pub const PRIMING_RECOLLAPSE_SECS: u64 = 15;

/// First-sight default: live Thinking/Activity expanded; settled collapsed.
/// Priming User starts collapsed. Other User / Answer / System stay open.
fn first_sight_collapsed(seg: &TranscriptSeg) -> bool {
    match seg {
        TranscriptSeg::Thinking { live, .. } | TranscriptSeg::Activity { live, .. } => !live,
        TranscriptSeg::User { is_priming: true, .. } => true,
        TranscriptSeg::User { is_priming: false, .. }
        | TranscriptSeg::System { .. }
        | TranscriptSeg::Answer { .. }
        | TranscriptSeg::UserChoiceQuestion { .. }
        | TranscriptSeg::UserChoiceAnswer { .. } => false,
    }
}

fn has_following_answer(segs: &[TranscriptSeg], idx: usize) -> bool {
    segs[idx + 1..]
        .iter()
        .any(|s| matches!(s, TranscriptSeg::Answer { .. }))
}

/// Sync collapse state with the current segment list.
///
/// - Resizes to match `segs` (truncates if shorter; appends first-sight defaults).
/// - Auto-collapses untoggled Thinking when a following Answer appears or the
///   segment is no longer live (turn settled — see Thinking `live` fixup in
///   [`build_transcript_segments`]).
/// - Auto-collapses untoggled Activity when a following Answer appears or the
///   turn settles (`live == false`).
/// - Keeps untoggled priming User collapsed (first-sight and on rebuild).
/// - Leaves `user_set` segments alone for auto-collapse (priming re-hide after
///   expand is a separate timer, not this sync path).
///
/// Thinking `live` means open-in-turn (streaming, no following Answer), not
/// "still receiving ReasoningDelta", so tool phases keep Thinking expanded.
pub fn sync_collapse_states(states: &mut Vec<CollapseState>, segs: &[TranscriptSeg]) {
    if states.len() > segs.len() {
        states.truncate(segs.len());
    }
    while states.len() < segs.len() {
        let i = states.len();
        states.push(CollapseState {
            collapsed: first_sight_collapsed(&segs[i]),
            user_set: false,
        });
    }

    for (i, seg) in segs.iter().enumerate() {
        if states[i].user_set {
            continue;
        }
        match seg {
            TranscriptSeg::Thinking { live, .. } | TranscriptSeg::Activity { live, .. } => {
                // Settle when Answer follows or the segment is no longer
                // open-in-turn (`!live`). For Thinking, `live` is fixed up so
                // committed reasoning during a tool phase stays open.
                if has_following_answer(segs, i) || !*live {
                    states[i].collapsed = true;
                }
            }
            TranscriptSeg::User { is_priming: true, .. } => {
                // Stay folded until the user clicks to inspect.
                states[i].collapsed = true;
            }
            TranscriptSeg::User { is_priming: false, .. }
            | TranscriptSeg::System { .. }
            | TranscriptSeg::Answer { .. }
            | TranscriptSeg::UserChoiceQuestion { .. }
            | TranscriptSeg::UserChoiceAnswer { .. } => {
                states[i].collapsed = false;
            }
        }
    }
}

/// User toggle: flip collapsed and mark as manually set so auto-collapse
/// will not override this segment again.
pub fn toggle_collapse(states: &mut [CollapseState], idx: usize) {
    if let Some(state) = states.get_mut(idx) {
        state.collapsed = !state.collapsed;
        state.user_set = true;
    }
}

/// Collapsed label for the synthetic priming user message.
pub fn priming_collapsed_label(lines: &[String]) -> String {
    let n = lines.len();
    if n == 1 {
        "Setup · 1 line".to_string()
    } else {
        format!("Setup · {n} lines")
    }
}

/// Force-collapse a priming segment after the expand timer. Callers gate on
/// expand generation so stale timers no-op.
pub fn recollapse_priming(states: &mut [CollapseState], idx: usize) {
    if let Some(state) = states.get_mut(idx) {
        state.collapsed = true;
        // Keep `user_set` so sync does not fight a later re-expand path that
        // also marks user_set; the timed re-hide is intentional UX.
    }
}

// ── Segment presentation helpers ───────────────────────────────────────────

/// Collapsed Thinking label: line count only (no duration).
///
/// Examples: `"Thinking · 1 line"`, `"Thinking · 12 lines"`.
pub fn thinking_collapsed_label(lines: &[String]) -> String {
    let n = lines.len();
    if n == 1 {
        "Thinking · 1 line".to_string()
    } else {
        format!("Thinking · {n} lines")
    }
}

/// Collapsed Activity summary: tool count plus sample names from the rows.
///
/// Example: `"4 tools · Read, Shell, Grep"`.
pub fn activity_collapsed_label(tools: &[ToolRow]) -> String {
    let n = tools.len();
    let count = if n == 1 {
        "1 tool".to_string()
    } else {
        format!("{n} tools")
    };
    const SAMPLE: usize = 3;
    let names: Vec<&str> = tools
        .iter()
        .take(SAMPLE)
        .map(|t| tool_display_name(&t.summary))
        .collect();
    if names.is_empty() {
        count
    } else {
        format!("{count} · {}", names.join(", "))
    }
}

/// Humanized tool verb from a summary line (`"Read · path"` → `"Read"`).
fn tool_display_name(summary: &str) -> &str {
    summary.split(" · ").next().unwrap_or(summary).trim()
}

/// Quiet row for an expanded Activity group. Status + summary on the row;
/// truncated output sits under it. Group expand only — no per-tool expand state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityRowView {
    pub status: ToolRowStatus,
    pub status_glyph: &'static str,
    pub summary: String,
    /// Truncated output lines under the row; empty while running or empty result.
    pub output_lines: Vec<String>,
}

/// Shape expanded Activity presentation as one quiet row per tool.
pub fn expanded_activity_rows(tools: &[ToolRow]) -> Vec<ActivityRowView> {
    tools
        .iter()
        .map(|t| ActivityRowView {
            status: t.status,
            status_glyph: tool_status_glyph(t.status),
            summary: t.summary.clone(),
            output_lines: t.output_lines.clone(),
        })
        .collect()
}

pub fn tool_status_glyph(status: ToolRowStatus) -> &'static str {
    match status {
        ToolRowStatus::Running => "●",
        ToolRowStatus::Done => "✓",
        ToolRowStatus::Error => "✗",
    }
}

pub fn truncate_output(output: &str) -> Vec<String> {
    const MAX_LINES: usize = 10;
    let cleaned = strip_ansi_escapes(output);
    let cleaned = strip_tool_wrapper_tags(&cleaned);
    let all_lines: Vec<String> = cleaned
        .lines()
        .map(sanitize_line)
        .map(|s| s.trim_end().to_string())
        .collect();
    let mut lines = if all_lines.len() > MAX_LINES {
        let mut truncated = all_lines[..MAX_LINES].to_vec();
        truncated.push(format!("… ({} more lines)", all_lines.len() - MAX_LINES));
        truncated
    } else {
        all_lines
    };
    while lines.last().is_some_and(|s| s.is_empty()) {
        lines.pop();
    }
    lines
}

/// Remove ANSI CSI escape sequences (e.g. `\x1B[32m`). Parses the sequence
/// greedily through its final byte so the parameter bytes don't leak through.
pub fn strip_ansi_escapes(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\x1B' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('[') => {
                for next in chars.by_ref() {
                    let n = next as u32;
                    if (0x40..=0x7E).contains(&n) {
                        break;
                    }
                }
            }
            Some(']') => {
                // OSC: terminate on BEL (0x07) or ESC \\.
                let mut prev_esc = false;
                for next in chars.by_ref() {
                    if next == '\x07' {
                        break;
                    }
                    if prev_esc && next == '\\' {
                        break;
                    }
                    prev_esc = next == '\x1B';
                }
            }
            Some(_) | None => {}
        }
    }
    out
}

/// Strip `<tool_use_error>` / `<tool_use_result>` wrapper tags that some
/// agent backends emit around tool output.
pub fn strip_tool_wrapper_tags(input: &str) -> String {
    input
        .replace("<tool_use_error>", "")
        .replace("</tool_use_error>", "")
        .replace("<tool_use_result>", "")
        .replace("</tool_use_result>", "")
}

/// Replace remaining non-printable / non-standard-whitespace characters with a
/// space to avoid rendering rectangles in the monospace font.
pub fn sanitize_line(line: &str) -> String {
    line.chars()
        .map(|c| if c == '\t' || c.is_control() { ' ' } else { c })
        .collect()
}

/// Tools that surface as mid-turn host chips (not Activity). Claude's
/// `AskUserQuestion` and humanized "Ask user question" forms match here.
pub fn is_host_choice_tool_name(name: &str) -> bool {
    let key = normalize_tool_key(name);
    matches!(
        key.as_str(),
        "ask_user_question" | "askuserquestion" | "ask_user" | "askuser"
    )
}

/// Produce a short human-readable summary of a tool call.
///
/// Shape: `Verb · detail` (or just `Verb`). Known Claude/Grok tools share a
/// calm display verb; unknown names are humanized. Never dumps raw JSON.
///
/// Examples: `Read · agent_chat.rs`, `Shell · cargo test -p duckboard`.
pub fn format_tool_summary(name: &str, input: &str) -> String {
    let verb = known_tool_verb(name)
        .map(str::to_string)
        .unwrap_or_else(|| humanize_tool_name(name));
    match tool_detail(input) {
        Some(detail) if !detail.is_empty() => format!("{verb} · {detail}"),
        _ => verb,
    }
}

/// Map known Claude / Grok tool names to a short display verb (case-insensitive).
/// Returns `None` for unmapped names (use [`humanize_tool_name`]).
fn known_tool_verb(name: &str) -> Option<&'static str> {
    let key = normalize_tool_key(name);
    match key.as_str() {
        // Shell
        "bash" | "shell" | "run_terminal_command" | "run_terminal" => Some("Shell"),
        // File read
        "read" | "read_file" => Some("Read"),
        // File write
        "write" | "write_file" => Some("Write"),
        // Edit / replace
        "edit" | "search_replace" | "multi_edit" | "str_replace" | "strreplace" => Some("Edit"),
        // Search
        "grep" | "rg" => Some("Grep"),
        // Glob / list
        "glob" => Some("Glob"),
        "ls" | "list" | "list_dir" => Some("List"),
        // Web
        "web_search" | "websearch" => Some("Search"),
        "web_fetch" | "webfetch" | "open_page" | "open_page_with_find" | "web_fetch_url" => {
            Some("Fetch")
        }
        // Misc agent tools
        "todo_write" | "todowrite" => Some("Todo"),
        "task" | "spawn_subagent" => Some("Task"),
        "image_gen" | "image_edit" => Some("Image"),
        _ => None,
    }
}

/// Normalize a tool name for alias matching: `WebSearch` → `web_search`,
/// `run-terminal-command` → `run_terminal_command`.
fn normalize_tool_key(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    for (i, c) in name.trim().chars().enumerate() {
        if c == '-' || c == ' ' {
            if !out.ends_with('_') {
                out.push('_');
            }
        } else if c.is_uppercase() {
            if i > 0 && !out.ends_with('_') {
                out.push('_');
            }
            for lower in c.to_lowercase() {
                out.push(lower);
            }
        } else {
            out.push(c);
        }
    }
    while out.contains("__") {
        out = out.replace("__", "_");
    }
    out.trim_matches('_').to_string()
}

/// Humanize an unknown tool name for display: `some_obscure_tool` →
/// `Some obscure tool`, `camelCase` → `Camel case`.
fn humanize_tool_name(name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return "Tool".to_string();
    }
    let key = normalize_tool_key(trimmed);
    let words: Vec<&str> = key.split('_').filter(|w| !w.is_empty()).collect();
    if words.is_empty() {
        return "Tool".to_string();
    }
    let mut parts = Vec::with_capacity(words.len());
    for (i, w) in words.iter().enumerate() {
        if i == 0 {
            // Title-case the first word.
            let mut chars = w.chars();
            let first = chars
                .next()
                .map(|c| c.to_uppercase().to_string())
                .unwrap_or_default();
            parts.push(format!("{first}{}", chars.as_str()));
        } else {
            parts.push(w.to_string());
        }
    }
    parts.join(" ")
}

/// Extract a single-line detail from tool input JSON for the summary row.
/// Prefer path, command, pattern/query; never multi-line bodies or full JSON.
fn tool_detail(input: &str) -> Option<String> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }
    let Ok(serde_json::Value::Object(map)) = serde_json::from_str(input) else {
        // Non-JSON input: one short line if it's already a simple string.
        let one = input.lines().next().unwrap_or(input).trim();
        if one.is_empty() || one.starts_with('{') {
            return None;
        }
        return Some(truncate_chars(one, 50).to_string());
    };

    // Path-like fields (shorten to last components).
    for key in [
        "file_path",
        "path",
        "target_file",
        "file",
        "filename",
        "target_directory",
    ] {
        if let Some(p) = map.get(key).and_then(|v| v.as_str()) {
            let p = p.trim();
            if !p.is_empty() {
                return Some(shorten_path(p));
            }
        }
    }

    // Shell command — first line only.
    if let Some(cmd) = map.get("command").and_then(|v| v.as_str()) {
        let one = cmd.lines().next().unwrap_or(cmd).trim();
        if !one.is_empty() {
            return Some(truncate_chars(one, 50).to_string());
        }
    }

    // Search pattern / query — quoted.
    for key in ["pattern", "query"] {
        if let Some(s) = map.get(key).and_then(|v| v.as_str()) {
            let s = s.trim();
            if !s.is_empty() {
                let t = truncate_chars(s, 40);
                return Some(format!("\"{t}\""));
            }
        }
    }

    // URL (web fetch / open).
    if let Some(url) = map.get("url").and_then(|v| v.as_str()) {
        let url = url.trim();
        if !url.is_empty() {
            return Some(truncate_chars(url, 48).to_string());
        }
    }

    // Fallback: first short single-line string field that isn't a bulky body.
    const SKIP: &[&str] = &[
        "contents",
        "content",
        "body",
        "old_string",
        "new_string",
        "output",
        "prompt",
        "text",
        "code",
        "diff",
    ];
    for (key, value) in &map {
        if SKIP.iter().any(|s| s.eq_ignore_ascii_case(key)) {
            continue;
        }
        let Some(s) = value.as_str() else {
            continue;
        };
        let s = s.trim();
        if s.is_empty() || s.contains('\n') {
            continue;
        }
        if s.chars().count() > 80 {
            continue;
        }
        return Some(truncate_chars(s, 40).to_string());
    }

    None
}

/// Shorten a path to at most the last three components.
fn shorten_path(p: &str) -> String {
    let short: String = p
        .rsplit('/')
        .take(3)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("/");
    if short.chars().count() > 48 {
        truncate_chars(&short, 48).to_string()
    } else {
        short
    }
}

/// Truncate a string to at most `max` characters, on char boundaries.
///
/// Slicing with a byte index (`&s[..n]`) panics when the index falls inside a
/// multibyte UTF-8 character, so we count by `char` instead. Returns the whole
/// string when it is already short enough.
pub fn truncate_chars(s: &str, max: usize) -> &str {
    match s.char_indices().nth(max) {
        Some((byte_idx, _)) => &s[..byte_idx],
        None => s,
    }
}



