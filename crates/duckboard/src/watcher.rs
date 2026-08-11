//! iced subscription adapter around duckcore file watching.

use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use tokio::sync::mpsc;

pub use duckcore::watcher::FileEvent;

/// Hashable wrapper so `Subscription::run_with` can deduplicate. Only
/// `project_root` contributes to the hash — the subscription should survive
/// changes to other fields so its watcher and in-flight debounce state are
/// preserved.
#[derive(Clone)]
struct WatchId {
    project_root: PathBuf,
    duckspec_root: Option<PathBuf>,
}

impl Hash for WatchId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        "file-watcher".hash(state);
        self.project_root.hash(state);
    }
}

/// Iced subscription that watches `project_root` for file changes.
///
/// Returns batches of [`FileEvent`]s after debouncing. The subscription keeps
/// running until the application exits.
pub fn watch_subscription(
    project_root: PathBuf,
    duckspec_root: Option<PathBuf>,
) -> iced::Subscription<Vec<FileEvent>> {
    iced::Subscription::run_with(
        WatchId {
            project_root,
            duckspec_root,
        },
        |id| watch_stream(id.project_root.clone(), id.duckspec_root.clone()),
    )
}

fn watch_stream(
    project_root: PathBuf,
    duckspec_root: Option<PathBuf>,
) -> impl iced::futures::Stream<Item = Vec<FileEvent>> {
    iced::stream::channel(
        32,
        |mut sender: iced::futures::channel::mpsc::Sender<Vec<FileEvent>>| async move {
            use iced::futures::SinkExt;

            let (tx, mut rx) = mpsc::channel::<Vec<FileEvent>>(32);
            let _handle = duckcore::watcher::spawn_watcher(project_root, duckspec_root, tx);

            while let Some(file_events) = rx.recv().await {
                tracing::debug!(count = file_events.len(), "forwarding file events to UI");
                if sender.send(file_events).await.is_err() {
                    tracing::debug!("file watcher iced receiver dropped");
                    break;
                }
            }
        },
    )
}
