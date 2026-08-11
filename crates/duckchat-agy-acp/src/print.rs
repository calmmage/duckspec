//! Cold `agy -p` spawn, log/session-id recovery, and failure classification.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;

use tokio::io::AsyncReadExt;
use tokio::process::{Child, Command};

use crate::agent::AgentError;

/// Env override: spawn this binary as argv[0] with no login-shell wrap (tests /
/// hermetic peers). Production leaves it unset so `agy` runs through the user
/// shell (Finder-launched Duckboard inherits a skeletal PATH).
pub(crate) const AGY_BIN_ENV: &str = "DUCKCHAT_AGY_BIN";

/// Arguments for one cold print-mode invocation.
#[derive(Debug, Clone)]
pub(crate) struct PrintSpawnArgs {
    pub cwd: PathBuf,
    pub prompt: String,
    pub model: Option<String>,
    pub conversation: Option<String>,
    pub log_file: PathBuf,
    pub print_timeout: String,
}

/// Builds a `Command` for `agy -p` (or a scripted test peer).
pub(crate) type AgySpawnFactory = Arc<dyn Fn(&PrintSpawnArgs) -> Command + Send + Sync>;

/// Optional direct binary override (`DUCKCHAT_AGY_BIN`). When set, skip the
/// login-shell wrap and spawn that path as argv[0].
pub(crate) fn agy_bin_override() -> Option<OsString> {
    std::env::var_os(AGY_BIN_ENV).filter(|v| !v.is_empty())
}

