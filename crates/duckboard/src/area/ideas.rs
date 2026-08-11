//! Ideas area — capture future work as files; flow them through
//! Inbox → Exploration → Change → Archive. List column only; content +
//! interaction columns are rendered by main.rs from the shared `state.tabs`
//! and `state.interactions`.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use iced::widget::{Space, button, column, container, row, text, text_input};
use iced::{Border, Center, Element, Length};

const ICON_IDEA: &[u8] = include_bytes!("../../assets/icon_idea.svg");
const ICON_EXPLORE: &[u8] = include_bytes!("../../assets/icon_explore.svg");
const ICON_BRANCH: &[u8] = include_bytes!("../../assets/icon_branch.svg");
const ICON_TAG: &[u8] = include_bytes!("../../assets/icon_tag.svg");
const ICON_CHANGE: &[u8] = include_bytes!("../../assets/icon_change.svg");
const ICON_ARCHIVE: &[u8] = include_bytes!("../../assets/icon_archive.svg");
const ICON_ORPHAN: &[u8] = include_bytes!("../../assets/icon_orphan.svg");

use super::interaction::{self, InteractionState};
use crate::data::ProjectData;
use crate::highlight::SyntaxHighlighter;
use crate::idea_store::{self, ArchiveKind, Idea, IdeaState};
use crate::scope::Scope;
use crate::theme;
use crate::widget::list_view::ListRow;
use crate::widget::text_edit::{EditorState, Pos};
use crate::widget::{collapsible, list_view, tab_bar, vertical_scroll};

/// Tab id prefix for the pinned idea body tab. Matches the
/// `file:` / `vcs:` prefix convention used by the change area's tab bar so
/// `widget::tab_bar` rendering and `main::handle_editor_action` can route
/// uniformly.
pub const PINNED_TAB_PREFIX: &str = "idea:";

/// `text_input::Id` for the inline tag-add field; used to focus on open.
pub const TAG_INPUT_ID: &str = "ideas-tag-input";

/// Focus id for the pinned idea body editor. Attached in `widget::tab_bar`
/// rendering when the active editor tab is the ideas pinned tab so that
/// `Message::AddIdea` can drop the cursor straight into the body.
pub const EDITOR_ID: &str = "ideas-editor";

pub fn pinned_tab_id(path: &Path) -> String {
    format!("{PINNED_TAB_PREFIX}{}", path.display())
}

// ── State ────────────────────────────────────────────────────────────────────

pub struct State {
    pub ideas: Vec<Idea>,
    /// Path of the currently selected idea (drives the pinned tab).
    pub selected: Option<PathBuf>,
    /// Per-section expand state. Defaults: all four sections expanded; entries
    /// only appear once the user collapses one.
    pub section_expanded: HashMap<IdeaState, bool>,
    /// Per-tag-tree-node *collapsed* set. Keys are
    /// `"<state-segment>/<tag1>/<tag2>"`. Tag groups default to expanded so
    /// new ideas surface immediately; the set is populated only when the
    /// user collapses a node.
    pub tag_collapsed: HashSet<String>,
    pub list_scroll: f32,
    /// Pending text in the inline tag-add input. `None` when the input is
    /// closed (showing as a `+ Tag` button); `Some` when the input is
    /// expanded and accepting keystrokes.
    pub tag_input: Option<String>,
    /// When `Some(idx)`, the input is editing an existing tag at that index
    /// (chip click). On submit, the tag is replaced (or removed if empty)
    /// instead of appended.
    pub tag_input_editing: Option<usize>,
    /// Per-idea format errors from the most recent save attempt. Cleared on
    /// the next successful format of that idea. Surfaced in the dashboard
    /// using the same UI as `ProjectAudit::artifact_errors`.
    pub format_errors: HashMap<PathBuf, Vec<String>>,
    /// Sort menu open (shared chrome; one open flag for Ideas headers).
    pub sort_menu_open: bool,
    /// Hovered idea path for mark outline / pillow overflow.
    pub hovered_idea: Option<PathBuf>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            ideas: Vec::new(),
            selected: None,
            section_expanded: HashMap::new(),
            tag_collapsed: HashSet::new(),
            list_scroll: 0.0,
            tag_input: None,
            tag_input_editing: None,
            format_errors: HashMap::new(),
            sort_menu_open: false,
            hovered_idea: None,
        }
    }
}

impl State {
    pub fn for_project(project_root: Option<&Path>) -> Self {
        let mut ideas = idea_store::load_all(project_root);
        ideas.sort_by(|a, b| b.frontmatter.created.cmp(&a.frontmatter.created));
        Self {
            ideas,
            ..Self::default()
        }
    }

    /// Find the idea whose chat scope key (`change` or `exploration`) equals
    /// `scope`. Used by main.rs to route subscription events back to the
    /// right interaction.
    pub fn idea_for_scope(&self, scope: &str) -> Option<&Idea> {
        self.ideas.iter().find(|i| i.scope_key() == Some(scope))
    }

    /// Find the idea attached to a given change name, if any.
    pub fn idea_path_for_change(&self, change_name: &str) -> Option<PathBuf> {
        self.ideas
            .iter()
            .find(|i| i.frontmatter.change.as_deref() == Some(change_name))
            .map(|i| i.abs_path.clone())
    }

    /// Compute the `Scope` for the idea at `path`, if any. Returns `None` for
    /// inbox-only ideas (no chat scope).
    pub fn scope_for_path(&self, path: &Path) -> Option<Scope> {
        let idea = self.ideas.iter().find(|i| i.abs_path == path)?;
        idea_scope(idea)
    }
}

fn idea_scope(idea: &Idea) -> Option<Scope> {
    if let Some(name) = idea.frontmatter.change.as_deref() {
        Some(Scope::Change(name.to_string()))
    } else {
        idea.frontmatter
            .exploration
            .as_deref()
            .map(|id| Scope::Exploration(id.to_string()))
    }
}

