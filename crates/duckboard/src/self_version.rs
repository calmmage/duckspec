//! Self-version evaluation: detect when this duckboard binary is behind the
//! open project's declared workspace version (dogfood loop signal only).

use std::path::{Path, PathBuf};

/// Running binary is behind the on-disk package version for a self project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleBuildInfo {
    pub running: String,
    pub disk: String,
    pub project_root: PathBuf,
}

const RUNNING_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Evaluate the open project. `None` when not self, unreadable, or not ahead.
pub fn evaluate(project_root: &Path) -> Option<StaleBuildInfo> {
    evaluate_with_running(project_root, RUNNING_VERSION)
}

fn evaluate_with_running(project_root: &Path, running: &str) -> Option<StaleBuildInfo> {
    if !is_duckboard_project(project_root) {
        return None;
    }
    let disk = read_disk_version(project_root)?;
    let running_triple = parse_triple(running)?;
    let disk_triple = parse_triple(&disk)?;
    if disk_triple <= running_triple {
        return None;
    }
    Some(StaleBuildInfo {
        running: running.to_string(),
        disk,
        project_root: project_root.to_path_buf(),
    })
}

/// `cd '<root>' && just install` with POSIX single-quote escaping.
pub fn install_command(project_root: &Path) -> String {
    format!("cd {} && just install", posix_single_quote(project_root))
}

fn is_duckboard_project(project_root: &Path) -> bool {
    let root_manifest = project_root.join("Cargo.toml");
    if !root_manifest.is_file() {
        return false;
    }
    // Root must at least parse as TOML.
    if read_toml_table(&root_manifest).is_none() {
        return false;
    }
    let pkg_manifest = project_root.join("crates/duckboard/Cargo.toml");
    let table = match read_toml_table(&pkg_manifest) {
        Some(t) => t,
        None => return false,
    };
    table
        .get("package")
        .and_then(|p| p.as_table())
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        == Some("duckboard")
}

fn read_disk_version(project_root: &Path) -> Option<String> {
    let table = read_toml_table(&project_root.join("Cargo.toml"))?;
    if let Some(v) = table
        .get("workspace")
        .and_then(|w| w.as_table())
        .and_then(|w| w.get("package"))
        .and_then(|p| p.as_table())
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
    {
        return Some(v.to_string());
    }
    table
        .get("package")
        .and_then(|p| p.as_table())
        .and_then(|p| p.get("version"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn read_toml_table(path: &Path) -> Option<toml::Table> {
    let content = std::fs::read_to_string(path).ok()?;
    content.parse().ok()
}

/// Parse `major.minor.patch`, optional leading `v`. Invalid → None.
fn parse_triple(raw: &str) -> Option<(u64, u64, u64)> {
    let s = raw.trim().strip_prefix('v').unwrap_or(raw.trim());
    let mut parts = s.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

fn posix_single_quote(path: &Path) -> String {
    // POSIX: wrap in single quotes; replace ' with '\''
    let s = path.to_string_lossy();
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for ch in s.chars() {
        if ch == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_dir() -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        let mut p = std::env::temp_dir();
        p.push(format!("duckboard-self-version-{nanos}-{counter}"));
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn write_duckboard_tree(root: &Path, workspace_version: Option<&str>) {
        let mut root_toml = String::from("[workspace]\nmembers = [\"crates/duckboard\"]\n");
        if let Some(v) = workspace_version {
            root_toml.push_str(&format!("\n[workspace.package]\nversion = \"{v}\"\n"));
        }
        fs::write(root.join("Cargo.toml"), root_toml).unwrap();
        let pkg = root.join("crates/duckboard");
        fs::create_dir_all(&pkg).unwrap();
        fs::write(
            pkg.join("Cargo.toml"),
            "[package]\nname = \"duckboard\"\nversion.workspace = true\n",
        )
        .unwrap();
    }

    // @spec shell/stale-build Self-project only: Non-duckboard project yields no stale signal
    #[test]
    fn non_duckboard_project_yields_no_stale_signal() {
        let root = temp_dir();
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"other\"\nversion = \"9.9.9\"\n",
        )
        .unwrap();
        assert_eq!(evaluate_with_running(&root, "0.1.0"), None);
        let _ = fs::remove_dir_all(&root);
    }

    // @spec shell/stale-build Self-project only: Duckboard package tree is eligible for evaluation
    // @spec shell/stale-build Ahead comparison: Higher disk version is stale
    #[test]
    fn duckboard_tree_with_higher_disk_version_is_stale() {
        let root = temp_dir();
        write_duckboard_tree(&root, Some("0.2.0"));
        let info = evaluate_with_running(&root, "0.1.0").expect("stale signal");
        assert_eq!(info.running, "0.1.0");
        assert_eq!(info.disk, "0.2.0");
        assert_eq!(info.project_root, root);
        let _ = fs::remove_dir_all(&root);
    }

    // @spec shell/stale-build Ahead comparison: Equal versions are not stale
    #[test]
    fn equal_versions_are_not_stale() {
        let root = temp_dir();
        write_duckboard_tree(&root, Some("0.1.0"));
        assert_eq!(evaluate_with_running(&root, "0.1.0"), None);
        let _ = fs::remove_dir_all(&root);
    }

    // @spec shell/stale-build Ahead comparison: Lower disk version is not stale
    #[test]
    fn lower_disk_version_is_not_stale() {
        let root = temp_dir();
        write_duckboard_tree(&root, Some("0.1.0"));
        assert_eq!(evaluate_with_running(&root, "0.2.0"), None);
        let _ = fs::remove_dir_all(&root);
    }

    // @spec shell/stale-build Ahead comparison: Unreadable disk version is not stale
    #[test]
    fn unreadable_disk_version_is_not_stale() {
        let root = temp_dir();
        // Eligible package tree, but no version on the root manifest.
        write_duckboard_tree(&root, None);
        assert_eq!(evaluate_with_running(&root, "0.1.0"), None);
        let _ = fs::remove_dir_all(&root);
    }

    // @spec shell/stale-build Install recipe command: Recipe uses just install under the project root
    #[test]
    fn recipe_uses_just_install_under_project_root() {
        let root = PathBuf::from("/tmp/duckspec");
        let cmd = install_command(&root);
        assert_eq!(cmd, "cd '/tmp/duckspec' && just install");
    }

    // @spec shell/stale-build Install recipe command: Recipe escapes single quotes in the path
    #[test]
    fn recipe_escapes_single_quotes_in_path() {
        let root = PathBuf::from("/tmp/duck's-spec");
        let cmd = install_command(&root);
        assert_eq!(cmd, "cd '/tmp/duck'\\''s-spec' && just install");
    }
}
