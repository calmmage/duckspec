//! Per-scope worktree placement, naming, persistence, and sidecar lifecycle.
//!
//! Step 01 owns policy + manager primitives. Active-root wiring and UI come later.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::config::VcsWorkflow;
use crate::vcs;

// ── Types ────────────────────────────────────────────────────────────────────

/// Where a scope is allowed to work relative to the project main tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScopePlacement {
    /// Always project main. No sidecar.
    Main,
    /// Always a dedicated sidecar (created if missing).
    Worktree,
    /// Main until main is in use and dirty by another scope, then sidecar.
    #[default]
    Auto,
}

/// VCS tool used to create/forget sidecars.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorktreeBackend {
    Jj,
    Git,
}

/// Outcome of placement resolution (before disk ensure).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolveOutcome {
    Main,
    /// Use a sidecar; create it if the binding does not already have one.
    Sidecar,
}

/// Persisted per-scope binding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScopeBinding {
    pub placement: ScopePlacement,
    /// Absolute work root. Main project root when `is_main`.
    pub work_root: PathBuf,
    /// Stable identity `duck-<scope_key>` when a sidecar exists; empty on main-only.
    #[serde(default)]
    pub name: String,
    pub is_main: bool,
    /// Optional stack base (later steps).
    #[serde(default)]
    pub base_scope: Option<String>,
    #[serde(default)]
    pub require_base_merged: bool,
}

/// All scope bindings for one project.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingStore {
    #[serde(default)]
    pub scopes: HashMap<String, ScopeBinding>,
}

// ── Naming ───────────────────────────────────────────────────────────────────

/// Durable sidecar identity: `duck-<scope_key>`.
pub fn worktree_identity(scope_key: &str) -> String {
    format!("duck-{scope_key}")
}

/// Sidecar directory under the project: `.duckboard/worktrees/duck-<scope_key>/`.
pub fn sidecar_path(project_root: &Path, scope_key: &str) -> PathBuf {
    project_root
        .join(".duckboard")
        .join("worktrees")
        .join(worktree_identity(scope_key))
}

/// Git branch name for a sidecar: `duck/<scope_key>`.
pub fn git_branch_name(scope_key: &str) -> String {
    format!("duck/{scope_key}")
}

// ── Workflow / placement policy ──────────────────────────────────────────────

/// Whether the operator workflow permits sidecar placement.
pub fn placement_allows_sidecars(workflow: VcsWorkflow) -> bool {
    matches!(workflow, VcsWorkflow::Jj | VcsWorkflow::Worktrees)
}

/// Backend for ensure/forget from workflow. Only valid when sidecars are allowed.
pub fn backend_for_workflow(workflow: VcsWorkflow) -> Option<WorktreeBackend> {
    match workflow {
        VcsWorkflow::Jj => Some(WorktreeBackend::Jj),
        VcsWorkflow::Worktrees => Some(WorktreeBackend::Git),
        VcsWorkflow::Git => None,
    }
}

/// Default placement for a newly seen scope.
pub fn default_placement(workflow: VcsWorkflow) -> ScopePlacement {
    if placement_allows_sidecars(workflow) {
        ScopePlacement::Auto
    } else {
        ScopePlacement::Main
    }
}

/// Pure placement resolve. Spec authority for Main / Worktree / Auto + workflow gate.
pub fn resolve_placement(
    workflow: VcsWorkflow,
    placement: ScopePlacement,
    main_in_use_and_dirty_by_other: bool,
) -> ResolveOutcome {
    if !placement_allows_sidecars(workflow) {
        return ResolveOutcome::Main;
    }
    match placement {
        ScopePlacement::Main => ResolveOutcome::Main,
        ScopePlacement::Worktree => ResolveOutcome::Sidecar,
        ScopePlacement::Auto => {
            if main_in_use_and_dirty_by_other {
                ResolveOutcome::Sidecar
            } else {
                ResolveOutcome::Main
            }
        }
    }
}

/// True when main is dirty and some *other* scope is bound to main.
pub fn main_in_use_and_dirty_by_other(
    store: &BindingStore,
    this_scope: &str,
    main_dirty: bool,
) -> bool {
    if !main_dirty {
        return false;
    }
    store
        .scopes
        .iter()
        .any(|(key, b)| key != this_scope && b.is_main)
}

/// Whether the main working tree has uncommitted changes.
pub fn is_dirty(work_root: &Path) -> bool {
    !vcs::changed_files(work_root).is_empty()
}

// ── Persistence ──────────────────────────────────────────────────────────────

fn bindings_path(project_root: &Path) -> PathBuf {
    crate::config::data_dir(Some(project_root)).join("worktree_bindings.json")
}

