//! Live agent + file-event application for the ducktui event loop.

use std::path::{Path, PathBuf};

use duckchat::{ContextHook, TurnRequest, UserChoiceAnswer};
use duckcore::agent::{self, AgentEvent, AgentHandle};
use duckcore::chat_store::{ChatMessage, ContentBlock, Role};
use duckcore::fast_response::from_user_choice;
use duckcore::scope::{
    AgentsMarkdownHook, ChangeScopeFacts, CurrentScopeHook, ScopeKind, SessionScope,
};
use duckcore::session_sharing::{self, ExternalSessionEvent};
use duckcore::watcher::{self, FileEvent};
use tokio::sync::mpsc;

use crate::chat_pane::{ChatAction, ChatPane};
use crate::navigator::BoundChat;
use crate::shell::Shell;

/// Path-reference note shared with duckboard first-turn orientation.
const PATH_REFERENCE_NOTE: &str = "When referencing project files in your replies, \
write the path relative to the project root, optionally with a 1-based line \
suffix — e.g. `crates/duckboard/src/main.rs:42`. Prefer this form over absolute \
paths or bare filenames; the UI turns such references into clickable links.";

/// Host-side agent subscription state for the active chat.
#[derive(Default)]
pub struct AgentRuntime {
    pub handle: Option<AgentHandle>,
    /// User text waiting for `Ready` before first `send_turn`.
    pub pending_prompt: Option<String>,
    pub harness: String,
    pub oneshot_model: Option<String>,
    /// True while a worker task is running (or starting).
    pub worker_alive: bool,
    /// Transcript dirty since last tick materialize.
    pub chat_dirty: bool,
    /// Last agent-facing prompt actually queued or sent (includes first-turn
    /// orientation when applied). Tests and diagnostics.
    pub last_agent_prompt: Option<String>,
}

impl AgentRuntime {
    pub fn harness_from_model(shell: &Shell) -> String {
        shell
            .model
            .as_ref()
            .map(|m| m.harness.clone())
            .unwrap_or_else(|| "grok".into())
    }

    pub fn model_id(shell: &Shell) -> Option<String> {
        shell.model.as_ref().map(|m| m.model.clone())
    }
}

/// Spawn file watcher for `project_root`; events forwarded as batches on `tx`.
pub fn start_watcher(
    project_root: PathBuf,
    tx: mpsc::Sender<Vec<FileEvent>>,
) -> std::thread::JoinHandle<()> {
    let duckspec = project_root.join("duckspec");
    let duckspec_root = duckspec.is_dir().then_some(duckspec);
    watcher::spawn_watcher(project_root, duckspec_root, tx)
}

/// Spawn duckcore agent worker; maps events into `AppEvent`-friendly channel.
pub fn start_agent_worker(
    project_root: PathBuf,
    harness: String,
    oneshot_model: Option<String>,
    tx: mpsc::Sender<AgentEvent>,
) {
    tokio::spawn(async move {
        agent::drive_harness(project_root, harness, oneshot_model, tx).await;
    });
}

/// Apply a batch of file events to the active chat session (session-sharing).
pub fn apply_file_events(shell: &mut Shell, events: &[FileEvent]) {
    let Some(root) = shell.project_root.clone() else {
        return;
    };
    let mut list = vec![shell.chat.session.clone()];
    for ev in events {
        let (path, removed) = match ev {
            FileEvent::Modified(p) => (p.as_path(), false),
            FileEvent::Removed(p) => (p.as_path(), true),
            FileEvent::VcsStateChanged(_) => continue,
        };
        let Some(ext) = session_sharing::external_event_for_path(path, removed, Some(&root))
        else {
            continue;
        };
        // Only apply when the event targets the active session scope/id.
        let (scope, id) = match &ext {
            ExternalSessionEvent::Modified { scope, session_id }
            | ExternalSessionEvent::Removed { scope, session_id } => {
                (scope.as_str(), session_id.as_str())
            }
        };
        if shell.chat.session.scope != scope || shell.chat.session.id != id {
            continue;
        }
        let _ = session_sharing::apply_external_to_list(&mut list, &ext, Some(&root));
    }
    if let Some(session) = list.into_iter().next() {
        shell.chat.load_from_session(session);
    } else {
        // Session removed externally
        let scope = shell.chat.session.scope.clone();
        shell.chat = ChatPane::new(scope);
    }
}

