//! Persistence scenarios that exercise duckboard area helpers over duckcore::chat_store.

#[cfg(test)]
mod tests {
    use crate::chat_store::{ContentBlock, Exploration, Role, load_sessions_for};
    use crate::test_support::{FsTmp, with_home};

    fn user_msg(text: &str) -> crate::chat_store::ChatMessage {
        crate::chat_store::ChatMessage {
            role: Role::User,
            content: vec![ContentBlock::Text(text.into())],
            timestamp: String::new(),
            is_priming: false,
        }
    }

    /// @spec chat/persistence In-flight turn durability: An in-flight turn survives a promotion
    #[test]
    fn in_flight_turn_survives_promotion() {
        use crate::area::change::{State, promote_exploration};
        use crate::area::interaction::{AgentSession, InteractionState};
        use crate::scope::{Scope, ScopeKind};
        use std::collections::HashMap;

        let tmp = FsTmp::new();
        with_home(tmp.path(), || {
            let root = tmp.path().join("project-promote");
            std::fs::create_dir_all(&root).unwrap();

            let mut state = State::new(Some(&root));
            let exp_id = "exploration-1".to_string();
            state.explorations.push(Exploration {
                id: exp_id.clone(),
                display_name: "Exp".into(),
                idea_path: None,
                archived_at: None,
                session_count: 0,
            });

            // GIVEN a session with messages streamed since its last persist:
            // the session lives only in memory (never saved to disk).
            let mut ax = AgentSession::new(exp_id.clone(), ScopeKind::Exploration);
            ax.session.id = "sess-1".into();
            ax.session.messages = vec![user_msg("streamed one"), user_msg("streamed two")];
            ax.needs_flush = true;
            ax.session.is_streaming = true;
            let mut ix = InteractionState::default();
            ix.sessions.push(ax);
            let mut interactions: HashMap<Scope, InteractionState> = HashMap::new();
            interactions.insert(Scope::Exploration(exp_id.clone()), ix);

            // WHEN the scope's in-memory state is migrated by a promotion.
            promote_exploration(
                &mut state,
                &mut interactions,
                &exp_id,
                "real-change",
                Some(&root),
            );

            // THEN the persisted session under the new scope includes those
            // streamed messages.
            let persisted = load_sessions_for("real-change", Some(&root));
            let sess = persisted.iter().find(|s| s.id == "sess-1").unwrap();
            assert_eq!(sess.messages.len(), 2);
        });
    }

    /// @spec chat/build-pilot Scope continuity: Armed pilot survives exploration promotion to change
    #[test]
    fn armed_pilot_survives_exploration_promotion_to_change() {
        use crate::area::change::{State, promote_exploration};
        use crate::area::interaction::{AgentSession, InteractionState};
        use crate::build_pilot::PilotState;
        use crate::scope::{Scope, ScopeKind};
        use crate::slash_commands::BuildPilotMode;
        use std::collections::HashMap;

        let tmp = FsTmp::new();
        with_home(tmp.path(), || {
            let root = tmp.path().join("project-pilot-promote");
            std::fs::create_dir_all(&root).unwrap();

            let mut state = State::new(Some(&root));
            let exp_id = "exploration-pilot".to_string();
            state.explorations.push(Exploration {
                id: exp_id.clone(),
                display_name: "Pilot exp".into(),
                idea_path: None,
                archived_at: None,
                session_count: 0,
            });

            // GIVEN an exploration session armed in auto mode
            let mut ax = AgentSession::new(exp_id.clone(), ScopeKind::Exploration);
            ax.session.id = "sess-pilot".into();
            ax.pilot = PilotState::Armed(BuildPilotMode::Auto);
            let mut ix = InteractionState::default();
            ix.sessions.push(ax);
            let mut interactions: HashMap<Scope, InteractionState> = HashMap::new();
            interactions.insert(Scope::Exploration(exp_id.clone()), ix);

            // WHEN that exploration is promoted into a change session
            promote_exploration(
                &mut state,
                &mut interactions,
                &exp_id,
                "pilot-change",
                Some(&root),
            );

            // THEN the change session pilot remains armed in auto mode
            let change_ix = interactions
                .get(&Scope::Change("pilot-change".into()))
                .expect("change scope");
            let sess = change_ix
                .sessions
                .iter()
                .find(|s| s.session.id == "sess-pilot")
                .expect("session");
            assert_eq!(
                sess.pilot,
                PilotState::Armed(BuildPilotMode::Auto)
            );
            assert_eq!(sess.scope_kind, ScopeKind::Change);
        });
    }

    /// @spec chat/persistence In-flight turn durability: Streamed messages are persisted before turn completion
    #[test]
    fn eager_flush_persists_streamed_messages_before_turn_completion() {
        use crate::area::interaction::{AgentSession, InteractionState, flush_dirty_sessions};
        use crate::scope::ScopeKind;

        let tmp = FsTmp::new();
        with_home(tmp.path(), || {
            let root = tmp.path().join("project-eager");
            std::fs::create_dir_all(&root).unwrap();

            // GIVEN a turn that has streamed messages and has not yet completed.
            let mut ax = AgentSession::new("eager-scope".into(), ScopeKind::Change);
            ax.session.id = "sess-eager".into();
            ax.session.messages = vec![user_msg("streamed so far")];
            ax.session.is_streaming = true;
            ax.needs_flush = true;
            let mut ix = InteractionState::default();
            ix.sessions.push(ax);

            // WHEN an eager flush occurs.
            flush_dirty_sessions(&mut ix, Some(&root));

            // THEN the persisted session includes the messages streamed so far.
            let persisted = load_sessions_for("eager-scope", Some(&root));
            let sess = persisted.iter().find(|s| s.id == "sess-eager").unwrap();
            assert_eq!(sess.messages.len(), 1);
        });
    }

    /// @spec chat/persistence In-flight turn durability: Eager flush includes pending reasoning as Reasoning content
    #[test]
    fn eager_flush_includes_pending_reasoning_as_reasoning_content() {
        use crate::area::interaction::{AgentSession, InteractionState, flush_dirty_sessions};
        use crate::scope::ScopeKind;

        let tmp = FsTmp::new();
        with_home(tmp.path(), || {
            let root = tmp.path().join("project-eager-reasoning");
            std::fs::create_dir_all(&root).unwrap();

            // GIVEN a turn that has streamed reasoning into the pending
            // reasoning buffer and has not yet completed.
            let mut ax = AgentSession::new("eager-reasoning-scope".into(), ScopeKind::Change);
            ax.session.id = "sess-eager-r".into();
            ax.session.pending_reasoning = "thinking out loud".into();
            ax.session.is_streaming = true;
            ax.needs_flush = true;
            let mut ix = InteractionState::default();
            ix.sessions.push(ax);

            // WHEN an eager flush occurs.
            flush_dirty_sessions(&mut ix, Some(&root));

            // THEN the persisted session includes that reasoning as Reasoning
            // content AND that body is not stored as Text content.
            let persisted = load_sessions_for("eager-reasoning-scope", Some(&root));
            let sess = persisted.iter().find(|s| s.id == "sess-eager-r").unwrap();
            assert_eq!(sess.messages.len(), 1);
            match &sess.messages[0].content[..] {
                [ContentBlock::Reasoning(body)] => {
                    assert_eq!(body, "thinking out loud");
                }
                other => panic!("expected Reasoning content, got {other:?}"),
            }
            // In-memory session is left untouched (snapshot only).
            assert_eq!(
                ix.sessions[0].session.pending_reasoning,
                "thinking out loud"
            );
        });
    }

}
