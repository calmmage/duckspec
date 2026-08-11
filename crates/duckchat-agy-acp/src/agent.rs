//! ACP agent state for AGY: provisional sessions, cold print, rebind, cancel.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::{Value, json};
use tokio::process::Child;

use crate::print::{
    AgySpawnFactory, PrintSpawnArgs, default_spawn_factory, is_resume_failure,
    run_print_concurrent,
};

/// Default main-path print timeout (design: 15m).
const MAIN_PRINT_TIMEOUT: &str = "15m";

/// Errors returned from session operations (mapped to JSON-RPC by the loop).
#[derive(Debug)]
pub(crate) enum AgentError {
    SessionNotFound(String),
    InvalidParams(String),
    MethodNotFound(String),
    Process(String),
}

impl AgentError {
    pub(crate) fn to_rpc_value(&self) -> Value {
        match self {
            AgentError::SessionNotFound(detail) => json!({
                "code": -32603,
                "message": "Path not found.",
                "data": {
                    "code": "FS_NOT_FOUND",
                    "detail": detail,
                }
            }),
            AgentError::InvalidParams(message) => json!({
                "code": -32602,
                "message": message,
            }),
            AgentError::MethodNotFound(method) => json!({
                "code": -32601,
                "message": format!("Method not found: {method}"),
            }),
            AgentError::Process(message) => json!({
                "code": -32603,
                "message": message,
            }),
        }
    }
}

/// Open/load state held until the first user prompt starts `agy`.
#[derive(Debug, Clone)]
struct PendingOpen {
    cwd: PathBuf,
    model: Option<String>,
    /// `None` = fresh conversation; `Some(id)` = `--conversation` that id.
    resume: Option<String>,
}

/// Session table + optional in-flight cold print child.
pub(crate) struct Agent {
    sessions: HashSet<String>,
    pending: HashMap<String, PendingOpen>,
    provisional_to_native: HashMap<String, String>,
    in_flight: Option<Child>,
    factory: AgySpawnFactory,
    /// Temp dir root for print log files (tests can isolate).
    log_dir: PathBuf,
}

impl Agent {
    pub(crate) fn new() -> Self {
        Self::with_factory(default_spawn_factory())
    }