/// Apply one agent event to shell chat state. Returns true if UI should rebuild.
pub fn apply_agent_event(
    shell: &mut Shell,
    runtime: &mut AgentRuntime,
    event: AgentEvent,
) -> bool {
    match event {
        AgentEvent::Ready(handle) => {
            runtime.handle = Some(handle);
            runtime.worker_alive = true;
            if let Some(prompt) = runtime.pending_prompt.take() {
                // pending_prompt is already fully assembled (orientation applied).
                send_turn_raw(shell, runtime, prompt);
            }
            false
        }
        AgentEvent::CommandsAvailable(commands) => {
            shell.apply_discovered_commands(commands);
            false
        }
        AgentEvent::ContentDelta { text } => {
            shell.chat.session.is_streaming = true;
            shell.chat.session.pending_text.push_str(&text);
            shell.chat.streaming = true;
            runtime.chat_dirty = true;
            true
        }
        AgentEvent::ReasoningDelta { text } => {
            shell.chat.session.is_streaming = true;
            shell.chat.session.pending_reasoning.push_str(&text);
            shell.chat.streaming = true;
            runtime.chat_dirty = true;
            true
        }
        AgentEvent::ToolUse { id, name, input } => {
            flush_pending_answer(&mut shell.chat.session);
            shell.chat.session.messages.push(ChatMessage {
                role: Role::Assistant,
                content: vec![ContentBlock::ToolUse { id, name, input }],
                timestamp: String::new(),
                is_priming: false,
            });
            shell.chat.session.is_streaming = true;
            runtime.chat_dirty = true;
            true
        }
        AgentEvent::ToolResult { id, name, output } => {
            shell.chat.session.messages.push(ChatMessage {
                role: Role::Assistant,
                content: vec![ContentBlock::ToolResult { id, name, output }],
                timestamp: String::new(),
                is_priming: false,
            });
            runtime.chat_dirty = true;
            true
        }
        AgentEvent::UsageUpdate {
            input_tokens,
            output_tokens,
        } => {
            shell.chat.session.context_tokens = input_tokens.saturating_add(output_tokens);
            shell.context.used = shell.chat.session.context_tokens;
            shell.turn = crate::shell::TurnState::Streaming;
            true
        }
        AgentEvent::SessionIdUpdated { session_id } => {
            shell.chat.session.agent_session_id = Some(session_id);
            false
        }
        AgentEvent::SessionNotFound => {
            shell.chat.session.agent_session_id = None;
            false
        }
        AgentEvent::UserChoiceRequest {
            correlation_id,
            prompt,
            options,
            allow_cancel: _,
        } => {
            shell.chat.awaiting_user = true;
            shell.chat.pending_choice_correlation = Some(correlation_id);
            shell.turn = crate::shell::TurnState::AwaitingChoice;
            let fr = from_user_choice(correlation_id, prompt, options);
            shell.chat.set_fast_response(fr, true);
            true
        }
        AgentEvent::TurnComplete => {
            flush_pending_answer(&mut shell.chat.session);
            flush_pending_reasoning(&mut shell.chat.session);
            shell.chat.session.is_streaming = false;
            shell.chat.streaming = false;
            shell.chat.awaiting_user = false;
            shell.turn = crate::shell::TurnState::Idle;
            shell.chat.drive_role = duckcore::session_sharing::DriveRole::Driven;
            let _ = shell.chat.persist_if_driven(shell.project_root.as_deref());
            shell.chat.load_from_session(shell.chat.session.clone());
            runtime.chat_dirty = false;
            true
        }
        AgentEvent::Error(msg) => {
            shell.chat.session.messages.push(ChatMessage {
                role: Role::System,
                content: vec![ContentBlock::Text(format!("Error: {msg}"))],
                timestamp: String::new(),
                is_priming: false,
            });
            shell.chat.session.is_streaming = false;
            shell.chat.streaming = false;
            shell.turn = crate::shell::TurnState::Idle;
            runtime.chat_dirty = true;
            true
        }
        AgentEvent::ProcessExited => {
            runtime.worker_alive = false;
            runtime.handle = None;
            false
        }
    }
}