// ── Messages ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Message {
    AddIdea,
    SelectIdea(PathBuf),
    DeleteIdea(PathBuf),
    ArchiveIdea(PathBuf),
    UnarchiveIdea(PathBuf),
    /// Spawn a new exploration for the currently-selected idea. Main loop
    /// intercepts to mint the `Exploration` record and seed chat state, then
    /// stamps `frontmatter.exploration` and saves the file.
    StartExploration(PathBuf),
    /// Navigate to the Change area for the idea's attached change.
    /// Main loop intercepts to switch areas + select.
    OpenChange(String),

    ToggleSection(IdeaState),
    ToggleTagNode(String),

    Interaction(interaction::Msg),
    ScrollList(f32),

    /// Cmd-S on the pinned tab — serialize editor body + frontmatter, save.
    SaveBody,

    /// Open the inline tag-add input for the selected idea. Caller (main.rs)
    /// follows up with a `text_input::focus` task.
    OpenTagInput,
    CancelTagInput,
    TagInputChanged(String),
    /// Submit the tag-add input. Empty submissions just close the input.
    SubmitTagInput,
    /// Remove the tag at `idx` from the selected idea. If primary changes,
    /// the file is renamed by `save_idea`.
    RemoveTag(usize),
    /// Move the tag at `idx` to position 0 (primary). Triggers a file move.
    PromoteTag(usize),
    /// Open the tag input pre-filled with the existing tag's value so a
    /// click-to-edit interaction can rename it. Submit replaces the tag at
    /// `idx`; cancel restores it unchanged.
    EditTag(usize),
    /// Chip body was clicked. Main.rs reads the live modifier state and
    /// re-dispatches as either `PromoteTag` (shift held) or `EditTag`.
    ChipClick(usize),
    CycleIdeaMark(PathBuf),
    ToggleSortMenu,
    SetListSortKey(crate::queue_list::SortKey),
    ToggleListTypePillows,
    ToggleListPhasePillows,
    HoverIdea(PathBuf),
    UnhoverIdea(PathBuf),
}

// ── Update ───────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)] // area update: state + tabs + interaction + project + flags
pub fn update(
    state: &mut State,
    tabs: &mut tab_bar::TabState,
    interactions: &mut HashMap<Scope, InteractionState>,
    message: Message,
    project: &ProjectData,
    highlighter: &SyntaxHighlighter,
    agent_input_hints: bool,
    window_w: f32,
    vcs_workflow: crate::config::VcsWorkflow,
    viewer_style: crate::config::ViewerStyle,
) {
    match message {
        Message::AddIdea => {
            if project.project_root.is_none() {
                tracing::warn!("ideas: ignoring AddIdea with no project loaded");
                return;
            }
            let mut idea = idea_store::new_idea();
            if let Err(e) = idea_store::save_idea(&mut idea, "", project.project_root.as_deref()) {
                tracing::warn!("failed to save new idea: {e}");
                return;
            }
            let path = idea.abs_path.clone();
            state.ideas.push(idea);
            state
                .ideas
                .sort_by(|a, b| b.frontmatter.created.cmp(&a.frontmatter.created));
            open_idea(
                state,
                tabs,
                interactions,
                &path,
                project,
                highlighter,
                window_w,
                viewer_style,
            );
        }
        Message::SelectIdea(path) => {
            open_idea(
                state,
                tabs,
                interactions,
                &path,
                project,
                highlighter,
                window_w,
                viewer_style,
            );
        }
        Message::DeleteIdea(path) => {
            if let Some(idx) = state.ideas.iter().position(|i| i.abs_path == path) {
                let idea = state.ideas.remove(idx);
                if let Some(scope) = idea_scope(&idea) {
                    interactions.remove(&scope);
                }
                idea_store::delete_idea(&idea, project.project_root.as_deref());
            }
            if state.selected.as_ref() == Some(&path) {
                state.selected = None;
                tabs.preview = None;
            }
        }
        Message::ArchiveIdea(path) => {
            let mut moved: Option<(PathBuf, String)> = None;
            if let Some(idea) = state.ideas.iter_mut().find(|i| i.abs_path == path) {
                idea.state = IdeaState::Archive;
                idea.frontmatter.archived = Some(ArchiveKind::Manual);
                let body = idea_store::read_body(&idea.abs_path).unwrap_or_default();
                if let Err(e) = idea_store::save_idea(idea, &body, project.project_root.as_deref())
                {
                    tracing::warn!("failed to archive idea: {e}");
                } else {
                    moved = Some((idea.abs_path.clone(), idea.display_title()));
                }
            }
            if let Some((new_path, new_title)) = moved {
                refresh_after_move(state, tabs, &path, &new_path, &new_title);
            }
        }
        Message::UnarchiveIdea(path) => {
            let mut moved: Option<(PathBuf, String)> = None;
            if let Some(idea) = state.ideas.iter_mut().find(|i| i.abs_path == path) {
                idea.frontmatter.archived = None;
                idea.state = if idea.frontmatter.change.is_some() {
                    IdeaState::Change
                } else if idea.frontmatter.exploration.is_some() {
                    IdeaState::Exploration
                } else {
                    IdeaState::Inbox
                };
                let body = idea_store::read_body(&idea.abs_path).unwrap_or_default();
                if let Err(e) = idea_store::save_idea(idea, &body, project.project_root.as_deref())
                {
                    tracing::warn!("failed to unarchive idea: {e}");
                } else {
                    moved = Some((idea.abs_path.clone(), idea.display_title()));
                }
            }
            if let Some((new_path, new_title)) = moved {
                refresh_after_move(state, tabs, &path, &new_path, &new_title);
            }
        }
        Message::StartExploration(_) | Message::OpenChange(_) => {
            // Both are intercepted in main.rs — main owns explorations, area
            // switching, and the cross-area selection.
        }
        Message::ToggleSection(s) => {
            let entry = state
                .section_expanded
                .entry(s)
                .or_insert_with(|| section_default_expanded(s));
            *entry = !*entry;
        }
        Message::ToggleTagNode(key) => {
            if !state.tag_collapsed.remove(&key) {
                state.tag_collapsed.insert(key);
            }
        }
        Message::Interaction(msg) => {
            handle_interaction(
                state,
                tabs,
                interactions,
                msg,
                project,
                highlighter,
                agent_input_hints,
                window_w,
                vcs_workflow,
                viewer_style,
            );
        }
        Message::ScrollList(offset) => {
            state.list_scroll = offset;
        }
        Message::SaveBody => {
            save_pinned_tab(state, tabs, interactions, project, highlighter);
        }
        Message::OpenTagInput => {
            state.tag_input = Some(String::new());
            state.tag_input_editing = None;
        }
        Message::CancelTagInput => {
            state.tag_input = None;
            state.tag_input_editing = None;
        }
        Message::TagInputChanged(s) => {
            if state.tag_input.is_some() {
                state.tag_input = Some(s);
            }
        }
        Message::SubmitTagInput => {
            let raw = state.tag_input.take().unwrap_or_default();
            let editing = state.tag_input_editing.take();
            let cleaned: String = raw.trim().trim_start_matches('#').trim().to_string();
            let Some(path) = state.selected.clone() else {
                return;
            };
            match editing {
                Some(idx) => {
                    apply_tag_change(state, tabs, project, &path, |tags| {
                        if idx >= tags.len() {
                            return;
                        }
                        if cleaned.is_empty() {
                            tags.remove(idx);
                            return;
                        }
                        tags[idx] = cleaned;
                        // Dedup keeping first occurrence — if the new value
                        // collides with another tag, the duplicate is dropped
                        // rather than yielding two identical entries.
                        let mut seen = std::collections::HashSet::new();
                        tags.retain(|t| seen.insert(t.clone()));
                    });
                }
                None => {
                    if cleaned.is_empty() {
                        return;
                    }
                    apply_tag_change(state, tabs, project, &path, |tags| {
                        if !tags.iter().any(|t| t == &cleaned) {
                            tags.push(cleaned);
                        }
                    });
                }
            }
        }
        Message::RemoveTag(idx) => {
            let Some(path) = state.selected.clone() else {
                return;
            };
            apply_tag_change(state, tabs, project, &path, |tags| {
                if idx < tags.len() {
                    tags.remove(idx);
                }
            });
        }
        Message::PromoteTag(idx) => {
            let Some(path) = state.selected.clone() else {
                return;
            };
            apply_tag_change(state, tabs, project, &path, |tags| {
                if idx > 0 && idx < tags.len() {
                    let t = tags.remove(idx);
                    tags.insert(0, t);
                }
            });
        }
        Message::EditTag(idx) => {
            let Some(path) = state.selected.as_deref() else {
                return;
            };
            let Some(idea) = state.ideas.iter().find(|i| i.abs_path == path) else {
                return;
            };
            let Some(tag) = idea.frontmatter.tags.get(idx) else {
                return;
            };
            state.tag_input = Some(tag.clone());
            state.tag_input_editing = Some(idx);
        }
        Message::ChipClick(_) => {
            // Routed by main.rs — the modifier state lives in a process-wide
            // cell maintained by the global key event handler, and main.rs
            // re-dispatches as PromoteTag (shift held) or EditTag.
        }
        Message::CycleIdeaMark(_)
        | Message::SetListSortKey(_)
        | Message::ToggleListTypePillows
        | Message::ToggleListPhasePillows => {
            // Intercepted by main.rs (config / save).
        }
        Message::ToggleSortMenu => {
            state.sort_menu_open = !state.sort_menu_open;
        }
        Message::HoverIdea(path) => {
            state.hovered_idea = Some(path);
        }
        Message::UnhoverIdea(path) => {
            if state.hovered_idea.as_deref() == Some(path.as_path()) {
                state.hovered_idea = None;
            }
        }
    }
}

