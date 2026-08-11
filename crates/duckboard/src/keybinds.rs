//! Focus-aware keybinding resolvers — one place to look for "what does this
//! action do given the current focus and area state?"
//!
//! Each `keybind_*` function is named for the *action* it represents, not the
//! key that triggers it today. Adding a new shortcut whose behavior depends
//! on focus belongs here; the dispatcher in `main::update` should stay a thin
//! `if mods.command() && key == … { keybind_thing(state) }` cascade so the
//! resolution rules don't drift back into the keypress arm.

use crate::State;
use crate::area::interaction::ActiveTab;
use crate::area::{self, Area};
use crate::widget::find::FindTarget;
use crate::widget::tab_bar::{self, ActiveTab as TabActive};

/// Which column the user last interacted with — drives focus-conditional
/// shortcut resolution (cmd+n, cmd+f, …). Updated lazily by
/// `main::update_focused_column` in response to chat/editor messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedColumn {
    Content,
    Chat,
}

// ── Action enums ────────────────────────────────────────────────────────────

/// What "new" should do given current focus + area.
#[derive(Debug, Clone)]
pub enum NewAction {
    /// Open the new-file modal. Seeded by the dispatcher with the focused
    /// editor tab's directory (or empty for project root).
    OpenNewFile,
    /// Add a new idea in the Ideas area.
    AddIdea,
    /// Spawn a new chat session in the active Change. Payload is the
    /// routing key the dispatcher already resolved.
    NewChatSession(String),
    /// Add a new exploration in the Change area.
    AddExploration,
}

/// What "save" should do. Currently only one focus-conditional flavor: the
/// Ideas pinned tab routes through the frontmatter-aware writer instead of
/// the generic file-save path.
#[derive(Debug, Clone)]
pub enum SaveAction {
    SaveIdeaBody,
}

// ── Resolvers ───────────────────────────────────────────────────────────────

/// `cmd+n` today. Content focus opens the new-file modal in every area;
/// otherwise area-scoped behavior takes over.
pub fn keybind_new(state: &State) -> Option<NewAction> {
    state.project.project_root.as_ref()?;
    if state.focused_column == Some(FocusedColumn::Content) {
        return Some(NewAction::OpenNewFile);
    }
    match state.active_area {
        Area::Ideas => Some(NewAction::AddIdea),
        Area::Change => {
            let routing_key = state.active_interaction_key();
            let real_change_selected =
                routing_key.is_some() && !state.change.is_exploration_selected();
            if real_change_selected {
                routing_key.map(NewAction::NewChatSession)
            } else {
                Some(NewAction::AddExploration)
            }
        }
        _ => None,
    }
}

/// `cmd+s` today. The generic in-editor save flows through the editor's own
/// `SaveRequested` action; this resolver only flags the Ideas-pinned-tab
/// case which needs frontmatter-aware handling.
pub fn keybind_save(state: &State) -> Option<SaveAction> {
    if state.active_area == Area::Ideas
        && matches!(state.tabs.active, TabActive::Preview)
        && state
            .tabs
            .active_tab()
            .is_some_and(|t| t.id.starts_with(area::ideas::PINNED_TAB_PREFIX))
    {
        return Some(SaveAction::SaveIdeaBody);
    }
    None
}

/// `cmd+w` today. Returns the logical tab index to close (accounting for the
/// preview slot), or `None` when focus belongs to the chat input / terminal,
/// or when no closable tab is active.
pub fn keybind_close(state: &State) -> Option<usize> {
    if !matches!(
        state.active_area,
        Area::Change | Area::Caps | Area::Codex | Area::Ideas
    ) {
        return None;
    }
    let TabActive::File(fi) = state.tabs.active else {
        return None;
    };
    let chat_focused = state
        .active_scope()
        .and_then(|scope| state.interactions.get(&scope))
        .and_then(|ix| ix.active())
        .is_some_and(|ax| ax.chat_input_focused);
    let terminal_focused = state
        .active_scope()
        .and_then(|scope| state.interactions.get(&scope))
        .is_some_and(|ix| ix.terminal_focused);
    if chat_focused || terminal_focused {
        return None;
    }
    Some(if state.tabs.preview.is_some() {
        fi + 1
    } else {
        fi
    })
}