fn flush_pending_answer(session: &mut duckcore::chat_store::ChatSession) {
    if session.pending_text.is_empty() {
        return;
    }
    let text = std::mem::take(&mut session.pending_text);
    session.messages.push(ChatMessage {
        role: Role::Assistant,
        content: vec![ContentBlock::Text(text)],
        timestamp: String::new(),
        is_priming: false,
    });
}

fn flush_pending_reasoning(session: &mut duckcore::chat_store::ChatSession) {
    if session.pending_reasoning.is_empty() {
        return;
    }
    let text = std::mem::take(&mut session.pending_reasoning);
    session.messages.push(ChatMessage {
        role: Role::Assistant,
        content: vec![ContentBlock::Reasoning(text)],
        timestamp: String::new(),
        is_priming: false,
    });
}

/// Materialize dirty streaming state into presentation segments.
pub fn materialize_if_dirty(shell: &mut Shell, runtime: &mut AgentRuntime) {
    if !runtime.chat_dirty {
        return;
    }
    shell.chat.load_from_session(shell.chat.session.clone());
    runtime.chat_dirty = false;
}

/// Route a composer (or hint) submit through shared slash parsing before any
/// agent turn. Single entry for all user text submits.
pub fn dispatch_user_submit(
    shell: &mut Shell,
    runtime: &mut AgentRuntime,
    agent_tx: mpsc::Sender<AgentEvent>,
    text: String,
) {
    use duckcore::slash_commands::{SubmitSlash, parse_submit_slash};

    match parse_submit_slash(&text) {
        SubmitSlash::LocalHelp => {
            run_local_help(shell);
        }
        SubmitSlash::LocalBuildPilot { .. } => {
            // No TUI build-pilot arm/kick — freeform agent path with typed text.
            shell.chat.push_user_text(text.clone());
            shell.chat.load_from_session(shell.chat.session.clone());
            shell.chat.agent_drive_requested = true;
            submit_user_prompt(shell, runtime, agent_tx, text);
        }
        SubmitSlash::Agent { display, prompt } => {
            shell.chat.push_user_text(display);
            shell.chat.load_from_session(shell.chat.session.clone());
            shell.chat.agent_drive_requested = true;
            submit_user_prompt(shell, runtime, agent_tx, prompt);
        }
    }
}

/// Local `/help`: user + system messages, driven persist, no agent turn.
fn run_local_help(shell: &mut Shell) {
    use duckcore::slash_commands::build_system_help_body;
    use duckcore::session_sharing::DriveRole;

    let catalog = shell.slash_catalog();
    let harness = shell.model.as_ref().map(|m| m.harness.as_str());
    let body = build_system_help_body(&catalog, harness);

    shell.chat.drive_role = DriveRole::Driven;
    shell.chat.agent_drive_requested = false;
    shell.chat.push_user_text("/help");
    shell.chat.push_system_text(body);
    shell.chat.session.is_streaming = false;
    shell.chat.streaming = false;
    shell.chat.session.pending_text.clear();
    shell.chat.session.pending_reasoning.clear();
    shell.turn = crate::shell::TurnState::Idle;
    let _ = shell.chat.persist_if_driven(shell.project_root.as_deref());
    shell.chat.load_from_session(shell.chat.session.clone());
}

