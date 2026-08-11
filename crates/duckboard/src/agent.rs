//! iced subscription adapter around duckcore agent.
//!
//! Catalog, harness dispatch, and turn driving live in `duckcore::agent`.
//! This module re-exports that API and adds the iced `Subscription` glue.

use std::path::PathBuf;

use iced::Subscription;
use tokio::sync::mpsc;

pub use duckcore::agent::{
    AgentEvent, AgentHandle, ModelInfo, SlashCommand, available_models, drive_harness,
    model_context_window, models_for_harness, refresh_model_catalog, resolve_oneshot_model,
    resolved_oneshot_model_for,
};

/// Create a subscription that manages one agent chat session.
///
/// `key` is an opaque routing token that gets echoed back with every event so
/// the caller can demultiplex when several sessions run in parallel. `harness`
/// selects the provider that runs the session's turns; it is folded into the
/// subscription's identity so switching harness respawns the worker on the new
/// backend.
pub fn agent_subscription(
    key: String,
    project_root: PathBuf,
    harness: String,
    oneshot_model: Option<String>,
) -> Subscription<(String, AgentEvent)> {
    Subscription::run_with(
        (key, project_root.clone(), harness, oneshot_model),
        |(key, root, harness, oneshot_model)| {
            use iced::futures::StreamExt;
            let key = key.clone();
            agent_stream(root.clone(), harness.clone(), oneshot_model.clone())
                .map(move |e| (key.clone(), e))
        },
    )
}

fn agent_stream(
    project_root: PathBuf,
    harness: String,
    oneshot_model: Option<String>,
) -> impl iced::futures::Stream<Item = AgentEvent> {
    iced::stream::channel(
        256,
        move |mut sender: iced::futures::channel::mpsc::Sender<AgentEvent>| async move {
            use iced::futures::SinkExt;

            let (tx, mut rx) = mpsc::channel::<AgentEvent>(256);
            tokio::spawn(async move {
                drive_harness(project_root, harness, oneshot_model, tx).await;
            });

            while let Some(event) = rx.recv().await {
                if sender.send(event).await.is_err() {
                    break;
                }
            }
        },
    )
}

/// Seed an unset global default on `config` from the catalog.
pub fn seed_global_default_if_unset(
    config: &mut crate::config::Config,
    catalog: &[ModelInfo],
) -> bool {
    duckcore::agent::seed_global_default_if_unset(&mut config.default_model, catalog)
}
