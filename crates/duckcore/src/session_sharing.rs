//! Cross-process session file coexistence for duckboard and ducktui.
//!
//! Persistence (`chat_store`) owns atomic single-process durability. This module
//! owns drive-versus-display write policy, external reload, and the in-flight
//! shield. Concurrent multi-process turns are unsupported: last atomic write
//! wins; there are no lock files.

use std::path::{Path, PathBuf};

use crate::chat_store::{self, ChatSession};
use crate::paths;

/// Whether this process may persist a loaded session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DriveRole {
    /// Loaded for viewing only; another process may own writes.
    #[default]
    Displayed,
    /// This process has driven (or is driving) turns on the session.
    Driven,
}

/// Outcome of a gated persist attempt.
#[derive(Debug)]
pub enum PersistOutcome {
    Written,
    SkippedDisplayed,
    Failed(anyhow::Error),
}

/// External session-file change observed via the project watcher.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalSessionEvent {
    Modified { scope: String, session_id: String },
    Removed { scope: String, session_id: String },
}

/// Result of applying an external event to an in-memory session list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyResult {
    Reloaded,
    Removed,
    IgnoredInFlight,
    NotPresent,
}

/// True when this process may write the session file.
pub fn may_persist(role: DriveRole) -> bool {
    matches!(role, DriveRole::Driven)
}

/// True while a local turn is open (stream buffers still live).
pub fn is_in_flight(session: &ChatSession) -> bool {
    session.is_streaming
}

/// Persist `session` only when `role` is [`DriveRole::Driven`].
///
/// Displayed sessions are never written. There is no multi-process lock file.
pub fn persist_driven(
    session: &ChatSession,
    role: DriveRole,
    project_root: Option<&Path>,
) -> PersistOutcome {
    if !may_persist(role) {
        return PersistOutcome::SkippedDisplayed;
    }
    match chat_store::save_session(session, project_root) {
        Ok(()) => PersistOutcome::Written,
        Err(e) => PersistOutcome::Failed(e),
    }
}

/// If `path` is a session file under the shared chats root, return `(scope, session_id)`.
pub fn classify_session_path(
    path: &Path,
    project_root: Option<&Path>,
) -> Option<(String, String)> {
    let chats = paths::data_dir(project_root).join("chats");
    let rel = path.strip_prefix(&chats).ok()?;
    let mut comps = rel.components();
    let scope = comps.next()?.as_os_str().to_str()?.to_string();
    let file = comps.next()?.as_os_str().to_str()?;
    if comps.next().is_some() {
        return None;
    }
    let id = file.strip_suffix(".json")?;
    if id.is_empty() || id.contains('.') {
        // Reject `foo.json.tmp` and empty ids.
        return None;
    }
    Some((scope, id.to_string()))
}

/// Map a watcher path + kind into an external session event, if it is a session file.
pub fn external_event_for_path(
    path: &Path,
    removed: bool,
    project_root: Option<&Path>,
) -> Option<ExternalSessionEvent> {
    let (scope, session_id) = classify_session_path(path, project_root)?;
    if removed {
        Some(ExternalSessionEvent::Removed { scope, session_id })
    } else {
        Some(ExternalSessionEvent::Modified { scope, session_id })
    }
}

/// Apply an external session-file event to the in-memory list for that scope.
///
/// In-flight local turns shield the matching session from mid-turn replacement.
/// Idle sessions reload from disk or drop on removal.
pub fn apply_external_to_list(
    sessions: &mut Vec<ChatSession>,
    event: &ExternalSessionEvent,
    project_root: Option<&Path>,
) -> ApplyResult {
    let (scope, session_id) = match event {
        ExternalSessionEvent::Modified { scope, session_id }
        | ExternalSessionEvent::Removed { scope, session_id } => (scope.as_str(), session_id.as_str()),
    };

    let idx = sessions
        .iter()
        .position(|s| s.scope == scope && s.id == session_id);
    let Some(idx) = idx else {
        // Removal of an unknown id: still a no-op for this list.
        // Modified of an unknown id: load into the list when the file exists.
        if let ExternalSessionEvent::Modified { .. } = event
            && let Some(loaded) = load_one(scope, session_id, project_root)
        {
            sessions.push(loaded);
            sessions.sort_by_key(|s| std::cmp::Reverse(s.created_at_nanos));
            return ApplyResult::Reloaded;
        }
        return ApplyResult::NotPresent;
    };

    if is_in_flight(&sessions[idx]) {
        return ApplyResult::IgnoredInFlight;
    }

    match event {
        ExternalSessionEvent::Removed { .. } => {
            sessions.remove(idx);
            ApplyResult::Removed
        }
        ExternalSessionEvent::Modified { .. } => match load_one(scope, session_id, project_root) {
            Some(loaded) => {
                sessions[idx] = loaded;
                ApplyResult::Reloaded
            }
            None => {
                // File vanished between event and read — treat as removal.
                sessions.remove(idx);
                ApplyResult::Removed
            }
        },
    }
}