/// Mutate the selected idea's tag list, then persist. Tag changes can rename
/// the file (primary tag drives the path), so the pinned tab and selection
/// are re-targeted at the post-save path. Body comes from the live editor
/// when available so the user's unsaved markdown isn't reverted from disk.
fn apply_tag_change(
    state: &mut State,
    tabs: &mut tab_bar::TabState,
    project: &ProjectData,
    path: &Path,
    mutate: impl FnOnce(&mut Vec<String>),
) {
    let body = current_body(tabs, path);
    let mut moved: Option<(PathBuf, String)> = None;
    if let Some(idea) = state.ideas.iter_mut().find(|i| i.abs_path == path) {
        mutate(&mut idea.frontmatter.tags);
        if let Err(e) = idea_store::save_idea(idea, &body, project.project_root.as_deref()) {
            tracing::warn!("failed to save idea on tag change: {e}");
        } else {
            moved = Some((idea.abs_path.clone(), idea.display_title()));
        }
    }
    if let Some((np, nt)) = moved {
        refresh_after_move(state, tabs, path, &np, &nt);
    }
}

/// Read the body markdown for `path`, preferring the open editor over disk so
/// in-flight edits aren't dropped when an action triggers a re-save (tag
/// changes, primary-tag promotions). Falls back to the on-disk body when no
/// pinned tab matches.
fn current_body(tabs: &tab_bar::TabState, path: &Path) -> String {
    if let Some(tab) = tabs.preview.as_ref()
        && tab.id == pinned_tab_id(path)
        && let tab_bar::TabView::Editor { editor, .. } = &tab.view
    {
        return editor.lines.join("\n");
    }
    idea_store::read_body(path).unwrap_or_default()
}

#[allow(clippy::too_many_arguments)] // mirrors area update signature for interaction routing
fn handle_interaction(
    state: &mut State,
    _tabs: &mut tab_bar::TabState,
    interactions: &mut HashMap<Scope, InteractionState>,
    msg: interaction::Msg,
    project: &ProjectData,
    highlighter: &SyntaxHighlighter,
    agent_input_hints: bool,
    window_w: f32,
    vcs_workflow: crate::config::VcsWorkflow,
    viewer_style: crate::config::ViewerStyle,
) {
    let Some(path) = state.selected.clone() else {
        return;
    };
    let snapshot = state
        .ideas
        .iter()
        .find(|i| i.abs_path == path)
        .map(|i| (i.frontmatter.clone(), i.display_title(), idea_scope(i)));
    let Some((fm, title, maybe_scope)) = snapshot else {
        return;
    };
    let Some(scope) = maybe_scope else {
        return;
    };
    let scope_kind = scope.kind();
    let scope_key = scope.key().to_string();
    let is_multi = matches!(scope, Scope::Change(_));
    let Some(ix) = interactions.get_mut(&scope) else {
        return;
    };
    match msg {
        interaction::Msg::NewSession if is_multi => {
            interaction::ensure_sessions_with_label(
                ix,
                &scope_key,
                &title,
                scope_kind,
                project.project_root.as_deref(),
                highlighter,
                viewer_style,
            );
            // Donor is still active — inherit next actions before insert.
            let new_session = interaction::new_session_with_inherited_next_actions(
                ix,
                scope_key.clone(),
                scope_kind,
            );
            let _ = new_session.persist(project.project_root.as_deref());
            ix.sessions.insert(0, new_session);
            ix.active_session = 0;
            interaction::reconcile_display_names(&mut ix.sessions, &title);
        }
        interaction::Msg::SelectSession(id) if is_multi => {
            if let Some(idx) = ix.find_session_index(&id) {
                ix.active_session = idx;
            }
        }
        interaction::Msg::ClearSession => {
            interaction::clear_single_session(
                ix,
                &scope_key,
                &title,
                scope_kind,
                project.project_root.as_deref(),
            );
        }
        interaction::Msg::NewSession | interaction::Msg::SelectSession(_) => {
            // Single-session pre-promotion: ignore session management.
        }
        other => {
            interaction::update_with_side_effects(
                ix,
                other,
                &scope_key,
                &title,
                scope_kind,
                project.project_root.as_deref(),
                highlighter,
                agent_input_hints,
                window_w,
                vcs_workflow,
                viewer_style,
            );
        }
    }
    // Mirror `area::change::update` — refresh fast response after any
    // session-touching interaction so a freshly-created chat on an
    // idea-promoted Change picks up the right lifecycle options. Without
    // this, cmd+N on a promoted change lands on a session with empty chrome
    // (this update path doesn't go through change::update).
    // Dirty unknown here; main's refresh_changed_files / change update
    // re-applies with the real flag. Gate row still tracks session emptiness.
    super::change::refresh_fast_response(interactions, project, agent_input_hints, false);
    let _ = fm;
}

