//! AGY agent harness.
//!
//! Thin provider over the shared [`crate::acp`] client. Builds a
//! [`duckchat-agy-acp`](crate::agy::agy_acp_launch) [`AgentLaunch`] and opens a
//! shared main ACP runtime. Title/reply use a local oneshot that never spawns
//! `agy` (main-only v1).

mod agent_bin;

pub use agent_bin::{AGY_ACP_BIN, AGY_ACP_ENV, agy_acp_launch, resolve_agy_acp_binary};

use std::path::Path;

use async_trait::async_trait;

use crate::acp::{AcpMainRuntime, AgentLaunch};
use crate::error::Error;
use crate::provider::{Capabilities, ModelInfo, Provider, SlashCommand};
use crate::request::{ReplySuggestionRequest, TitleRequest};
use crate::runtime::{MainRuntime, OneshotKind, OneshotRuntime};
use crate::title::clean_title;

/// Stable harness id shared by every model this provider owns.
const HARNESS: &str = "agy";

/// Curated AGY model labels (`agy --model` accepts display labels).
const AGY_MODELS: &[(&str, &str)] = &[
    ("Gemini 3.5 Flash (Low)", "Gemini 3.5 Flash (Low)"),
    ("Gemini 3.5 Flash (Medium)", "Gemini 3.5 Flash (Medium)"),
    ("Gemini 3.5 Flash (High)", "Gemini 3.5 Flash (High)"),
    ("Gemini 3.1 Pro (Low)", "Gemini 3.1 Pro (Low)"),
    ("Gemini 3.1 Pro (High)", "Gemini 3.1 Pro (High)"),
    ("Claude Sonnet 4.6 (Thinking)", "Claude Sonnet 4.6 (Thinking)"),
    ("Claude Opus 4.6 (Thinking)", "Claude Opus 4.6 (Thinking)"),
    ("GPT-OSS 120B (Medium)", "GPT-OSS 120B (Medium)"),
];

/// [`Provider`] over the owned AGY ACP agent (`duckchat-agy-acp`).
#[derive(Clone)]
pub struct AgyProvider {
    launch: AgentLaunch,
}

impl AgyProvider {
    pub fn new() -> Self {
        Self {
            launch: agy_acp_launch(),
        }
    }

    /// Construct with a custom launch. Test-only seam.
    #[cfg(test)]
    fn with_launch(launch: AgentLaunch) -> Self {
        Self { launch }
    }
}

impl Default for AgyProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for AgyProvider {
    fn id(&self) -> &str {
        HARNESS
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            streaming: false,
            tool_use: false,
            resume: true,
            reasoning: false,
            slash_commands: false,
        }
    }

    fn list_models(&self) -> Vec<ModelInfo> {
        AGY_MODELS
            .iter()
            .map(|(id, display)| ModelInfo {
                harness: self.id().to_string(),
                id: (*id).to_string(),
                display: (*display).to_string(),
                context_window: None,
            })
            .collect()
    }

    fn list_commands(&self, _project_root: &Path) -> Vec<SlashCommand> {
        Vec::new()
    }

    fn open_main_runtime(&self, working_dir: &Path) -> Box<dyn MainRuntime> {
        Box::new(AcpMainRuntime::new(self.launch.clone(), working_dir))
    }

    fn open_oneshot_runtime(
        &self,
        _working_dir: &Path,
        _preferred_model: Option<String>,
    ) -> Box<dyn OneshotRuntime> {
        // Main-only v1: never spawn agy for title/reply chrome.
        Box::new(LocalOneshotRuntime)
    }

    async fn title_summary(&self, req: TitleRequest, _working_dir: &Path) -> Result<String, Error> {
        Ok(local_title_from_message(&req.user_message))
    }

    async fn reply_suggestions(
        &self,
        _req: ReplySuggestionRequest,
        _working_dir: &Path,
    ) -> Result<Vec<String>, Error> {
        Ok(Vec::new())
    }
}

/// Oneshot that never launches AGY: local title heuristic, empty reply text.
struct LocalOneshotRuntime;

#[async_trait]
impl OneshotRuntime for LocalOneshotRuntime {
    async fn ensure_hot(&mut self) -> Result<(), Error> {
        Ok(())
    }

    async fn prompt(&mut self, kind: OneshotKind, text: String) -> Result<String, Error> {
        match kind {
            OneshotKind::Title => {
                let message = extract_user_message_from_title_prompt(&text)
                    .unwrap_or(text.as_str())
                    .to_string();
                Ok(local_title_from_message(&message))
            }
            OneshotKind::ReplySuggest => Ok(String::new()),
        }
    }

    async fn rotate(&mut self) -> Result<(), Error> {
        Ok(())
    }

    async fn shutdown(&mut self) {}
}

fn extract_user_message_from_title_prompt(prompt: &str) -> Option<&str> {
    let start = prompt.find("<user_message>\n")? + "<user_message>\n".len();
    let rest = &prompt[start..];
    let end = rest.find("\n</user_message>")?;
    Some(&rest[..end])
}