fn load_one(scope: &str, session_id: &str, project_root: Option<&Path>) -> Option<ChatSession> {
    chat_store::load_sessions_for(scope, project_root)
        .into_iter()
        .find(|s| s.id == session_id)
}

/// Absolute path of a session file (test and watcher helpers).
pub fn session_file_path(scope: &str, session_id: &str, project_root: Option<&Path>) -> PathBuf {
    paths::data_dir(project_root)
        .join("chats")
        .join(scope)
        .join(format!("{session_id}.json"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat_store::{ChatMessage, ContentBlock, Role};
    use crate::test_support::{FsTmp, with_home};

    fn user_msg(text: &str) -> ChatMessage {
        ChatMessage {
            role: Role::User,
            content: vec![ContentBlock::Text(text.into())],
            timestamp: String::new(),
            is_priming: false,
        }
    }

    fn session(scope: &str, id: &str, msgs: Vec<ChatMessage>) -> ChatSession {
        let mut s = ChatSession::new(scope.into());
        s.id = id.into();
        s.messages = msgs;
        s
    }

    fn file_bytes(path: &Path) -> Vec<u8> {
        std::fs::read(path).unwrap_or_default()
    }

    // @spec chat/session-sharing Drive-only writes: Displayed-only session is never written
    #[test]
    fn displayed_only_session_is_never_written() {
        let tmp = FsTmp::new();
        with_home(tmp.path(), || {
            let root = tmp.path().join("project");
            std::fs::create_dir_all(&root).unwrap();

            // GIVEN a session already on disk and loaded for display only
            let mut s = session("scope-a", "1", vec![user_msg("disk")]);
            chat_store::save_session(&s, Some(&root)).unwrap();
            let path = session_file_path("scope-a", "1", Some(&root));
            let before = file_bytes(&path);

            // Local UI-only mutation of the in-memory copy
            s.messages.push(user_msg("ui-only"));

            // WHEN the app updates other local UI state (would-be persist while displayed)
            let outcome = persist_driven(&s, DriveRole::Displayed, Some(&root));

            // THEN the session file on disk is unchanged
            assert!(matches!(outcome, PersistOutcome::SkippedDisplayed));
            assert_eq!(file_bytes(&path), before);
            let loaded = chat_store::load_sessions_for("scope-a", Some(&root));
            assert_eq!(loaded[0].messages.len(), 1);
        });
    }

    // @spec chat/session-sharing Drive-only writes: Completing a local turn writes the session
    #[test]
    fn completing_a_local_turn_writes_the_session() {
        let tmp = FsTmp::new();
        with_home(tmp.path(), || {
            let root = tmp.path().join("project");
            std::fs::create_dir_all(&root).unwrap();

            // GIVEN a session the app is actively driving
            let mut s = session("scope-b", "2", vec![user_msg("hello")]);
            s.is_streaming = true;

            // WHEN a local turn on that session completes
            s.is_streaming = false;
            s.messages.push(ChatMessage {
                role: Role::Assistant,
                content: vec![ContentBlock::Text("done".into())],
                timestamp: String::new(),
                is_priming: false,
            });
            let outcome = persist_driven(&s, DriveRole::Driven, Some(&root));

            // THEN the session file on disk reflects the completed turn
            assert!(matches!(outcome, PersistOutcome::Written));
            let loaded = chat_store::load_sessions_for("scope-b", Some(&root));
            assert_eq!(loaded.len(), 1);
            assert_eq!(loaded[0].messages.len(), 2);
        });
    }

    // @spec chat/session-sharing External change reload: External write reloads a displayed idle session
    #[test]
    fn external_write_reloads_a_displayed_idle_session() {
        let tmp = FsTmp::new();
        with_home(tmp.path(), || {
            let root = tmp.path().join("project");
            std::fs::create_dir_all(&root).unwrap();

            // GIVEN a session displayed locally with no in-flight turn
            let local = session("scope-c", "3", vec![user_msg("old")]);
            chat_store::save_session(&local, Some(&root)).unwrap();
            let mut list = vec![local];
            assert!(!is_in_flight(&list[0]));

            // WHEN another process writes that session file
            let external = session("scope-c", "3", vec![user_msg("old"), user_msg("from-other")]);
            chat_store::save_session(&external, Some(&root)).unwrap();
            let event = ExternalSessionEvent::Modified {
                scope: "scope-c".into(),
                session_id: "3".into(),
            };
            let result = apply_external_to_list(&mut list, &event, Some(&root));

            // THEN the in-memory session matches the newly persisted content
            assert_eq!(result, ApplyResult::Reloaded);
            assert_eq!(list[0].messages.len(), 2);
        });
    }

    // @spec chat/session-sharing External change reload: External removal drops a displayed idle session
    #[test]
    fn external_removal_drops_a_displayed_idle_session() {
        let tmp = FsTmp::new();
        with_home(tmp.path(), || {
            let root = tmp.path().join("project");
            std::fs::create_dir_all(&root).unwrap();

            // GIVEN a session displayed locally with no in-flight turn
            let local = session("scope-d", "4", vec![user_msg("bye")]);
            chat_store::save_session(&local, Some(&root)).unwrap();
            let mut list = vec![local];

            // WHEN another process removes that session file
            chat_store::delete_session("scope-d", "4", Some(&root));
            let event = ExternalSessionEvent::Removed {
                scope: "scope-d".into(),
                session_id: "4".into(),
            };
            let result = apply_external_to_list(&mut list, &event, Some(&root));

            // THEN the session is no longer present in the local session list
            assert_eq!(result, ApplyResult::Removed);
            assert!(list.is_empty());
        });
    }

    // @spec chat/session-sharing In-flight external shield: External change during a local turn does not replace mid-turn state
    #[test]
    fn external_change_during_a_local_turn_does_not_replace_mid_turn_state() {
        let tmp = FsTmp::new();
        with_home(tmp.path(), || {
            let root = tmp.path().join("project");
            std::fs::create_dir_all(&root).unwrap();

            // GIVEN a session with a local in-flight turn
            let mut local = session("scope-e", "5", vec![user_msg("local")]);
            local.is_streaming = true;
            local.pending_text = "streaming…".into();
            chat_store::save_session(
                &session("scope-e", "5", vec![user_msg("stale-on-disk")]),
                Some(&root),
            )
            .unwrap();
            let mut list = vec![local.clone()];

            // WHEN another process writes that session file before the turn settles
            let external = session("scope-e", "5", vec![user_msg("external-overwrite")]);
            chat_store::save_session(&external, Some(&root)).unwrap();
            let event = ExternalSessionEvent::Modified {
                scope: "scope-e".into(),
                session_id: "5".into(),
            };
            let result = apply_external_to_list(&mut list, &event, Some(&root));

            // THEN in-memory continues to reflect the local turn; external not applied mid-turn
            assert_eq!(result, ApplyResult::IgnoredInFlight);
            assert!(list[0].is_streaming);
            assert_eq!(list[0].pending_text, "streaming…");
            assert_eq!(list[0].messages.len(), 1);
            // External file had a different single message — shield kept local.
            let on_disk = chat_store::load_sessions_for("scope-e", Some(&root));
            assert_eq!(on_disk[0].messages.len(), 1);
        });
    }

    // @spec chat/session-sharing In-flight external shield: After local turn settlement, a later external change reloads
    #[test]
    fn after_local_turn_settlement_a_later_external_change_reloads() {
        let tmp = FsTmp::new();
        with_home(tmp.path(), || {
            let root = tmp.path().join("project");
            std::fs::create_dir_all(&root).unwrap();

            // GIVEN a session whose local turn has just settled
            let mut local = session("scope-f", "6", vec![user_msg("settled")]);
            local.is_streaming = false;
            chat_store::save_session(&local, Some(&root)).unwrap();
            let mut list = vec![local];

            // WHEN another process writes that session file afterward
            let external = session(
                "scope-f",
                "6",
                vec![user_msg("settled"), user_msg("later-external")],
            );
            chat_store::save_session(&external, Some(&root)).unwrap();
            let event = ExternalSessionEvent::Modified {
                scope: "scope-f".into(),
                session_id: "6".into(),
            };
            let result = apply_external_to_list(&mut list, &event, Some(&root));

            // THEN the in-memory session matches the newly persisted content
            assert_eq!(result, ApplyResult::Reloaded);
            assert_eq!(list[0].messages.len(), 2);
        });
    }

    // @spec chat/session-sharing Concurrent drive is last-write-wins: Later write is the persisted content
    #[test]
    fn later_write_is_the_persisted_content() {
        let tmp = FsTmp::new();
        with_home(tmp.path(), || {
            let root = tmp.path().join("project");
            std::fs::create_dir_all(&root).unwrap();

            // GIVEN two apps each completing a write of the same session
            // (no lock files — both use atomic rename)
            let first = session("scope-g", "7", vec![user_msg("first-writer")]);
            let second = session("scope-g", "7", vec![user_msg("second-writer")]);

            // WHEN the second write finishes after the first
            assert!(matches!(
                persist_driven(&first, DriveRole::Driven, Some(&root)),
                PersistOutcome::Written
            ));
            assert!(matches!(
                persist_driven(&second, DriveRole::Driven, Some(&root)),
                PersistOutcome::Written
            ));

            // THEN the session file on disk matches the second write
            let loaded = chat_store::load_sessions_for("scope-g", Some(&root));
            assert_eq!(loaded.len(), 1);
            assert_eq!(loaded[0].messages.len(), 1);
            match &loaded[0].messages[0].content[..] {
                [ContentBlock::Text(t)] => assert_eq!(t, "second-writer"),
                other => panic!("unexpected content {other:?}"),
            }
            // No lock file beside the session
            let path = session_file_path("scope-g", "7", Some(&root));
            let dir = path.parent().unwrap();
            let locks: Vec<_> = std::fs::read_dir(dir)
                .unwrap()
                .flatten()
                .filter(|e| {
                    e.path()
                        .extension()
                        .is_some_and(|ext| ext == "lock" || ext == "lck")
                })
                .collect();
            assert!(locks.is_empty(), "no multi-process lock files");
        });
    }
}