/// Start or reuse agent worker and send user prompt.
///
/// Caller is responsible for appending the user display message first.
/// When the session has no resumable `agent_session_id`, first-turn scope
/// orientation is folded into the agent-facing prompt body.
pub fn submit_user_prompt(
    shell: &mut Shell,
    runtime: &mut AgentRuntime,
    agent_tx: mpsc::Sender<AgentEvent>,
    prompt: String,
) {
    let Some(root) = shell.project_root.clone() else {
        return;
    };
    shell.chat.drive_role = duckcore::session_sharing::DriveRole::Driven;
    let _ = shell.chat.persist_if_driven(Some(&root));

    let agent_prompt = assemble_agent_prompt(shell, &prompt);
    runtime.last_agent_prompt = Some(agent_prompt.clone());

    runtime.harness = AgentRuntime::harness_from_model(shell);
    runtime.oneshot_model = shell
        .shared_config
        .chat
        .oneshot_model(&runtime.harness)
        .map(str::to_string);

    if let Some(handle) = runtime.handle.clone() {
        let mut req = TurnRequest::new(agent_prompt, root);
        req.session_id = shell.chat.session.agent_session_id.clone();
        req.model = AgentRuntime::model_id(shell);
        handle.send_turn(req);
        shell.chat.session.is_streaming = true;
        shell.chat.streaming = true;
        shell.turn = crate::shell::TurnState::Streaming;
        return;
    }

    if !runtime.worker_alive {
        runtime.worker_alive = true;
        runtime.pending_prompt = Some(agent_prompt);
        start_agent_worker(
            root,
            runtime.harness.clone(),
            runtime.oneshot_model.clone(),
            agent_tx,
        );
    } else {
        runtime.pending_prompt = Some(agent_prompt);
    }
    shell.chat.session.is_streaming = true;
    shell.chat.streaming = true;
    shell.turn = crate::shell::TurnState::Streaming;
}

/// Send an already-assembled agent prompt (no second orientation pass).
fn send_turn_raw(shell: &mut Shell, runtime: &mut AgentRuntime, prompt: String) {
    let Some(root) = shell.project_root.clone() else {
        return;
    };
    let Some(handle) = runtime.handle.clone() else {
        runtime.pending_prompt = Some(prompt);
        return;
    };
    runtime.last_agent_prompt = Some(prompt.clone());
    let mut req = TurnRequest::new(prompt, root);
    req.session_id = shell.chat.session.agent_session_id.clone();
    req.model = AgentRuntime::model_id(shell);
    handle.send_turn(req);
    shell.chat.session.is_streaming = true;
    shell.chat.streaming = true;
    shell.turn = crate::shell::TurnState::Streaming;
}

/// Fold first-turn orientation into the agent prompt when the session is not
/// resumable. Resume turns pass the user prompt through unchanged.
fn assemble_agent_prompt(shell: &Shell, user_prompt: &str) -> String {
    if shell.chat.session.agent_session_id.is_some() {
        return user_prompt.to_string();
    }
    let mut parts: Vec<String> = Vec::new();
    if let Some(root) = shell.project_root.as_ref()
        && let Some(out) = AgentsMarkdownHook.compute(root)
    {
        parts.push(out.text);
    }
    let scope = session_scope_for_shell(shell);
    if let Some(out) = CurrentScopeHook.compute(&scope) {
        parts.push(out.text);
    }
    parts.push(PATH_REFERENCE_NOTE.to_string());
    parts.push(user_prompt.to_string());
    parts.join("\n\n")
}

/// Build orientation input for the active chat scope.
pub fn session_scope_for_shell(shell: &Shell) -> SessionScope {
    let scope_key = shell.chat.session.scope.clone();
    let kind = scope_kind_for(shell, &scope_key);
    let (change_facts, has_inputs_ledger) = match kind {
        ScopeKind::Change => {
            let facts = shell
                .project_root
                .as_deref()
                .and_then(|root| load_change_facts(root, &scope_key));
            let has_inputs = shell
                .project_root
                .as_deref()
                .is_some_and(|root| duckcore::inputs_ledger::inputs_ledger_is_present(root, &scope_key));
            (facts, has_inputs)
        }
        _ => (None, false),
    };
    SessionScope {
        kind,
        scope_key,
        change_facts,
        has_inputs_ledger,
    }
}

