//! Live `agy models` discovery with a curated fallback table.

use std::ffi::OsString;
use std::process::{Command, Stdio};

use crate::provider::ModelInfo;

use super::HARNESS;

/// Env override: spawn this binary as argv[0] with no login-shell wrap.
/// Same name as print-mode (`duckchat-agy-acp`) so tests share one seam.
pub(crate) const AGY_BIN_ENV: &str = "DUCKCHAT_AGY_BIN";

/// Curated AGY model labels (`agy --model` accepts display labels).
pub(crate) const AGY_MODELS: &[(&str, &str)] = &[
    ("Gemini 3.5 Flash (Low)", "Gemini 3.5 Flash (Low)"),
    ("Gemini 3.5 Flash (Medium)", "Gemini 3.5 Flash (Medium)"),
    ("Gemini 3.5 Flash (High)", "Gemini 3.5 Flash (High)"),
    ("Gemini 3.1 Pro (Low)", "Gemini 3.1 Pro (Low)"),
    ("Gemini 3.1 Pro (High)", "Gemini 3.1 Pro (High)"),
    (
        "Claude Sonnet 4.6 (Thinking)",
        "Claude Sonnet 4.6 (Thinking)",
    ),
    ("Claude Opus 4.6 (Thinking)", "Claude Opus 4.6 (Thinking)"),
    ("GPT-OSS 120B (Medium)", "GPT-OSS 120B (Medium)"),
];

/// Parse `agy models` stdout: skip lines without a tab; keep `id<TAB>display`.
pub(crate) fn parse_agy_models_output(stdout: &str) -> Vec<(String, String)> {
    stdout
        .lines()
        .filter_map(|line| {
            let (id, name) = line.split_once('\t')?;
            let id = id.trim();
            let name = name.trim();
            if id.is_empty() || name.is_empty() {
                return None;
            }
            Some((id.to_string(), name.to_string()))
        })
        .collect()
}

pub(crate) fn curated_models() -> Vec<ModelInfo> {
    models_from_display_names(AGY_MODELS.iter().map(|(_, display)| *display))
}

fn models_from_display_names<'a>(names: impl IntoIterator<Item = &'a str>) -> Vec<ModelInfo> {
    names
        .into_iter()
        .map(|display| ModelInfo {
            harness: HARNESS.to_string(),
            id: display.to_string(),
            display: display.to_string(),
            context_window: None,
        })
        .collect()
}

/// Live rows win when non-empty; otherwise the curated table.
pub(crate) fn resolve_listed_models(live: Option<Vec<(String, String)>>) -> Vec<ModelInfo> {
    match live {
        Some(rows) if !rows.is_empty() => {
            models_from_display_names(rows.iter().map(|(_, display)| display.as_str()))
        }
        _ => curated_models(),
    }
}

/// Run `agy models` and return parsed rows. `None` on spawn/wait/empty failure
/// of the process itself (caller still applies curated fallback).
pub(crate) fn discover_agy_models() -> Option<Vec<(String, String)>> {
    let output = agy_models_command().output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let rows = parse_agy_models_output(&stdout);
    if rows.is_empty() { None } else { Some(rows) }
}

fn agy_models_command() -> Command {
    let prefix = agy_argv_prefix();
    let mut iter = prefix.into_iter();
    let program = iter.next().expect("agy argv prefix non-empty");
    let mut cmd = Command::new(program);
    for arg in iter {
        cmd.arg(arg);
    }
    cmd.arg("models")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd
}

fn agy_argv_prefix() -> Vec<OsString> {
    if let Some(bin) = std::env::var_os(AGY_BIN_ENV).filter(|v| !v.is_empty()) {
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