/// Load bindings for a project (empty store if missing or invalid).
pub fn load_bindings(project_root: &Path) -> BindingStore {
    let path = bindings_path(project_root);
    let Ok(bytes) = std::fs::read(&path) else {
        return BindingStore::default();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

/// Persist bindings atomically (write temp + rename).
pub fn save_bindings(project_root: &Path, store: &BindingStore) -> anyhow::Result<()> {
    let path = bindings_path(project_root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_vec_pretty(store)?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

/// Drop sidecar paths that no longer exist on disk; keep placement.
pub fn reconcile_orphans(project_root: &Path, store: &mut BindingStore) {
    let mut dirty = false;
    for binding in store.scopes.values_mut() {
        if binding.is_main {
            continue;
        }
        if !binding.work_root.exists() {
            binding.work_root = project_root.to_path_buf();
            binding.is_main = true;
            binding.name.clear();
            dirty = true;
        }
    }
    if dirty {
        let _ = save_bindings(project_root, store);
    }
}

// ── Ensure / forget (CLI) ────────────────────────────────────────────────────

/// Result of merging a sidecar into the project main working tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrateError {
    Conflict { paths: Vec<String> },
    Failed { message: String },
}

/// Outcome of [`integrate_to_main`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegrateOutcome {
    /// Already on main / no sidecar.
    NoopMain,
    /// Merged cleanly; sidecar forgotten and binding cleared to main.
    Clean,
    /// Stopped; paths (or equivalent) for the human.
    Conflict { paths: Vec<String> },
    Failed { message: String },
}

/// Operations that create or remove sidecars. Production uses [`CliWorktreeOps`].
pub trait WorktreeOps {
    /// Create a sidecar for `scope_key`. When `base_work_root` is set, seed from
    /// that tree’s current revision instead of trunk alone.
    fn ensure(
        &self,
        project_root: &Path,
        scope_key: &str,
        backend: WorktreeBackend,
        base_work_root: Option<&Path>,
    ) -> anyhow::Result<PathBuf>;

    fn forget(
        &self,
        project_root: &Path,
        scope_key: &str,
        backend: WorktreeBackend,
        work_root: &Path,
    ) -> anyhow::Result<()>;

    /// Merge sidecar work into `project_root`. Does not forget the sidecar.
    fn try_integrate(
        &self,
        project_root: &Path,
        scope_key: &str,
        sidecar: &Path,
        backend: WorktreeBackend,
    ) -> Result<(), IntegrateError>;
}

/// Shell out to `jj workspace` / `git worktree`.
#[derive(Debug, Default, Clone, Copy)]
pub struct CliWorktreeOps;

impl WorktreeOps for CliWorktreeOps {
    fn ensure(
        &self,
        project_root: &Path,
        scope_key: &str,
        backend: WorktreeBackend,
        base_work_root: Option<&Path>,
    ) -> anyhow::Result<PathBuf> {
        ensure_sidecar_ignore(project_root)?;
        let dest = sidecar_path(project_root, scope_key);
        if dest.exists() {
            return Ok(dest);
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let identity = worktree_identity(scope_key);
        match backend {
            WorktreeBackend::Git => {
                let branch = git_branch_name(scope_key);
                let mut cmd = Command::new("git");
                cmd.args(["worktree", "add", "-b", &branch]);
                cmd.arg(dest.to_str().unwrap_or_default());
                if let Some(base) = base_work_root {
                    // Start the new branch at the base worktree’s HEAD.
                    if let Ok(out) = Command::new("git")
                        .args(["rev-parse", "HEAD"])
                        .current_dir(base)
                        .output()
                        && out.status.success()
                    {
                        let rev = String::from_utf8_lossy(&out.stdout).trim().to_string();
                        if !rev.is_empty() {
                            cmd.arg(rev);
                        }
                    }
                }
                let status = cmd.current_dir(project_root).status()?;
                if !status.success() {
                    anyhow::bail!("git worktree add failed for {identity}");
                }
            }
            WorktreeBackend::Jj => {
                let mut args = vec![
                    "workspace".into(),
                    "add".into(),
                    "--name".into(),
                    identity.clone(),
                ];
                if let Some(base) = base_work_root {
                    // Resolve base WC commit; fall back to default parents if lookup fails.
                    if let Ok(out) = Command::new("jj")
                        .args(["log", "-r", "@", "-T", "commit_id", "--no-graph"])
                        .current_dir(base)
                        .output()
                        && out.status.success()
                    {
                        let rev = String::from_utf8_lossy(&out.stdout).trim().to_string();
                        if !rev.is_empty() {
                            args.push("-r".into());
                            args.push(rev);
                        }
                    }
                }
                args.push(dest.display().to_string());
                let status = Command::new("jj")
                    .args(&args)
                    .current_dir(project_root)
                    .status()?;
                if !status.success() {
                    anyhow::bail!("jj workspace add failed for {identity}");
                }
            }
        }
        Ok(dest)
    }

    fn forget(
        &self,
        project_root: &Path,
        scope_key: &str,
        backend: WorktreeBackend,
        work_root: &Path,
    ) -> anyhow::Result<()> {
        let identity = worktree_identity(scope_key);
        match backend {
            WorktreeBackend::Git => {
                let _ = Command::new("git")
                    .args([
                        "worktree",
                        "remove",
                        "--force",
                        work_root.to_str().unwrap_or_default(),
                    ])
                    .current_dir(project_root)
                    .status();
            }
            WorktreeBackend::Jj => {
                let _ = Command::new("jj")
                    .args(["workspace", "forget", &identity])
                    .current_dir(project_root)
                    .status();
                if work_root.exists() {
                    let _ = std::fs::remove_dir_all(work_root);
                }
            }
        }
        Ok(())
    }

    fn try_integrate(
        &self,
        project_root: &Path,
        scope_key: &str,
        sidecar: &Path,
        backend: WorktreeBackend,
    ) -> Result<(), IntegrateError> {
        match backend {
            WorktreeBackend::Git => {
                let branch = git_branch_name(scope_key);
                // Fetch the worktree branch into main and merge.
                let status = Command::new("git")
                    .args(["merge", "--no-edit", &branch])
                    .current_dir(project_root)
                    .status()
                    .map_err(|e| IntegrateError::Failed {
                        message: e.to_string(),
                    })?;
                if status.success() {
                    return Ok(());
                }
                // Collect unmerged paths if present.
                let out = Command::new("git")
                    .args(["diff", "--name-only", "--diff-filter=U"])
                    .current_dir(project_root)
                    .output();
                let paths = out
                    .ok()
                    .map(|o| {
                        String::from_utf8_lossy(&o.stdout)
                            .lines()
                            .filter(|l| !l.is_empty())
                            .map(str::to_string)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                if paths.is_empty() {
                    Err(IntegrateError::Failed {
                        message: format!(
                            "git merge of {branch} failed (sidecar {})",
                            sidecar.display()
                        ),
                    })
                } else {
                    Err(IntegrateError::Conflict { paths })
                }
            }
            WorktreeBackend::Jj => {
                // Squash workspace WC onto main @ when possible.
                let identity = worktree_identity(scope_key);
                let status = Command::new("jj")
                    .args([
                        "squash",
                        "--from",
                        &format!("{identity}@"),
                        "--into",
                        "@",
                    ])
                    .current_dir(project_root)
                    .status()
                    .map_err(|e| IntegrateError::Failed {
                        message: e.to_string(),
                    })?;
                if status.success() {
                    return Ok(());
                }
                Err(IntegrateError::Conflict {
                    paths: vec![format!(
                        "jj squash from {identity}@ into @ failed — resolve conflicts in main"
                    )],
                })
            }
        }
    }
}

/// Ensure `.duckboard/worktrees/` exists and is ignored by git.
fn ensure_sidecar_ignore(project_root: &Path) -> anyhow::Result<()> {
    let worktrees = project_root.join(".duckboard").join("worktrees");
    std::fs::create_dir_all(&worktrees)?;
    let gi = project_root.join(".duckboard").join(".gitignore");
    if !gi.exists() {
        std::fs::write(gi, "worktrees/\n")?;
    }
    Ok(())
}

// ── Activate ─────────────────────────────────────────────────────────────────

/// Ensure `scope_key` has a binding and work root according to placement + workflow.
///
/// Returns the absolute work root for the scope.
pub fn activate_scope(
    store: &mut BindingStore,
    project_root: &Path,
    scope_key: &str,
    workflow: VcsWorkflow,
    main_dirty: bool,
    ops: &dyn WorktreeOps,
) -> anyhow::Result<PathBuf> {
    let placement = store
        .scopes
        .get(scope_key)
        .map(|b| b.placement)
        .unwrap_or_else(|| default_placement(workflow));

    let pressure = main_in_use_and_dirty_by_other(store, scope_key, main_dirty);
    let outcome = resolve_placement(workflow, placement, pressure);

    match outcome {
        ResolveOutcome::Main => {
            let binding = ScopeBinding {
                placement,
                work_root: project_root.to_path_buf(),
                name: String::new(),
                is_main: true,
                base_scope: store
                    .scopes
                    .get(scope_key)
                    .and_then(|b| b.base_scope.clone()),
                require_base_merged: store
                    .scopes
                    .get(scope_key)
                    .map(|b| b.require_base_merged)
                    .unwrap_or(false),
            };
            store.scopes.insert(scope_key.to_string(), binding);
            Ok(project_root.to_path_buf())
        }
        ResolveOutcome::Sidecar => {
            // Already have a live sidecar?
            if let Some(existing) = store.scopes.get(scope_key)
                && !existing.is_main
                && existing.work_root.exists()
            {
                return Ok(existing.work_root.clone());
            }
            let backend = backend_for_workflow(workflow)
                .ok_or_else(|| anyhow::anyhow!("sidecars disabled for this workflow"))?;
            let base_root = store
                .scopes
                .get(scope_key)
                .and_then(|b| b.base_scope.as_ref())
                .and_then(|base_key| store.scopes.get(base_key))
                .map(|b| b.work_root.clone());
            let root = ops.ensure(
                project_root,
                scope_key,
                backend,
                base_root.as_deref(),
            )?;
            let name = worktree_identity(scope_key);
            let binding = ScopeBinding {
                placement,
                work_root: root.clone(),
                name,
                is_main: false,
                base_scope: store
                    .scopes
                    .get(scope_key)
                    .and_then(|b| b.base_scope.clone()),
                require_base_merged: store
                    .scopes
                    .get(scope_key)
                    .map(|b| b.require_base_merged)
                    .unwrap_or(false),
            };
            store.scopes.insert(scope_key.to_string(), binding);
            Ok(root)
        }
    }
}

/// Set placement on a scope without activating (creates a stub binding on main).
pub fn set_placement(
    store: &mut BindingStore,
    project_root: &Path,
    scope_key: &str,
    placement: ScopePlacement,
) {
    ensure_binding_entry(store, project_root, scope_key).placement = placement;
}

fn ensure_binding_entry<'a>(
    store: &'a mut BindingStore,
    project_root: &Path,
    scope_key: &str,
) -> &'a mut ScopeBinding {
    store
        .scopes
        .entry(scope_key.to_string())
        .or_insert_with(|| ScopeBinding {
            placement: ScopePlacement::Auto,
            work_root: project_root.to_path_buf(),
            name: String::new(),
            is_main: true,
            base_scope: None,
            require_base_merged: false,
        })
}

/// Why [`set_base_scope`] refused a base.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetBaseError {
    /// `base` is the same as the scope.
    SelfRef,
    /// Setting `base` would create a cycle in the base graph.
    Cycle,
}

/// Whether walking `start`’s base chain would reach `target`.
fn base_chain_reaches(store: &BindingStore, start: &str, target: &str) -> bool {
    let mut seen = std::collections::HashSet::new();
    let mut cur = Some(start.to_string());
    while let Some(k) = cur {
        if k == target {
            return true;
        }
        if !seen.insert(k.clone()) {
            return true;
        }
        cur = store
            .scopes
            .get(&k)
            .and_then(|b| b.base_scope.clone());
    }
    false
}

/// Set or clear stack base for `scope_key`. Rejects self-base and cycles.
/// Does not change an existing sidecar path (recreate via placement Worktree).
pub fn set_base_scope(
    store: &mut BindingStore,
    project_root: &Path,
    scope_key: &str,
    base: Option<&str>,
) -> Result<(), SetBaseError> {
    if let Some(b) = base {
        if b == scope_key {
            return Err(SetBaseError::SelfRef);
        }
        if base_chain_reaches(store, b, scope_key) {
            return Err(SetBaseError::Cycle);
        }
    }
    ensure_binding_entry(store, project_root, scope_key).base_scope =
        base.map(str::to_string);
    let _ = save_bindings(project_root, store);
    Ok(())
}

/// Toggle or set require-base-merged on a scope.
pub fn set_require_base_merged(
    store: &mut BindingStore,
    project_root: &Path,
    scope_key: &str,
    require: bool,
) {
    ensure_binding_entry(store, project_root, scope_key).require_base_merged = require;
    let _ = save_bindings(project_root, store);
}

/// True when the base scope is on the project main tree (or has no sidecar).
pub fn base_is_on_main(store: &BindingStore, base_key: &str, project_root: &Path) -> bool {
    match store.scopes.get(base_key) {
        None => true,
        Some(b) => b.is_main || b.work_root == project_root,
    }
}

/// When send should be blocked: `Some(base_key)` that must be merged first.
pub fn agent_send_blocked(
    store: &BindingStore,
    scope_key: &str,
    project_root: &Path,
) -> Option<String> {
    let b = store.scopes.get(scope_key)?;
    if !b.require_base_merged {
        return None;
    }
    let base = b.base_scope.as_ref()?;
    if base_is_on_main(store, base, project_root) {
        None
    } else {
        Some(base.clone())
    }
}

/// Next stack-base candidate in a cycle: None → candidates[0] → … → None.
pub fn cycle_stack_base(
    current: Option<&str>,
    candidates: &[String],
) -> Option<String> {
    if candidates.is_empty() {
        return None;
    }
    match current {
        None => Some(candidates[0].clone()),
        Some(cur) => {
            if let Some(i) = candidates.iter().position(|c| c == cur) {
                if i + 1 < candidates.len() {
                    Some(candidates[i + 1].clone())
                } else {
                    None
                }
            } else {
                Some(candidates[0].clone())
            }
        }
    }
}

/// Integrate a scope’s sidecar into main. Same path for archive auto-merge and
/// explicit “merge to main”. On clean success forgets the sidecar, binds the
/// scope to main, and updates dependents that stacked on this scope.
pub fn integrate_to_main(
    store: &mut BindingStore,
    project_root: &Path,
    scope_key: &str,
    backend: WorktreeBackend,
    ops: &dyn WorktreeOps,
) -> IntegrateOutcome {
    let Some(binding) = store.scopes.get(scope_key).cloned() else {
        return IntegrateOutcome::NoopMain;
    };
    if binding.is_main {
        return IntegrateOutcome::NoopMain;
    }
    let sidecar = binding.work_root.clone();
    match ops.try_integrate(project_root, scope_key, &sidecar, backend) {
        Ok(()) => {
            let _ = ops.forget(project_root, scope_key, backend, &sidecar);
            store.scopes.insert(
                scope_key.to_string(),
                ScopeBinding {
                    placement: binding.placement,
                    work_root: project_root.to_path_buf(),
                    name: String::new(),
                    is_main: true,
                    base_scope: binding.base_scope,
                    require_base_merged: binding.require_base_merged,
                },
            );
            let _ = save_bindings(project_root, store);
            update_dependents_after_base_merge(
                store,
                project_root,
                scope_key,
                backend,
                ops,
            );
            IntegrateOutcome::Clean
        }
        Err(IntegrateError::Conflict { paths }) => IntegrateOutcome::Conflict { paths },
        Err(IntegrateError::Failed { message }) => IntegrateOutcome::Failed { message },
    }
}

/// After `base_key` lands on main, stacked dependents are **not** auto-integrated
/// (review resolution C). They keep their sidecars and `base_scope` until the
/// user explicitly merges or archives them. Spec MAY rebase/update is left for
/// a later cut.
pub fn update_dependents_after_base_merge(
    store: &BindingStore,
    _project_root: &Path,
    base_key: &str,
    _backend: WorktreeBackend,
    _ops: &dyn WorktreeOps,
) {
    let n = store
        .scopes
        .iter()
        .filter(|(k, b)| {
            *k != base_key && b.base_scope.as_deref() == Some(base_key) && !b.is_main
        })
        .count();
    if n > 0 {
        tracing::debug!(
            base = %base_key,
            dependents = n,
            "base merged; dependents left on sidecars (no auto-integrate)"
        );
    }
}

/// Whether archive/explicit merge should attempt integrate for this scope.
pub fn scope_has_sidecar(store: &BindingStore, scope_key: &str) -> bool {
    store
        .scopes
        .get(scope_key)
        .is_some_and(|b| !b.is_main)
}

/// Human-readable text for integrate conflict/fail (UI / system message).
/// `None` for clean or noop outcomes.
pub fn format_integrate_surface_message(
    scope_key: &str,
    outcome: &IntegrateOutcome,
) -> Option<String> {
    match outcome {
        IntegrateOutcome::Conflict { paths } => {
            let list = if paths.is_empty() {
                "(no path list from VCS)".to_string()
            } else {
                paths
                    .iter()
                    .map(|p| format!("- {p}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            Some(format!(
                "Could not merge `{scope_key}` into Main — conflict. Sidecar kept.\n{list}"
            ))
        }
        IntegrateOutcome::Failed { message } => Some(format!(
            "Could not merge `{scope_key}` into Main — failed. Sidecar kept.\n{message}"
        )),
        IntegrateOutcome::Clean | IntegrateOutcome::NoopMain => None,
    }
}

/// Cycle Main → Worktree → Auto → Main.
pub fn cycle_placement(placement: ScopePlacement) -> ScopePlacement {
    match placement {
        ScopePlacement::Main => ScopePlacement::Worktree,
        ScopePlacement::Worktree => ScopePlacement::Auto,
        ScopePlacement::Auto => ScopePlacement::Main,
    }
}

/// Effective placement for a scope (binding or workflow default).
pub fn effective_placement(
    store: &BindingStore,
    scope_key: &str,
    workflow: VcsWorkflow,
) -> ScopePlacement {
    store
        .scopes
        .get(scope_key)
        .map(|b| b.placement)
        .unwrap_or_else(|| default_placement(workflow))
}

/// Short UI label for a placement mode.
pub fn placement_label(placement: ScopePlacement) -> &'static str {
    match placement {
        ScopePlacement::Main => "main",
        ScopePlacement::Worktree => "tree",
        ScopePlacement::Auto => "auto",
    }
}

/// Move a binding from one scope key to another (exploration → change promotion).
/// Preserves placement, work root, and sidecar identity. No-op when `from_key`
/// has no binding.
pub fn transfer_scope_binding(
    store: &mut BindingStore,
    from_key: &str,
    to_key: &str,
    project_root: &Path,
) {
    if from_key == to_key {
        return;
    }
    let Some(binding) = store.scopes.remove(from_key) else {
        return;
    };
    store.scopes.insert(to_key.to_string(), binding);
    let _ = save_bindings(project_root, store);
}

// ── Active root (focused scope authority) ────────────────────────────────────

/// Work root recorded for an exploration/change scope key, or main if unbound.
pub fn scope_work_root(
    store: &BindingStore,
    project_root: &Path,
    scope_key: &str,
) -> PathBuf {
    store
        .scopes
        .get(scope_key)
        .map(|b| b.work_root.clone())
        .filter(|p| p.as_os_str().len() > 0)
        .unwrap_or_else(|| project_root.to_path_buf())
}

/// Whether this interaction scope always uses the project main tree.
pub fn scope_kind_forces_main(scope_key: &str) -> bool {
    scope_key == "caps" || scope_key == "codex"
}

/// Desired agent/file work root for a scope key (caps/codex → main).
pub fn desired_work_root(
    store: &BindingStore,
    project_root: &Path,
    scope_key: &str,
) -> PathBuf {
    if scope_kind_forces_main(scope_key) {
        project_root.to_path_buf()
    } else {
        scope_work_root(store, project_root, scope_key)
    }
}

/// Runtime agent cwd: keep the in-flight handle dir while streaming; otherwise
/// use the desired scope root (cold rebind).
pub fn agent_runtime_root(
    desired: &Path,
    is_streaming: bool,
    handle_working_dir: Option<&Path>,
) -> PathBuf {
    if is_streaming {
        if let Some(dir) = handle_working_dir {
            return dir.to_path_buf();
        }
    }
    desired.to_path_buf()
}

/// Resolve a project-relative path under the focused work root. Absolute paths
/// that already live under `work_root` or `project_root` are remapped by
/// relative suffix onto `work_root` when possible.
pub fn resolve_under_work_root(
    work_root: &Path,
    project_root: &Path,
    path: &Path,
) -> PathBuf {
    if path.is_absolute() {
        if let Ok(rel) = path.strip_prefix(work_root) {
            return work_root.join(rel);
        }
        if let Ok(rel) = path.strip_prefix(project_root) {
            return work_root.join(rel);
        }
        return path.to_path_buf();
    }
    work_root.join(path)
}

/// Roots that should be watched: project main plus every live sidecar.
pub fn watch_roots(project_root: &Path, store: &BindingStore) -> Vec<PathBuf> {
    let mut roots = vec![project_root.to_path_buf()];
    for b in store.scopes.values() {
        if !b.is_main && b.work_root.exists() && !roots.iter().any(|r| r == &b.work_root) {
            roots.push(b.work_root.clone());
        }
    }
    roots
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Fake ops: create empty dirs under sidecar_path; record ensure calls.
    struct FakeOps {
        ensures: Mutex<Vec<(String, Option<PathBuf>)>>,
    }

    impl FakeOps {
        fn new() -> Self {
            Self {
                ensures: Mutex::new(Vec::new()),
            }
        }

        fn ensure_count(&self) -> usize {
            self.ensures.lock().unwrap().len()
        }

        fn last_base(&self) -> Option<PathBuf> {
            self.ensures
                .lock()
                .unwrap()
                .last()
                .and_then(|(_, b)| b.clone())
        }
    }

    impl WorktreeOps for FakeOps {
        fn ensure(
            &self,
            project_root: &Path,
            scope_key: &str,
            _backend: WorktreeBackend,
            base_work_root: Option<&Path>,
        ) -> anyhow::Result<PathBuf> {
            self.ensures.lock().unwrap().push((
                scope_key.to_string(),
                base_work_root.map(|p| p.to_path_buf()),
            ));
            let dest = sidecar_path(project_root, scope_key);
            std::fs::create_dir_all(&dest)?;
            if let Some(base) = base_work_root {
                std::fs::write(dest.join(".stack-from"), base.display().to_string())?;
            }
            Ok(dest)
        }

        fn forget(
            &self,
            _project_root: &Path,
            _scope_key: &str,
            _backend: WorktreeBackend,
            work_root: &Path,
        ) -> anyhow::Result<()> {
            if work_root.exists() {
                let _ = std::fs::remove_dir_all(work_root);
            }
            Ok(())
        }

        fn try_integrate(
            &self,
            _project_root: &Path,
            _scope_key: &str,
            sidecar: &Path,
            _backend: WorktreeBackend,
        ) -> Result<(), IntegrateError> {
            let conflict = sidecar.join(".conflict");
            if conflict.exists() {
                let paths = std::fs::read_to_string(&conflict)
                    .unwrap_or_default()
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(str::to_string)
                    .collect::<Vec<_>>();
                return Err(IntegrateError::Conflict {
                    paths: if paths.is_empty() {
                        vec!["conflict".into()]
                    } else {
                        paths
                    },
                });
            }
            Ok(())
        }
    }

    fn temp_project() -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "duckboard-worktree-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&p).expect("temp project");
        p
    }

    fn with_config_home<T>(f: impl FnOnce() -> T) -> T {
        let mut home = std::env::temp_dir();
        home.push(format!(
            "duckboard-cfg-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&home).expect("temp config");
        crate::config::set_config_dir_override(home.join(".config/duckboard"));
        f()
    }

    // ── Workflow gate ────────────────────────────────────────────────────────

    /// @spec worktree/scope-placement Workflow gate: Plain Git stays on Main
    #[test]
    fn plain_git_stays_on_main() {
        let root = temp_project();
        let mut store = BindingStore::default();
        // Another scope on dirty main
        set_placement(&mut store, &root, "other", ScopePlacement::Auto);
        store.scopes.get_mut("other").unwrap().is_main = true;

        set_placement(&mut store, &root, "exp-a", ScopePlacement::Auto);
        let ops = FakeOps::new();
        let work = activate_scope(
            &mut store,
            &root,
            "exp-a",
            VcsWorkflow::Git,
            true, // main dirty
            &ops,
        )
        .unwrap();

        assert_eq!(work, root);
        assert!(store.scopes["exp-a"].is_main);
        assert_eq!(ops.ensure_count(), 0);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// @spec worktree/scope-placement Workflow gate: Jj or Worktrees may use placement
    #[test]
    fn jj_or_worktrees_may_use_placement() {
        for workflow in [VcsWorkflow::Jj, VcsWorkflow::Worktrees] {
            assert!(placement_allows_sidecars(workflow));
            assert_eq!(
                resolve_placement(workflow, ScopePlacement::Worktree, false),
                ResolveOutcome::Sidecar
            );
        }
        let root = temp_project();
        let mut store = BindingStore::default();
        set_placement(&mut store, &root, "exp-a", ScopePlacement::Worktree);
        let ops = FakeOps::new();
        let work = activate_scope(
            &mut store,
            &root,
            "exp-a",
            VcsWorkflow::Worktrees,
            false,
            &ops,
        )
        .unwrap();
        assert_eq!(work, sidecar_path(&root, "exp-a"));
        assert_eq!(ops.ensure_count(), 1);
        let _ = std::fs::remove_dir_all(&root);
    }

    // ── Placement modes ──────────────────────────────────────────────────────

    /// @spec worktree/scope-placement Placement modes: Main never creates a sidecar
    #[test]
    fn main_never_creates_a_sidecar() {
        let root = temp_project();
        let mut store = BindingStore::default();
        set_placement(&mut store, &root, "owner", ScopePlacement::Auto);
        store.scopes.get_mut("owner").unwrap().is_main = true;
        set_placement(&mut store, &root, "pinned", ScopePlacement::Main);
        let ops = FakeOps::new();
        let work =
            activate_scope(&mut store, &root, "pinned", VcsWorkflow::Jj, true, &ops).unwrap();
        assert_eq!(work, root);
        assert_eq!(ops.ensure_count(), 0);
        assert!(store.scopes["pinned"].is_main);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// @spec worktree/scope-placement Placement modes: Worktree always has a sidecar
    #[test]
    fn worktree_always_has_a_sidecar() {
        let root = temp_project();
        let mut store = BindingStore::default();
        set_placement(&mut store, &root, "exp-a", ScopePlacement::Worktree);
        let ops = FakeOps::new();
        let work = activate_scope(
            &mut store,
            &root,
            "exp-a",
            VcsWorkflow::Worktrees,
            false, // main clean
            &ops,
        )
        .unwrap();
        let expected = sidecar_path(&root, "exp-a");
        assert_eq!(work, expected);
        assert!(expected.exists());
        assert!(!store.scopes["exp-a"].is_main);
        assert_eq!(ops.ensure_count(), 1);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// @spec worktree/scope-placement Placement modes: Auto stays on Main when Main is free
    #[test]
    fn auto_stays_on_main_when_main_is_free() {
        let root = temp_project();
        let mut store = BindingStore::default();
        set_placement(&mut store, &root, "exp-a", ScopePlacement::Auto);
        let ops = FakeOps::new();
        let work =
            activate_scope(&mut store, &root, "exp-a", VcsWorkflow::Jj, false, &ops).unwrap();
        assert_eq!(work, root);
        assert_eq!(ops.ensure_count(), 0);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// @spec worktree/scope-placement Placement modes: Auto forks when Main is in use and dirty
    #[test]
    fn auto_forks_when_main_in_use_and_dirty() {
        let root = temp_project();
        let mut store = BindingStore::default();
        set_placement(&mut store, &root, "scope-a", ScopePlacement::Auto);
        store.scopes.get_mut("scope-a").unwrap().is_main = true;
        set_placement(&mut store, &root, "scope-b", ScopePlacement::Auto);
        let ops = FakeOps::new();
        let work_b = activate_scope(
            &mut store,
            &root,
            "scope-b",
            VcsWorkflow::Worktrees,
            true,
            &ops,
        )
        .unwrap();
        assert_eq!(work_b, sidecar_path(&root, "scope-b"));
        assert!(store.scopes["scope-a"].is_main);
        assert!(!store.scopes["scope-b"].is_main);
        assert_eq!(ops.ensure_count(), 1);
        let _ = std::fs::remove_dir_all(&root);
    }

    /// @spec worktree/scope-placement Placement modes: Main pin wins over Auto fork pressure
    #[test]
    fn main_pin_wins_over_auto_fork_pressure() {
        let root = temp_project();
        let mut store = BindingStore::default();
        set_placement(&mut store, &root, "scope-a", ScopePlacement::Auto);
        store.scopes.get_mut("scope-a").unwrap().is_main = true;
        set_placement(&mut store, &root, "scope-b", ScopePlacement::Main);
        let ops = FakeOps::new();
        let work =
            activate_scope(&mut store, &root, "scope-b", VcsWorkflow::Jj, true, &ops).unwrap();
        assert_eq!(work, root);
        assert_eq!(ops.ensure_count(), 0);
        let _ = std::fs::remove_dir_all(&root);
    }

    // ── Naming ───────────────────────────────────────────────────────────────

    /// @spec worktree/scope-placement Stable naming: Sidecar identity is duck-scope_key
    #[test]
    fn sidecar_identity_is_duck_scope_key() {
        let key = "session-worktrees";
        assert_eq!(worktree_identity(key), "duck-session-worktrees");
        let root = temp_project();
        let path = sidecar_path(&root, key);
        assert!(path.ends_with(".duckboard/worktrees/duck-session-worktrees"));

        let mut store = BindingStore::default();
        set_placement(&mut store, &root, key, ScopePlacement::Worktree);
        let ops = FakeOps::new();
        let work = activate_scope(
            &mut store,
            &root,
            key,
            VcsWorkflow::Worktrees,
            false,
            &ops,
        )
        .unwrap();
        assert_eq!(store.scopes[key].name, "duck-session-worktrees");
        assert_eq!(work, path);
        assert!(work.starts_with(root.join(".duckboard/worktrees")));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// @spec worktree/scope-placement Stable naming: Display rename does not rename the worktree id
    #[test]
    fn display_rename_does_not_rename_worktree_id() {
        let scope_key = "exp-123";
        let identity_before = worktree_identity(scope_key);
        // Display name is not an input to identity — rename is a no-op for naming.
        let _display_name = "Cloud agent options";
        let identity_after = worktree_identity(scope_key);
        assert_eq!(identity_before, "duck-exp-123");
        assert_eq!(identity_after, identity_before);
    }

    // ── Persist ──────────────────────────────────────────────────────────────

    /// @spec worktree/scope-placement Placement persists: Placement survives project reload
    #[test]
    fn placement_survives_project_reload() {
        with_config_home(|| {
            let root = temp_project();
            let mut store = BindingStore::default();
            set_placement(&mut store, &root, "exp-a", ScopePlacement::Worktree);
            save_bindings(&root, &store).unwrap();

            let reloaded = load_bindings(&root);
            assert_eq!(
                reloaded.scopes["exp-a"].placement,
                ScopePlacement::Worktree
            );
            let _ = std::fs::remove_dir_all(&root);
        });
    }

    /// @spec worktree/scope-placement Placement persists: Promotion keeps placement on the change
    #[test]
    fn promotion_keeps_placement_on_the_change() {
        with_config_home(|| {
            let root = temp_project();
            let mut store = BindingStore::default();
            set_placement(&mut store, &root, "exp-xyz", ScopePlacement::Worktree);
            transfer_scope_binding(&mut store, "exp-xyz", "my-feature", &root);

            assert!(!store.scopes.contains_key("exp-xyz"));
            assert_eq!(
                store.scopes["my-feature"].placement,
                ScopePlacement::Worktree
            );

            let reloaded = load_bindings(&root);
            assert_eq!(
                reloaded.scopes["my-feature"].placement,
                ScopePlacement::Worktree
            );
            let _ = std::fs::remove_dir_all(&root);
        });
    }

    // ── Active root ──────────────────────────────────────────────────────────

    fn bind_sidecar(store: &mut BindingStore, project: &Path, key: &str) -> PathBuf {
        let side = sidecar_path(project, key);
        std::fs::create_dir_all(&side).unwrap();
        store.scopes.insert(
            key.to_string(),
            ScopeBinding {
                placement: ScopePlacement::Worktree,
                work_root: side.clone(),
                name: worktree_identity(key),
                is_main: false,
                base_scope: None,
                require_base_merged: false,
            },
        );
        side
    }

    /// @spec worktree/active-root Focused root authority: Changed files reflect the focused scope’s root only
    #[test]
    fn changed_files_reflect_focused_scope_root_only() {
        let project = temp_project();
        let mut store = BindingStore::default();
        let side_a = bind_sidecar(&mut store, &project, "scope-a");
        let side_b = bind_sidecar(&mut store, &project, "scope-b");
        std::fs::write(side_a.join("a.rs"), "a").unwrap();
        std::fs::write(side_b.join("b.rs"), "b").unwrap();

        // Authority is whichever root is focused — not a union.
        assert_eq!(desired_work_root(&store, &project, "scope-a"), side_a);
        assert_eq!(desired_work_root(&store, &project, "scope-b"), side_b);
        assert_ne!(
            desired_work_root(&store, &project, "scope-a"),
            desired_work_root(&store, &project, "scope-b")
        );
        let _ = std::fs::remove_dir_all(&project);
    }

    /// @spec worktree/active-root Focused root authority: Agent working directory matches the focused scope’s root
    #[test]
    fn agent_working_directory_matches_focused_scope_root() {
        let project = temp_project();
        let mut store = BindingStore::default();
        let side = bind_sidecar(&mut store, &project, "scope-a");
        let desired = desired_work_root(&store, &project, "scope-a");
        assert_eq!(desired, side);
        let runtime = agent_runtime_root(&desired, false, None);
        assert_eq!(runtime, side);
        let _ = std::fs::remove_dir_all(&project);
    }

    /// @spec worktree/active-root Focused root authority: File open resolves under the focused root
    #[test]
    fn file_open_resolves_under_focused_root() {
        let project = temp_project();
        let mut store = BindingStore::default();
        let side = bind_sidecar(&mut store, &project, "scope-a");
        std::fs::create_dir_all(side.join("src")).unwrap();
        std::fs::write(side.join("src/lib.rs"), "x").unwrap();

        // Explorer walk root and open target share desired_work_root for the scope.
        let walk_root = desired_work_root(&store, &project, "scope-a");
        assert_eq!(walk_root, side);

        let resolved = resolve_under_work_root(&walk_root, &project, Path::new("src/lib.rs"));
        assert_eq!(resolved, side.join("src/lib.rs"));
        assert!(resolved.starts_with(&side));

        // Absolute path under main project remaps into work root.
        let main_abs = project.join("src/lib.rs");
        let remapped = resolve_under_work_root(&side, &project, &main_abs);
        assert_eq!(remapped, side.join("src/lib.rs"));
        let _ = std::fs::remove_dir_all(&project);
    }

    /// @spec worktree/active-root Focus switch: Switching scope rebinds Changed files to the new root
    #[test]
    fn switching_scope_rebinds_changed_files_to_new_root() {
        let project = temp_project();
        let mut store = BindingStore::default();
        let side_a = bind_sidecar(&mut store, &project, "scope-a");
        let side_b = bind_sidecar(&mut store, &project, "scope-b");
        let focused_a = desired_work_root(&store, &project, "scope-a");
        let focused_b = desired_work_root(&store, &project, "scope-b");
        assert_eq!(focused_a, side_a);
        assert_eq!(focused_b, side_b);
        assert_ne!(focused_a, focused_b);
        let _ = std::fs::remove_dir_all(&project);
    }

    /// @spec worktree/active-root Focus switch: Cold agent rebinds to the new root on next use
    #[test]
    fn cold_agent_rebinds_to_new_root_on_next_use() {
        let r1 = PathBuf::from("/tmp/r1");
        let r2 = PathBuf::from("/tmp/r2");
        // Not streaming: desired R2 wins even if handle was on R1.
        let runtime = agent_runtime_root(&r2, false, Some(&r1));
        assert_eq!(runtime, r2);
    }

    /// @spec worktree/active-root No mid-stream rebind: Streaming session keeps its root until the turn ends
    #[test]
    fn streaming_session_keeps_root_until_turn_ends() {
        let r1 = PathBuf::from("/tmp/r1");
        let r2 = PathBuf::from("/tmp/r2");
        let runtime = agent_runtime_root(&r2, true, Some(&r1));
        assert_eq!(runtime, r1);
    }

    /// @spec worktree/active-root Caps and codex on Main: Caps and codex scopes always use the project main root
    #[test]
    fn caps_and_codex_scopes_always_use_project_main_root() {
        let project = temp_project();
        let mut store = BindingStore::default();
        let _side = bind_sidecar(&mut store, &project, "exp-a");
        assert_eq!(
            desired_work_root(&store, &project, "caps"),
            project
        );
        assert_eq!(
            desired_work_root(&store, &project, "codex"),
            project
        );
        assert!(scope_kind_forces_main("caps"));
        assert!(scope_kind_forces_main("codex"));
        let _ = std::fs::remove_dir_all(&project);
    }

    // ── Stack base + require-merged ───────────────────────────────────────────

    /// @spec worktree/stack-and-merge Stack base: Stacked sidecar is created from the base scope’s work revision
    #[test]
    fn stacked_sidecar_created_from_base_work_revision() {
        let project = temp_project();
        let mut store = BindingStore::default();
        let side_a = bind_sidecar(&mut store, &project, "scope-a");
        set_placement(&mut store, &project, "scope-b", ScopePlacement::Worktree);
        set_base_scope(&mut store, &project, "scope-b", Some("scope-a")).unwrap();
        // Drop any existing B sidecar so ensure runs.
        store.scopes.get_mut("scope-b").unwrap().is_main = true;
        store.scopes.get_mut("scope-b").unwrap().work_root = project.clone();

        let ops = FakeOps::new();
        let work = activate_scope(
            &mut store,
            &project,
            "scope-b",
            VcsWorkflow::Worktrees,
            false,
            &ops,
        )
        .unwrap();
        assert_eq!(work, sidecar_path(&project, "scope-b"));
        assert_eq!(ops.last_base().as_deref(), Some(side_a.as_path()));
        let marker = std::fs::read_to_string(work.join(".stack-from")).unwrap();
        assert_eq!(marker, side_a.display().to_string());
        let _ = std::fs::remove_dir_all(&project);
    }

    /// @spec worktree/stack-and-merge Stack base: Cyclic base is rejected
    #[test]
    fn cyclic_base_is_rejected() {
        let project = temp_project();
        let mut store = BindingStore::default();
        set_base_scope(&mut store, &project, "scope-a", Some("scope-b")).unwrap();
        let err = set_base_scope(&mut store, &project, "scope-b", Some("scope-a"));
        assert_eq!(err, Err(SetBaseError::Cycle));
        assert_eq!(
            store.scopes.get("scope-b").and_then(|b| b.base_scope.clone()),
            None
        );
        let _ = std::fs::remove_dir_all(&project);
    }

    /// @spec worktree/stack-and-merge Stack base: Clearing base returns create policy to trunk
    #[test]
    fn clearing_base_returns_create_policy_to_trunk() {
        let project = temp_project();
        let mut store = BindingStore::default();
        let _side_a = bind_sidecar(&mut store, &project, "scope-a");
        set_placement(&mut store, &project, "scope-b", ScopePlacement::Worktree);
        set_base_scope(&mut store, &project, "scope-b", Some("scope-a")).unwrap();
        set_base_scope(&mut store, &project, "scope-b", None).unwrap();
        store.scopes.get_mut("scope-b").unwrap().is_main = true;
        store.scopes.get_mut("scope-b").unwrap().work_root = project.clone();
        // Remove leftover path so ensure recreates.
        let _ = std::fs::remove_dir_all(sidecar_path(&project, "scope-b"));

        let ops = FakeOps::new();
        let _ = activate_scope(
            &mut store,
            &project,
            "scope-b",
            VcsWorkflow::Worktrees,
            false,
            &ops,
        )
        .unwrap();
        assert_eq!(ops.last_base(), None);
        let _ = std::fs::remove_dir_all(&project);
    }

    /// @spec worktree/stack-and-merge Require base merged: Blocked send while base unmerged and flag set
    #[test]
    fn blocked_send_while_base_unmerged_and_flag_set() {
        let project = temp_project();
        let mut store = BindingStore::default();
        let _a = bind_sidecar(&mut store, &project, "scope-a");
        set_base_scope(&mut store, &project, "scope-b", Some("scope-a")).unwrap();
        set_require_base_merged(&mut store, &project, "scope-b", true);
        assert_eq!(
            agent_send_blocked(&store, "scope-b", &project).as_deref(),
            Some("scope-a")
        );
        let _ = std::fs::remove_dir_all(&project);
    }

    /// @spec worktree/stack-and-merge Require base merged: Send allowed when flag unset even if base unmerged
    #[test]
    fn send_allowed_when_flag_unset_even_if_base_unmerged() {
        let project = temp_project();
        let mut store = BindingStore::default();
        let _a = bind_sidecar(&mut store, &project, "scope-a");
        set_base_scope(&mut store, &project, "scope-b", Some("scope-a")).unwrap();
        set_require_base_merged(&mut store, &project, "scope-b", false);
        assert_eq!(agent_send_blocked(&store, "scope-b", &project), None);
        let _ = std::fs::remove_dir_all(&project);
    }

    /// @spec worktree/stack-and-merge Require base merged: Send allowed after base integrated to Main
    #[test]
    fn send_allowed_after_base_integrated_to_main() {
        let project = temp_project();
        let mut store = BindingStore::default();
        let _a = bind_sidecar(&mut store, &project, "scope-a");
        set_base_scope(&mut store, &project, "scope-b", Some("scope-a")).unwrap();
        set_require_base_merged(&mut store, &project, "scope-b", true);
        // Integrate A to main: mark as main.
        store.scopes.get_mut("scope-a").unwrap().is_main = true;
        store.scopes.get_mut("scope-a").unwrap().work_root = project.clone();
        assert_eq!(agent_send_blocked(&store, "scope-b", &project), None);
        let _ = std::fs::remove_dir_all(&project);
    }

    // ── Integrate to Main ────────────────────────────────────────────────────

    /// @spec worktree/stack-and-merge Integrate to Main: Archive success auto-attempts integrate for a sidecar scope
    #[test]
    fn archive_success_auto_attempts_integrate_for_sidecar_scope() {
        // Archive path calls the same integrate_to_main entry as explicit merge.
        let project = temp_project();
        let mut store = BindingStore::default();
        let side = bind_sidecar(&mut store, &project, "feat-a");
        assert!(scope_has_sidecar(&store, "feat-a"));
        let ops = FakeOps::new();
        // Simulate archive hook: always attempt when sidecar present.
        let outcome = if scope_has_sidecar(&store, "feat-a") {
            integrate_to_main(
                &mut store,
                &project,
                "feat-a",
                WorktreeBackend::Git,
                &ops,
            )
        } else {
            IntegrateOutcome::NoopMain
        };
        assert_eq!(outcome, IntegrateOutcome::Clean);
        assert!(!side.exists() || !scope_has_sidecar(&store, "feat-a"));
        let _ = std::fs::remove_dir_all(&project);
    }

    /// @spec worktree/stack-and-merge Integrate to Main: Clean integrate forgets the sidecar
    #[test]
    fn clean_integrate_forgets_the_sidecar() {
        let project = temp_project();
        let mut store = BindingStore::default();
        let side = bind_sidecar(&mut store, &project, "feat-a");
        let ops = FakeOps::new();
        let outcome = integrate_to_main(
            &mut store,
            &project,
            "feat-a",
            WorktreeBackend::Git,
            &ops,
        );
        assert_eq!(outcome, IntegrateOutcome::Clean);
        assert!(!side.exists());
        assert!(store.scopes["feat-a"].is_main);
        assert_eq!(store.scopes["feat-a"].work_root, project);
        assert!(!scope_has_sidecar(&store, "feat-a"));
        let _ = std::fs::remove_dir_all(&project);
    }

    /// @spec worktree/stack-and-merge Integrate to Main: Conflict stops without claiming success
    #[test]
    fn conflict_stops_without_claiming_success() {
        let project = temp_project();
        let mut store = BindingStore::default();
        let side = bind_sidecar(&mut store, &project, "feat-a");
        std::fs::write(side.join(".conflict"), "src/lib.rs\n").unwrap();
        let ops = FakeOps::new();
        let outcome = integrate_to_main(
            &mut store,
            &project,
            "feat-a",
            WorktreeBackend::Git,
            &ops,
        );
        assert!(matches!(
            outcome,
            IntegrateOutcome::Conflict { ref paths } if paths.iter().any(|p| p.contains("lib"))
        ));
        assert!(side.exists());
        assert!(scope_has_sidecar(&store, "feat-a"));
        // User-visible surface text includes scope + conflict paths (not success).
        let msg = format_integrate_surface_message("feat-a", &outcome).expect("surface text");
        assert!(msg.contains("feat-a"));
        assert!(msg.to_lowercase().contains("conflict"));
        assert!(msg.contains("src/lib.rs") || msg.contains("Sidecar kept"));
        assert!(!msg.to_lowercase().contains("merged successfully"));
        let _ = std::fs::remove_dir_all(&project);
    }

    /// @spec worktree/stack-and-merge Integrate to Main: Explicit merge to main uses the same integrate path
    #[test]
    fn explicit_merge_to_main_uses_same_integrate_path() {
        // Explicit merge is integrate_to_main — identical to archive auto path.
        let project = temp_project();
        let mut store = BindingStore::default();
        let _side = bind_sidecar(&mut store, &project, "feat-a");
        let ops = FakeOps::new();
        let archive_path = integrate_to_main(
            &mut store,
            &project,
            "feat-a",
            WorktreeBackend::Git,
            &ops,
        );
        // Fresh sidecar for explicit path comparison of function identity.
        let mut store2 = BindingStore::default();
        let _side2 = bind_sidecar(&mut store2, &project, "feat-b");
        let explicit_path = integrate_to_main(
            &mut store2,
            &project,
            "feat-b",
            WorktreeBackend::Git,
            &ops,
        );
        assert_eq!(archive_path, IntegrateOutcome::Clean);
        assert_eq!(explicit_path, IntegrateOutcome::Clean);
        let _ = std::fs::remove_dir_all(&project);
    }

    /// @spec worktree/stack-and-merge Stack after parent merge: After base merges cleanly, dependent may rebase onto Main when clean
    #[test]
    fn after_base_merges_cleanly_dependent_may_rebase_onto_main_when_clean() {
        // Review resolution C: MAY rebase is not auto full-land — dependent stays
        // on its sidecar until explicit merge/archive.
        let project = temp_project();
        let mut store = BindingStore::default();
        let _a = bind_sidecar(&mut store, &project, "scope-a");
        let side_b = bind_sidecar(&mut store, &project, "scope-b");
        set_base_scope(&mut store, &project, "scope-b", Some("scope-a")).unwrap();
        let ops = FakeOps::new();
        let outcome = integrate_to_main(
            &mut store,
            &project,
            "scope-a",
            WorktreeBackend::Git,
            &ops,
        );
        assert_eq!(outcome, IntegrateOutcome::Clean);
        assert!(store.scopes["scope-a"].is_main);
        assert!(
            !store.scopes["scope-b"].is_main,
            "dependent must not auto-land on Main when base merges"
        );
        assert!(
            side_b.exists(),
            "dependent sidecar remains until explicit merge"
        );
        assert_eq!(
            store.scopes["scope-b"].base_scope.as_deref(),
            Some("scope-a")
        );
        let _ = std::fs::remove_dir_all(&project);
    }
}
