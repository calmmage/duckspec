//! Self-version evaluation: detect when this duckboard binary is behind the
//! open project's package version or source fingerprint (dogfood signal only).

use std::path::{Path, PathBuf};

use crate::source_fingerprint;

/// Running binary is behind the open self-project (version and/or source).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleBuildInfo {
    /// Display string for the running binary (version; source when relevant).
    pub running: String,
    /// Display string for the open tree (version; source when relevant).
    pub disk: String,
    pub project_root: PathBuf,
}

const RUNNING_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Baked at compile time by `build.rs`. Empty when VCS was unavailable.
const RUNNING_SOURCE: &str = env!("DUCKBOARD_SOURCE_ID");

/// Evaluate the open project. `None` when not self, unreadable, or not ahead.
pub fn evaluate(project_root: &Path) -> Option<StaleBuildInfo> {
    let disk_source = read_disk_source_id(project_root).unwrap_or_default();
    evaluate_with(
        project_root,
        RUNNING_VERSION,
        RUNNING_SOURCE,
        Some(disk_source.as_str()),
    )
}

/// Testable entry: optional `disk_source_override` skips live VCS (`None` = read disk).
fn evaluate_with(
    project_root: &Path,
    running_version: &str,
    running_source: &str,
    disk_source_override: Option<&str>,
) -> Option<StaleBuildInfo> {
    if !is_duckboard_project(project_root) {
        return None;
    }

    let disk_version = read_disk_version(project_root);
    let disk_source = match disk_source_override {
        Some(s) => s.to_string(),
        None => read_disk_source_id(project_root).unwrap_or_default(),
    };

    let version_ahead = match (disk_version.as_deref(), parse_triple(running_version)) {
        (Some(dv), Some(rv)) => parse_triple(dv).is_some_and(|d| d > rv),
        _ => false,
    };

    let source_ahead = !running_source.is_empty()
        && !disk_source.is_empty()
        && running_source != disk_source;

    if !version_ahead && !source_ahead {
        return None;
    }

    let disk_ver_display = disk_version.as_deref().unwrap_or("?");
    Some(StaleBuildInfo {
        running: display_label(running_version, running_source, source_ahead),
        disk: display_label(disk_ver_display, &disk_source, source_ahead),
        project_root: project_root.to_path_buf(),
    })
}