fn scope_kind_for(shell: &Shell, scope_key: &str) -> ScopeKind {
    if let Some(bound) = shell.navigator.bound.as_ref() {
        return match bound {
            BoundChat::Change(_) => ScopeKind::Change,
            BoundChat::Exploration(_) => ScopeKind::Exploration,
            BoundChat::Codex => ScopeKind::Codex,
        };
    }
    match scope_key {
        "codex" => ScopeKind::Codex,
        "caps" => ScopeKind::Caps,
        s if s.starts_with("exploration-") => ScopeKind::Exploration,
        _ => ScopeKind::Change,
    }
}

/// Lightweight change facts from on-disk artifacts (no duckboard ProjectData).
fn load_change_facts(project_root: &Path, name: &str) -> Option<ChangeScopeFacts> {
    let dir = project_root.join("duckspec").join("changes").join(name);
    if !dir.is_dir() {
        return None;
    }
    let has_proposal = dir.join("proposal.md").is_file();
    let has_design = dir.join("design.md").is_file();
    let has_caps = dir.join("caps").is_dir();
    let steps_dir = dir.join("steps");
    let step_files: Vec<PathBuf> = std::fs::read_dir(&steps_dir)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|ext| ext == "md"))
                .collect()
        })
        .unwrap_or_default();
    let step_count = step_files.len();
    let steps_done = step_files
        .iter()
        .filter(|p| step_file_is_complete(p))
        .count();
    let current_review = highest_review_name(&dir.join("reviews"));

    let (phase, next): (&'static str, &[&str]) = if step_count > 0 && steps_done < step_count {
        ("implementing steps", &["ds-apply", "ds-review", "ds-followup"])
    } else if step_count > 0 && steps_done == step_count {
        if current_review.is_some() {
            (
                "all steps complete, review on file",
                &["ds-step", "ds-spec", "ds-review", "ds-followup", "ds-archive"],
            )
        } else {
            (
                "all steps complete",
                &["ds-archive", "ds-review", "ds-followup"],
            )
        }
    } else if has_caps {
        ("specs drafted, steps not yet written", &["ds-step", "ds-archive"])
    } else if has_design {
        ("designing", &["ds-spec", "ds-design"])
    } else if has_proposal {
        ("proposal written", &["ds-design"])
    } else {
        ("new change", &["ds-propose", "ds-design"])
    };

    Some(ChangeScopeFacts {
        phase,
        steps_done,
        step_count,
        active_step_tasks: None,
        next_command: next.first().map(|s| (*s).to_string()),
        current_review,
    })
}

fn step_file_is_complete(path: &Path) -> bool {
    let Ok(text) = std::fs::read_to_string(path) else {
        return false;
    };
    let mut saw_task = false;
    for line in text.lines() {
        let t = line.trim_start();
        if t.starts_with("- [ ]") {
            return false;
        }
        if t.starts_with("- [x]") || t.starts_with("- [X]") {
            saw_task = true;
        }
    }
    saw_task
}