/// Cheap local title: first non-empty line, cleaned, truncated to ~6 words.
fn local_title_from_message(message: &str) -> String {
    let first = message.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    let cleaned = clean_title(first);
    let words: Vec<&str> = cleaned.split_whitespace().take(6).collect();
    let joined = words.join(" ");
    if joined.is_empty() {
        "Chat".to_string()
    } else {
        joined
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acp::AgentLaunch;
    use crate::cancel::CancelToken;
    use crate::event::AgentEvent;
    use crate::request::TurnRequest;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use tempfile::TempDir;
    use tokio::process::Command;
    use tokio::sync::mpsc;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn write_executable(path: &std::path::Path, body: &str) {
        let mut f = std::fs::File::create(path).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        f.flush().unwrap();
        let mut perms = std::fs::metadata(path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms).unwrap();
    }

    fn install_fake_acp_agent(dir: &TempDir) -> PathBuf {
        let path = dir.path().join("fake-agy-acp");
        write_executable(
            &path,
            r#"#!/usr/bin/env python3
import json, sys

def send(msg):
    sys.stdout.write(json.dumps(msg) + "\n")
    sys.stdout.flush()

def respond(req_id, result):
    send({"jsonrpc": "2.0", "id": req_id, "result": result})

for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    msg = json.loads(line)
    method = msg.get("method")
    req_id = msg.get("id")
    if method == "initialize":
        respond(req_id, {
            "protocolVersion": 1,
            "agentCapabilities": {"loadSession": True},
            "_meta": {"modelState": {"availableModels": [
                {"modelId": "Gemini 3.5 Flash (High)", "name": "Gemini 3.5 Flash (High)"},
            ]}},
        })
    elif method == "session/new":
        respond(req_id, {"sessionId": "agy-native-sess-1"})
    elif method == "session/load":
        respond(req_id, {})
    elif method == "session/prompt":
        send({
            "jsonrpc": "2.0",
            "method": "session/update",
            "params": {
                "sessionId": "agy-native-sess-1",
                "update": {
                    "sessionUpdate": "agent_message_chunk",
                    "content": {"type": "text", "text": "from-owned-agy-agent"},
                },
            },
        })
        respond(req_id, {"stopReason": "end_turn"})
    elif method == "session/cancel":
        pass
    elif req_id is not None:
        respond(req_id, {})
"#,
        );
        path
    }

    /// @spec harness/agy Owned ACP agent over headless AGY: An AGY turn is driven through the owned ACP agent process
    #[tokio::test]
    async fn agy_turn_driven_through_owned_acp_agent() {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = TempDir::new().unwrap();
        let agent = install_fake_acp_agent(&tmp);

        let launch = AgentLaunch::new({
            let agent = agent.clone();
            move || Command::new(&agent)
        });
        let provider = AgyProvider::with_launch(launch);

        let cmd = agy_acp_launch().command();
        let program = cmd.as_std().get_program().to_string_lossy().into_owned();
        assert!(
            program.contains(AGY_ACP_BIN) || program.ends_with(AGY_ACP_BIN),
            "default AGY launch must target {AGY_ACP_BIN}, got program={program}"
        );

        let mut rt = provider.open_main_runtime(tmp.path());
        let (tx, mut rx) = mpsc::channel(16);
        let req = TurnRequest::new("hello", tmp.path().to_path_buf());
        let outcome = rt
            .run_turn(
                req,
                tx,
                CancelToken::new(),
                crate::event::PendingUserChoices::shared(),
            )
            .await
            .expect("turn through owned ACP agent");
        assert_eq!(outcome.session_id, "agy-native-sess-1");

        let mut saw_content = false;
        while let Ok(ev) = rx.try_recv() {
            if let AgentEvent::ContentDelta { text } = ev {
                if text.contains("from-owned-agy-agent") {
                    saw_content = true;
                }
            }
        }
        assert!(
            saw_content,
            "host ACP client must receive profile content from the owned AGY agent"
        );
    }

    /// @spec harness/agy Main-only title and reply suggestions: Title summary does not spawn agy
    #[tokio::test]
    async fn title_summary_does_not_spawn_agy() {
        let provider = AgyProvider::new();
        let mut oneshot = provider.open_oneshot_runtime(Path::new("/tmp"), None);
        // Local oneshot: ensure_hot is free, prompt never needs a binary.
        oneshot.ensure_hot().await.unwrap();
        let title = oneshot
            .prompt(
                OneshotKind::Title,
                "instruction\n\n<user_message>\nImplement AGY harness support\n</user_message>"
                    .into(),
            )
            .await
            .unwrap();
        assert!(
            title.to_lowercase().contains("implement") || title.to_lowercase().contains("agy"),
            "expected local title from user message, got {title:?}"
        );

        // Provider-level API also avoids agy.
        let via_provider = provider
            .title_summary(
                TitleRequest::new("Fix login redirect flow"),
                Path::new("/tmp"),
            )
            .await
            .unwrap();
        assert!(!via_provider.is_empty());
    }

    /// @spec harness/agy Main-only title and reply suggestions: Reply suggestions are empty without spawning agy
    #[tokio::test]
    async fn reply_suggestions_empty_without_spawning_agy() {
        let provider = AgyProvider::new();
        let mut oneshot = provider.open_oneshot_runtime(Path::new("/tmp"), None);
        let raw = oneshot
            .prompt(OneshotKind::ReplySuggest, "anything".into())
            .await
            .unwrap();
        assert!(raw.is_empty(), "local reply oneshot must return empty text");

        let suggestions = provider
            .reply_suggestions(
                ReplySuggestionRequest::new("What next?"),
                Path::new("/tmp"),
            )
            .await
            .unwrap();
        assert!(suggestions.is_empty());
    }

    /// @spec harness/agy Offered AGY models: Offered AGY models are tagged with the agy harness
    #[test]
    fn offered_agy_models_tagged_with_agy_harness() {
        let provider = AgyProvider::new();
        let models = provider.list_models();
        assert!(!models.is_empty());
        for m in models {
            assert_eq!(m.harness, "agy", "model {} has harness {}", m.id, m.harness);
        }
    }
}