fn display_label(version: &str, source: &str, include_source: bool) -> String {
    if include_source && !source.is_empty() {
        format!("{version} ({source})")
    } else {
        version.to_string()
    }
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

/// Same rules as bake-time (`source_fingerprint` via `build.rs`).
fn read_disk_source_id(project_root: &Path) -> Option<String> {
    let id = source_fingerprint::resolve_source_id(project_root);
    if id.is_empty() {
        None
    } else {
        Some(id)
    }
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

/// Project-root justfile path for recipe shape tests.
#[cfg(test)]
fn project_justfile_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../justfile")
}

/// Body of the `install:` recipe in the root justfile (lines until next recipe).
#[cfg(test)]
fn install_recipe_body(justfile: &str) -> Option<String> {
    let mut lines = justfile.lines();
    // Find a line that is exactly `install:` (recipe header).
    loop {
        let line = lines.next()?;
        if line.trim() == "install:" {
            break;
        }
    }
    let mut body = String::new();
    for line in lines {
        // Next recipe: non-empty, non-comment line that does not start with whitespace
        // and ends with `:` (just recipe header), or starts with `#` at col 0 only if
        // we already finished — stop on unindented non-shebang content that looks
        // like a new recipe `name:`.
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            // blank lines can appear inside bash recipes; keep going if next is indented
            body.push('\n');
            continue;
        }
        if !line.starts_with(char::is_whitespace)
            && !line.starts_with('#')
            && trimmed.ends_with(':')
            && !trimmed.contains(' ')
        {
            break;
        }
        // Comment lines at column 0 that document the next section end the recipe
        // only when followed by a recipe — keep `#` lines that are part of docs
        // before the next recipe: if line starts with `# ` at col 0 after body
        // started with content, treat as end of recipe block when body already
        // has non-empty content and line is a section comment.
        if !line.starts_with(char::is_whitespace)
            && line.starts_with('#')
            && !body.trim().is_empty()
        {
            break;
        }
        body.push_str(line);
        body.push('\n');
    }
    let body = body.trim().to_string();
    if body.is_empty() {
        None
    } else {
        Some(body)
    }
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

    fn eval(
        root: &Path,
        running_version: &str,
        running_source: &str,
        disk_source: &str,
    ) -> Option<StaleBuildInfo> {
        evaluate_with(
            root,
            running_version,
            running_source,
            Some(disk_source),
        )
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
        assert_eq!(eval(&root, "0.1.0", "", ""), None);
        let _ = fs::remove_dir_all(&root);
    }

    // @spec shell/stale-build Self-project only: Duckboard package tree is eligible for evaluation
    // @spec shell/stale-build Ahead comparison: Higher disk version is stale
    #[test]
    fn duckboard_tree_with_higher_disk_version_is_stale() {
        let root = temp_dir();
        write_duckboard_tree(&root, Some("0.2.0"));
        let info = eval(&root, "0.1.0", "", "").expect("stale signal");
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
        // Matching empty sources — no source-ahead either.
        assert_eq!(eval(&root, "0.1.0", "", ""), None);
        // Matching non-empty sources — still not stale.
        assert_eq!(eval(&root, "0.1.0", "abc", "abc"), None);
        let _ = fs::remove_dir_all(&root);
    }

    // @spec shell/stale-build Ahead comparison: Lower disk version is not stale
    #[test]
    fn lower_disk_version_is_not_stale() {
        let root = temp_dir();
        write_duckboard_tree(&root, Some("0.1.0"));
        assert_eq!(eval(&root, "0.2.0", "", ""), None);
        let _ = fs::remove_dir_all(&root);
    }

    // @spec shell/stale-build Ahead comparison: Unreadable disk version is not stale
    #[test]
    fn unreadable_disk_version_is_not_stale() {
        let root = temp_dir();
        // Eligible package tree, but no version on the root manifest.
        write_duckboard_tree(&root, None);
        assert_eq!(eval(&root, "0.1.0", "", ""), None);
        let _ = fs::remove_dir_all(&root);
    }

    // @spec shell/stale-build Ahead comparison: Differing source fingerprints are stale at equal version
    #[test]
    fn differing_source_fingerprints_are_stale_at_equal_version() {
        let root = temp_dir();
        write_duckboard_tree(&root, Some("0.1.0"));
        let info = eval(&root, "0.1.0", "abc", "def+").expect("source stale");
        assert!(
            info.running.contains("abc"),
            "running display should include source: {}",
            info.running
        );
        assert!(
            info.disk.contains("def+"),
            "disk display should include source: {}",
            info.disk
        );
        assert!(info.running.contains("0.1.0"));
        assert!(info.disk.contains("0.1.0"));
        let _ = fs::remove_dir_all(&root);
    }

    // @spec shell/stale-build Ahead comparison: Missing source fingerprint skips source comparison
    #[test]
    fn missing_source_fingerprint_skips_source_comparison() {
        let root = temp_dir();
        write_duckboard_tree(&root, Some("0.1.0"));
        // Empty running source.
        assert_eq!(eval(&root, "0.1.0", "", "def"), None);
        // Empty disk source.
        assert_eq!(eval(&root, "0.1.0", "abc", ""), None);
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

    // @spec shell/local-install Cargo bins and Applications app: Install recipe deploys cargo bins and Applications
    #[test]
    fn install_recipe_deploys_cargo_bins_and_applications() {
        let path = project_justfile_path();
        let justfile = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("read justfile at {}: {e}", path.display()));
        let body = install_recipe_body(&justfile)
            .expect("justfile should contain an install: recipe");

        assert!(
            body.contains("cargo install --path crates/duckspec"),
            "install must cargo-install duckspec: {body}"
        );
        assert!(
            body.contains("cargo install --path crates/duckboard"),
            "install must cargo-install duckboard: {body}"
        );
        assert!(
            body.contains("cargo install --path crates/duckchat-claude-acp"),
            "install must cargo-install duckchat-claude-acp: {body}"
        );
        assert!(
            body.contains("just bundle") || body.contains("bundle"),
            "install must assemble the macOS app bundle: {body}"
        );
        assert!(
            body.contains("/Applications/Duckboard.app"),
            "install must deploy to /Applications/Duckboard.app: {body}"
        );
    }
}