fn highest_review_name(reviews_dir: &Path) -> Option<String> {
    let rd = std::fs::read_dir(reviews_dir).ok()?;
    let mut names: Vec<String> = rd
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_str()?.to_string();
            if name.ends_with(".md") {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    names.sort();
    names.pop()
}

/// Activate a numbered next-token or fast-response hint against the live agent.
pub fn activate_hint(
    shell: &mut Shell,
    runtime: &mut AgentRuntime,
    agent_tx: mpsc::Sender<AgentEvent>,
    action: ChatAction,
) {
    match action {
        ChatAction::ActivatedNext(send) => {
            dispatch_user_submit(shell, runtime, agent_tx, send);
        }
        ChatAction::ActivatedFastResponse(id) => {
            if let (Some(handle), Some(corr)) = (
                runtime.handle.clone(),
                shell.chat.pending_choice_correlation,
            ) {
                handle.answer_user_choice(
                    corr,
                    UserChoiceAnswer::Selected { option_id: id },
                );
                shell.chat.awaiting_user = false;
                shell.chat.pending_choice_correlation = None;
                shell.chat.fast_hints.clear();
            } else {
                // Fallback: freeform user text through the same router.
                dispatch_user_submit(shell, runtime, agent_tx, id);
            }
        }
        ChatAction::Submitted(text) => {
            dispatch_user_submit(shell, runtime, agent_tx, text);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::Shell;
    use duckcore::agent::AgentEvent;
    use duckcore::chat_store::{ContentBlock, Role};
    use duckcore::test_support::{FsTmp, with_home};
    use tokio::sync::mpsc;

    fn agent_tx() -> mpsc::Sender<AgentEvent> {
        mpsc::channel(8).0
    }

    #[test]
    fn content_delta_marks_dirty_and_appends_pending() {
        let mut shell = Shell::new(Some(PathBuf::from("/tmp/p")));
        let mut rt = AgentRuntime::default();
        let dirty = apply_agent_event(
            &mut shell,
            &mut rt,
            AgentEvent::ContentDelta {
                text: "hi".into(),
            },
        );
        assert!(dirty);
        assert!(rt.chat_dirty);
        assert_eq!(shell.chat.session.pending_text, "hi");
        materialize_if_dirty(&mut shell, &mut rt);
        assert!(!rt.chat_dirty);
        assert!(
            shell
                .chat
                .all_rendered_lines()
                .iter()
                .any(|l| l.contains("hi"))
        );
    }

    #[test]
    fn turn_complete_clears_streaming() {
        let mut shell = Shell::new(Some(PathBuf::from("/tmp/p")));
        let mut rt = AgentRuntime::default();
        apply_agent_event(
            &mut shell,
            &mut rt,
            AgentEvent::ContentDelta {
                text: "done".into(),
            },
        );
        apply_agent_event(&mut shell, &mut rt, AgentEvent::TurnComplete);
        assert!(!shell.chat.session.is_streaming);
        assert_eq!(shell.turn, crate::shell::TurnState::Idle);
    }

    // @spec chat/slash-commands Local system submit: Bare /help does not start an agent turn
    #[test]
    fn bare_help_does_not_start_an_agent_turn() {
        let mut shell = Shell::new(Some(PathBuf::from("/tmp/p")));
        let mut rt = AgentRuntime::default();
        dispatch_user_submit(&mut shell, &mut rt, agent_tx(), "/help".into());
        assert!(!shell.chat.agent_drive_requested);
        assert!(rt.pending_prompt.is_none());
        assert!(!rt.worker_alive);
        assert!(!shell.chat.session.is_streaming);
        assert!(!shell.chat.streaming);
        assert_eq!(shell.turn, crate::shell::TurnState::Idle);
    }

    // @spec chat/slash-commands Local system submit: Bare /help records user then system messages
    #[test]
    fn bare_help_records_user_then_system_messages() {
        let tmp = FsTmp::new();
        with_home(tmp.path(), || {
            let root = tmp.path().join("proj");
            std::fs::create_dir_all(&root).unwrap();
            let mut shell = Shell::new(Some(root));
            let mut rt = AgentRuntime::default();
            dispatch_user_submit(&mut shell, &mut rt, agent_tx(), "/help".into());

            assert!(shell.chat.session.messages.len() >= 2);
            let user = &shell.chat.session.messages[0];
            let system = &shell.chat.session.messages[1];
            assert_eq!(user.role, Role::User);
            assert_eq!(system.role, Role::System);
            let user_text = match &user.content[0] {
                ContentBlock::Text(t) => t.as_str(),
                _ => panic!("expected user text"),
            };
            assert_eq!(user_text, "/help");
            let sys_text = match &system.content[0] {
                ContentBlock::Text(t) => t.as_str(),
                _ => panic!("expected system text"),
            };
            assert!(
                sys_text.contains("Running system command `/help`"),
                "{sys_text}"
            );
            assert!(sys_text.contains("//help"), "{sys_text}");
        });
    }

    // @spec chat/slash-commands Double-slash agent escape: Bare //help is an agent turn with prompt /help
    #[test]
    fn bare_double_slash_help_is_agent_turn_with_prompt_help() {
        let mut shell = Shell::new(Some(PathBuf::from("/tmp/p")));
        // Resume id set so first-turn orientation does not rewrite the prompt.
        shell.chat.session.agent_session_id = Some("sess".into());
        // Pretend a worker is already live so we do not `tokio::spawn` in a unit test.
        let mut rt = AgentRuntime {
            worker_alive: true,
            ..Default::default()
        };
        dispatch_user_submit(&mut shell, &mut rt, agent_tx(), "//help".into());
        assert!(shell.chat.agent_drive_requested);
        assert_eq!(rt.pending_prompt.as_deref(), Some("/help"));
        assert_eq!(rt.last_agent_prompt.as_deref(), Some("/help"));
        assert!(shell.chat.session.is_streaming);
        assert_eq!(shell.turn, crate::shell::TurnState::Streaming);
    }

    // @spec chat/slash-commands Double-slash agent escape: Escape keeps typed //help as the user message text
    #[test]
    fn escape_keeps_typed_double_slash_as_user_message() {
        let mut shell = Shell::new(Some(PathBuf::from("/tmp/p")));
        let mut rt = AgentRuntime {
            worker_alive: true,
            ..Default::default()
        };
        dispatch_user_submit(&mut shell, &mut rt, agent_tx(), "//help".into());
        let user = shell
            .chat
            .session
            .messages
            .iter()
            .find(|m| m.role == Role::User)
            .expect("user message");
        let text = match &user.content[0] {
            ContentBlock::Text(t) => t.as_str(),
            _ => panic!("expected text"),
        };
        assert_eq!(text, "//help");
    }

    // @spec session/scope Reliable first-turn delivery: The first turn's message body carries the scope orientation
    #[test]
    fn first_turn_message_body_carries_scope_orientation() {
        let mut shell = Shell::new(Some(PathBuf::from("/tmp/p")));
        shell.chat = crate::chat_pane::ChatPane::new("build-pilot");
        shell.navigator.bound = Some(BoundChat::Change("build-pilot".into()));
        let mut rt = AgentRuntime {
            worker_alive: true,
            ..Default::default()
        };
        assert!(shell.chat.session.agent_session_id.is_none());
        dispatch_user_submit(&mut shell, &mut rt, agent_tx(), "hello".into());
        let body = rt.last_agent_prompt.expect("agent prompt recorded");
        assert!(
            body.contains("build-pilot"),
            "orientation must name the change: {body}"
        );
        assert!(
            body.contains("duckspec/changes/build-pilot/"),
            "orientation must state change path: {body}"
        );
        assert!(
            body.contains("Change-acting commands"),
            "orientation must assert default command target: {body}"
        );
        assert!(
            body.contains("hello"),
            "user prompt still present: {body}"
        );
        assert!(
            body.contains("When referencing project files"),
            "path-reference note rides first turn: {body}"
        );
    }

    // @spec session/scope Reliable first-turn delivery: A resumed session does not repeat the orientation
    #[test]
    fn resumed_session_does_not_repeat_orientation() {
        let mut shell = Shell::new(Some(PathBuf::from("/tmp/p")));
        shell.chat = crate::chat_pane::ChatPane::new("build-pilot");
        shell.navigator.bound = Some(BoundChat::Change("build-pilot".into()));
        shell.chat.session.agent_session_id = Some("sess-abc".into());
        let mut rt = AgentRuntime {
            worker_alive: true,
            ..Default::default()
        };
        dispatch_user_submit(&mut shell, &mut rt, agent_tx(), "follow-up".into());
        let body = rt.last_agent_prompt.expect("agent prompt recorded");
        assert_eq!(body, "follow-up");
        assert!(
            !body.contains("Change-acting commands"),
            "resumed turn must not re-send orientation: {body}"
        );
        assert!(
            !body.contains("When referencing project files"),
            "resumed turn must not re-send path note: {body}"
        );
    }
}