/// Argv prefix before print flags: either override binary alone, or
/// `SHELL -ilc 'exec "$@"' duckchat-wrap agy`.
pub(crate) fn agy_argv_prefix() -> Vec<OsString> {
    if let Some(bin) = agy_bin_override() {
        return vec![bin];
    }
    let shell = std::env::var_os("SHELL").unwrap_or_else(|| OsString::from("/bin/zsh"));
    vec![
        shell,
        OsString::from("-ilc"),
        OsString::from(r#"exec "$@""#),
        OsString::from("duckchat-wrap"),
        OsString::from("agy"),
    ]
}

/// Build a production (or override) command for one cold print turn.
pub(crate) fn build_agy_command(args: &PrintSpawnArgs) -> Command {
    let prefix = agy_argv_prefix();
    let mut iter = prefix.into_iter();
    let program = iter.next().expect("agy argv prefix non-empty");
    let mut cmd = Command::new(program);
    for arg in iter {
        cmd.arg(arg);
    }

    cmd.arg("-p")
        .arg("--dangerously-skip-permissions")
        .arg("--print-timeout")
        .arg(&args.print_timeout)
        .arg("--log-file")
        .arg(&args.log_file)
        .current_dir(&args.cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    if let Some(model) = args.model.as_deref() {
        cmd.arg("--model").arg(model);
    }
    if let Some(conv) = args.conversation.as_deref() {
        cmd.arg("--conversation").arg(conv);
    }
    // Prompt last as positional (print mode).
    cmd.arg(&args.prompt);
    cmd
}

/// Default factory: login-shell-wrapped `agy` (or `DUCKCHAT_AGY_BIN` override).
pub(crate) fn default_spawn_factory() -> AgySpawnFactory {
    Arc::new(build_agy_command)
}

/// Count spawns for tests.
#[cfg(test)]
pub(crate) fn counting_factory(
    inner: AgySpawnFactory,
    counter: Arc<std::sync::atomic::AtomicUsize>,
) -> AgySpawnFactory {
    Arc::new(move |args: &PrintSpawnArgs| {
        counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        inner(args)
    })
}

/// Result of a completed print run.
#[derive(Debug)]
pub(crate) struct PrintOutcome {
    pub stdout: String,
    pub stderr: String,
    pub exit_ok: bool,
    pub conversation_id: Option<String>,
}

/// Spawn, concurrently drain stdout/stderr, wait for exit.
///
/// While the child is live, `slot` holds it so cancel can kill mid-flight.
pub(crate) async fn run_print_concurrent(
    factory: &AgySpawnFactory,
    args: &PrintSpawnArgs,
    slot: &mut Option<Child>,
) -> Result<PrintOutcome, AgentError> {
    let mut cmd = factory(args);
    let mut child = cmd
        .spawn()
        .map_err(|e| AgentError::Process(format!("failed to spawn agy: {e}")))?;

    let mut stdout_pipe = child
        .stdout
        .take()
        .ok_or_else(|| AgentError::Process("agy stdout not piped".into()))?;
    let mut stderr_pipe = child
        .stderr
        .take()
        .ok_or_else(|| AgentError::Process("agy stderr not piped".into()))?;

    *slot = Some(child);

    let stdout_task = async {
        let mut buf = Vec::new();
        stdout_pipe.read_to_end(&mut buf).await.map(|_| buf)
    };
    let stderr_task = async {
        let mut buf = Vec::new();
        stderr_pipe.read_to_end(&mut buf).await.map(|_| buf)
    };

    let (stdout_res, stderr_res) = tokio::join!(stdout_task, stderr_task);
    let stdout_bytes =
        stdout_res.map_err(|e| AgentError::Process(format!("read agy stdout: {e}")))?;
    let stderr_bytes =
        stderr_res.map_err(|e| AgentError::Process(format!("read agy stderr: {e}")))?;

    let status = if let Some(mut child) = slot.take() {
        child
            .wait()
            .await
            .map_err(|e| AgentError::Process(format!("wait agy: {e}")))?
    } else {
        // Cancelled: child was taken and killed elsewhere.
        return Err(AgentError::Process("agy turn cancelled".into()));
    };

    let stdout = String::from_utf8_lossy(&stdout_bytes).into_owned();
    let stderr = String::from_utf8_lossy(&stderr_bytes).into_owned();
    let conversation_id =
        parse_conversation_from_log(&args.log_file).or_else(|| read_last_conversation(&args.cwd));

    Ok(PrintOutcome {
        stdout,
        stderr,
        exit_ok: status.success(),
        conversation_id,
    })
}

/// Parse `Print mode: conversation=<uuid>, sending message` from a log file.
pub(crate) fn parse_conversation_from_log(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    for line in text.lines() {
        if let Some(id) = extract_conversation_id(line) {
            return Some(id);
        }
    }
    None
}

fn extract_conversation_id(line: &str) -> Option<String> {
    const MARKER: &str = "conversation=";
    let idx = line.find(MARKER)?;
    let rest = &line[idx + MARKER.len()..];
    let end = rest
        .find(|c: char| c == ',' || c.is_whitespace())
        .unwrap_or(rest.len());
    let id = rest[..end].trim();
    if id.is_empty() {
        None
    } else {
        Some(id.to_string())
    }
}

/// Fallback: `~/.gemini/antigravity-cli/cache/last_conversations.json` keyed by cwd.
pub(crate) fn read_last_conversation(cwd: &Path) -> Option<String> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    let path = home
        .join(".gemini")
        .join("antigravity-cli")
        .join("cache")
        .join("last_conversations.json");
    let text = std::fs::read_to_string(path).ok()?;
    let map: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&text).ok()?;
    let key = cwd.to_string_lossy();
    let key_norm = key.trim_end_matches('/');
    map.get(key_norm)
        .or_else(|| map.get(key.as_ref()))
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

/// True when stderr/stdout indicate resume failure / timeout waiting for response.
pub(crate) fn is_resume_failure(stdout: &str, stderr: &str, was_resume: bool) -> bool {
    if !was_resume {
        return false;
    }
    let blob = format!("{stdout}\n{stderr}").to_lowercase();
    blob.contains("timeout waiting for response")
        || blob.contains("conversation not found")
        || blob.contains("no conversation")
        || blob.contains("session not found")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Serialize env-touching tests on `DUCKCHAT_AGY_BIN`.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn extract_conversation_from_log_line() {
        let line = "I0711 23:04:15.566935 printmode.go:191] Print mode: conversation=70a979a1-6588-40ca-89a9-ff08fab014fc, sending message";
        assert_eq!(
            extract_conversation_id(line).as_deref(),
            Some("70a979a1-6588-40ca-89a9-ff08fab014fc")
        );
    }

    #[test]
    fn resume_failure_detects_timeout() {
        assert!(is_resume_failure(
            "",
            "Error: timeout waiting for response",
            true
        ));
        assert!(!is_resume_failure(
            "",
            "Error: timeout waiting for response",
            false
        ));
    }

    #[test]
    fn production_prefix_uses_login_shell_wrap() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Ensure no override so production prefix is exercised.
        // Safety: serialized by ENV_LOCK; restored below.
        let prev = std::env::var_os(AGY_BIN_ENV);
        // SAFETY: test-only, guarded by ENV_LOCK.
        unsafe { std::env::remove_var(AGY_BIN_ENV) };

        let prefix = agy_argv_prefix();
        let strings: Vec<String> = prefix
            .iter()
            .map(|s| s.to_string_lossy().into_owned())
            .collect();
        assert!(
            strings.iter().any(|s| s == "-ilc"),
            "production prefix must use login-interactive shell, got {strings:?}"
        );
        assert!(
            strings.iter().any(|s| s == "agy"),
            "production prefix must end with agy, got {strings:?}"
        );
        assert_ne!(
            strings.first().map(String::as_str),
            Some("agy"),
            "production argv0 must be the shell, not bare agy"
        );

        match prev {
            Some(v) => unsafe { std::env::set_var(AGY_BIN_ENV, v) },
            None => unsafe { std::env::remove_var(AGY_BIN_ENV) },
        }
    }

    #[test]
    fn env_override_skips_shell_wrap() {
        let _guard = ENV_LOCK.lock().unwrap();
        let prev = std::env::var_os(AGY_BIN_ENV);
        unsafe { std::env::set_var(AGY_BIN_ENV, "/tmp/fake-agy-peer") };

        let prefix = agy_argv_prefix();
        assert_eq!(prefix.len(), 1);
        assert_eq!(
            prefix[0].to_string_lossy(),
            "/tmp/fake-agy-peer",
            "override must be sole argv0 without shell wrap"
        );

        match prev {
            Some(v) => unsafe { std::env::set_var(AGY_BIN_ENV, v) },
            None => unsafe { std::env::remove_var(AGY_BIN_ENV) },
        }
    }

    #[test]
    fn build_agy_command_chains_print_flags_after_prefix() {
        let _guard = ENV_LOCK.lock().unwrap();
        let prev = std::env::var_os(AGY_BIN_ENV);
        unsafe { std::env::set_var(AGY_BIN_ENV, "/tmp/fake-agy-peer") };

        let args = PrintSpawnArgs {
            cwd: PathBuf::from("/proj"),
            prompt: "hello".into(),
            model: Some("Gemini 3.5 Flash (High)".into()),
            conversation: Some("conv-1".into()),
            log_file: PathBuf::from("/tmp/log"),
            print_timeout: "15m".into(),
        };
        let cmd = build_agy_command(&args);
        let program = cmd.as_std().get_program().to_string_lossy().into_owned();
        let argv: Vec<String> = cmd
            .as_std()
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(program, "/tmp/fake-agy-peer");
        assert!(argv.iter().any(|a| a == "-p"));
        assert!(argv.iter().any(|a| a == "--dangerously-skip-permissions"));
        assert!(argv.iter().any(|a| a == "--print-timeout"));
        assert!(argv.iter().any(|a| a == "15m"));
        assert!(argv.iter().any(|a| a == "--log-file"));
        assert!(argv.iter().any(|a| a == "/tmp/log"));
        assert!(argv.iter().any(|a| a == "--model"));
        assert!(argv.iter().any(|a| a == "Gemini 3.5 Flash (High)"));
        assert!(argv.iter().any(|a| a == "--conversation"));
        assert!(argv.iter().any(|a| a == "conv-1"));
        assert_eq!(argv.last().map(String::as_str), Some("hello"));

        match prev {
            Some(v) => unsafe { std::env::set_var(AGY_BIN_ENV, v) },
            None => unsafe { std::env::remove_var(AGY_BIN_ENV) },
        }
    }
}