pub fn open_idea(
    state: &mut State,
    tabs: &mut tab_bar::TabState,
    interactions: &mut HashMap<Scope, InteractionState>,
    path: &Path,
    project: &ProjectData,
    highlighter: &SyntaxHighlighter,
    window_w: f32,
    viewer_style: crate::config::ViewerStyle,
) {
    let snapshot = state
        .ideas
        .iter()
        .find(|i| i.abs_path == path)
        .map(|i| (i.frontmatter.clone(), i.display_title(), idea_scope(i)));
    let Some((fm, title, maybe_scope)) = snapshot else {
        return;
    };

    let body = idea_store::read_body(path).unwrap_or_default();
    let mut editor = EditorState::new(&body);
    let syntax = highlighter.find_syntax("md");
    editor.highlight_spans = Some(highlighter.highlight_lines(&editor.lines, syntax));
    tabs.preview = Some(tab_bar::Tab {
        id: pinned_tab_id(path),
        title: title.clone(),
        view: tab_bar::TabView::Editor {
            editor,
            // None — Cmd-S routes through Message::SaveBody for frontmatter handling.
            path: None,
        },
    });
    tabs.active = tab_bar::ActiveTab::Preview;
    state.selected = Some(path.to_path_buf());

    if let Some(scope) = maybe_scope {
        let scope_kind = scope.kind();
        let scope_key = scope.key().to_string();
        let ix = interactions
            .entry(scope)
            .or_insert_with(|| InteractionState::for_window(window_w));
        interaction::ensure_sessions_with_label(
            ix,
            &scope_key,
            &title,
            scope_kind,
            project.project_root.as_deref(),
            highlighter,
            viewer_style,
        );
        if let Some(ax) = ix.active_mut() {
            ax.idea_description = Some(body.clone());
        }
        interaction::show_panel(ix, window_w);
    }
    let _ = fm;
}

/// Re-target the list selection and pinned tab to follow an idea whose file
/// just moved (Archive/Unarchive, Explore, change-promotion). Without this,
/// `state.selected` and `tabs.preview.id` keep the pre-move path so the list
/// row loses its highlight and the tab no longer matches the idea.
pub fn refresh_after_move(
    state: &mut State,
    tabs: &mut tab_bar::TabState,
    old_path: &Path,
    new_path: &Path,
    new_title: &str,
) {
    if old_path == new_path {
        return;
    }
    if state.selected.as_deref() == Some(old_path) {
        state.selected = Some(new_path.to_path_buf());
    }
    let old_id = pinned_tab_id(old_path);
    if let Some(tab) = tabs.preview.as_mut()
        && tab.id == old_id
    {
        tab.id = pinned_tab_id(new_path);
        tab.title = new_title.to_string();
    }
}

fn save_pinned_tab(
    state: &mut State,
    tabs: &mut tab_bar::TabState,
    interactions: &mut HashMap<Scope, InteractionState>,
    project: &ProjectData,
    highlighter: &SyntaxHighlighter,
) {
    let Some(path) = state.selected.clone() else {
        return;
    };
    let body = match tabs.preview.as_ref() {
        Some(tab) => match &tab.view {
            tab_bar::TabView::Editor { editor, .. } => editor.lines.join("\n"),
            _ => return,
        },
        None => return,
    };
    let format_result = crate::idea_format::format_body(&body, project.duckspec_root.as_deref());
    let body_to_save: String = match &format_result {
        Ok(formatted) => formatted.clone(),
        Err(_) => body.clone(),
    };
    let Some(idea) = state.ideas.iter_mut().find(|i| i.abs_path == path) else {
        return;
    };
    let prev_path = idea.abs_path.clone();
    if let Err(e) = idea_store::save_idea(idea, &body_to_save, project.project_root.as_deref()) {
        tracing::warn!("failed to save idea body: {e}");
        return;
    }
    let new_path = idea.abs_path.clone();
    let new_title = idea.display_title();
    let scope = idea_scope(idea);
    if new_path != prev_path {
        state.selected = Some(new_path.clone());
    }
    state.format_errors.remove(&prev_path);
    state.format_errors.remove(&new_path);
    match &format_result {
        Ok(_) => {}
        Err(errors) => {
            state.format_errors.insert(new_path.clone(), errors.clone());
        }
    }
    if let Some(tab) = tabs.preview.as_mut() {
        tab.id = pinned_tab_id(&new_path);
        tab.title = new_title.clone();
        if let tab_bar::TabView::Editor { editor, .. } = &mut tab.view {
            if let Ok(formatted) = &format_result {
                let mut next = EditorState::new(formatted);
                let last_line_idx = next.lines.len().saturating_sub(1);
                let row = editor.cursor.line.min(last_line_idx);
                let col = editor.cursor.col.min(next.lines[row].len());
                next.cursor = Pos::new(row, col);
                next.scroll_y = editor.scroll_y;
                let syntax = highlighter.find_syntax("md");
                next.highlight_spans = Some(highlighter.highlight_lines(&next.lines, syntax));
                *editor = next;
            } else {
                editor.dirty = false;
            }
        }
    }
    state
        .ideas
        .sort_by(|a, b| b.frontmatter.created.cmp(&a.frontmatter.created));
    // Idea title doubles as the chat session label for explorations and
    // change-promoted ideas; reconcile so an H1 rename rolls into the
    // session dropdown.
    if let Some(scope) = scope
        && let Some(ix) = interactions.get_mut(&scope)
    {
        interaction::reconcile_display_names(&mut ix.sessions, &new_title);
    }
}

// ── List column ──────────────────────────────────────────────────────────────

