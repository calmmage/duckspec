//! File-system watcher with debouncing and .gitignore filtering.
//!
//! Watches the project root recursively, debounces events, filters through
//! the ignore crate (respects .gitignore), and emits classified file events
//! over a `tokio::mpsc` channel.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

use ignore::gitignore::GitignoreBuilder;
use notify_debouncer_mini::{DebouncedEventKind, new_debouncer};
use tokio::sync::mpsc::Sender;

// ── Event types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum FileEvent {
    /// File content was modified (or created — debouncer merges both).
    Modified(PathBuf),
    /// File or directory was removed.
    Removed(PathBuf),
    /// Git metadata changed outside the app (commit, checkout, ref update).
    VcsStateChanged(PathBuf),
}

// ── Watch loop ──────────────────────────────────────────────────────────────

/// Spawn a background watcher on `project_root` that sends debounced
/// [`FileEvent`] batches to `sender`. Returns a join handle for the blocking
/// notify thread (the async forwarder runs as a detached task when using the
/// iced adapter; here the caller owns the receiver).
///
/// Dropping all receivers stops the forward path; the notify thread exits when
/// its channel closes.
pub fn spawn_watcher(
    project_root: PathBuf,
    duckspec_root: Option<PathBuf>,
    sender: Sender<Vec<FileEvent>>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let (notify_tx, notify_rx) = mpsc::channel();

        let mut debouncer = match new_debouncer(Duration::from_millis(250), notify_tx) {
            Ok(d) => d,
            Err(e) => {
                tracing::error!("failed to create file watcher: {e}");
                return;
            }
        };

        if let Err(e) = debouncer.watcher().watch(
            &project_root,
            notify_debouncer_mini::notify::RecursiveMode::Recursive,
        ) {
            tracing::error!("failed to watch {}: {e}", project_root.display());
            return;
        }

        tracing::info!("file watcher active on {}", project_root.display());

        let gitignore = build_gitignore(&project_root);

        loop {
            match notify_rx.recv() {
                Ok(Ok(events)) => {
                    let file_events: Vec<FileEvent> = events
                        .into_iter()
                        .filter_map(|ev| {
                            if let Some(vcs_ev) = classify_vcs_state(&ev.path, &project_root) {
                                return Some(vcs_ev);
                            }
                            if is_ignored(&ev.path, &gitignore, duckspec_root.as_deref()) {
                                return None;
                            }
                            classify(&ev.path, ev.kind)
                        })
                        .collect();

                    if !file_events.is_empty() && sender.blocking_send(file_events).is_err() {
                        break;
                    }
                }
                Ok(Err(err)) => {
                    tracing::warn!("file watcher error: {err}");
                }
                Err(_) => {
                    tracing::debug!("file watcher notify channel closed");
                    break;
                }
            }
        }

        drop(debouncer);
    })
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn build_gitignore(project_root: &Path) -> ignore::gitignore::Gitignore {
    let mut builder = GitignoreBuilder::new(project_root);
    let gitignore_path = project_root.join(".gitignore");
    if gitignore_path.exists() {
        let _ = builder.add(&gitignore_path);
    }
    let _ = builder.add_line(None, ".git/");
    let _ = builder.add_line(None, ".jj/");
    let _ = builder.add_line(None, "target/");
    builder.build().unwrap_or_else(|e| {
        tracing::warn!("failed to build gitignore matcher: {e}");
        GitignoreBuilder::new(project_root).build().unwrap()
    })
}

fn is_ignored(
    path: &Path,
    gitignore: &ignore::gitignore::Gitignore,
    duckspec_root: Option<&Path>,
) -> bool {
    if let Some(root) = duckspec_root
        && path.starts_with(root)
    {
        return false;
    }
    let is_dir = path.is_dir();
    gitignore
        .matched_path_or_any_parents(path, is_dir)
        .is_ignore()
}

fn classify_vcs_state(path: &Path, project_root: &Path) -> Option<FileEvent> {
    let rel = path.strip_prefix(project_root).ok()?;
    let mut comps = rel.components();
    if comps.next()?.as_os_str() != ".git" {
        return None;
    }
    let rest: PathBuf = comps.collect();
    let is_state =
        rest.starts_with("HEAD") || rest.starts_with("index") || rest.starts_with("refs");
    is_state.then(|| FileEvent::VcsStateChanged(path.to_path_buf()))
}

fn classify(path: &Path, kind: DebouncedEventKind) -> Option<FileEvent> {
    match kind {
        DebouncedEventKind::Any => {
            if path.exists() {
                Some(FileEvent::Modified(path.to_path_buf()))
            } else {
                Some(FileEvent::Removed(path.to_path_buf()))
            }
        }
        DebouncedEventKind::AnyContinuous | _ => None,
    }
}