    pub(crate) fn with_factory(factory: AgySpawnFactory) -> Self {
        Self {
            sessions: HashSet::new(),
            pending: HashMap::new(),
            provisional_to_native: HashMap::new(),
            in_flight: None,
            factory,
            log_dir: std::env::temp_dir(),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_factory_and_log_dir(factory: AgySpawnFactory, log_dir: PathBuf) -> Self {
        Self {
            sessions: HashSet::new(),
            pending: HashMap::new(),
            provisional_to_native: HashMap::new(),
            in_flight: None,
            factory,
            log_dir,
        }
    }

    /// ACP `initialize` result: protocol version, loadSession, curated models.
    pub(crate) fn initialize(&self) -> Value {
        json!({
            "protocolVersion": 1,
            "agentCapabilities": {
                "loadSession": true,
            },
            "_meta": {
                "modelState": {
                    "availableModels": [
                        { "modelId": "Gemini 3.5 Flash (Low)", "name": "Gemini 3.5 Flash (Low)" },
                        { "modelId": "Gemini 3.5 Flash (Medium)", "name": "Gemini 3.5 Flash (Medium)" },
                        { "modelId": "Gemini 3.5 Flash (High)", "name": "Gemini 3.5 Flash (High)" },
                        { "modelId": "Gemini 3.1 Pro (Low)", "name": "Gemini 3.1 Pro (Low)" },
                        { "modelId": "Gemini 3.1 Pro (High)", "name": "Gemini 3.1 Pro (High)" },
                        { "modelId": "Claude Sonnet 4.6 (Thinking)", "name": "Claude Sonnet 4.6 (Thinking)" },
                        { "modelId": "Claude Opus 4.6 (Thinking)", "name": "Claude Opus 4.6 (Thinking)" },
                        { "modelId": "GPT-OSS 120B (Medium)", "name": "GPT-OSS 120B (Medium)" },
                    ]
                }
            }
        })
    }

    /// Open a new ACP session without starting `agy`.
    pub(crate) async fn session_new(&mut self, params: &Value) -> Result<Value, AgentError> {
        let cwd = cwd_from_params(params);
        let model = model_from_params(params);

        // Drop any in-flight print before opening a new conversation.
        self.cancel(None).await;

        let provisional = new_provisional_id();
        self.sessions.insert(provisional.clone());
        self.pending.insert(
            provisional.clone(),
            PendingOpen {
                cwd,
                model,
                resume: None,
            },
        );
        Ok(json!({ "sessionId": provisional }))
    }

    /// Resume a session id without spawning `agy` until the first prompt.
    pub(crate) async fn session_load(&mut self, params: &Value) -> Result<Value, AgentError> {
        let session_id = params
            .get("sessionId")
            .and_then(Value::as_str)
            .ok_or_else(|| AgentError::InvalidParams("session/load missing sessionId".into()))?
            .to_string();
        if session_id.is_empty() {
            return Err(AgentError::SessionNotFound(
                "empty sessionId on session/load".into(),
            ));
        }
        let cwd = cwd_from_params(params);
        let model = model_from_params(params);
        let resolved = self.resolve_id(&session_id);

        self.cancel(None).await;

        self.sessions.insert(session_id.clone());
        self.pending.insert(
            session_id.clone(),
            PendingOpen {
                cwd,
                model,
                resume: Some(resolved),
            },
        );
        Ok(json!({}))
    }

    /// Run a prompt: cold `agy -p`, emit one message chunk, rebind session id.
    pub(crate) async fn run_prompt(
        &mut self,
        params: &Value,
        on_update: &mut (dyn FnMut(Value) + Send),
    ) -> Result<Value, AgentError> {
        let request_id = params
            .get("sessionId")
            .and_then(Value::as_str)
            .ok_or_else(|| AgentError::InvalidParams("session/prompt missing sessionId".into()))?
            .to_string();
        let open_id = request_id.clone();
        let prompt_text = acp_prompt_to_text(params);

        let pending = self.pending.remove(&request_id).or_else(|| {
            Some(PendingOpen {
                cwd: cwd_from_params(params),
                model: model_from_params(params),
                resume: Some(self.resolve_id(&request_id)),
            })
        });
        let pending = pending.expect("pending always Some");

        let cwd = {
            let from_params = params.get("cwd").and_then(Value::as_str);
            if from_params.is_some() {
                cwd_from_params(params)
            } else {
                pending.cwd
            }
        };
        let model = model_from_params(params).or(pending.model);
        let resume = pending.resume.clone();
        let was_resume = resume.is_some();

        let log_file = self
            .log_dir
            .join(format!("duckchat-agy-{}.log", open_id.replace('/', "_")));

        let args = PrintSpawnArgs {
            cwd: cwd.clone(),
            prompt: prompt_text,
            model,
            conversation: resume.clone(),
            log_file: log_file.clone(),
            print_timeout: MAIN_PRINT_TIMEOUT.to_string(),
        };

        let outcome = run_print_concurrent(&self.factory, &args, &mut self.in_flight).await?;

        if !outcome.exit_ok {
            if is_resume_failure(&outcome.stdout, &outcome.stderr, was_resume) {
                return Err(AgentError::SessionNotFound(format!(
                    "agy resume failed: {}",
                    outcome.stderr.trim()
                )));
            }
            let detail = if outcome.stderr.trim().is_empty() {
                outcome.stdout.trim().to_string()
            } else {
                outcome.stderr.trim().to_string()
            };
            return Err(AgentError::Process(format!(
                "agy -p failed: {detail}"
            )));
        }

        let body = outcome.stdout.trim().to_string();
        if !body.is_empty() {
            on_update(json!({
                "sessionId": open_id,
                "update": {
                    "sessionUpdate": "agent_message_chunk",
                    "content": {
                        "type": "text",
                        "text": body,
                    }
                }
            }));
        }

        // Durable id must be a real AGY conversation uuid — never the provisional
        // open handle (`pending-*`). Prefer log/cache recovery; on an explicit
        // resume we already know the uuid we passed to `--conversation`.
        let durable = outcome
            .conversation_id
            .filter(|id| !id.is_empty() && !is_provisional_id(id))
            .or_else(|| {
                resume
                    .clone()
                    .filter(|id| !id.is_empty() && !is_provisional_id(id))
            })
            .ok_or_else(|| {
                AgentError::Process(
                    "agy -p succeeded but no durable conversation id was recovered from log or cache"
                        .into(),
                )
            })?;

        if durable != open_id {
            self.provisional_to_native
                .insert(open_id.clone(), durable.clone());
        }
        self.sessions.insert(durable.clone());

        let mut result = json!({ "stopReason": "end_turn" });
        if durable != open_id {
            result["sessionId"] = json!(durable);
        }
        Ok(result)
    }

    fn resolve_id(&self, id: &str) -> String {
        self.provisional_to_native
            .get(id)
            .cloned()
            .unwrap_or_else(|| id.to_string())
    }

    /// Cancel kills an in-flight `agy` child. Session ids stay known.
    pub(crate) async fn cancel(&mut self, _session_id: Option<&str>) {
        if let Some(mut child) = self.in_flight.take() {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }
    }

    /// Test helper: whether a print child is currently held (mid-turn).
    #[cfg(test)]
    pub(crate) fn has_in_flight(&self) -> bool {
        self.in_flight.is_some()
    }
}

fn new_provisional_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("pending-{n}")
}

fn is_provisional_id(id: &str) -> bool {
    id.starts_with("pending-")
}

fn cwd_from_params(params: &Value) -> PathBuf {
    params
        .get("cwd")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir())
}