pub fn view_list<'a>(
    state: &'a State,
    project: &'a ProjectData,
    _tabs: &'a tab_bar::TabState,
    list_prefs: &'a crate::queue_list::ListConfig,
    phase_pill_list: bool,
) -> Element<'a, Message> {
    let mut sections = column![].spacing(0.0);
    for s in IdeaState::ALL {
        sections = sections.push(view_section(state, project, s, list_prefs, phase_pill_list));
    }

    vertical_scroll::view(state.list_scroll, Message::ScrollList, sections)
}

/// Archive starts collapsed so the list column stays quiet; other sections
/// default open so new ideas surface immediately.
fn section_default_expanded(section: IdeaState) -> bool {
    !matches!(section, IdeaState::Archive)
}

fn view_section<'a>(
    state: &'a State,
    project: &'a ProjectData,
    section: IdeaState,
    list_prefs: &'a crate::queue_list::ListConfig,
    phase_pill_list: bool,
) -> Element<'a, Message> {
    let label = match section {
        IdeaState::Inbox => "Inbox",
        IdeaState::Exploration => "Exploration",
        IdeaState::Change => "Change",
        IdeaState::Archive => "Archive",
    };
    let expanded = section_is_expanded(state, section);

    let count = state.ideas.iter().filter(|i| i.state == section).count();
    let header_label = format!("{label}  ({count})");

    let body: Element<'a, Message> = if expanded {
        let mut rows: Vec<ListRow<'a, Message>> = Vec::new();
        let selected = state.selected.as_deref();
        collect_section_rows(
            state,
            project,
            section,
            &[],
            0,
            selected,
            list_prefs,
            phase_pill_list,
            &mut rows,
        );
        list_view::view(rows, None)
    } else {
        Space::new().into()
    };

    let mut actions = row![].spacing(theme::SPACING_XS).align_y(iced::Center);
    actions = actions.push(ideas_sort_menu(
        state.sort_menu_open,
        list_prefs,
        phase_pill_list,
    ));
    if matches!(section, IdeaState::Inbox) {
        actions = actions.push(collapsible::add_button(Message::AddIdea));
    }

    collapsible::view_with_add_owned(
        header_label,
        expanded,
        Message::ToggleSection(section),
        Some(actions.into()),
        body,
    )
}

fn ideas_sort_menu<'a>(
    open: bool,
    prefs: &crate::queue_list::ListConfig,
    phase_pill_list: bool,
) -> Element<'a, Message> {
    let sort_btn = button(text("Sort").size(theme::font_sm()))
        .on_press(Message::ToggleSortMenu)
        .padding([theme::SPACING_XS, theme::SPACING_SM])
        .style(theme::icon_button);
    if !open {
        return sort_btn.into();
    }
    let mut menu = column![sort_btn].spacing(theme::SPACING_XS);
    for key in crate::queue_list::SortKey::ALL {
        let label = if prefs.sort_key == key {
            format!("• {}", key.label())
        } else {
            format!("  {}", key.label())
        };
        menu = menu.push(
            button(text(label).size(theme::font_sm()))
                .on_press(Message::SetListSortKey(key))
                .padding([theme::SPACING_XS, theme::SPACING_SM])
                .style(theme::icon_button),
        );
    }
    let type_label = if prefs.show_type_pillows {
        "• Type tags"
    } else {
        "  Type tags"
    };
    menu = menu.push(
        button(text(type_label).size(theme::font_sm()))
            .on_press(Message::ToggleListTypePillows)
            .padding([theme::SPACING_XS, theme::SPACING_SM])
            .style(theme::icon_button),
    );
    if !phase_pill_list {
        let phase_label = if prefs.show_phase_pillows {
            "• Phase"
        } else {
            "  Phase"
        };
        menu = menu.push(
            button(text(phase_label).size(theme::font_sm()))
                .on_press(Message::ToggleListPhasePillows)
                .padding([theme::SPACING_XS, theme::SPACING_SM])
                .style(theme::icon_button),
        );
    }
    menu.into()
}

// ── Shared Ideas paint-order (list view + digit index) ───────────────────────
//
// One pure path for idea ordering and tag expand. Digits index idea paths only
// (no tag chrome). The list walk paints chevrons but must not re-implement
// sort / child-tag / expand logic for digits alone.

/// Whether a lifecycle section body is expanded (list paint + digit index).
fn section_is_expanded(state: &State, section: IdeaState) -> bool {
    state
        .section_expanded
        .get(&section)
        .copied()
        .unwrap_or_else(|| section_default_expanded(section))
}

/// Collapse key for a tag folder under a section (`"{segment}/{tag/path}"`).
fn tag_node_key(section: IdeaState, tag_path: &[String]) -> String {
    format!("{}/{}", section.segment(), tag_path.join("/"))
}

fn tag_node_expanded(state: &State, section: IdeaState, tag_path: &[String]) -> bool {
    !state.tag_collapsed.contains(&tag_node_key(section, tag_path))
}

/// Sorted idea rows at one tag prefix (queue sort + fav pins). Shared by the
/// list body and digit paint-order walk.
fn sorted_direct_ideas_at<'a>(
    state: &'a State,
    project: &ProjectData,
    section: IdeaState,
    prefix: &[String],
    list_prefs: &crate::queue_list::ListConfig,
) -> Vec<&'a Idea> {
    let direct: Vec<&'a Idea> = state
        .ideas
        .iter()
        .filter(|i| i.state == section && i.primary_tag_path == prefix)
        .collect();
    let mut metas: Vec<crate::queue_list::QueueRowMeta> = direct
        .iter()
        .map(|idea| {
            // Always compute phase for SortKey::Phase ranking; display pillows
            // are suppressed separately when short phase pills own chrome.
            let phase = idea
                .frontmatter
                .change
                .as_deref()
                .and_then(|n| super::change::change_scope_facts(n, project))
                .map(|f| f.phase.to_string());
            crate::queue_list::QueueRowMeta {
                key: crate::queue_list::QueueKey::Idea(idea.abs_path.clone()),
                title: idea.display_title(),
                mark: idea.frontmatter.mark,
                favored_at: idea.frontmatter.favored_at.clone(),
                type_tags: crate::queue_list::type_tags_from_idea_tags(&idea.frontmatter.tags),
                phase,
                last_message_at: idea.scope_key().and_then(|scope| {
                    let sessions =
                        crate::chat_store::load_sessions_for(scope, project.project_root.as_deref());
                    crate::queue_list::last_message_activity_nanos(&sessions)
                }),
                created_at: Some(idea.frontmatter.created.clone()),
            }
        })
        .collect();
    crate::queue_list::sort_queue(&mut metas, list_prefs.sort_key);
    metas
        .iter()
        .filter_map(|m| match &m.key {
            crate::queue_list::QueueKey::Idea(p) => {
                direct.iter().copied().find(|i| i.abs_path == *p)
            }
            _ => None,
        })
        .collect()
}