/// `cmd+k` today. True when the chat tab is the visible, active interaction
/// tab — the runtime check for "is there actually a tentative to pin" stays
/// at the call site since it needs `&mut`.
pub fn keybind_pin_selection(state: &State) -> bool {
    let Some(scope) = state.active_scope() else {
        return false;
    };
    let Some(ix) = state.interactions.get(&scope) else {
        return false;
    };
    ix.visible && ix.active_tab == ActiveTab::Chat && ix.active().is_some()
}

/// `cmd+r` today. True when chat input is focused and there's at least one
/// attachment (pinned or tentative) to clear.
pub fn keybind_clear_attachments(state: &State) -> bool {
    let Some(scope) = state.active_scope() else {
        return false;
    };
    let Some(ix) = state.interactions.get(&scope) else {
        return false;
    };
    if !ix.visible || ix.active_tab != ActiveTab::Chat {
        return false;
    }
    let Some(ax) = ix.active() else {
        return false;
    };
    ax.chat_input_focused && (!ax.selection_pinned.is_empty() || ax.selection_tentative.is_some())
}

/// `cmd+f` today. The local-find target for the focused column, or `None`
/// when neither column is in a state to host find (terminal focused, no
/// editor tab open, no chat session, search-stack active).
pub fn keybind_find(state: &State) -> Option<FindTarget> {
    match state.focused_column? {
        FocusedColumn::Content => {
            let tab = state.tabs.active_tab()?;
            match &tab.view {
                tab_bar::TabView::Editor { .. } | tab_bar::TabView::Diff { .. } => {
                    Some(FindTarget::editor(tab.id.clone()))
                }
                tab_bar::TabView::SearchStack { .. } => None,
            }
        }
        FocusedColumn::Chat => {
            let scope = state.active_scope()?;
            let ix = state.interactions.get(&scope)?;
            let ax = ix.active()?;
            Some(FindTarget::chat(ix.instance_id, ax.session.id.clone()))
        }
    }
}

/// ⌘↑/↓/←/→ chat landmark jumps when eligible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatLandmarkAction {
    HistoryTop,
    HistoryBottom,
    PrevAnswer,
    NextAnswer,
}

/// True when chat landmark shortcuts may run: chat tab visible with an
/// active session, terminal not focused. Modal ownership is enforced by
/// the caller's early-returns (find, file finder, …).
pub fn keybind_chat_landmarks(state: &State) -> bool {
    let Some(scope) = state.active_scope() else {
        return false;
    };
    let Some(ix) = state.interactions.get(&scope) else {
        return false;
    };
    if ix.terminal_focused {
        return false;
    }
    ix.visible && ix.active_tab == ActiveTab::Chat && ix.active().is_some()
}

// ── List digit switch (Ctrl+1/2/3) ──────────────────────────────────────────

/// Target of a resolved list-digit chord.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListDigitAction {
    /// Change or exploration id (same payload as `SelectChange`).
    SelectChange(String),
    /// Idea absolute path (same payload as `SelectIdea`).
    SelectIdea(std::path::PathBuf),
}