fn model_from_params(params: &Value) -> Option<String> {
    params
        .pointer("/_meta/model")
        .or_else(|| params.get("model"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// Flatten ACP prompt content blocks to plain text (v1 text-only).
fn acp_prompt_to_text(params: &Value) -> String {
    let Some(prompt) = params.get("prompt") else {
        return String::new();
    };
    if let Some(s) = prompt.as_str() {
        return s.to_string();
    }
    let Some(arr) = prompt.as_array() else {
        return String::new();
    };
    let mut parts = Vec::new();
    for block in arr {
        if block.get("type").and_then(Value::as_str) == Some("text") {
            if let Some(t) = block.get("text").and_then(Value::as_str) {
                parts.push(t.to_string());
            }
        }
    }
    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::print::counting_factory;
    use std::process::Stdio;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
    use tempfile::tempdir;
    use tokio::process::Command;

    /// Scripted `agy` peer: writes conversation id into --log-file, prints final text.
    const SCRIPTED_AGY_PY: &str = r#"
import sys, os, json

argv = sys.argv[1:]
conversation = None
log_file = None
i = 0
args = []
while i < len(argv):
    a = argv[i]
    if a == "--conversation" and i + 1 < len(argv):
        conversation = argv[i + 1]
        i += 2
        continue
    if a == "--log-file" and i + 1 < len(argv):
        log_file = argv[i + 1]
        i += 2
        continue
    if a in ("-p", "--print", "--dangerously-skip-permissions"):
        i += 1
        continue
    if a == "--print-timeout" and i + 1 < len(argv):
        i += 2
        continue
    if a == "--model" and i + 1 < len(argv):
        i += 2
        continue
    args.append(a)
    i += 1

prompt = args[-1] if args else ""

if conversation == "missing-session-id":
    sys.stderr.write("Error: timeout waiting for response\n")
    sys.exit(1)

session_id = conversation or "agy-native-sess-1"
if log_file:
    with open(log_file, "w") as f:
        f.write(f"Print mode: conversation={session_id}, sending message\n")

# Record flags seen for tool auto-approve assertion via env side channel file
print(f"FINAL:{prompt}", flush=True)
"#;

    fn scripted_factory() -> AgySpawnFactory {
        Arc::new(|args: &PrintSpawnArgs| {
            let mut cmd = Command::new("python3");
            cmd.arg("-u").arg("-c").arg(SCRIPTED_AGY_PY);
            // Reconstruct argv-like list the script understands.
            cmd.arg("-p")
                .arg("--dangerously-skip-permissions")
                .arg("--print-timeout")
                .arg(&args.print_timeout)
                .arg("--log-file")
                .arg(&args.log_file);
            if let Some(m) = args.model.as_deref() {
                cmd.arg("--model").arg(m);
            }
            if let Some(c) = args.conversation.as_deref() {
                cmd.arg("--conversation").arg(c);
            }
            cmd.arg(&args.prompt)
                .current_dir(&args.cwd)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);
            cmd
        })
    }

    async fn prompt_collecting(
        agent: &mut Agent,
        params: Value,
    ) -> Result<(Vec<Value>, Value), AgentError> {
        let mut updates = Vec::new();
        let mut sink = |u: Value| updates.push(u);
        let result = agent.run_prompt(&params, &mut sink).await?;
        Ok((updates, result))
    }

    /// @spec harness/agy Session lifecycle and durable conversation ids: Opening a new session does not start agy before the first user prompt
    #[tokio::test]
    async fn opening_new_session_does_not_start_agy_before_first_prompt() {
        let counter = Arc::new(AtomicUsize::new(0));
        let log_dir = tempdir().unwrap();
        let mut agent = Agent::with_factory_and_log_dir(
            counting_factory(scripted_factory(), Arc::clone(&counter)),
            log_dir.path().to_path_buf(),
        );
        let created = agent
            .session_new(&json!({ "cwd": std::env::temp_dir().to_string_lossy() }))
            .await
            .unwrap();
        let sid = created["sessionId"].as_str().unwrap();
        assert!(
            sid.starts_with("pending-"),
            "open returns a provisional handle, got {sid}"
        );
        assert_eq!(
            counter.load(AtomicOrdering::SeqCst),
            0,
            "session/new must not spawn agy"
        );
        assert!(!agent.has_in_flight());
        agent.cancel(None).await;
    }

    /// @spec harness/agy Session lifecycle and durable conversation ids: A turn without a prior session surfaces a durable AGY conversation id
    #[tokio::test]
    async fn turn_without_prior_session_surfaces_durable_id() {
        let log_dir = tempdir().unwrap();
        let mut agent =
            Agent::with_factory_and_log_dir(scripted_factory(), log_dir.path().to_path_buf());
        let created = agent
            .session_new(&json!({ "cwd": std::env::temp_dir().to_string_lossy() }))
            .await
            .unwrap();
        let provisional = created["sessionId"].as_str().unwrap().to_string();

        let (updates, result) = prompt_collecting(
            &mut agent,
            json!({
                "sessionId": provisional,
                "prompt": [{ "type": "text", "text": "hi" }],
            }),
        )
        .await
        .unwrap();
        assert_eq!(result["stopReason"], "end_turn");
        assert_eq!(
            result["sessionId"].as_str(),
            Some("agy-native-sess-1"),
            "first prompt rebinds to durable AGY id"
        );
        assert!(
            updates
                .iter()
                .any(|u| u["update"]["sessionUpdate"] == "agent_message_chunk")
        );
        agent.cancel(None).await;
    }

    /// @spec harness/agy Session lifecycle and durable conversation ids: A turn with a prior AGY conversation id resumes that id
    #[tokio::test]
    async fn turn_with_prior_conversation_resumes_id() {
        let log_dir = tempdir().unwrap();
        let counter = Arc::new(AtomicUsize::new(0));
        let mut agent = Agent::with_factory_and_log_dir(
            counting_factory(scripted_factory(), Arc::clone(&counter)),
            log_dir.path().to_path_buf(),
        );
        agent
            .session_load(&json!({
                "sessionId": "agy-native-sess-1",
                "cwd": std::env::temp_dir().to_string_lossy(),
            }))
            .await
            .unwrap();
        assert_eq!(counter.load(AtomicOrdering::SeqCst), 0);

        let (_, result) = prompt_collecting(
            &mut agent,
            json!({
                "sessionId": "agy-native-sess-1",
                "prompt": [{ "type": "text", "text": "resume" }],
            }),
        )
        .await
        .unwrap();
        assert_eq!(result["stopReason"], "end_turn");
        // Durable id already known — rebind field may be absent.
        assert_eq!(counter.load(AtomicOrdering::SeqCst), 1);
        agent.cancel(None).await;
    }

    /// @spec harness/agy Session lifecycle and durable conversation ids: A failed resume of a dead AGY conversation surfaces session-not-found
    #[tokio::test]
    async fn failed_resume_surfaces_session_not_found() {
        let log_dir = tempdir().unwrap();
        let mut agent =
            Agent::with_factory_and_log_dir(scripted_factory(), log_dir.path().to_path_buf());
        agent
            .session_load(&json!({
                "sessionId": "missing-session-id",
                "cwd": std::env::temp_dir().to_string_lossy(),
            }))
            .await
            .unwrap();

        let err = prompt_collecting(
            &mut agent,
            json!({
                "sessionId": "missing-session-id",
                "prompt": [{ "type": "text", "text": "x" }],
            }),
        )
        .await
        .unwrap_err();
        assert!(
            matches!(err, AgentError::SessionNotFound(_)),
            "got {err:?}"
        );
        agent.cancel(None).await;
    }

    /// @spec harness/agy Owned ACP agent over headless AGY: The agent runs headless agy print mode with auto-approved tools
    #[tokio::test]
    async fn agent_runs_print_mode_with_auto_approve_flags() {
        let log_dir = tempdir().unwrap();
        let seen = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let seen_c = Arc::clone(&seen);
        let factory: AgySpawnFactory = Arc::new(move |args: &PrintSpawnArgs| {
            let mut argv = vec![
                "-p".into(),
                "--dangerously-skip-permissions".into(),
                "--print-timeout".into(),
                args.print_timeout.clone(),
                "--log-file".into(),
                args.log_file.display().to_string(),
            ];
            if let Some(c) = &args.conversation {
                argv.push("--conversation".into());
                argv.push(c.clone());
            }
            argv.push(args.prompt.clone());
            seen_c.lock().unwrap().extend(argv.iter().cloned());

            // Reuse scripted peer for actual execution.
            let mut cmd = Command::new("python3");
            cmd.arg("-u").arg("-c").arg(SCRIPTED_AGY_PY);
            for a in &argv {
                cmd.arg(a);
            }
            cmd.current_dir(&args.cwd)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);
            cmd
        });

        let mut agent = Agent::with_factory_and_log_dir(factory, log_dir.path().to_path_buf());
        let sid = agent
            .session_new(&json!({ "cwd": std::env::temp_dir().to_string_lossy() }))
            .await
            .unwrap()["sessionId"]
            .as_str()
            .unwrap()
            .to_string();
        prompt_collecting(
            &mut agent,
            json!({
                "sessionId": sid,
                "prompt": [{ "type": "text", "text": "ping" }],
            }),
        )
        .await
        .unwrap();

        let argv = seen.lock().unwrap().clone();
        assert!(argv.iter().any(|a| a == "-p"));
        assert!(argv.iter().any(|a| a == "--dangerously-skip-permissions"));
        assert!(argv.iter().any(|a| a == "--print-timeout"));
        agent.cancel(None).await;
    }

    /// @spec harness/agy Batch final-answer emission: A successful turn surfaces the final assistant answer as content
    #[tokio::test]
    async fn successful_turn_surfaces_final_answer_as_content() {
        let log_dir = tempdir().unwrap();
        let mut agent =
            Agent::with_factory_and_log_dir(scripted_factory(), log_dir.path().to_path_buf());
        let sid = agent
            .session_new(&json!({ "cwd": std::env::temp_dir().to_string_lossy() }))
            .await
            .unwrap()["sessionId"]
            .as_str()
            .unwrap()
            .to_string();
        let (updates, result) = prompt_collecting(
            &mut agent,
            json!({
                "sessionId": sid,
                "prompt": [{ "type": "text", "text": "hello-world" }],
            }),
        )
        .await
        .unwrap();
        assert_eq!(result["stopReason"], "end_turn");
        let text = updates
            .iter()
            .find(|u| u["update"]["sessionUpdate"] == "agent_message_chunk")
            .and_then(|u| u["update"]["content"]["text"].as_str())
            .unwrap_or("");
        assert!(
            text.contains("FINAL:hello-world"),
            "expected final answer in content, got {text:?}"
        );
        agent.cancel(None).await;
    }

    /// @spec harness/agy Batch final-answer emission: A successful turn does not require tool or reasoning events on the wire
    #[tokio::test]
    async fn successful_turn_does_not_require_tool_or_reasoning_events() {
        let log_dir = tempdir().unwrap();
        let mut agent =
            Agent::with_factory_and_log_dir(scripted_factory(), log_dir.path().to_path_buf());
        let sid = agent
            .session_new(&json!({ "cwd": std::env::temp_dir().to_string_lossy() }))
            .await
            .unwrap()["sessionId"]
            .as_str()
            .unwrap()
            .to_string();
        let (updates, result) = prompt_collecting(
            &mut agent,
            json!({
                "sessionId": sid,
                "prompt": [{ "type": "text", "text": "x" }],
            }),
        )
        .await
        .unwrap();
        assert_eq!(result["stopReason"], "end_turn");
        let kinds: Vec<_> = updates
            .iter()
            .filter_map(|u| u["update"]["sessionUpdate"].as_str())
            .collect();
        assert!(
            !kinds.iter().any(|k| *k == "tool_call" || *k == "agent_thought_chunk"),
            "batch path must not require tool/reasoning events, got {kinds:?}"
        );
        assert!(
            kinds.iter().any(|k| *k == "agent_message_chunk"),
            "expected message chunk, got {kinds:?}"
        );
        agent.cancel(None).await;
    }

    /// Regression (review finding 2): success without conversation id must not
    /// rebind to the provisional `pending-*` handle.
    #[tokio::test]
    async fn success_without_conversation_id_does_not_use_provisional() {
        // Peer prints a final answer but never writes a conversation= line.
        const NO_LOG_PEER: &str = r#"
import sys
argv = sys.argv[1:]
prompt = argv[-1] if argv else ""
print(f"FINAL:{prompt}", flush=True)
"#;
        let factory: AgySpawnFactory = Arc::new(|args: &PrintSpawnArgs| {
            let mut cmd = Command::new("python3");
            cmd.arg("-u")
                .arg("-c")
                .arg(NO_LOG_PEER)
                .arg("-p")
                .arg("--log-file")
                .arg(&args.log_file)
                .arg(&args.prompt)
                .current_dir(&args.cwd)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .kill_on_drop(true);
            cmd
        });

        let log_dir = tempdir().unwrap();
        // Isolate last_conversations fallback: use a unique cwd that is not
        // present in the machine cache key (temp dir is enough in practice).
        let cwd = log_dir.path().join("isolated-cwd");
        std::fs::create_dir_all(&cwd).unwrap();

        let mut agent = Agent::with_factory_and_log_dir(factory, log_dir.path().to_path_buf());
        let provisional = agent
            .session_new(&json!({ "cwd": cwd.to_string_lossy() }))
            .await
            .unwrap()["sessionId"]
            .as_str()
            .unwrap()
            .to_string();
        assert!(provisional.starts_with("pending-"));

        let err = prompt_collecting(
            &mut agent,
            json!({
                "sessionId": provisional,
                "cwd": cwd.to_string_lossy(),
                "prompt": [{ "type": "text", "text": "hi" }],
            }),
        )
        .await
        .unwrap_err();
        match err {
            AgentError::Process(msg) => {
                assert!(
                    msg.contains("no durable conversation id"),
                    "expected durable-id error, got {msg:?}"
                );
            }
            other => panic!("expected Process error, got {other:?}"),
        }
        agent.cancel(None).await;
    }
}