/// Immediate child tag segment names under `prefix` (sorted, unique).
fn child_tag_names_at(state: &State, section: IdeaState, prefix: &[String]) -> Vec<String> {
    let mut children: Vec<&str> = state
        .ideas
        .iter()
        .filter(|i| i.state == section && i.primary_tag_path.starts_with(prefix))
        .filter_map(|i| i.primary_tag_path.get(prefix.len()).map(String::as_str))
        .collect();
    children.sort();
    children.dedup();
    children.into_iter().map(str::to_string).collect()
}

/// Build a flat sequence of `ListRow`s for one section's tag tree, matching
/// the tree styling used by Change/Caps/Codex: chevron-leading rows for tag
/// groups, indent-leading rows for ideas. Ideas at depth `N` and tag groups
/// at depth `N` use the same indent so they sit at the same visual level.
/// Idea order and tag expand come from the shared paint-order helpers above.
fn collect_section_rows<'a>(
    state: &'a State,
    project: &'a ProjectData,
    section: IdeaState,
    prefix: &[String],
    depth: usize,
    selected: Option<&Path>,
    list_prefs: &crate::queue_list::ListConfig,
    phase_pill_list: bool,
    out: &mut Vec<ListRow<'a, Message>>,
) {
    let ordered = sorted_direct_ideas_at(state, project, section, prefix, list_prefs);
    let children = child_tag_names_at(state, section, prefix);
    let has_tag_siblings = !children.is_empty();

    for idea in &ordered {
        out.push(idea_list_row(
            idea,
            project,
            depth,
            selected,
            has_tag_siblings,
            list_prefs,
            phase_pill_list,
            state.hovered_idea.as_deref() == Some(idea.abs_path.as_path()),
        ));
    }

    for child in children {
        let mut next_prefix = prefix.to_vec();
        next_prefix.push(child.clone());
        let key = tag_node_key(section, &next_prefix);
        let expanded = tag_node_expanded(state, section, &next_prefix);
        let leading = collapsible::chevron(expanded);
        out.push(
            ListRow::new(child)
                .leading(leading)
                .icon(ICON_TAG)
                .indent(depth)
                .on_press(Message::ToggleTagNode(key)),
        );
        if expanded {
            collect_section_rows(
                state,
                project,
                section,
                &next_prefix,
                depth + 1,
                selected,
                list_prefs,
                phase_pill_list,
                out,
            );
        }
    }
}

/// Selectable idea paths in paint order for expanded sections only. Same
/// sort/expand walk as the Ideas list view for idea rows; tag-folder chrome is
/// not indexed (only `SelectIdea` targets).
pub(crate) fn painted_idea_paths(
    state: &State,
    project: &ProjectData,
    list_prefs: &crate::queue_list::ListConfig,
) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for section in IdeaState::ALL {
        if !section_is_expanded(state, section) {
            continue;
        }
        collect_section_idea_paths(state, project, section, &[], list_prefs, &mut out);
    }
    out
}

fn collect_section_idea_paths(
    state: &State,
    project: &ProjectData,
    section: IdeaState,
    prefix: &[String],
    list_prefs: &crate::queue_list::ListConfig,
    out: &mut Vec<PathBuf>,
) {
    for idea in sorted_direct_ideas_at(state, project, section, prefix, list_prefs) {
        out.push(idea.abs_path.clone());
    }
    for child in child_tag_names_at(state, section, prefix) {
        let mut next_prefix = prefix.to_vec();
        next_prefix.push(child);
        if tag_node_expanded(state, section, &next_prefix) {
            collect_section_idea_paths(
                state,
                project,
                section,
                &next_prefix,
                list_prefs,
                out,
            );
        }
    }
}

fn idea_list_row<'a>(
    idea: &'a Idea,
    project: &'a ProjectData,
    depth: usize,
    selected: Option<&Path>,
    has_tag_siblings: bool,
    list_prefs: &crate::queue_list::ListConfig,
    phase_pill_list: bool,
    hovered: bool,
) -> ListRow<'a, Message> {
    let is_selected = selected == Some(idea.abs_path.as_path());
    let icon_bytes: &'static [u8] = match idea.frontmatter.archived {
        Some(ArchiveKind::Orphaned) => ICON_ORPHAN,
        Some(ArchiveKind::Manual) => ICON_ARCHIVE,
        _ => {
            if idea.frontmatter.change.is_some() {
                ICON_BRANCH
            } else if idea.frontmatter.exploration.is_some() {
                ICON_EXPLORE
            } else {
                ICON_IDEA
            }
        }
    };
    // Title stays plain; mark lives only on the interactive after_icon control.
    let title_base = idea.display_title();
    let phase = if phase_pill_list {
        None
    } else {
        idea.frontmatter
            .change
            .as_deref()
            .and_then(|n| super::change::change_scope_facts(n, project))
            .map(|f| f.phase)
    };
    let pillows =
        crate::queue_list::project_row_pillows(&idea.frontmatter.tags, phase, list_prefs);
    let mut row = ListRow::new(title_base.clone())
        .icon(icon_bytes)
        .indent(depth)
        .selected(is_selected)
        .on_press(Message::SelectIdea(idea.abs_path.clone()))
        .on_hover(
            Message::HoverIdea(idea.abs_path.clone()),
            Message::UnhoverIdea(idea.abs_path.clone()),
        );
    if let Some(g) = crate::queue_list::mark_glyph(idea.frontmatter.mark, hovered) {
        row = row.after_icon(
            button(text(g).size(theme::font_sm()))
                .on_press(Message::CycleIdeaMark(idea.abs_path.clone()))
                .padding(0)
                .style(theme::icon_button)
                .into(),
        );
    } else if let Some(change_name) = idea.frontmatter.change.as_deref()
        && change_resolves(project, change_name)
    {
        row = row.after_icon(change_link_button(change_name));
    }
    if let Some(p) = crate::queue_list::steady_row_pillows(
        &title_base,
        &pillows,
        crate::queue_list::DEFAULT_ROW_CHAR_BUDGET,
        hovered,
    ) {
        row = row.trailing(idea_pillow_trail(p));
    }
    if has_tag_siblings {
        let leading: Element<'a, Message> = row![Space::new().width(theme::font_sm())].into();
        row = row.leading(leading);
    }
    row
}

