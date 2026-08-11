//! Source fingerprint for dogfood stale-build detection.
//!
//! Shared by `build.rs` (bake into the binary) and runtime (read the open tree).
//! Prefer jj working-copy change id; fall back to short git commit. Append `+`
//! when the working copy has local changes. Empty string when neither VCS works.

use std::path::Path;
use std::process::Command;

/// Resolve a source fingerprint for `root` (repo root).
pub fn resolve_source_id(root: &Path) -> String {
    if let Some(id) = jj_change_id(root) {
        return with_dirty(id, jj_dirty(root));
    }
    if let Some(id) = git_commit_short(root) {
        return with_dirty(id, git_dirty(root));
    }
    String::new()
}

fn with_dirty(id: String, dirty: bool) -> String {
    if dirty {
        format!("{id}+")
    } else {
        id
    }
}

fn jj_change_id(root: &Path) -> Option<String> {
    let out = Command::new("jj")
        .args([
            "log",
            "-r",
            "@",
            "--no-graph",
            "-T",
            "change_id.short() ++ \"\\n\"",
        ])
        .current_dir(root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout);
    let id = s.lines().next()?.trim();
    if id.is_empty() {
        None
    } else {
        Some(id.to_string())
    }
}

fn jj_dirty(root: &Path) -> bool {
    // `jj diff --summary` non-empty means working-copy changes vs parent.
    let out = Command::new("jj")
        .args(["diff", "--summary"])
        .current_dir(root)
        .output();
    match out {
        Ok(o) if o.status.success() => !String::from_utf8_lossy(&o.stdout).trim().is_empty(),
        _ => false,
    }
}

fn git_commit_short(root: &Path) -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let id = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if id.is_empty() {
        None
    } else {
        Some(id)
    }
}

fn git_dirty(root: &Path) -> bool {
    let out = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output();
    match out {
        Ok(o) if o.status.success() => !String::from_utf8_lossy(&o.stdout).trim().is_empty(),
        _ => false,
    }
}