/// `Ctrl+1/2/3` today. Resolves the nth painted row of the focused Change or
/// Ideas list. Returns `None` when gated out or when `n` has no painted row.
/// Does not apply selection — the dispatcher owns side effects.
pub fn keybind_list_digit(state: &State, n: u8) -> Option<ListDigitAction> {
    if !(1..=3).contains(&n) {
        return None;
    }
    state.project.project_root.as_ref()?;
    if state.navigation_keys_captured() {
        return None;
    }
    let idx = (n as usize) - 1;
    match state.active_area {
        Area::Change => {
            let ids = area::change::painted_live_queue_ids(
                &state.change,
                &state.project,
                &state.ideas,
                state.list_prefs(),
                state.project.project_root.as_deref(),
                state.change.changed_files.as_slice(),
            );
            ids.get(idx)
                .cloned()
                .map(ListDigitAction::SelectChange)
        }
        Area::Ideas => {
            let paths =
                area::ideas::painted_idea_paths(&state.ideas, &state.project, state.list_prefs());
            paths
                .get(idx)
                .cloned()
                .map(ListDigitAction::SelectIdea)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat_store::Exploration;
    use crate::data::ChangeData;
    use crate::idea_store::{Frontmatter, Idea, IdeaState};
    use crate::scope::{Scope, ScopeKind};
    use crate::area::interaction::{AgentSession, InteractionState};
    use std::path::PathBuf;

    fn change_data(name: &str) -> ChangeData {
        ChangeData {
            name: name.into(),
            prefix: "changes".into(),
            has_proposal: false,
            has_design: false,
            cap_tree: vec![],
            steps: vec![],
            reviews: vec![],
            shallow_mtime_nanos: None,
        }
    }

    fn with_project(state: &mut State) {
        state.project.project_root = Some(PathBuf::from("/tmp/list-digit-test-project"));
    }

    fn seed_live_change(state: &mut State, name: &str) {
        state.project.active_changes.push(change_data(name));
    }

    fn seed_exploration(state: &mut State, id: &str) {
        let mut exp = Exploration::new(1);
        exp.id = id.into();
        exp.display_name = id.into();
        state.change.explorations.push(exp);
    }

    fn seed_idea(state: &mut State, path: &str, section: IdeaState, tag_path: Vec<String>) {
        state.ideas.ideas.push(Idea {
            abs_path: PathBuf::from(path),
            state: section,
            primary_tag_path: tag_path,
            frontmatter: Frontmatter {
                title: path.into(),
                created: "2026-01-01T00:00:00Z".into(),
                ..Default::default()
            },
        });
    }

    // @spec shell/list-digit-switch Chord and eligibility: Change area with project resolves digit
    #[test]
    fn change_area_with_project_resolves_digit() {
        let mut state = State::for_tests();
        with_project(&mut state);
        state.active_area = Area::Change;
        seed_live_change(&mut state, "alpha");
        let action = keybind_list_digit(&state, 1);
        assert_eq!(
            action,
            Some(ListDigitAction::SelectChange("alpha".into()))
        );
    }

    // @spec shell/list-digit-switch Chord and eligibility: Ideas area with project resolves digit
    #[test]
    fn ideas_area_with_project_resolves_digit() {
        let mut state = State::for_tests();
        with_project(&mut state);
        state.active_area = Area::Ideas;
        seed_idea(&mut state, "/ideas/inbox/a.md", IdeaState::Inbox, vec![]);
        let action = keybind_list_digit(&state, 1);
        assert_eq!(
            action,
            Some(ListDigitAction::SelectIdea(PathBuf::from(
                "/ideas/inbox/a.md"
            )))
        );
    }

    // @spec shell/list-digit-switch Chord and eligibility: Other area does not resolve
    #[test]
    fn other_area_does_not_resolve() {
        let mut state = State::for_tests();
        with_project(&mut state);
        state.active_area = Area::Caps;
        seed_live_change(&mut state, "alpha");
        assert_eq!(keybind_list_digit(&state, 1), None);
    }

    // @spec shell/list-digit-switch Chord and eligibility: Modal or rename capture does not resolve
    #[test]
    fn modal_or_rename_capture_does_not_resolve() {
        let mut state = State::for_tests();
        with_project(&mut state);
        state.active_area = Area::Change;
        seed_live_change(&mut state, "alpha");
        state.file_finder.visible = true;
        assert_eq!(keybind_list_digit(&state, 1), None);
        state.file_finder.visible = false;
        state.change.renaming_exploration = Some("exp".into());
        assert_eq!(keybind_list_digit(&state, 1), None);
    }

    // @spec shell/list-digit-switch Chord and eligibility: Eligible with chat focused in Change or Ideas
    #[test]
    fn eligible_with_chat_focused_in_change_or_ideas() {
        let mut state = State::for_tests();
        with_project(&mut state);
        state.active_area = Area::Change;
        seed_live_change(&mut state, "alpha");
        state.change.selected_change = Some("alpha".into());
        let mut ax = AgentSession::new("alpha".into(), ScopeKind::Change);
        ax.chat_input_focused = true;
        let mut ix = InteractionState::default();
        ix.sessions.push(ax);
        ix.visible = true;
        ix.active_tab = ActiveTab::Chat;
        state
            .interactions
            .insert(Scope::Change("alpha".into()), ix);
        state.focused_column = Some(FocusedColumn::Chat);
        assert_eq!(
            keybind_list_digit(&state, 1),
            Some(ListDigitAction::SelectChange("alpha".into()))
        );
    }

    // @spec shell/list-digit-switch Painted-row index: Change nth row matches live-queue order
    #[test]
    fn change_nth_row_matches_live_queue_order() {
        let mut state = State::for_tests();
        with_project(&mut state);
        state.active_area = Area::Change;
        // Fixed created times so sort by Created is deterministic.
        seed_exploration(&mut state, "exploration-first");
        seed_live_change(&mut state, "beta-change");
        // Default sort is last-message; with no activity, order is stable via
        // title under the ordinary sort after pin prefix — use explicit sort.
        state.config.list.sort_key = crate::queue_list::SortKey::Created;
        // Prefer asserting against painted_live_queue_ids then digit target.
        let ids = area::change::painted_live_queue_ids(
            &state.change,
            &state.project,
            &state.ideas,
            state.list_prefs(),
            state.project.project_root.as_deref(),
            &[],
        );
        assert!(ids.len() >= 2, "expected two painted rows, got {ids:?}");
        let second = ids[1].clone();
        assert_eq!(
            keybind_list_digit(&state, 2),
            Some(ListDigitAction::SelectChange(second))
        );
    }

    // @spec shell/list-digit-switch Painted-row index: Ideas skips collapsed sections
    #[test]
    fn ideas_skips_collapsed_sections() {
        let mut state = State::for_tests();
        with_project(&mut state);
        state.active_area = Area::Ideas;
        seed_idea(
            &mut state,
            "/ideas/inbox/only.md",
            IdeaState::Inbox,
            vec![],
        );
        seed_idea(
            &mut state,
            "/ideas/archive/hidden.md",
            IdeaState::Archive,
            vec![],
        );
        // Archive defaults collapsed; only inbox paints.
        let paths =
            area::ideas::painted_idea_paths(&state.ideas, &state.project, state.list_prefs());
        assert_eq!(paths, vec![PathBuf::from("/ideas/inbox/only.md")]);
        assert_eq!(
            keybind_list_digit(&state, 1),
            Some(ListDigitAction::SelectIdea(PathBuf::from(
                "/ideas/inbox/only.md"
            )))
        );
        assert_eq!(keybind_list_digit(&state, 2), None);
    }

    // @spec shell/list-digit-switch Painted-row index: Ideas includes nested painted rows in order
    #[test]
    fn ideas_includes_nested_painted_rows_in_order() {
        let mut state = State::for_tests();
        with_project(&mut state);
        state.active_area = Area::Ideas;
        seed_idea(
            &mut state,
            "/ideas/inbox/root.md",
            IdeaState::Inbox,
            vec![],
        );
        seed_idea(
            &mut state,
            "/ideas/inbox/parser/nested.md",
            IdeaState::Inbox,
            vec!["parser".into()],
        );
        let paths =
            area::ideas::painted_idea_paths(&state.ideas, &state.project, state.list_prefs());
        assert_eq!(
            paths,
            vec![
                PathBuf::from("/ideas/inbox/root.md"),
                PathBuf::from("/ideas/inbox/parser/nested.md"),
            ]
        );
        assert_eq!(
            keybind_list_digit(&state, 2),
            Some(ListDigitAction::SelectIdea(PathBuf::from(
                "/ideas/inbox/parser/nested.md"
            )))
        );
    }

    // @spec shell/list-digit-switch Painted-row index: Out-of-range digit selects nothing
    #[test]
    fn out_of_range_digit_selects_nothing() {
        let mut state = State::for_tests();
        with_project(&mut state);
        state.active_area = Area::Change;
        seed_live_change(&mut state, "only");
        assert_eq!(keybind_list_digit(&state, 3), None);
    }
}