fn idea_pillow_trail<'a>(pillows: &crate::queue_list::RowPillows) -> Element<'a, Message> {
    let mut r = row![].spacing(theme::SPACING_XS).align_y(iced::Center);
    for tag in &pillows.type_tags {
        let label = crate::queue_list::truncate_tag_display(
            tag,
            crate::queue_list::TYPE_TAG_DISPLAY_MAX,
        );
        r = r.push(
            container(
                text(format!("#{label}"))
                    .size(theme::font_sm())
                    .color(theme::text_muted()),
            )
            .padding([1.0, theme::SPACING_XS])
            .style(|_t: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(theme::bg_elevated())),
                border: iced::Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        );
    }
    if let Some(phase) = &pillows.phase {
        let label = crate::queue_list::truncate_tag_display(phase, 20);
        r = r.push(
            container(
                text(label)
                    .size(theme::font_sm())
                    .color(theme::text_secondary()),
            )
            .padding([1.0, theme::SPACING_XS])
            .style(|_t: &iced::Theme| container::Style {
                background: Some(iced::Background::Color(theme::bg_elevated())),
                border: iced::Border {
                    radius: 8.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
        );
    }
    r.into()
}

fn change_resolves(project: &ProjectData, change_name: &str) -> bool {
    project.active_changes.iter().any(|c| c.name == change_name)
        || project.archived_changes.iter().any(|c| {
            c.name == change_name || crate::data::strip_archive_prefix(&c.name) == Some(change_name)
        })
}

fn change_link_button<'a>(change_name: &str) -> Element<'a, Message> {
    let icon = iced::widget::svg(iced::widget::svg::Handle::from_memory(ICON_CHANGE))
        .width(list_view::ICON_SIZE)
        .height(list_view::ICON_SIZE)
        .style(theme::svg_tint(theme::accent()));
    button(icon)
        .on_press(Message::OpenChange(change_name.to_string()))
        .padding(0.0)
        .style(theme::icon_button)
        .into()
}

// ── Pinned-tab toolbar (shown by main.rs above the tab content) ──────────────

/// Per-idea action toolbar shown below the tab bar when the pinned idea tab is
/// active. Returned for main.rs to render between the tab bar and tab content.
/// Replaces the generic file-path bar (suppressed in `tab_bar::view_content`)
/// because an idea's storage path isn't user-facing; tag chips and lifecycle
/// actions live here instead.
pub fn view_pinned_toolbar<'a>(
    state: &'a State,
    tabs: &tab_bar::TabState,
) -> Option<Element<'a, Message>> {
    if !matches!(tabs.active, tab_bar::ActiveTab::Preview) {
        return None;
    }
    let path = state.selected.as_deref()?;
    let idea = state.ideas.iter().find(|i| i.abs_path == path)?;

    // ── Tag chips + add-tag input ────────────────────────────────────────
    let mut tag_row = row![].spacing(theme::SPACING_XS).align_y(Center);
    let editing_idx = state.tag_input_editing;
    for (i, tag) in idea.frontmatter.tags.iter().enumerate() {
        if editing_idx == Some(i)
            && let Some(value) = state.tag_input.as_ref()
        {
            tag_row = tag_row.push(view_tag_input(value));
        } else {
            tag_row = tag_row.push(view_tag_chip(i, tag, i == 0));
        }
    }
    // Adding a new tag: input renders at the end. Editing replaces a chip
    // in place (handled above), so we only show the trailing input here.
    if state.tag_input.is_some() && editing_idx.is_none() {
        tag_row = tag_row.push(view_tag_input(
            state.tag_input.as_deref().unwrap_or_default(),
        ));
    } else if state.tag_input.is_none() {
        tag_row = tag_row.push(
            button(text("+ Tag").size(theme::font_sm()))
                .on_press(Message::OpenTagInput)
                .padding([2.0, theme::SPACING_SM])
                .style(theme::session_bar_button),
        );
    }

    // ── Lifecycle actions ────────────────────────────────────────────────
    let mut actions = row![].spacing(theme::SPACING_SM).align_y(Center);

    if let Some(change_name) = idea.frontmatter.change.as_deref() {
        actions = actions.push(
            button(text(format!("Change · {change_name}")).size(theme::font_sm()))
                .on_press(Message::OpenChange(change_name.to_string()))
                .padding([2.0, theme::SPACING_SM])
                .style(theme::session_bar_button),
        );
    } else if idea.frontmatter.exploration.is_none() && !matches!(idea.state, IdeaState::Archive) {
        actions = actions.push(
            button(text("Explore").size(theme::font_sm()))
                .on_press(Message::StartExploration(idea.abs_path.clone()))
                .padding([2.0, theme::SPACING_SM])
                .style(theme::session_bar_button),
        );
    }

    let archive_btn: Element<'a, Message> = if matches!(idea.state, IdeaState::Archive) {
        button(text("Unarchive").size(theme::font_sm()))
            .on_press(Message::UnarchiveIdea(idea.abs_path.clone()))
            .padding([2.0, theme::SPACING_SM])
            .style(theme::session_bar_button)
            .into()
    } else {
        button(text("Archive").size(theme::font_sm()))
            .on_press(Message::ArchiveIdea(idea.abs_path.clone()))
            .padding([2.0, theme::SPACING_SM])
            .style(theme::session_bar_button)
            .into()
    };
    actions = actions.push(archive_btn);
    actions = actions.push(
        button(text("Delete").size(theme::font_sm()))
            .on_press(Message::DeleteIdea(idea.abs_path.clone()))
            .padding([2.0, theme::SPACING_SM])
            .style(theme::session_bar_button_destructive),
    );

    let layout = row![tag_row, Space::new().width(Length::Fill), actions,]
        .spacing(theme::SPACING_MD)
        .align_y(Center);

    let bar = container(layout)
        .padding([theme::SPACING_XS, theme::SPACING_SM])
        .width(Length::Fill);
    let border = container(Space::new().width(Length::Fill).height(1.0)).style(theme::divider);
    Some(column![bar, border].into())
}

const TAG_INPUT_PLACEHOLDER: &str = "tag (or parent/child)";

/// `text_input` reserves this many pixels of headroom past the trailing
/// grapheme so the caret stays inside the content area. If the field's
/// content area is narrower than `text_width + CARET_HEADROOM`, the widget
/// scrolls left to keep the cursor visible — clipping the leading
/// character. See `measure_cursor_and_scroll_offset` in
/// `iced_widget/src/text_input.rs`.
const CARET_HEADROOM: f32 = 5.0;

