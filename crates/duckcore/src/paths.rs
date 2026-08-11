//! Shared config and project data paths (same on-disk layout for every UI).

use std::path::{Path, PathBuf};

thread_local! {
    /// Per-thread redirect for `config_dir`, set by tests so that everything
    /// derived from it (`data_dir`, chats, …) lands in a temp directory instead
    /// of the real `~/.config/duckboard`. Thread-local keeps parallel tests
    /// isolated without a serialization dependency.
    static CONFIG_DIR_OVERRIDE: std::cell::RefCell<Option<PathBuf>> =
        const { std::cell::RefCell::new(None) };
}

/// Redirect `config_dir()` to `dir` for the current thread (tests).
pub fn set_config_dir_override(dir: PathBuf) {
    CONFIG_DIR_OVERRIDE.with(|c| *c.borrow_mut() = Some(dir));
}

pub fn config_dir() -> PathBuf {
    if let Some(dir) = CONFIG_DIR_OVERRIDE.with(|c| c.borrow().clone()) {
        return dir;
    }
    dirs::home_dir()
        .expect("home directory must exist")
        .join(".config")
        .join("duckboard")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn data_dir(project_root: Option<&Path>) -> PathBuf {
    let base = config_dir().join("data");
    match project_root {
        Some(root) => base.join("projects").join(project_hash(root)),
        None => base,
    }
}

pub fn project_hash(project_root: &Path) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    project_root.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
