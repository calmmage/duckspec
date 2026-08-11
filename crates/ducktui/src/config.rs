//! Shared config read/write for the same `config.toml` as duckboard.
//!
//! Only harness/model/oneshot and project recents are owned here. Save merges
//! into the existing document so duckboard-only keys (ui fonts, list prefs, …)
//! survive. Theme stays app-local and is never written.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use duckchat::ModelRef;
use duckcore::paths;
use serde::{Deserialize, Serialize};

/// Maximum entries kept in `projects.recent` (matches duckboard).
const RECENT_CAP: usize = 12;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SharedConfig {
    pub default_model: Option<ModelRef>,
    pub projects: ProjectsConfig,
    pub chat: ChatConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectsConfig {
    /// Most-recently-opened first.
    pub recent: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ChatConfig {
    /// After a turn, run a cheap oneshot for freeform reply chips when eligible.
    pub agent_input_hints: bool,
    /// Preferred oneshot model id per harness (global, not per-project).
    pub oneshot_models: HashMap<String, String>,
}

impl ChatConfig {
    pub fn oneshot_model(&self, harness: &str) -> Option<&str> {
        self.oneshot_models.get(harness).map(String::as_str)
    }

    pub fn set_oneshot_model(&mut self, harness: &str, model: Option<String>) {
        match model {
            Some(m) => {
                self.oneshot_models.insert(harness.to_string(), m);
            }
            None => {
                self.oneshot_models.remove(harness);
            }
        }
    }
}

impl ProjectsConfig {
    /// Promote `path` to the head of the recent list (duckboard-compatible).
    pub fn touch(&mut self, path: &Path) {
        if path.as_os_str().is_empty() {
            return;
        }
        let canonical = path.canonicalize().ok();
        let target = canonical.as_deref().unwrap_or(path);
        self.recent.retain(|p| {
            let pc = p.canonicalize().ok();
            pc.as_deref().unwrap_or(p.as_path()) != target
        });
        self.recent.insert(0, target.to_path_buf());
        if self.recent.len() > RECENT_CAP {
            self.recent.truncate(RECENT_CAP);
        }
    }
}

pub fn load() -> SharedConfig {
    let path = paths::config_path();
    match std::fs::read_to_string(&path) {
        Ok(data) => match toml::from_str(&data) {
            Ok(cfg) => cfg,
            Err(e) => {
                tracing::warn!(path = %path.display(), "failed to parse config: {e}");
                SharedConfig::default()
            }
        },
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            tracing::warn!(path = %path.display(), "failed to read config: {e}");
            SharedConfig::default()
        }
        Err(_) => SharedConfig::default(),
    }
}

/// Persist shared fields, merging into any existing document so unrelated keys
/// (duckboard UI fonts, list prefs, VCS, …) are not wiped.
pub fn save(config: &SharedConfig) -> anyhow::Result<()> {
    let dir = paths::config_dir();
    std::fs::create_dir_all(&dir)?;
    let path = paths::config_path();

    let mut root: toml::map::Map<String, toml::Value> = match std::fs::read_to_string(&path) {
        Ok(data) => data.parse().unwrap_or_default(),
        Err(_) => toml::map::Map::new(),
    };

    match &config.default_model {
        Some(m) => {
            let mut table = toml::map::Map::new();
            table.insert(
                "harness".to_string(),
                toml::Value::String(m.harness.clone()),
            );
            table.insert(
                "model".to_string(),
                toml::Value::String(m.model.clone()),
            );
            root.insert("default_model".to_string(), toml::Value::Table(table));
        }
        None => {
            root.remove("default_model");
        }
    }

    {
        let projects = root
            .entry("projects".to_string())
            .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
        if let toml::Value::Table(t) = projects {
            let recent = config
                .projects
                .recent
                .iter()
                .map(|p| toml::Value::String(p.display().to_string()))
                .collect();
            t.insert("recent".to_string(), toml::Value::Array(recent));
        }
    }

    {
        let chat = root
            .entry("chat".to_string())
            .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
        if let toml::Value::Table(t) = chat {
            t.insert(
                "agent_input_hints".to_string(),
                toml::Value::Boolean(config.chat.agent_input_hints),
            );
            let mut oneshot = toml::map::Map::new();
            for (harness, model) in &config.chat.oneshot_models {
                oneshot.insert(harness.clone(), toml::Value::String(model.clone()));
            }
            t.insert("oneshot_models".to_string(), toml::Value::Table(oneshot));
        }
    }

    let data = toml::to_string_pretty(&toml::Value::Table(root))?;
    std::fs::write(path, data)?;
    Ok(())
}

/// Load, touch `path` into recents, and save. Returns the updated config.
pub fn promote_recent(path: &Path) -> SharedConfig {
    let mut cfg = load();
    cfg.projects.touch(path);
    if let Err(e) = save(&cfg) {
        tracing::warn!("failed to persist recent projects: {e}");
    }
    cfg
}