fn view_tag_input<'a>(value: &str) -> Element<'a, Message> {
    // Grow/shrink with content so the field reads as a chip that's
    // currently being typed into, not a fixed pill. Width tracks the longer
    // of the value and the placeholder so an empty input still has room to
    // show its prompt. `text_input` shapes its content with
    // `Shaping::Advanced`, so the measurement must match — a `Basic`
    // measurement underestimates and clips the leading characters once the
    // widget's internal cursor scroll kicks in.
    let measured = if value.is_empty() {
        interaction::measure_text_advanced(TAG_INPUT_PLACEHOLDER, theme::font_md())
    } else {
        interaction::measure_text_advanced(value, theme::font_md())
    }
    .ceil();

    // The widget's text starts at `padding.left` and the trailing
    // `CARET_HEADROOM` pixels live inside the content area on the right.
    // Adding the same amount to the left padding keeps the visible
    // margin around the text symmetric.
    let visual_pad = theme::SPACING_SM;
    let padding = iced::Padding {
        top: 1.0,
        right: visual_pad,
        bottom: 1.0,
        left: visual_pad + CARET_HEADROOM,
    };
    let width = measured + visual_pad * 2.0 + CARET_HEADROOM * 2.0;

    text_input(TAG_INPUT_PLACEHOLDER, value)
        .id(TAG_INPUT_ID)
        .on_input(Message::TagInputChanged)
        .on_submit(Message::SubmitTagInput)
        .size(theme::font_md())
        .padding(padding)
        .width(width)
        .style(tag_input_style)
        .into()
}

fn tag_input_style(
    _theme: &iced::Theme,
    status: iced::widget::text_input::Status,
) -> iced::widget::text_input::Style {
    use iced::widget::text_input::Status;
    let border_color = match status {
        Status::Focused { .. } => theme::accent(),
        _ => theme::border_color(),
    };
    iced::widget::text_input::Style {
        background: theme::bg_section_header().into(),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: theme::BORDER_RADIUS.into(),
        },
        icon: theme::text_muted(),
        placeholder: theme::text_muted(),
        value: theme::text_primary(),
        selection: theme::bg_list_selected(),
    }
}

fn view_tag_chip<'a>(idx: usize, tag: &'a str, is_primary: bool) -> Element<'a, Message> {
    let label_style: fn(&iced::Theme, button::Status) -> button::Style = if is_primary {
        chip_label_primary_style
    } else {
        chip_label_style
    };
    let display_tag = tag.replace('/', " / ");
    let label_btn = button(text(format!("# {display_tag}")).size(theme::font_md()))
        .on_press(Message::ChipClick(idx))
        .padding([1.0, theme::SPACING_SM])
        .style(label_style);
    let remove_btn = button(text("×").size(theme::font_md()))
        .on_press(Message::RemoveTag(idx))
        .padding([1.0, theme::SPACING_XS])
        .style(chip_remove_style);
    container(row![label_btn, remove_btn].align_y(Center))
        .style(chip_style)
        .into()
}

fn chip_style(_theme: &iced::Theme) -> container::Style {
    container::Style {
        background: Some(theme::bg_section_header().into()),
        border: Border {
            color: theme::border_color(),
            width: 1.0,
            radius: theme::BORDER_RADIUS.into(),
        },
        ..Default::default()
    }
}

/// Chip-label button style — transparent background (the chip's `container`
/// owns the visual frame), no per-button hover bg so the chip reads as a
/// single pill rather than two segmented buttons. Hover lifts text to
/// `accent()` to telegraph the click-to-promote affordance.
fn chip_label_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    button::Style {
        background: None,
        text_color: match status {
            button::Status::Hovered => theme::accent(),
            _ => theme::text_primary(),
        },
        border: Border::default(),
        ..Default::default()
    }
}

/// Variant for the primary tag — already accent-colored at rest, so hover
/// just brightens slightly via `text_primary` swap.
fn chip_label_primary_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    button::Style {
        background: None,
        text_color: match status {
            button::Status::Hovered => theme::text_primary(),
            _ => theme::accent(),
        },
        border: Border::default(),
        ..Default::default()
    }
}

/// `×` remove-button style: muted at rest, error red on hover so the
/// destructive action is unambiguous. Same transparent bg as the label
/// half so the chip stays a single pill.
fn chip_remove_style(_theme: &iced::Theme, status: button::Status) -> button::Style {
    button::Style {
        background: None,
        text_color: match status {
            button::Status::Hovered => theme::error(),
            _ => theme::text_muted(),
        },
        border: Border::default(),
        ..Default::default()
    }
}

// ── Breadcrumbs ──────────────────────────────────────────────────────────────

pub fn breadcrumbs(state: &State) -> Vec<String> {
    let mut crumbs = vec!["Ideas".into()];
    if let Some(path) = state.selected.as_deref()
        && let Some(idea) = state.ideas.iter().find(|i| i.abs_path == path)
    {
        crumbs.push(idea.display_title());
    }
    crumbs
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @spec ideas/reconcile Selection and editor follow relocations: The selected idea stays selected after relocation
    #[test]
    fn selection_follows_relocation() {
        let old_path = PathBuf::from("/ideas/change/my-idea.md");
        let new_path = PathBuf::from("/ideas/archive/my-idea.md");

        let mut state = State {
            selected: Some(old_path.clone()),
            ..State::default()
        };
        let mut tabs = tab_bar::TabState::default();

        refresh_after_move(&mut state, &mut tabs, &old_path, &new_path, "My idea");

        assert_eq!(state.selected, Some(new_path));
    }

    /// @spec ideas/reconcile Selection and editor follow relocations: The open idea editor tracks the idea after relocation
    #[test]
    fn open_editor_tracks_relocation() {
        let old_path = PathBuf::from("/ideas/change/my-idea.md");
        let new_path = PathBuf::from("/ideas/archive/my-idea.md");

        let mut state = State {
            selected: Some(old_path.clone()),
            ..State::default()
        };
        let mut tabs = tab_bar::TabState::default();
        tabs.open_preview(
            pinned_tab_id(&old_path),
            "Old title".to_string(),
            String::new(),
            Some(old_path.clone()),
        );

        refresh_after_move(&mut state, &mut tabs, &old_path, &new_path, "New title");

        let preview = tabs.preview.expect("preview tab present");
        assert_eq!(preview.id, pinned_tab_id(&new_path));
        assert_eq!(preview.title, "New title");
    }

    /// @spec archive/browse Archived section visibility: Ideas Archive section starts collapsed
    #[test]
    fn ideas_archive_section_starts_collapsed() {
        let state = State::default();
        assert!(state.section_expanded.is_empty());
        assert!(!section_default_expanded(IdeaState::Archive));
        assert!(section_default_expanded(IdeaState::Inbox));
        assert!(section_default_expanded(IdeaState::Exploration));
        assert!(section_default_expanded(IdeaState::Change));
    }
}
