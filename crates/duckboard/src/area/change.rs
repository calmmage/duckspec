//! Change area — single change workspace with three-column layout.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use iced::Element;
use iced::Length;
use iced::widget::{Space, button, column, container, row, svg, text, text_input};

use crate::chat_store::Exploration;
use crate::data::{ChangeData, ProjectData, StepCompletion, TreeNode};
use crate::scope::{Scope, ScopeKind};
use crate::theme;
use crate::vcs::{ChangedFile, FileStatus};
use crate::widget::list_view::{self, ListRow};
use crate::widget::{collapsible, tab_bar, tree_view, vertical_scroll};

use super::interaction::{self, AgentSession, InteractionState};

const ICON_BRANCH: &[u8] = include_bytes!("../../assets/icon_branch.svg");
const ICON_FILE: &[u8] = include_bytes!("../../assets/icon_file.svg");
const ICON_SPEC: &[u8] = include_bytes!("../../assets/icon_spec.svg");
const ICON_DOC: &[u8] = include_bytes!("../../assets/icon_doc.svg");
const ICON_SPEC_DELTA: &[u8] = include_bytes!("../../assets/icon_spec_delta.svg");
const ICON_DOC_DELTA: &[u8] = include_bytes!("../../assets/icon_doc_delta.svg");
const ICON_STEP: &[u8] = include_bytes!("../../assets/icon_step.svg");
const ICON_STEP_DONE: &[u8] = include_bytes!("../../assets/icon_step_done.svg");
const ICON_STEP_PARTIAL: &[u8] = include_bytes!("../../assets/icon_step_partial.svg");
const ICON_EXPLORE: &[u8] = include_bytes!("../../assets/icon_explore.svg");
const ICON_IDEAS: &[u8] = include_bytes!("../../assets/icon_idea.svg");
const ICON_PENCIL: &[u8] = include_bytes!("../../assets/icon_pencil.svg");
const ICON_REFRESH: &[u8] = include_bytes!("../../assets/icon_refresh.svg");

/// Headroom past the trailing grapheme so the rename caret stays visible
/// (matches tag-input sizing in ideas).
const RENAME_CARET_HEADROOM: f32 = 5.0;
/// Minimum rename field width so short names stay comfortable to edit.
const RENAME_MIN_WIDTH: f32 = 96.0;

/// `text_input::Id` for the inline exploration rename field.
pub const RENAME_INPUT_ID: &str = "change-exploration-rename";

/// Section key for the Files explorer in `expanded_sections`. Absent by
/// default — the explorer starts collapsed with its header pinned to the
/// bottom of the list column.
pub const FILES_SECTION: &str = "files";

/// Container id wrapping the list column's scroll viewport. Measured by the
/// scroll-into-view operation in `main::reveal_active_file_in_explorer`.
pub const EXPLORER_VIEWPORT_ID: &str = "change-list-viewport";

/// Container id wrapping the Files explorer's row block (uniform-height
/// rows). Measured together with `EXPLORER_VIEWPORT_ID` to derive a row's
/// position inside the scroll content.
pub const EXPLORER_CONTENT_ID: &str = "files-explorer-content";

// ── State ────────────────────────────────────────────────────────────────────

pub struct State {
    pub selected_change: Option<String>,
    pub expanded_sections: HashSet<String>,
    pub expanded_nodes: HashSet<String>,
    /// Directory paths (repo-relative, as display strings) expanded in the
    /// changed-files tree.
    pub expanded_file_dirs: HashSet<String>,
    /// Directory paths previously surfaced by `set_changed_files`. Used to
    /// auto-expand only directories the user has never seen, so refreshes
    /// don't keep re-opening folders the user explicitly collapsed.
    known_file_dirs: HashSet<String>,
    pub changed_files: Vec<ChangedFile>,
    /// Precomputed flat rows for the Changed Files section. Rebuilt when
    /// `changed_files` or `expanded_file_dirs` change — not in `view()`.
    changed_file_rows: Vec<ChangedFileRow>,
    /// Full project file tree (gitignore-respecting, hidden files excluded)
    /// shown in the Files explorer section. Dir node ids are root-relative
    /// paths; file node ids are `file:<rel-path>` so they match file tab ids
    /// and row highlighting derives directly from the active tab.
    pub explorer_tree: Vec<TreeNode>,
    /// Directory node ids expanded in the Files explorer tree.
    pub expanded_explorer_dirs: HashSet<String>,
    /// Virtual exploration changes (not persisted to duckspec). Each carries
    /// a stable `id` used as the on-disk scope key plus a mutable
    /// `display_name` the UI shows.
    pub explorations: Vec<Exploration>,
    /// Counter for seeding default exploration display names.
    pub exploration_counter: usize,
    /// Id of the exploration row currently under the cursor, if any. When
    /// set, the exploration row's icon slot renders a close button instead.
    pub hovered_exploration: Option<String>,
    /// Id of the exploration whose close button has been clicked once and
    /// is now "armed" — the next click commits the destructive delete.
    /// Cleared on hover-leave, on a different selection, on `AddExploration`,
    /// and whenever the destructive delete actually fires. Skipped entirely
    /// for explorations whose `session_count` is zero (nothing to lose).
    pub armed_remove_exploration: Option<String>,
    /// Vertical scroll offset for the list column.
    pub list_scroll: f32,
    /// Folder-slug → originating exploration id, recorded when an exploration
    /// session's agent runs `ds create change`. Consumed by
    /// `reload_and_reconcile` to attribute the new folder to the session that
    /// created it. Not persisted; not cleaned up (changes are infrequent).
    pub pending_bindings: HashMap<String, String>,
    /// Exploration id currently being renamed inline, if any.
    pub renaming_exploration: Option<String>,
    /// Draft text for the active rename input.
    pub rename_draft: String,
    /// Sort menu open under the Change section header.
    pub sort_menu_open: bool,
    /// Hovered live queue row id (exploration id or change name) for mark/pillow chrome.
    pub hovered_queue_row: Option<String>,
}

impl State {
    pub fn new(project_root: Option<&Path>) -> Self {
        let mut sections = HashSet::new();
        sections.insert("picker".to_string());
        sections.insert("overview".to_string());
        sections.insert("capabilities".to_string());
        sections.insert("reviews".to_string());
        sections.insert("steps".to_string());
        sections.insert("changed_files".to_string());
        let (explorations, exploration_counter) =
            crate::chat_store::load_explorations(project_root);
        Self {
            selected_change: None,
            expanded_sections: sections,
            expanded_nodes: HashSet::new(),
            expanded_file_dirs: HashSet::new(),
            known_file_dirs: HashSet::new(),
            changed_files: vec![],
            changed_file_rows: vec![],
            explorer_tree: vec![],
            expanded_explorer_dirs: HashSet::new(),
            explorations,
            exploration_counter,
            hovered_exploration: None,
            armed_remove_exploration: None,
            list_scroll: 0.0,
            pending_bindings: HashMap::new(),
            renaming_exploration: None,
            rename_draft: String::new(),
            sort_menu_open: false,
            hovered_queue_row: None,
        }
    }

    /// Replace the changed-files list. Auto-expands only directories the
    /// user has never seen before, so a freshly-loaded changeset surfaces
    /// new files without re-opening folders the user explicitly collapsed
    /// during a previous refresh. Dirs that no longer appear are forgotten,
    /// so they auto-expand again if they ever come back.
    pub fn set_changed_files(&mut self, files: Vec<ChangedFile>) {
        let mut current_dirs: HashSet<String> = HashSet::new();
        for f in &files {
            let parts: Vec<&str> = f
                .path
                .components()
                .filter_map(|c| c.as_os_str().to_str())
                .collect();
            if parts.len() < 2 {
                continue;
            }
            let mut current = PathBuf::new();
            for part in &parts[..parts.len() - 1] {
                current.push(part);
                current_dirs.insert(current.display().to_string());
            }
        }

        for dir in &current_dirs {
            if !self.known_file_dirs.contains(dir) && !is_collapse_by_default(dir) {
                self.expanded_file_dirs.insert(dir.clone());
            }
        }
        self.expanded_file_dirs.retain(|d| current_dirs.contains(d));
        self.known_file_dirs = current_dirs;

        self.changed_files = files;
        self.refresh_changed_file_rows();
    }

    /// Rebuild [`Self::changed_file_rows`] from `changed_files` + expand set.
    fn refresh_changed_file_rows(&mut self) {
        self.changed_file_rows =
            rebuild_changed_file_rows(&self.changed_files, &self.expanded_file_dirs);
    }

    /// Replace the Files explorer contents from a fresh project walk.
    /// `files` are root-relative paths. Expanded state is pruned to
    /// directories that still exist; everything stays collapsed by default.
    pub fn set_project_files(&mut self, files: &[PathBuf]) {
        let mut dirs = HashSet::new();
        self.explorer_tree = build_explorer_tree(files, &mut dirs);
        self.expanded_explorer_dirs.retain(|d| dirs.contains(d));
    }

    /// Expand every ancestor directory of a root-relative file path in the
    /// Files explorer, so the file's row is present in the flattened tree.
    pub fn expand_explorer_ancestors(&mut self, rel: &str) {
        let Some((dirs, _file)) = rel.rsplit_once('/') else {
            return;
        };
        let mut prefix = String::new();
        for part in dirs.split('/') {
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(part);
            self.expanded_explorer_dirs.insert(prefix.clone());
        }
    }

    /// Index of `target_id` among the visible (flattened) explorer rows,
    /// plus the total visible row count. Mirrors `tree_view`'s flatten
    /// order: a node counts one row, children only when their parent is
    /// expanded.
    pub fn explorer_flat_position(&self, target_id: &str) -> Option<(usize, usize)> {
        fn walk(
            nodes: &[TreeNode],
            expanded: &HashSet<String>,
            target: &str,
            index: &mut usize,
            found: &mut Option<usize>,
        ) {
            for node in nodes {
                if node.id == target {
                    *found = Some(*index);
                }
                *index += 1;
                if expanded.contains(&node.id) {
                    walk(&node.children, expanded, target, index, found);
                }
            }
        }
        let mut index = 0;
        let mut found = None;
        walk(
            &self.explorer_tree,
            &self.expanded_explorer_dirs,
            target_id,
            &mut index,
            &mut found,
        );
        found.map(|f| (f, index))
    }

    /// Whether the currently selected change is an exploration (virtual).
    pub fn is_exploration_selected(&self) -> bool {
        self.selected_change
            .as_deref()
            .is_some_and(|id| self.explorations.iter().any(|e| e.id == id))
    }

    /// Classify a scope key: is it an exploration or a real change?
    pub fn scope_kind_for(&self, scope: &str) -> ScopeKind {
        if self.explorations.iter().any(|e| e.id == scope) {
            ScopeKind::Exploration
        } else {
            ScopeKind::Change
        }
    }

    /// Build a `Scope` from a raw scope key, classifying via `explorations`.
    pub fn scope_for(&self, scope: &str) -> Scope {
        if self.explorations.iter().any(|e| e.id == scope) {
            Scope::Exploration(scope.to_string())
        } else {
            Scope::Change(scope.to_string())
        }
    }

    /// Human-readable label for a scope: exploration display_name if the
    /// scope is an exploration id, else the scope key itself.
    pub fn scope_display_label(&self, scope: &str) -> String {
        self.explorations
            .iter()
            .find(|e| e.id == scope)
            .map(|e| e.display_name.clone())
            .unwrap_or_else(|| scope.to_string())
    }
}

/// Build the Files explorer tree from root-relative paths. Directories
/// come first (sorted), then files (sorted), matching the changed-files
/// tree. Every directory id encountered is also collected into `dirs_out`
/// so the caller can prune stale expanded state.
fn build_explorer_tree(files: &[PathBuf], dirs_out: &mut HashSet<String>) -> Vec<TreeNode> {
    #[derive(Default)]
    struct Dir {
        dirs: BTreeMap<String, Dir>,
        files: Vec<String>,
    }

    let mut root = Dir::default();
    for path in files {
        let parts: Vec<&str> = path
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect();
        let Some((file_name, dir_parts)) = parts.split_last() else {
            continue;
        };
        let mut node = &mut root;
        for part in dir_parts {
            node = node.dirs.entry((*part).to_string()).or_default();
        }
        node.files.push((*file_name).to_string());
    }

    fn convert(dir: Dir, prefix: &str, dirs_out: &mut HashSet<String>) -> Vec<TreeNode> {
        let join = |name: &str| {
            if prefix.is_empty() {
                name.to_string()
            } else {
                format!("{prefix}/{name}")
            }
        };
        let mut nodes = Vec::new();
        for (name, sub) in dir.dirs {
            let path = join(&name);
            dirs_out.insert(path.clone());
            let children = convert(sub, &path, dirs_out);
            nodes.push(TreeNode {
                id: path,
                label: name,
                children,
            });
        }
        let mut files = dir.files;
        files.sort_unstable();
        for name in files {
            nodes.push(TreeNode {
                id: format!("file:{}", join(&name)),
                label: name,
                children: vec![],
            });
        }
        nodes
    }

    convert(root, "", dirs_out)
}

/// Directories the changed-files tree should leave collapsed even on first
/// appearance. The duckspec root is usually noise — the user is typically
/// looking at the project's own changes, not their tracked spec edits — but
/// can still be expanded by hand when wanted.
fn is_collapse_by_default(dir: &str) -> bool {
    dir == "duckspec"
}

/// Promote an exploration to a real change: remove from explorations list,
/// migrate interaction state and chat sessions from the exploration's id
/// scope to the new change name.
pub fn promote_exploration(
    state: &mut State,
    interactions: &mut HashMap<Scope, InteractionState>,
    exploration_id: &str,
    real_name: &str,
    project_root: Option<&Path>,
) {
    // Flush-before-mutate: persist every session this exploration holds before
    // its in-memory state is migrated, so an in-flight turn can't be lost by
    // the promotion.
    if let Some(ix) = interactions.get(&Scope::Exploration(exploration_id.to_string())) {
        interaction::flush_sessions(ix, project_root);
    }
    state.explorations.retain(|e| e.id != exploration_id);
    if let Some(mut ix) = interactions.remove(&Scope::Exploration(exploration_id.to_string())) {
        for ax in ix.sessions.iter_mut() {
            ax.session.scope = real_name.to_string();
            ax.scope_kind = ScopeKind::Change;
        }
        let target = Scope::Change(real_name.to_string());
        if let Some(existing) = interactions.get_mut(&target) {
            // Target scope is already live — fold the exploration's sessions in
            // rather than overwrite, preserving the target's subscriptions.
            interaction::merge_sessions(existing, ix.sessions, real_name);
        } else {
            interaction::reconcile_display_names(&mut ix.sessions, real_name);
            interactions.insert(target, ix);
        }
    }
    if state.selected_change.as_deref() == Some(exploration_id) {
        state.selected_change = Some(real_name.to_string());
    }
    crate::chat_store::merge_scope(exploration_id, real_name, project_root);
    crate::chat_store::save_explorations(
        &state.explorations,
        state.exploration_counter,
        project_root,
    );
}

/// Migrate interaction state and chat sessions from a change that was just
/// archived externally (via CLI, agent tool, etc.) to its new archived name.
pub fn archive_change(
    state: &mut State,
    interactions: &mut HashMap<Scope, InteractionState>,
    tabs: &mut tab_bar::TabState,
    old_name: &str,
    archived_name: &str,
    project_root: Option<&Path>,
) {
    if let Some(mut ix) = interactions.remove(&Scope::Change(old_name.to_string())) {
        for ax in ix.sessions.iter_mut() {
            ax.session.scope = archived_name.to_string();
        }
        interaction::reconcile_display_names(&mut ix.sessions, archived_name);
        interactions.insert(Scope::Change(archived_name.to_string()), ix);
    }
    if state.selected_change.as_deref() == Some(old_name) {
        state.selected_change = Some(archived_name.to_string());
    }
    rewrite_tab_ids_for_archive(tabs, old_name, archived_name);
    crate::chat_store::rename_scope(old_name, archived_name, project_root);
}

fn row_icon_button<'a>(bytes: &'static [u8], on_press: Message) -> Element<'a, Message> {
    let icon = svg(svg::Handle::from_memory(bytes))
        .width(list_view::ICON_SIZE)
        .height(list_view::ICON_SIZE)
        .style(theme::svg_tint(theme::text_muted()));
    button(icon)
        .on_press(on_press)
        .padding(0.0)
        .style(theme::icon_button)
        .into()
}

fn rename_input<'a>(draft: &'a str) -> Element<'a, Message> {
    let measured = if draft.is_empty() {
        interaction::measure_text_advanced("Exploration name", theme::font_md())
    } else {
        interaction::measure_text_advanced(draft, theme::font_md())
    }
    .ceil();
    let visual_pad = theme::SPACING_SM;
    let width = (measured + visual_pad * 2.0 + RENAME_CARET_HEADROOM * 2.0).max(RENAME_MIN_WIDTH);
    text_input("Exploration name", draft)
        .id(RENAME_INPUT_ID)
        .on_input(Message::RenameDraft)
        .on_submit(Message::CommitRenameExploration)
        .padding(iced::Padding {
            top: theme::SPACING_XS,
            right: visual_pad,
            bottom: theme::SPACING_XS,
            left: visual_pad + RENAME_CARET_HEADROOM,
        })
        .size(theme::font_md())
        .width(Length::Fixed(width))
        .into()
}

fn begin_rename(state: &mut State, id: String) {
    let draft = state
        .explorations
        .iter()
        .find(|e| e.id == id)
        .map(|e| e.display_name.clone())
        .unwrap_or_default();
    state.renaming_exploration = Some(id);
    state.rename_draft = draft;
}

/// Rewrite tab IDs that reference a change being archived so breadcrumbs, the
/// path header below the tab bar, and content lookups point to the new archive
/// location. Handles artifact tabs (`changes/<old>/…`) and VCS diff tabs
/// (`vcs:…/changes/<old>/…`). Tab titles are unchanged (they're filenames).
fn rewrite_tab_ids_for_archive(tabs: &mut tab_bar::TabState, old_name: &str, archived_name: &str) {
    let artifact_old = format!("changes/{old_name}/");
    let artifact_new = format!("archive/{archived_name}/");
    let vcs_old = format!("/changes/{old_name}/");
    let vcs_new = format!("/archive/{archived_name}/");

    let rewrite = |id: &str| -> Option<String> {
        if let Some(rest) = id.strip_prefix(&artifact_old) {
            return Some(format!("{artifact_new}{rest}"));
        }
        if let Some(rest) = id.strip_prefix("vcs:")
            && let Some(idx) = rest.find(&vcs_old)
        {
            let (lead, tail) = rest.split_at(idx);
            let tail = &tail[vcs_old.len()..];
            return Some(format!("vcs:{lead}{vcs_new}{tail}"));
        }
        None
    };

    if let Some(tab) = tabs.preview.as_mut()
        && let Some(new_id) = rewrite(&tab.id)
    {
        tab.id = new_id;
    }
    for tab in tabs.file_tabs.iter_mut() {
        if let Some(new_id) = rewrite(&tab.id) {
            tab.id = new_id;
        }
    }
}

// ── Messages ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Message {
    SelectChange(String),
    ToggleSection(String),
    ToggleNode(String),
    SelectItem(String),
    Interaction(interaction::Msg),
    SelectChangedFile(PathBuf),
    ToggleFileDir(String),
    /// Toggle a directory node in the Files explorer tree.
    ToggleExplorerDir(String),
    /// A file row in the Files explorer was clicked. Payload is the row's
    /// node id (`file:<rel-path>`). Intercepted by `main::update`, which
    /// opens the file as a regular file tab.
    SelectExplorerFile(String),
    AddExploration,
    /// Soft-archive a live exploration (stamp `archived_at`, keep chats).
    ArchiveExploration(String),
    /// First click on the close button of an exploration that has chat
    /// sessions. Sets `armed_remove_exploration` so the next
    /// `RemoveExploration` for the same id commits.
    ArmRemoveExploration(String),
    RemoveExploration(String),
    HoverExploration(String),
    /// Payload is the exploration name the row thinks it's clearing. Only
    /// clear the hover state if it still matches — otherwise a stale exit
    /// from row N can wipe a fresh enter from row N+1 when both fire in
    /// the same event dispatch.
    UnhoverExploration(String),
    /// Navigate to a change and open one of its artifacts.
    OpenArtifact {
        change: String,
        artifact_id: String,
    },
    /// Navigate to the idea linked to a given change. Handled by the main
    /// loop (switches `active_area` to Ideas and selects the idea); a no-op
    /// here so the message body can be a plain String.
    OpenIdeaForChange(String),
    /// `+` on the Changed Files header → open the new-file modal seeded with
    /// the project root. Intercepted by `main::update`.
    AddFile,
    ScrollList(f32),
    /// Draft text for the active exploration rename input.
    RenameDraft(String),
    /// Explicit rename affordance (pencil) — opens the inline rename field.
    StartRenameExploration(String),
    /// Commit the open exploration rename (Enter in the rename field).
    CommitRenameExploration,
    /// Abandon the open exploration rename without writing.
    CancelRenameExploration,
    /// Re-run the title summarizer for the exploration's active session.
    /// Intercepted by `main::update` (needs AgentHandle + oneshot Task).
    RefreshExplorationTitle(String),
    /// Cycle exclusive mark on the idea linked to this queue key (main saves).
    CycleQueueMark(crate::idea_store::QueueLinkKey),
    ToggleSortMenu,
    SetListSortKey(crate::queue_list::SortKey),
    ToggleListTypePillows,
    ToggleListPhasePillows,
    HoverQueueRow(String),
    UnhoverQueueRow(String),
    /// Phase-pill activation from the change list. Main selects the target
    /// scope then submits `text` into that chat session.
    PhasePillSend {
        target: String,
        text: String,
    },
    /// Cycle worktree placement (Main → Worktree → Auto) for a scope key.
    /// Intercepted by `main::update` (owns `worktree_bindings`).
    CyclePlacement(String),
    /// Cycle stack base among other live scopes (or clear). Main owns store.
    CycleStackBase(String),
    /// Toggle require-base-merged for a scope. Main owns store.
    ToggleRequireBaseMerged(String),
    /// Explicit merge of this scope’s sidecar into Main. Main owns integrate.
    MergeToMain(String),
}

// ── Update ───────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)] // area update: state + tabs + interaction + project + flags
pub fn update(
    state: &mut State,
    tabs: &mut tab_bar::TabState,
    interactions: &mut HashMap<Scope, InteractionState>,
    message: Message,
    project: &ProjectData,
    highlighter: &crate::highlight::SyntaxHighlighter,
    agent_input_hints: bool,
    window_w: f32,
    vcs_workflow: crate::config::VcsWorkflow,
    viewer_style: crate::config::ViewerStyle,
) {
    match message {
        Message::SelectChange(name) => {
            // Second click on an already-selected exploration opens rename.
            if state.selected_change.as_deref() == Some(name.as_str())
                && state.explorations.iter().any(|e| e.id == name)
            {
                begin_rename(state, name);
                return;
            }

            state.selected_change = Some(name.clone());
            state.expanded_nodes.clear();
            state.armed_remove_exploration = None;
            state.renaming_exploration = None;
            state.rename_draft.clear();

            let is_exploration = state.explorations.iter().any(|e| e.id == name);
            if !is_exploration
                && let Some(change) = project
                    .active_changes
                    .iter()
                    .chain(project.archived_changes.iter())
                    .find(|c| c.name == name)
            {
                crate::data::TreeNode::collect_parent_ids(
                    &change.cap_tree,
                    &mut state.expanded_nodes,
                );
            }

            // Reveal the row in the list when navigation arrives from
            // another area (the toolbar Change button on an idea, etc.) —
            // the archived section is collapsed by default, so a fresh
            // selection there would otherwise be invisible. Pending archives
            // live on the Change list (`picker`), not under Archived.
            let dirty = state.changed_files.as_slice();
            let in_finished_archived = is_list_finished_archive(&name, project, dirty)
                || state
                    .explorations
                    .iter()
                    .any(|e| e.id == name && e.is_archived());
            let section = if in_finished_archived {
                "archived"
            } else {
                "picker"
            };
            state.expanded_sections.insert(section.to_string());

            let kind = state.scope_kind_for(&name);
            let label = state.scope_display_label(&name);
            let scope = state.scope_for(&name);
            let ix = interactions
                .entry(scope)
                .or_insert_with(|| InteractionState::for_window(window_w));
            interaction::ensure_sessions_with_label(
                ix,
                &name,
                &label,
                kind,
                project.project_root.as_deref(),
                highlighter,
                viewer_style,
            );
            if !ix.visible {
                interaction::show_panel(ix, window_w);
            }
        }
        Message::ToggleSection(id) => {
            if !state.expanded_sections.remove(&id) {
                state.expanded_sections.insert(id);
            }
        }
        Message::ToggleNode(id) => {
            if !state.expanded_nodes.remove(&id) {
                state.expanded_nodes.insert(id);
            }
        }
        Message::ToggleFileDir(id) => {
            if !state.expanded_file_dirs.remove(&id) {
                state.expanded_file_dirs.insert(id);
            }
            state.refresh_changed_file_rows();
        }
        Message::ToggleExplorerDir(id) => {
            if !state.expanded_explorer_dirs.remove(&id) {
                state.expanded_explorer_dirs.insert(id);
            }
        }
        Message::SelectExplorerFile(_) => {
            // Intercepted by `main::update` (needs `open_path_in_tab`).
        }
        Message::SelectItem(id) => {
            open_artifact(tabs, &id, project, highlighter);
        }
        Message::Interaction(msg) => {
            let scope_key = match state.selected_change.clone() {
                Some(n) => n,
                None => return,
            };
            let kind = state.scope_kind_for(&scope_key);
            let label = state.scope_display_label(&scope_key);
            let scope = state.scope_for(&scope_key);
            match msg {
                interaction::Msg::NewSession => {
                    let ix = interactions
                        .entry(scope.clone())
                        .or_insert_with(|| InteractionState::for_window(window_w));
                    interaction::ensure_sessions_with_label(
                        ix,
                        &scope_key,
                        &label,
                        kind,
                        project.project_root.as_deref(),
                        highlighter,
                        viewer_style,
                    );
                    // Donor is still active — inherit next actions before insert.
                    let new_session = interaction::new_session_with_inherited_next_actions(
                        ix,
                        scope_key.clone(),
                        kind,
                    );
                    let _ = new_session.persist(project.project_root.as_deref());
                    ix.sessions.insert(0, new_session);
                    ix.active_session = 0;
                    interaction::reconcile_display_names(&mut ix.sessions, &label);
                }
                interaction::Msg::SelectSession(id) => {
                    let Some(ix) = interactions.get_mut(&scope) else {
                        return;
                    };
                    if let Some(idx) = ix.find_session_index(&id) {
                        ix.active_session = idx;
                    }
                }
                interaction::Msg::ClearSession => {
                    let Some(ix) = interactions.get_mut(&scope) else {
                        return;
                    };
                    clear_active_session(
                        ix,
                        &scope_key,
                        &label,
                        kind,
                        project.project_root.as_deref(),
                    );
                }
                other => {
                    let Some(ix) = interactions.get_mut(&scope) else {
                        return;
                    };
                    interaction::update_with_side_effects(
                        ix,
                        other,
                        &scope_key,
                        &label,
                        kind,
                        project.project_root.as_deref(),
                        highlighter,
                        agent_input_hints,
                        window_w,
                        vcs_workflow,
                        viewer_style,
                    );
                }
            }
        }
        Message::SelectChangedFile(_) => {
            // Intercepted by `main::update` so the async diff-highlight
            // `Task` can be propagated to the runtime.
        }
        Message::RenameDraft(text) => {
            state.rename_draft = text;
        }
        Message::StartRenameExploration(id) => {
            if !state.explorations.iter().any(|e| e.id == id) {
                return;
            }
            // Pencil can open rename without requiring a prior select click.
            if state.selected_change.as_deref() != Some(id.as_str()) {
                state.selected_change = Some(id.clone());
                state.expanded_nodes.clear();
                state.armed_remove_exploration = None;
                let kind = state.scope_kind_for(&id);
                let label = state.scope_display_label(&id);
                let scope = state.scope_for(&id);
                let ix = interactions.entry(scope).or_default();
                interaction::ensure_sessions_with_label(
                    ix,
                    &id,
                    &label,
                    kind,
                    project.project_root.as_deref(),
                    highlighter,
                    viewer_style,
                );
                if !ix.visible {
                    ix.visible = true;
                }
            }
            begin_rename(state, id);
        }
        Message::CommitRenameExploration => {
            let Some(id) = state.renaming_exploration.take() else {
                return;
            };
            let draft = std::mem::take(&mut state.rename_draft);
            let root = project.project_root.as_deref();
            let changed = if let Some(exp) = state.explorations.iter_mut().find(|e| e.id == id) {
                crate::chat_store::rename_exploration(exp, &draft)
            } else {
                false
            };
            if changed {
                crate::chat_store::save_explorations(
                    &state.explorations,
                    state.exploration_counter,
                    root,
                );
                let label = state
                    .explorations
                    .iter()
                    .find(|e| e.id == id)
                    .map(|e| e.display_name.clone())
                    .unwrap_or(draft);
                if let Some(ix) = interactions.get_mut(&Scope::Exploration(id)) {
                    interaction::reconcile_display_names(&mut ix.sessions, &label);
                }
            }
        }
        Message::CancelRenameExploration => {
            state.renaming_exploration = None;
            state.rename_draft.clear();
        }
        Message::RefreshExplorationTitle(_) => {
            // Intercepted by `main::update` (oneshot Task).
        }
        Message::AddExploration => {
            state.exploration_counter += 1;
            let exp = Exploration::new(state.exploration_counter);
            let id = exp.id.clone();
            let display_name = exp.display_name.clone();
            state.explorations.push(exp);
            state.selected_change = Some(id.clone());
            state.armed_remove_exploration = None;
            state.renaming_exploration = None;
            state.rename_draft.clear();
            crate::chat_store::save_explorations(
                &state.explorations,
                state.exploration_counter,
                project.project_root.as_deref(),
            );
            let ix = interactions
                .entry(Scope::Exploration(id.clone()))
                .or_insert_with(|| InteractionState::for_window(window_w));
            interaction::ensure_sessions_with_label(
                ix,
                &id,
                &display_name,
                ScopeKind::Exploration,
                project.project_root.as_deref(),
                highlighter,
                viewer_style,
            );
            interaction::show_panel(ix, window_w);
            crate::chat_store::recount_explorations(
                &mut state.explorations,
                project.project_root.as_deref(),
            );
        }
        Message::ArchiveExploration(id) => {
            if let Some(exp) = state.explorations.iter_mut().find(|e| e.id == id) {
                exp.mark_archived();
            }
            if state.armed_remove_exploration.as_deref() == Some(id.as_str()) {
                state.armed_remove_exploration = None;
            }
            crate::chat_store::save_explorations(
                &state.explorations,
                state.exploration_counter,
                project.project_root.as_deref(),
            );
        }
        Message::ArmRemoveExploration(id) => {
            state.armed_remove_exploration = Some(id);
        }
        Message::RemoveExploration(id) => {
            state.explorations.retain(|e| e.id != id);
            interactions.remove(&Scope::Exploration(id.clone()));
            if state.selected_change.as_deref() == Some(&id) {
                state.selected_change = None;
            }
            if state.hovered_exploration.as_deref() == Some(&id) {
                state.hovered_exploration = None;
            }
            if state.armed_remove_exploration.as_deref() == Some(&id) {
                state.armed_remove_exploration = None;
            }
            crate::chat_store::delete_scope(&id, project.project_root.as_deref());
            crate::chat_store::save_explorations(
                &state.explorations,
                state.exploration_counter,
                project.project_root.as_deref(),
            );
        }
        Message::HoverExploration(id) => {
            state.hovered_exploration = Some(id);
        }
        Message::UnhoverExploration(id) => {
            if state.hovered_exploration.as_deref() == Some(id.as_str()) {
                state.hovered_exploration = None;
            }
            // Moving the cursor off an armed row disarms it — matches the
            // visual disappearance of the red icon.
            if state.armed_remove_exploration.as_deref() == Some(id.as_str()) {
                state.armed_remove_exploration = None;
            }
        }
        Message::OpenArtifact {
            change,
            artifact_id,
        } => {
            state.selected_change = Some(change.clone());
            state.expanded_nodes.clear();
            if let Some(ch) = project
                .active_changes
                .iter()
                .chain(project.archived_changes.iter())
                .find(|c| c.name == change)
            {
                crate::data::TreeNode::collect_parent_ids(&ch.cap_tree, &mut state.expanded_nodes);
            }
            open_artifact(tabs, &artifact_id, project, highlighter);
        }
        Message::OpenIdeaForChange(_) => {
            // Handled in main.rs — crosses area boundaries.
        }
        Message::AddFile => {
            // Handled in main.rs — opens the global new-file modal.
        }
        Message::ScrollList(offset) => {
            state.list_scroll = offset;
        }
        Message::CyclePlacement(_)
        | Message::CycleStackBase(_)
        | Message::ToggleRequireBaseMerged(_)
        | Message::MergeToMain(_) => {
            // Handled in `main::update` (needs worktree binding store).
        }
        Message::CycleQueueMark(_)
        | Message::SetListSortKey(_)
        | Message::ToggleListTypePillows
        | Message::ToggleListPhasePillows
        | Message::PhasePillSend { .. } => {
            // Intercepted by main.rs (needs ideas + config / send path).
        }
        Message::ToggleSortMenu => {
            state.sort_menu_open = !state.sort_menu_open;
        }
        Message::HoverQueueRow(id) => {
            state.hovered_queue_row = Some(id);
        }
        Message::UnhoverQueueRow(id) => {
            if state.hovered_queue_row.as_deref() == Some(id.as_str()) {
                state.hovered_queue_row = None;
            }
        }
    }

    let vcs_dirty = !state.changed_files.is_empty();
    refresh_fast_response(interactions, project, agent_input_hints, vcs_dirty);
    // Cheap: one `read_dir` per exploration. Keeps `Exploration.session_count`
    // in sync so the close-button arming logic doesn't `read_dir` per frame.
    crate::chat_store::recount_explorations(
        &mut state.explorations,
        project.project_root.as_deref(),
    );
}

/// Compute the suggested next /ds-* command (without the leading slash) given
/// the selected change's artifact state. Returns `None` for archived changes
/// or when nothing is selected. Test-only — production paths refresh
/// `scope_facts` / next-action bootstrap via `refresh_fast_response`.
#[cfg(test)]
fn compute_lifecycle_command(state: &State, project: &ProjectData) -> Option<String> {
    let selected = state.selected_change.as_deref()?;

    // Exploration (virtual) — always orient first.
    if state.explorations.iter().any(|e| e.id == selected) {
        return Some("ds-explore".into());
    }

    lifecycle_command_from_artifacts(selected, project)
}

/// Re-export shared lifecycle facts (defined in duckcore).
pub use crate::scope::ChangeScopeFacts;

fn scope_facts(
    phase: &'static str,
    steps_done: usize,
    step_count: usize,
    active_step_tasks: Option<(usize, usize)>,
    lifecycle: &[&str],
    current_review: Option<String>,
) -> ChangeScopeFacts {
    let next_command = lifecycle.first().map(|s| (*s).to_string());
    ChangeScopeFacts {
        phase,
        steps_done,
        step_count,
        active_step_tasks,
        next_command,
        current_review,
    }
}

/// Inspect a change directory's artifact and step state and return its
/// lifecycle facts. Pure function over `project` — independent of
/// `state.selected_change`, so it can describe any change session (e.g.
/// freshly-promoted idea sessions where the user is still in the Ideas
/// area and the Changes area's selection hasn't moved). Returns `None` for
/// archived or unknown changes, which have no next stage.
pub fn change_scope_facts(name: &str, project: &ProjectData) -> Option<ChangeScopeFacts> {
    if project.archived_changes.iter().any(|c| c.name == name) {
        return None;
    }

    let change = project.active_changes.iter().find(|c| c.name == name)?;

    // The current review is the highest-numbered review (reviews are sorted
    // ascending). Computed before the phase branches and set in every arm so
    // it surfaces at any lifecycle stage — including a pre-implementation
    // review under a proposal-only change. When present, review-aware arms
    // below take priority over the plain ladder.
    let current_review = change.reviews.last().cloned();
    let has_review = current_review.is_some();

    let steps_done = change
        .steps
        .iter()
        .filter(|s| matches!(s.completion, StepCompletion::Done))
        .count();
    let step_count = change.steps.len();
    let has_steps = step_count > 0;
    let all_done = has_steps && steps_done == step_count;
    let open = has_steps && !all_done;
    let active_step_tasks = change.steps.iter().find_map(|s| match s.completion {
        StepCompletion::Partial(done, total) => Some((done, total)),
        _ => None,
    });

    // Open steps: apply first; review and followup stay available (including
    // when a critique file already exists — re-critique mid-impl).
    if open {
        return Some(scope_facts(
            "implementing steps",
            steps_done,
            step_count,
            active_step_tasks,
            &["ds-apply", "ds-review", "ds-followup"],
            current_review,
        ));
    }

    // No open steps + review: rework + re-critique + archive.
    if has_review {
        return Some(scope_facts(
            if all_done {
                "all steps complete, review on file"
            } else {
                "review on file, no open steps"
            },
            steps_done,
            step_count,
            active_step_tasks,
            &[
                "ds-step",
                "ds-spec",
                "ds-review",
                "ds-followup",
                "ds-archive",
            ],
            current_review,
        ));
    }

    // All steps complete, no review → archive + both critique modes.
    if all_done {
        return Some(scope_facts(
            "all steps complete",
            steps_done,
            step_count,
            active_step_tasks,
            &["ds-archive", "ds-review", "ds-followup"],
            current_review,
        ));
    }

    // Caps on disk, no steps — step or no-code archive (no re-entry to ds-spec).
    if !change.cap_tree.is_empty() {
        return Some(scope_facts(
            "specs drafted, steps not yet written",
            0,
            0,
            None,
            &["ds-step", "ds-archive"],
            current_review,
        ));
    }

    // No caps yet — feature-flow ladder (design optional).
    if change.has_design {
        return Some(scope_facts(
            "design drafted, specs not yet written",
            0,
            0,
            None,
            &["ds-spec", "ds-step"],
            current_review,
        ));
    }
    if change.has_proposal {
        return Some(scope_facts(
            "proposal drafted, design not yet written",
            0,
            0,
            None,
            &["ds-design", "ds-spec"],
            current_review,
        ));
    }
    Some(scope_facts(
        "newly created, no artifacts yet",
        0,
        0,
        None,
        &["ds-propose"],
        current_review,
    ))
}

/// Compact stage for phase-pill face (UI only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseShort {
    Explore,
    Empty,
    Proposal,
    Design,
    Specs,
    Steps,
    Review,
    Ready,
    Archived,
}

impl PhaseShort {
    pub fn label(self) -> &'static str {
        match self {
            Self::Explore => "explore",
            Self::Empty => "empty",
            Self::Proposal => "proposal",
            Self::Design => "design",
            Self::Specs => "specs",
            Self::Steps => "steps",
            Self::Review => "review",
            Self::Ready => "ready",
            Self::Archived => "archived",
        }
    }

    fn is_late(self) -> bool {
        matches!(self, Self::Ready | Self::Archived)
    }
}

/// Working-tree pill on late stages only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VcsPill {
    /// Working tree clean vs HEAD.
    Committed,
    /// Any repo-wide dirty path.
    Uncommitted,
}

impl VcsPill {
    pub fn label(self) -> &'static str {
        match self {
            Self::Committed => "committed",
            Self::Uncommitted => "uncommitted",
        }
    }
}

/// Which pill was activated (lifecycle stage vs VCS).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhasePillKind {
    Lifecycle,
    Vcs,
}

/// Projected phase chrome for list/composer pills — not a stored status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseDisplay {
    pub short: PhaseShort,
    /// Hover body for the lifecycle pill.
    pub lifecycle_hover: String,
    /// Empty-send text for lifecycle click (`/ds-…`), if clickable.
    pub lifecycle_send: Option<String>,
    pub vcs: Option<VcsPill>,
    pub vcs_hover: Option<&'static str>,
    /// `Some("Commit")` only when `vcs == Uncommitted`.
    pub vcs_send: Option<&'static str>,
}

const VCS_HOVER_DIRTY: &str = "working tree has uncommitted changes (repo-wide)";
const VCS_HOVER_CLEAN: &str = "working tree clean vs HEAD";
const VCS_HOVER_ARCHIVE_PACKAGE: &str = "this archive package has uncommitted paths";
const EXPLORE_HOVER: &str = "exploration — early brainstorming, no formal change artifacts";
const ARCHIVED_HOVER: &str = "archived";
/// Always-on trailing plaque on pending-commit Change rows.
pub const PENDING_UNCOMMITTED_LABEL: &str = "uncommitted";

fn short_from_facts(facts: &ChangeScopeFacts) -> PhaseShort {
    match facts.phase {
        "newly created, no artifacts yet" => PhaseShort::Empty,
        "proposal drafted, design not yet written" => PhaseShort::Proposal,
        "design drafted, specs not yet written" => PhaseShort::Design,
        "specs drafted, steps not yet written" => PhaseShort::Specs,
        "implementing steps" => PhaseShort::Steps,
        "all steps complete" => PhaseShort::Ready,
        "all steps complete, review on file" | "review on file, no open steps" => PhaseShort::Review,
        // Unknown phase strings still need a face; treat as empty rather than panic.
        _ => PhaseShort::Empty,
    }
}

fn vcs_fields(short: PhaseShort, vcs_dirty: bool) -> (Option<VcsPill>, Option<&'static str>, Option<&'static str>) {
    if !short.is_late() {
        return (None, None, None);
    }
    if vcs_dirty {
        (
            Some(VcsPill::Uncommitted),
            Some(VCS_HOVER_DIRTY),
            Some("Commit"),
        )
    } else {
        (
            Some(VcsPill::Committed),
            Some(VCS_HOVER_CLEAN),
            None,
        )
    }
}

fn phase_display_from_facts(facts: &ChangeScopeFacts, vcs_dirty: bool) -> PhaseDisplay {
    let short = short_from_facts(facts);
    let lifecycle_send = facts
        .next_command
        .as_deref()
        .and_then(crate::fast_response::format_lifecycle_command);
    let (vcs, vcs_hover, vcs_send) = vcs_fields(short, vcs_dirty);
    PhaseDisplay {
        short,
        lifecycle_hover: facts.phase.to_string(),
        lifecycle_send,
        vcs,
        vcs_hover,
        vcs_send,
    }
}

/// Build display for an active change name. `None` if unknown / not active.
pub fn phase_display_for_change(
    name: &str,
    project: &ProjectData,
    vcs_dirty: bool,
) -> Option<PhaseDisplay> {
    let facts = change_scope_facts(name, project)?;
    Some(phase_display_from_facts(&facts, vcs_dirty))
}

/// Archived row: short `archived`, no lifecycle send; VCS pill always present.
pub fn phase_display_for_archived(vcs_dirty: bool) -> PhaseDisplay {
    let short = PhaseShort::Archived;
    let (vcs, vcs_hover, vcs_send) = vcs_fields(short, vcs_dirty);
    PhaseDisplay {
        short,
        lifecycle_hover: ARCHIVED_HOVER.into(),
        lifecycle_send: None,
        vcs,
        vcs_hover,
        vcs_send,
    }
}

/// Pending-commit archive on the Change list: package is dirty by definition.
/// VCS hover is scoped to this archive folder (not repo-wide).
pub fn phase_display_for_pending_archive() -> PhaseDisplay {
    PhaseDisplay {
        short: PhaseShort::Archived,
        lifecycle_hover: ARCHIVED_HOVER.into(),
        lifecycle_send: None,
        vcs: Some(VcsPill::Uncommitted),
        vcs_hover: Some(VCS_HOVER_ARCHIVE_PACKAGE),
        vcs_send: Some("Commit"),
    }
}

/// Activate uncommitted chrome for a pending archive: chat send only (no VCS).
pub fn pending_uncommitted_activate(archive_id: &str) -> Message {
    Message::PhasePillSend {
        target: archive_id.to_string(),
        text: "Commit".to_string(),
    }
}

/// Exploration row.
pub fn phase_display_for_exploration(session_empty: bool) -> PhaseDisplay {
    PhaseDisplay {
        short: PhaseShort::Explore,
        lifecycle_hover: EXPLORE_HOVER.into(),
        lifecycle_send: if session_empty {
            crate::fast_response::format_lifecycle_command("ds-explore")
        } else {
            None
        },
        vcs: None,
        vcs_hover: None,
        vcs_send: None,
    }
}

/// Prompt text submitted when a phase pill is activated.
pub fn phase_pill_activation_send(
    display: &PhaseDisplay,
    kind: PhasePillKind,
) -> Option<String> {
    match kind {
        PhasePillKind::Lifecycle => display.lifecycle_send.clone(),
        PhasePillKind::Vcs => display.vcs_send.map(str::to_string),
    }
}

/// Suggested next `/ds-*` command for a change, derived from its lifecycle
/// facts. Thin caller over `change_scope_facts` so the placeholder and the
/// scope orientation share one source of truth. Production paths derive the
/// command from already-computed facts in `refresh_fast_response`; this
/// wrapper exists for the test-only `compute_lifecycle_command`.
#[cfg(test)]
fn lifecycle_command_from_artifacts(name: &str, project: &ProjectData) -> Option<String> {
    change_scope_facts(name, project).and_then(|f| f.next_command)
}

/// Refresh `scope_facts` and re-sync fast-response chips on every session of
/// every change / exploration interaction. `scope_facts` drives orientation and
/// empty-session next-action bootstrap. Shell is re-derived from settled oneshot
/// when eligible; a parked user-choice fill is left alone.
pub fn refresh_fast_response(
    interactions: &mut HashMap<Scope, InteractionState>,
    project: &ProjectData,
    agent_input_hints: bool,
    _vcs_dirty: bool,
) {
    for (scope, ix) in interactions.iter_mut() {
        if matches!(scope, Scope::Caps | Scope::Codex) {
            continue;
        }
        // Facts once per change scope; non-change scopes carry none.
        let facts = match scope {
            Scope::Change(name) => change_scope_facts(name, project),
            Scope::Exploration(_) | Scope::Caps | Scope::Codex => None,
        };
        for ax in ix.sessions.iter_mut() {
            ax.scope_facts = facts.clone();
            // Next-action list uses scope_facts bootstrap and last assistant.
            // Not a turn boundary — keep Tab index if the list is unchanged.
            ax.refresh_next_actions(false);
            // Re-sync oneshot chips after next-actions (eligibility depends on
            // empty next-action list). Leaves UserChoice fills alone.
            super::interaction::sync_oneshot_chips(ax, agent_input_hints);
        }
    }
}

// ── Breadcrumbs ──────────────────────────────────────────────────────────────

pub fn breadcrumbs(state: &State, project: &ProjectData, tabs: &tab_bar::TabState) -> Vec<String> {
    let Some(selected) = state.selected_change.as_deref() else {
        return vec!["Changes".into()];
    };

    if let Some(exp) = state.explorations.iter().find(|e| e.id == selected) {
        return vec!["Explorations".into(), exp.display_name.clone()];
    }

    // Finished archives use Archive chrome; pending packages stay under Changes.
    let list_archived =
        is_list_finished_archive(selected, project, state.changed_files.as_slice());

    if let Some(tab) = tabs.active_tab() {
        return tab_breadcrumbs(&tab.id, selected, list_archived);
    }

    let root = if list_archived { "Archive" } else { "Changes" };
    vec![root.into(), selected.into()]
}

/// Breadcrumb root uses list presentation (`selected_list_archived`), not disk
/// location alone — pending archives keep `archive/…` tab ids but list as Changes.
fn tab_breadcrumbs(id: &str, selected: &str, selected_list_archived: bool) -> Vec<String> {
    if let Some(path) = id.strip_prefix("file:") {
        return vec!["Files".into(), path.into()];
    }

    if let Some(path) = id.strip_prefix("vcs:") {
        let root = if selected_list_archived {
            "Archive"
        } else {
            "Changes"
        };
        return vec![
            root.into(),
            selected.into(),
            "Changed files".into(),
            path.into(),
        ];
    }

    let root_rest = id
        .strip_prefix("changes/")
        .map(|r| ("Changes", r))
        .or_else(|| {
            id.strip_prefix("archive/").map(|r| {
                let root = if selected_list_archived {
                    "Archive"
                } else {
                    "Changes"
                };
                (root, r)
            })
        });

    if let Some((root, rest)) = root_rest {
        let (change, inner) = rest.split_once('/').unwrap_or((rest, ""));
        let mut segs = vec![root.into(), change.into()];
        segs.extend(parse_change_inner(inner));
        return segs;
    }

    vec![id.into()]
}

fn parse_change_inner(path: &str) -> Vec<String> {
    if path.is_empty() {
        return vec![];
    }
    if path == "proposal.md" {
        return vec!["Proposal".into()];
    }
    if path == "design.md" {
        return vec!["Design".into()];
    }
    if let Some(rest) = path.strip_prefix("caps/") {
        let mut segs = vec!["Capabilities".into()];
        segs.extend(rest.split('/').map(str::to_string));
        return segs;
    }
    if let Some(rest) = path.strip_prefix("steps/") {
        return vec!["Steps".into(), rest.into()];
    }
    if let Some(rest) = path.strip_prefix("reviews/") {
        return vec!["Reviews".into(), rest.into()];
    }
    path.split('/').map(str::to_string).collect()
}

/// Hover leading-control action for an exploration row.
/// Live → soft archive (one click). Archived → remove with arm when sessions remain.
#[derive(Debug, Clone, PartialEq, Eq)]
enum ExplorationHoverAction {
    Archive(String),
    ArmRemove(String),
    /// `armed` tints the control red (second click after arming).
    Remove {
        id: String,
        armed: bool,
    },
}

fn exploration_hover_action(exp: &Exploration, armed: bool) -> ExplorationHoverAction {
    if !exp.is_archived() {
        ExplorationHoverAction::Archive(exp.id.clone())
    } else if exp.session_count == 0 {
        ExplorationHoverAction::Remove {
            id: exp.id.clone(),
            armed: false,
        }
    } else if armed {
        ExplorationHoverAction::Remove {
            id: exp.id.clone(),
            armed: true,
        }
    } else {
        ExplorationHoverAction::ArmRemove(exp.id.clone())
    }
}

fn exploration_hover_button<'a>(action: ExplorationHoverAction) -> Element<'a, Message> {
    match action {
        ExplorationHoverAction::Archive(id) => {
            collapsible::close_button_sized(Message::ArchiveExploration(id), list_view::ICON_SIZE)
        }
        ExplorationHoverAction::Remove { id, armed: true } => {
            collapsible::close_button_sized_tinted(
                Message::RemoveExploration(id),
                list_view::ICON_SIZE,
                theme::error(),
            )
        }
        ExplorationHoverAction::Remove { id, armed: false } => {
            collapsible::close_button_sized(Message::RemoveExploration(id), list_view::ICON_SIZE)
        }
        ExplorationHoverAction::ArmRemove(id) => {
            collapsible::close_button_sized(Message::ArmRemoveExploration(id), list_view::ICON_SIZE)
        }
    }
}

/// Whether dirty path `rel` lies under `duckspec/archive/<archive_id>/`
/// (or is exactly that archive directory path). Paths are project-root
/// relative as emitted by `vcs::changed_files`.
pub fn archive_package_dirty(archive_id: &str, dirty: &[ChangedFile]) -> bool {
    let folder = format!("duckspec/archive/{archive_id}");
    let prefix = format!("{folder}/");
    dirty.iter().any(|f| {
        let p = f.path.to_string_lossy();
        // Normalize Windows separators if any ever leak through.
        let p = p.replace('\\', "/");
        p == folder || p.starts_with(&prefix)
    })
}

/// Archived `ChangeData` is pending commit when its archive folder still has
/// residual dirty paths. `prefix` is `archive/<folder-id>` from the loader.
pub fn is_pending_commit_archive(ch: &ChangeData, dirty: &[ChangedFile]) -> bool {
    ch.prefix.starts_with("archive/") && archive_package_dirty(&ch.name, dirty)
}

/// Finished archive for list presentation: on disk under archive and not pending.
/// Pending packages use Change-list chrome (`Changes` breadcrumb, `picker` section).
pub fn is_list_finished_archive(
    name: &str,
    project: &ProjectData,
    dirty: &[ChangedFile],
) -> bool {
    project
        .archived_changes
        .iter()
        .any(|ch| ch.name == name && !is_pending_commit_archive(ch, dirty))
}

/// Unified Archived list row (Change list + Dashboard).
#[derive(Debug, Clone, Copy)]
pub enum ArchivedEntry<'a> {
    Change(&'a ChangeData),
    Exploration(&'a Exploration),
}

impl ArchivedEntry<'_> {
    /// Sort key: higher is more recent (string-desc works for folder prefixes
    /// and ISO archive stamps when compared lexicographically).
    fn sort_key(self) -> String {
        match self {
            ArchivedEntry::Change(ch) => ch.name.clone(),
            ArchivedEntry::Exploration(exp) => exp.archived_at.clone().unwrap_or_default(),
        }
    }
}

/// Archived packages still pending path-scoped commit, newest archive id first.
pub fn pending_archives<'a>(
    changes: &'a [ChangeData],
    dirty: &[ChangedFile],
) -> Vec<&'a ChangeData> {
    let mut pending: Vec<&'a ChangeData> = changes
        .iter()
        .filter(|ch| is_pending_commit_archive(ch, dirty))
        .collect();
    pending.sort_by(|a, b| b.name.cmp(&a.name));
    pending
}

/// Non–idea-owned archived explorations + **finished** archived changes,
/// newest first. Pending packages (dirty under their archive folder) are
/// omitted — they live on the Change list instead.
pub fn archived_entries<'a>(
    changes: &'a [ChangeData],
    explorations: &'a [Exploration],
    dirty: &[ChangedFile],
) -> Vec<ArchivedEntry<'a>> {
    let mut entries: Vec<ArchivedEntry<'a>> = changes
        .iter()
        .filter(|ch| !is_pending_commit_archive(ch, dirty))
        .map(ArchivedEntry::Change)
        .chain(
            explorations
                .iter()
                .filter(|e| e.is_on_archived_list())
                .map(ArchivedEntry::Exploration),
        )
        .collect();
    entries.sort_by_key(|e| std::cmp::Reverse(e.sort_key()));
    entries
}

pub fn has_archived_section(
    changes: &[ChangeData],
    explorations: &[Exploration],
    dirty: &[ChangedFile],
) -> bool {
    changes
        .iter()
        .any(|ch| !is_pending_commit_archive(ch, dirty))
        || explorations.iter().any(|e| e.is_on_archived_list())
}

/// Rows under the Change picker: live explorations + active changes + pending archives.
fn change_section_count(
    explorations: &[Exploration],
    active: &[ChangeData],
    pending_archive_count: usize,
) -> usize {
    explorations.iter().filter(|e| e.is_on_live_list()).count()
        + active.len()
        + pending_archive_count
}

enum LiveQueueEntry<'a> {
    Exploration(&'a Exploration),
    Change(&'a ChangeData),
}

fn ordered_live_queue<'a>(
    state: &'a State,
    project: &'a ProjectData,
    ideas: &'a super::ideas::State,
    list_prefs: &crate::queue_list::ListConfig,
    project_root: Option<&Path>,
    dirty: &[ChangedFile],
) -> Vec<LiveQueueEntry<'a>> {
    use crate::queue_list::{QueueKey, QueueRowMeta, sort_queue};

    let mut metas: Vec<QueueRowMeta> = Vec::new();
    let mut entries: Vec<LiveQueueEntry<'a>> = Vec::new();

    for exp in state.explorations.iter().filter(|e| e.is_on_live_list()) {
        let idea = crate::idea_store::idea_for_exploration(&ideas.ideas, &exp.id);
        let sessions = crate::chat_store::load_sessions_for(&exp.id, project_root);
        metas.push(QueueRowMeta {
            key: QueueKey::Exploration(exp.id.clone()),
            title: exp.display_name.clone(),
            mark: idea.map(|i| i.frontmatter.mark).unwrap_or_default(),
            favored_at: idea.and_then(|i| i.frontmatter.favored_at.clone()),
            type_tags: idea
                .map(|i| crate::queue_list::type_tags_from_idea_tags(&i.frontmatter.tags))
                .unwrap_or_default(),
            phase: None,
            last_message_at: crate::queue_list::last_message_activity_nanos(&sessions),
            created_at: idea.map(|i| i.frontmatter.created.clone()),
        });
        entries.push(LiveQueueEntry::Exploration(exp));
    }
    for ch in &project.active_changes {
        let idea = crate::idea_store::idea_for_change(&ideas.ideas, &ch.name);
        let sessions = crate::chat_store::load_sessions_for(&ch.name, project_root);
        let phase = change_scope_facts(&ch.name, project).map(|f| f.phase.to_string());
        metas.push(QueueRowMeta {
            key: QueueKey::Change(ch.name.clone()),
            title: ch.name.clone(),
            mark: idea.map(|i| i.frontmatter.mark).unwrap_or_default(),
            favored_at: idea.and_then(|i| i.frontmatter.favored_at.clone()),
            type_tags: idea
                .map(|i| crate::queue_list::type_tags_from_idea_tags(&i.frontmatter.tags))
                .unwrap_or_default(),
            phase,
            last_message_at: crate::queue_list::last_message_activity_nanos(&sessions),
            created_at: idea.map(|i| i.frontmatter.created.clone()),
        });
        entries.push(LiveQueueEntry::Change(ch));
    }

    sort_queue(&mut metas, list_prefs.sort_key);
    let mut out = Vec::with_capacity(metas.len());
    for m in &metas {
        let entry = match &m.key {
            QueueKey::Exploration(id) => state
                .explorations
                .iter()
                .find(|e| e.id == *id)
                .map(LiveQueueEntry::Exploration),
            QueueKey::Change(name) => project
                .active_changes
                .iter()
                .find(|c| c.name == *name)
                .map(LiveQueueEntry::Change),
            QueueKey::Idea(_) => None,
        };
        if let Some(e) = entry {
            out.push(e);
        }
    }
    let _ = entries;
    // Pending archives stay after live WIP, newest archive id first — not
    // mixed into queue sort.
    for ch in pending_archives(&project.archived_changes, dirty) {
        out.push(LiveQueueEntry::Change(ch));
    }
    out
}

/// Selection keys for painted live-queue rows in paint order (same as
/// [`ordered_live_queue`]): exploration id or change name.
pub(crate) fn painted_live_queue_ids(
    state: &State,
    project: &ProjectData,
    ideas: &super::ideas::State,
    list_prefs: &crate::queue_list::ListConfig,
    project_root: Option<&Path>,
    dirty: &[ChangedFile],
) -> Vec<String> {
    ordered_live_queue(state, project, ideas, list_prefs, project_root, dirty)
        .into_iter()
        .map(|e| match e {
            LiveQueueEntry::Exploration(exp) => exp.id.clone(),
            LiveQueueEntry::Change(ch) => ch.name.clone(),
        })
        .collect()
}

fn list_phase_pill_pair<'a>(
    display: &PhaseDisplay,
    target: &str,
) -> Element<'a, Message> {
    let on_lifecycle = display.lifecycle_send.as_ref().map(|text| Message::PhasePillSend {
        target: target.to_string(),
        text: text.clone(),
    });
    let on_vcs = display.vcs_send.map(|text| Message::PhasePillSend {
        target: target.to_string(),
        text: text.to_string(),
    });
    // Clone display into owned data for the lifetime of the element: labels are
    // &'static but hover is String. view_pair borrows PhaseDisplay — we need
    // the hover strings to live. Build pills with owned hover via view_pill
    // through a small owned wrapper.
    crate::widget::phase_pill::view_pair_owned(display, on_lifecycle, on_vcs)
}

/// Always-on uncommitted plaque for pending-commit rows (click → Commit chat send).
fn pending_uncommitted_plaque<'a>(archive_id: &str) -> Element<'a, Message> {
    button(
        text(PENDING_UNCOMMITTED_LABEL)
            .size(theme::font_sm())
            .color(theme::warning()),
    )
    .on_press(pending_uncommitted_activate(archive_id))
    .padding([0.0, theme::SPACING_XS])
    .style(theme::icon_button)
    .into()
}

fn pillow_trail<'a>(pillows: &crate::queue_list::RowPillows) -> Element<'a, Message> {
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

/// Sort menu for the Change list. When `phase_pill_list` is true, short phase
/// pills own list phase chrome and Sort "Phase" is omitted.
fn sort_menu_controls<'a>(
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
    // Short clickable pills (Settings) own phase face when enabled — no
    // parallel long-phase Sort toggle.
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

/// Rows under Archived: interleaved archived entries length.
#[cfg(test)]
fn archived_section_count(
    changes: &[ChangeData],
    explorations: &[Exploration],
    dirty: &[ChangedFile],
) -> usize {
    archived_entries(changes, explorations, dirty).len()
}

// ── View ─────────────────────────────────────────────────────────────────────

pub fn view_list<'a>(
    state: &'a State,
    project: &'a ProjectData,
    ideas: &'a super::ideas::State,
    tabs: &'a tab_bar::TabState,
    list_prefs: &'a crate::queue_list::ListConfig,
    project_root: Option<&'a Path>,
    phase_pill_list: bool,
    vcs_dirty: bool,
    placement_enabled: bool,
    placements: &'a crate::worktree::BindingStore,
    vcs_workflow: crate::config::VcsWorkflow,
) -> Element<'a, Message> {
    let dirty = state.changed_files.as_slice();
    let ordered = ordered_live_queue(state, project, ideas, list_prefs, project_root, dirty);
    let mut rows: Vec<ListRow<'a, Message>> = vec![];

    for entry in &ordered {
        match entry {
            LiveQueueEntry::Exploration(exp) => {
                let is_selected = state.selected_change.as_deref() == Some(exp.id.as_str());
                let is_hovered = state.hovered_exploration.as_deref() == Some(exp.id.as_str())
                    || state.hovered_queue_row.as_deref() == Some(exp.id.as_str());
                let is_renaming = state.renaming_exploration.as_deref() == Some(exp.id.as_str());
                let mark = crate::idea_store::mark_for_exploration(&ideas.ideas, &exp.id);
                let idea = crate::idea_store::idea_for_exploration(&ideas.ideas, &exp.id);
                let tags = idea.map(|i| i.frontmatter.tags.as_slice()).unwrap_or(&[]);
                let pillows = crate::queue_list::project_row_pillows(tags, None, list_prefs);
                // Title stays plain; mark lives only on the interactive after_icon control.
                let title_base = exp.display_name.as_str();
                let sessions =
                    crate::chat_store::load_sessions_for(&exp.id, project_root);
                let session_empty =
                    sessions.is_empty() || sessions.iter().all(|s| s.messages.is_empty());
                let can_refresh = sessions
                    .iter()
                    .any(|s| crate::chat_store::title_refresh_target(s).is_some());
                let phase_disp = phase_pill_list
                    .then(|| phase_display_for_exploration(session_empty));

                let mut r = if is_renaming {
                    ListRow::new("")
                        .selected(true)
                        .label_content(rename_input(&state.rename_draft))
                        .on_hover(
                            Message::HoverExploration(exp.id.clone()),
                            Message::UnhoverExploration(exp.id.clone()),
                        )
                } else {
                    let mut row = ListRow::new(title_base.to_string())
                        .selected(is_selected)
                        .on_press(Message::SelectChange(exp.id.clone()))
                        .on_hover(
                            Message::HoverExploration(exp.id.clone()),
                            Message::UnhoverExploration(exp.id.clone()),
                        );
                    if let Some(g) = crate::queue_list::mark_glyph(mark, is_hovered) {
                        row = row.after_icon(
                            button(text(g).size(theme::font_sm()))
                                .on_press(Message::CycleQueueMark(
                                    crate::idea_store::QueueLinkKey::Exploration(exp.id.clone()),
                                ))
                                .padding(0)
                                .style(theme::icon_button)
                                .into(),
                        );
                    }
                    let show_p = crate::queue_list::steady_row_pillows(
                        title_base,
                        &pillows,
                        crate::queue_list::DEFAULT_ROW_CHAR_BUDGET,
                        is_hovered,
                    );
                    if is_hovered || is_selected {
                        let mut actions = row![].spacing(theme::SPACING_XS).align_y(iced::Center);
                        if let Some(ref d) = phase_disp {
                            actions = actions.push(list_phase_pill_pair(d, &exp.id));
                        } else if let Some(p) = show_p {
                            actions = actions.push(pillow_trail(p));
                        }
                        if placement_enabled {
                            actions = actions.push(placement_cycle_button(
                                &exp.id,
                                placements,
                                vcs_workflow,
                            ));
                            actions = actions.push(stack_base_button(&exp.id, placements));
                            actions =
                                actions.push(require_base_button(&exp.id, placements));
                            if crate::worktree::scope_has_sidecar(placements, &exp.id) {
                                actions = actions.push(merge_to_main_button(&exp.id));
                            }
                        }
                        actions = actions.push(row_icon_button(
                            ICON_PENCIL,
                            Message::StartRenameExploration(exp.id.clone()),
                        ));
                        // Only show ↻ when there is summarizable chat; otherwise it
                        // looks clickable but silently no-ops.
                        if can_refresh {
                            actions = actions.push(row_icon_button(
                                ICON_REFRESH,
                                Message::RefreshExplorationTitle(exp.id.clone()),
                            ));
                        }
                        row = row.trailing(actions.into());
                    } else if let Some(ref d) = phase_disp {
                        row = row.trailing(list_phase_pill_pair(d, &exp.id));
                    } else if let Some(p) = show_p {
                        row = row.trailing(pillow_trail(p));
                    }
                    row
                };

                if is_hovered && !is_renaming {
                    let armed =
                        state.armed_remove_exploration.as_deref() == Some(exp.id.as_str());
                    let action = exploration_hover_action(exp, armed);
                    r = r.leading(exploration_hover_button(action));
                } else {
                    r = r.icon(ICON_EXPLORE);
                }
                rows.push(r);
            }
            LiveQueueEntry::Change(ch) => {
                let is_selected = state.selected_change.as_ref() == Some(&ch.name);
                let is_hovered = state.hovered_queue_row.as_deref() == Some(ch.name.as_str());
                let has_err = project
                    .validations
                    .get(&ch.name)
                    .is_some_and(|v| v.total_count() > 0);
                // Ideas link by base slug; archive folder ids strip the date prefix.
                let idea_key = crate::data::strip_archive_prefix(&ch.name).unwrap_or(&ch.name);
                let mark = crate::idea_store::mark_for_change(&ideas.ideas, idea_key);
                let idea = crate::idea_store::idea_for_change(&ideas.ideas, idea_key);
                let tags = idea.map(|i| i.frontmatter.tags.as_slice()).unwrap_or(&[]);
                // Prefer short clickable phase pills when enabled; otherwise keep
                // the dense long-phase queue pillow.
                let phase = if phase_pill_list {
                    None
                } else {
                    change_scope_facts(&ch.name, project).map(|f| f.phase)
                };
                let is_pending_archive = is_pending_commit_archive(ch, dirty);
                let pillows = crate::queue_list::project_row_pillows(tags, phase, list_prefs);
                let phase_disp = if phase_pill_list {
                    if is_pending_archive {
                        Some(phase_display_for_pending_archive())
                    } else if ch.prefix.starts_with("archive/") {
                        Some(phase_display_for_archived(vcs_dirty))
                    } else {
                        phase_display_for_change(&ch.name, project, vcs_dirty)
                    }
                } else {
                    None
                };
                // Title stays plain; mark lives only on the interactive after_icon control.
                let mut r = ListRow::new(ch.name.clone())
                    .icon(ICON_BRANCH)
                    .selected(is_selected)
                    .errored(has_err)
                    .on_press(Message::SelectChange(ch.name.clone()))
                    .on_hover(
                        Message::HoverQueueRow(ch.name.clone()),
                        Message::UnhoverQueueRow(ch.name.clone()),
                    );
                if let Some(g) = crate::queue_list::mark_glyph(mark, is_hovered) {
                    r = r.after_icon(
                        button(text(g).size(theme::font_sm()))
                            .on_press(Message::CycleQueueMark(
                                crate::idea_store::QueueLinkKey::Change(idea_key.to_string()),
                            ))
                            .padding(0)
                            .style(theme::icon_button)
                            .into(),
                    );
                } else if ideas.idea_path_for_change(idea_key).is_some() {
                    r = r.after_icon(idea_link_button(idea_key));
                }
                let mut trail = row![].spacing(theme::SPACING_XS).align_y(iced::Center);
                let mut has_trail = false;
                if let Some(ref d) = phase_disp {
                    trail = trail.push(list_phase_pill_pair(d, &ch.name));
                    has_trail = true;
                } else if !is_pending_archive
                    && let Some(p) = crate::queue_list::steady_row_pillows(
                        &ch.name,
                        &pillows,
                        crate::queue_list::DEFAULT_ROW_CHAR_BUDGET,
                        is_hovered,
                    )
                {
                    trail = trail.push(pillow_trail(p));
                    has_trail = true;
                }
                if is_pending_archive {
                    trail = trail.push(pending_uncommitted_plaque(&ch.name));
                    has_trail = true;
                }
                if ideas.idea_path_for_change(idea_key).is_some()
                    && crate::queue_list::mark_glyph(mark, is_hovered).is_some()
                {
                    trail = trail.push(idea_link_button(idea_key));
                    has_trail = true;
                }
                if placement_enabled && (is_hovered || is_selected) {
                    trail = trail.push(placement_cycle_button(
                        &ch.name,
                        placements,
                        vcs_workflow,
                    ));
                    trail = trail.push(stack_base_button(&ch.name, placements));
                    trail = trail.push(require_base_button(&ch.name, placements));
                    if crate::worktree::scope_has_sidecar(placements, &ch.name) {
                        trail = trail.push(merge_to_main_button(&ch.name));
                    }
                    has_trail = true;
                }
                if has_trail {
                    r = r.trailing(trail.into());
                }
                rows.push(r);
            }
        }
    }

    let pending_n = pending_archives(&project.archived_changes, dirty).len();
    let change_count =
        change_section_count(&state.explorations, &project.active_changes, pending_n);
    debug_assert_eq!(rows.len(), change_count);
    let selector = list_view::view(rows, None);

    let archived_list =
        archived_entries(&project.archived_changes, &state.explorations, dirty);
    let archived_count = archived_list.len();
    let archived_rows: Vec<ListRow<'a, Message>> = archived_list
        .into_iter()
        .map(|entry| match entry {
            ArchivedEntry::Change(ch) => {
                let is_selected = state.selected_change.as_ref() == Some(&ch.name);
                let has_err = project
                    .validations
                    .get(&ch.name)
                    .is_some_and(|v| v.total_count() > 0);
                let base = crate::data::strip_archive_prefix(&ch.name).unwrap_or(&ch.name);
                let mut r = ListRow::new(ch.name.as_str())
                    .icon(ICON_BRANCH)
                    .selected(is_selected)
                    .errored(has_err)
                    .on_press(Message::SelectChange(ch.name.clone()));
                if ideas.idea_path_for_change(base).is_some() {
                    r = r.after_icon(idea_link_button(base));
                }
                if phase_pill_list {
                    let d = phase_display_for_archived(vcs_dirty);
                    r = r.trailing(list_phase_pill_pair(&d, &ch.name));
                }
                r
            }
            ArchivedEntry::Exploration(exp) => {
                let is_selected = state.selected_change.as_deref() == Some(exp.id.as_str());
                let is_hovered = state.hovered_exploration.as_deref() == Some(exp.id.as_str());
                let mut r = ListRow::new(exp.display_name.as_str())
                    .selected(is_selected)
                    .on_press(Message::SelectChange(exp.id.clone()))
                    .on_hover(
                        Message::HoverExploration(exp.id.clone()),
                        Message::UnhoverExploration(exp.id.clone()),
                    );
                if is_hovered {
                    let armed = state.armed_remove_exploration.as_deref() == Some(exp.id.as_str());
                    r = r.leading(exploration_hover_button(exploration_hover_action(
                        exp, armed,
                    )));
                } else {
                    r = r.icon(ICON_EXPLORE);
                }
                if phase_pill_list {
                    let sessions =
                        crate::chat_store::load_sessions_for(&exp.id, project_root);
                    let session_empty =
                        sessions.is_empty() || sessions.iter().all(|s| s.messages.is_empty());
                    let d = phase_display_for_exploration(session_empty);
                    r = r.trailing(list_phase_pill_pair(&d, &exp.id));
                }
                r
            }
        })
        .collect();

    let archived_section =
        if has_archived_section(&project.archived_changes, &state.explorations, dirty) {
            Some(collapsible::view_with_add_owned(
                format!("Archived  ({archived_count})"),
                state.expanded_sections.contains("archived"),
                Message::ToggleSection("archived".to_string()),
                None,
                list_view::view(archived_rows, None),
            ))
        } else {
        None
    };

    let header_actions = row![
        sort_menu_controls(state.sort_menu_open, list_prefs, phase_pill_list),
        collapsible::add_button(Message::AddExploration),
    ]
    .spacing(theme::SPACING_XS)
    .align_y(iced::Center);
    let change_section = collapsible::view_with_add_owned(
        format!("Change  ({change_count})"),
        state.expanded_sections.contains("picker"),
        Message::ToggleSection("picker".to_string()),
        Some(header_actions.into()),
        selector,
    );

    let change = find_change(state, project);
    let is_exploration = state.is_exploration_selected();
    let mut list_col = column![change_section].spacing(0.0);

    if let Some(section) = archived_section {
        list_col = list_col.push(section);
    }

    if is_exploration {
        list_col = list_col.push(
            container(
                text("Exploration mode — use the agent or terminal to work freely.")
                    .size(theme::font_md())
                    .color(theme::text_muted()),
            )
            .padding([theme::SPACING_SM, theme::SPACING_SM]),
        );
    } else if let Some(change) = change {
        let error_ids: HashSet<String> = project
            .validations
            .get(&change.name)
            .map(|v| v.file_errors.iter().map(|(p, _)| p.clone()).collect())
            .unwrap_or_default();
        list_col = list_col.push(view_overview_section(tabs, state, change, &error_ids));
        list_col = list_col.push(view_caps_section(tabs, state, change, &error_ids));
        list_col = list_col.push(view_reviews_section(tabs, state, change, &error_ids));
        list_col = list_col.push(view_steps_section(tabs, state, change, &error_ids));
    }

    list_col = list_col.push(view_changed_files_section(tabs, state));

    let files_expanded = state.expanded_sections.contains(FILES_SECTION);
    if files_expanded {
        list_col = list_col.push(view_files_section(tabs, state));
    }

    let scroll: Element<'a, Message> = container(vertical_scroll::view(
        state.list_scroll,
        Message::ScrollList,
        list_col,
    ))
    .width(iced::Length::Fill)
    .height(iced::Length::Fill)
    .id(EXPLORER_VIEWPORT_ID)
    .into();

    if files_expanded {
        scroll
    } else {
        // Collapsed: pin the Files header below the scroll viewport so it
        // stays anchored to the bottom edge of the list column regardless
        // of scroll position.
        column![scroll, view_files_section(tabs, state)]
            .height(iced::Length::Fill)
            .into()
    }
}

fn view_overview_section<'a>(
    tabs: &'a tab_bar::TabState,
    state: &'a State,
    change: &'a ChangeData,
    error_ids: &HashSet<String>,
) -> Element<'a, Message> {
    let active_id = tabs.active_tab().map(|t| t.id.as_str());
    let mut rows: Vec<ListRow<'a, Message>> = vec![];

    let mut push_file = |label: &'static str, id: String, has_err: bool| {
        let r = ListRow::new(label)
            .icon(icon_for_artifact(label))
            .selected(active_id == Some(id.as_str()))
            .errored(has_err)
            .on_press(Message::SelectItem(id));
        rows.push(r);
    };

    if change.has_proposal {
        let id = format!("{}/proposal.md", change.prefix);
        let has_err = error_ids.contains(&id);
        push_file("proposal.md", id, has_err);
    }
    if change.has_design {
        let id = format!("{}/design.md", change.prefix);
        let has_err = error_ids.contains(&id);
        push_file("design.md", id, has_err);
    }

    collapsible::view(
        "Overview",
        state.expanded_sections.contains("overview"),
        Message::ToggleSection("overview".to_string()),
        list_view::view(rows, Some("No overview files")),
    )
}

fn view_caps_section<'a>(
    tabs: &'a tab_bar::TabState,
    state: &'a State,
    change: &'a ChangeData,
    error_ids: &HashSet<String>,
) -> Element<'a, Message> {
    let content = if change.cap_tree.is_empty() {
        container(
            text("No capability changes")
                .size(theme::font_md())
                .color(theme::text_muted()),
        )
        .padding([theme::SPACING_XS, theme::SPACING_SM])
        .into()
    } else {
        tree_view::view(
            &change.cap_tree,
            &state.expanded_nodes,
            tabs.active_tab().map(|t| t.id.as_str()),
            error_ids,
            Message::ToggleNode,
            Message::SelectItem,
        )
    };

    collapsible::view(
        "Capabilities",
        state.expanded_sections.contains("capabilities"),
        Message::ToggleSection("capabilities".to_string()),
        content,
    )
}

fn view_reviews_section<'a>(
    tabs: &'a tab_bar::TabState,
    state: &'a State,
    change: &'a ChangeData,
    error_ids: &HashSet<String>,
) -> Element<'a, Message> {
    let active_id = tabs.active_tab().map(|t| t.id.as_str());
    let rows: Vec<ListRow<'a, Message>> = change
        .reviews
        .iter()
        .map(|filename| {
            let id = format!("{}/reviews/{}", change.prefix, filename);
            let has_err = error_ids.contains(&id);
            ListRow::new(filename.as_str())
                .icon(ICON_DOC)
                .selected(active_id == Some(id.as_str()))
                .errored(has_err)
                .on_press(Message::SelectItem(id))
        })
        .collect();

    collapsible::view(
        "Reviews",
        state.expanded_sections.contains("reviews"),
        Message::ToggleSection("reviews".to_string()),
        list_view::view(rows, Some("No reviews")),
    )
}

fn view_steps_section<'a>(
    tabs: &'a tab_bar::TabState,
    state: &'a State,
    change: &'a ChangeData,
    error_ids: &HashSet<String>,
) -> Element<'a, Message> {
    let active_id = tabs.active_tab().map(|t| t.id.as_str());
    let rows: Vec<ListRow<'a, Message>> = change
        .steps
        .iter()
        .map(|step| {
            let (icon_bytes, icon_tint): (&'static [u8], Option<iced::Color>) =
                match step.completion {
                    StepCompletion::Done => (ICON_STEP_DONE, Some(theme::success())),
                    StepCompletion::Partial(0, _) | StepCompletion::NoTasks => (ICON_STEP, None),
                    StepCompletion::Partial(_, _) => (ICON_STEP_PARTIAL, Some(theme::warning())),
                };
            let has_err = error_ids.contains(&step.id);
            let mut r = ListRow::new(step.label.as_str())
                .icon(icon_bytes)
                .selected(active_id == Some(step.id.as_str()))
                .errored(has_err)
                .on_press(Message::SelectItem(step.id.clone()));
            if let Some(tint) = icon_tint {
                r = r.icon_tint(tint);
            }
            r
        })
        .collect();

    collapsible::view(
        "Steps",
        state.expanded_sections.contains("steps"),
        Message::ToggleSection("steps".to_string()),
        list_view::view(rows, Some("No steps")),
    )
}

/// Tree of changed files grouped by directory.
struct FileTree {
    dirs: BTreeMap<String, FileTree>,
    files: Vec<ChangedFile>,
    path: PathBuf,
}

impl FileTree {
    fn new(path: PathBuf) -> Self {
        Self {
            dirs: BTreeMap::new(),
            files: vec![],
            path,
        }
    }

    fn insert(&mut self, file: ChangedFile) {
        let parts: Vec<String> = file
            .path
            .components()
            .filter_map(|c| c.as_os_str().to_str().map(|s| s.to_string()))
            .collect();
        if parts.is_empty() {
            return;
        }
        let mut node = self;
        let mut current_path = PathBuf::new();
        for part in &parts[..parts.len() - 1] {
            current_path.push(part);
            node = node
                .dirs
                .entry(part.clone())
                .or_insert_with(|| FileTree::new(current_path.clone()));
        }
        node.files.push(file);
    }
}

fn aggregate_status(node: &FileTree) -> Option<FileStatus> {
    fn visit(node: &FileTree, seen: &mut Option<FileStatus>) -> bool {
        for file in &node.files {
            match seen {
                None => *seen = Some(file.status),
                Some(s) if *s == file.status => {}
                Some(_) => return false,
            }
        }
        for sub in node.dirs.values() {
            if !visit(sub, seen) {
                return false;
            }
        }
        true
    }
    let mut seen = None;
    if visit(node, &mut seen) { seen } else { None }
}

/// Owned flat row for the Changed Files section (view maps this; no tree rebuild).
#[derive(Debug, Clone, PartialEq, Eq)]
enum ChangedFileRow {
    Dir {
        key: String,
        name: String,
        depth: usize,
        is_expanded: bool,
        agg: Option<FileStatus>,
    },
    File {
        path: PathBuf,
        status: FileStatus,
        name: String,
        depth: usize,
    },
}

fn rebuild_changed_file_rows(
    files: &[ChangedFile],
    expanded: &HashSet<String>,
) -> Vec<ChangedFileRow> {
    if files.is_empty() {
        return vec![];
    }
    let mut tree = FileTree::new(PathBuf::new());
    for cf in files {
        tree.insert(cf.clone());
    }
    let mut flat = Vec::new();
    flatten_file_tree(&tree, 0, expanded, &mut flat);
    flat
}

fn flatten_file_tree(
    node: &FileTree,
    depth: usize,
    expanded: &HashSet<String>,
    out: &mut Vec<ChangedFileRow>,
) {
    for (name, sub) in &node.dirs {
        let key = sub.path.display().to_string();
        let is_expanded = expanded.contains(&key);
        let agg = aggregate_status(sub);
        out.push(ChangedFileRow::Dir {
            key,
            name: name.clone(),
            depth,
            is_expanded,
            agg,
        });
        if is_expanded {
            flatten_file_tree(sub, depth + 1, expanded, out);
        }
    }
    let mut files: Vec<&ChangedFile> = node.files.iter().collect();
    files.sort_by_key(|f| {
        f.path
            .file_name()
            .map(|s| s.to_os_string())
            .unwrap_or_default()
    });
    for file in files {
        let name = file
            .path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| file.path.display().to_string());
        out.push(ChangedFileRow::File {
            path: file.path.clone(),
            status: file.status,
            name,
            depth,
        });
    }
}

fn status_char(status: FileStatus) -> &'static str {
    match status {
        FileStatus::Modified => "M",
        FileStatus::Added => "A",
        FileStatus::Deleted => "D",
    }
}

fn view_changed_files_section<'a>(
    tabs: &'a tab_bar::TabState,
    state: &'a State,
) -> Element<'a, Message> {
    let active_tab_id = tabs.active_tab().map(|t| t.id.as_str());
    let rows: Vec<ListRow<'a, Message>> = state
        .changed_file_rows
        .iter()
        .map(|row_data| match row_data {
            ChangedFileRow::Dir {
                key,
                name,
                depth,
                is_expanded,
                agg,
            } => {
                let (sc, color) = match agg {
                    Some(s) => (status_char(*s), theme::vcs_status_color(s)),
                    None => ("~", theme::text_muted()),
                };
                let leading: Element<'a, Message> = row![
                    collapsible::chevron(*is_expanded),
                    text(sc)
                        .size(theme::font_md())
                        .font(theme::content_font())
                        .color(color),
                ]
                .spacing(theme::SPACING_SM)
                .align_y(iced::Center)
                .into();
                ListRow::new(format!("{}/", name))
                    .leading(leading)
                    .indent(*depth)
                    .spacing(theme::SPACING_SM)
                    .on_press(Message::ToggleFileDir(key.clone()))
            }
            ChangedFileRow::File {
                path,
                status,
                name,
                depth,
            } => {
                let sc = status_char(*status);
                let color = theme::vcs_status_color(status);
                let tab_id = format!("vcs:{}", path.display());
                let is_active = active_tab_id == Some(tab_id.as_str());
                let leading: Element<'a, Message> = row![
                    Space::new().width(theme::font_sm()),
                    text(sc)
                        .size(theme::font_md())
                        .font(theme::content_font())
                        .color(color),
                ]
                .spacing(theme::SPACING_SM)
                .align_y(iced::Center)
                .into();
                ListRow::new(name.clone())
                    .leading(leading)
                    .indent(*depth)
                    .spacing(theme::SPACING_SM)
                    .selected(is_active)
                    .on_press(Message::SelectChangedFile(path.clone()))
            }
        })
        .collect();

    collapsible::view_with_add(
        "Changed Files",
        state.expanded_sections.contains("changed_files"),
        Message::ToggleSection("changed_files".to_string()),
        Some(collapsible::add_button(Message::AddFile)),
        list_view::view(rows, Some("No changes")),
    )
}

/// Files explorer section — the full project tree, gitignore-respecting.
/// File rows carry `file:<rel>` node ids, so the row of the file open in
/// the content column highlights without any extra selection state.
fn view_files_section<'a>(tabs: &'a tab_bar::TabState, state: &'a State) -> Element<'a, Message> {
    let expanded = state.expanded_sections.contains(FILES_SECTION);

    let content: Element<'a, Message> = if !expanded {
        // Collapsible sections skip their content when collapsed.
        Space::new().into()
    } else if state.explorer_tree.is_empty() {
        container(
            text("No files")
                .size(theme::font_md())
                .color(theme::text_muted()),
        )
        .padding([theme::SPACING_XS, theme::SPACING_SM])
        .into()
    } else {
        let no_errors = HashSet::new();
        let tints = explorer_vcs_tints(&state.changed_files);
        container(tree_view::view_with_tints(
            &state.explorer_tree,
            &state.expanded_explorer_dirs,
            tabs.active_tab().map(|t| t.id.as_str()),
            &no_errors,
            &tints,
            Message::ToggleExplorerDir,
            Message::SelectExplorerFile,
        ))
        .id(EXPLORER_CONTENT_ID)
        .into()
    };

    collapsible::view_with_add(
        "Files",
        expanded,
        Message::ToggleSection(FILES_SECTION.to_string()),
        Some(collapsible::add_button(Message::AddFile)),
        content,
    )
}

/// Per-node tints for the Files explorer: VCS-changed files take their
/// status color, and every ancestor directory takes the aggregate color of
/// the changes inside it — falling back to the modified color when statuses
/// are mixed — so collapsed directories still signal where changes live.
/// Deleted files have no row in the explorer (they're gone from the working
/// tree) but still tint their ancestors.
fn explorer_vcs_tints(changed: &[ChangedFile]) -> HashMap<String, iced::Color> {
    let mut tints = HashMap::new();
    let mut dir_status: HashMap<String, Option<FileStatus>> = HashMap::new();
    for cf in changed {
        let parts: Vec<&str> = cf
            .path
            .components()
            .filter_map(|c| c.as_os_str().to_str())
            .collect();
        if parts.is_empty() {
            continue;
        }
        if cf.status != FileStatus::Deleted {
            tints.insert(
                format!("file:{}", parts.join("/")),
                theme::vcs_status_color(&cf.status),
            );
        }
        let mut prefix = String::new();
        for part in &parts[..parts.len() - 1] {
            if !prefix.is_empty() {
                prefix.push('/');
            }
            prefix.push_str(part);
            dir_status
                .entry(prefix.clone())
                .and_modify(|s| {
                    if *s != Some(cf.status) {
                        *s = None;
                    }
                })
                .or_insert(Some(cf.status));
        }
    }
    for (dir, status) in dir_status {
        let color = theme::vcs_status_color(&status.unwrap_or(FileStatus::Modified));
        tints.insert(dir, color);
    }
    tints
}

fn icon_for_artifact(label: &str) -> &'static [u8] {
    match label {
        l if l.starts_with("spec.delta") => ICON_SPEC_DELTA,
        l if l.starts_with("spec") => ICON_SPEC,
        l if l.starts_with("doc.delta") => ICON_DOC_DELTA,
        l if l.starts_with("doc") => ICON_DOC,
        _ => ICON_FILE,
    }
}

fn placement_cycle_button<'a>(
    scope_key: &str,
    placements: &crate::worktree::BindingStore,
    workflow: crate::config::VcsWorkflow,
) -> Element<'a, Message> {
    let placement = crate::worktree::effective_placement(placements, scope_key, workflow);
    let label = crate::worktree::placement_label(placement);
    button(text(label).size(theme::font_sm()).color(theme::text_muted()))
        .on_press(Message::CyclePlacement(scope_key.to_string()))
        .padding([0.0, theme::SPACING_XS])
        .style(theme::icon_button)
        .into()
}

fn stack_base_button<'a>(
    scope_key: &str,
    placements: &crate::worktree::BindingStore,
) -> Element<'a, Message> {
    let label = placements
        .scopes
        .get(scope_key)
        .and_then(|b| b.base_scope.as_deref())
        .map(|b| {
            // Short tail so the row stays readable.
            let short = b.rsplit('-').next().unwrap_or(b);
            format!("on:{short}")
        })
        .unwrap_or_else(|| "base".into());
    button(text(label).size(theme::font_sm()).color(theme::text_muted()))
        .on_press(Message::CycleStackBase(scope_key.to_string()))
        .padding([0.0, theme::SPACING_XS])
        .style(theme::icon_button)
        .into()
}

fn require_base_button<'a>(
    scope_key: &str,
    placements: &crate::worktree::BindingStore,
) -> Element<'a, Message> {
    let on = placements
        .scopes
        .get(scope_key)
        .map(|b| b.require_base_merged)
        .unwrap_or(false);
    let label = if on { "lock" } else { "free" };
    button(text(label).size(theme::font_sm()).color(theme::text_muted()))
        .on_press(Message::ToggleRequireBaseMerged(scope_key.to_string()))
        .padding([0.0, theme::SPACING_XS])
        .style(theme::icon_button)
        .into()
}

fn merge_to_main_button<'a>(scope_key: &str) -> Element<'a, Message> {
    button(text("merge").size(theme::font_sm()).color(theme::text_muted()))
        .on_press(Message::MergeToMain(scope_key.to_string()))
        .padding([0.0, theme::SPACING_XS])
        .style(theme::icon_button)
        .into()
}

fn idea_link_button<'a>(change_name: &str) -> Element<'a, Message> {
    let icon = iced::widget::svg(iced::widget::svg::Handle::from_memory(ICON_IDEAS))
        .width(list_view::ICON_SIZE)
        .height(list_view::ICON_SIZE)
        .style(theme::svg_tint(theme::accent()));
    button(icon)
        .on_press(Message::OpenIdeaForChange(change_name.to_string()))
        .padding(0.0)
        .style(theme::icon_button)
        .into()
}

/// Errors associated with the active artifact tab. Used by main.rs's content
/// column renderer to draw the error panel below the editor.
pub fn error_panel_for<'a>(
    state: &State,
    project: &'a ProjectData,
    tabs: &tab_bar::TabState,
) -> Option<&'a [String]> {
    let tab = tabs.active_tab()?;
    let change_name = state.selected_change.as_ref()?;
    let validation = project.validations.get(change_name)?;
    validation
        .file_errors
        .iter()
        .find(|(path, _)| *path == tab.id)
        .map(|(_, errs)| errs.as_slice())
        .filter(|errs| !errs.is_empty())
}

fn find_change<'a>(state: &State, project: &'a ProjectData) -> Option<&'a ChangeData> {
    let name = state.selected_change.as_ref()?;
    project
        .active_changes
        .iter()
        .chain(project.archived_changes.iter())
        .find(|c| &c.name == name)
}

fn open_artifact(
    tabs: &mut tab_bar::TabState,
    id: &str,
    project: &ProjectData,
    highlighter: &crate::highlight::SyntaxHighlighter,
) {
    if let Some(content) = project.read_artifact(id) {
        let title = id.rsplit('/').next().unwrap_or(id).to_string();
        let path = project.duckspec_root.as_ref().map(|r| r.join(id));
        crate::open_artifact_tab(tabs, id.to_string(), title, content, id, path, highlighter);
    }
}

/// Reset the active session for a scope: cancel agent, delete persisted file,
/// and replace with a fresh empty session under a new id.
fn clear_active_session(
    ix: &mut InteractionState,
    scope: &str,
    scope_label: &str,
    scope_kind: ScopeKind,
    project_root: Option<&Path>,
) {
    if ix.sessions.is_empty() {
        ix.sessions
            .push(AgentSession::new(scope.to_string(), scope_kind));
        ix.active_session = 0;
        return;
    }
    let idx = ix.active_session.min(ix.sessions.len() - 1);
    if let Some(ax) = ix.sessions.get(idx) {
        if let Some(handle) = &ax.agent_handle {
            handle.cancel();
        }
        crate::chat_store::delete_session(&ax.session.scope, &ax.session.id, project_root);
    }
    ix.sessions[idx] = AgentSession::new(scope.to_string(), scope_kind);
    ix.active_session = idx;
    interaction::reconcile_display_names(&mut ix.sessions, scope_label);
}

#[cfg(test)]
mod breadcrumb_tests {
    use super::*;

    #[test]
    fn tab_proposal() {
        assert_eq!(
            tab_breadcrumbs("changes/foo/proposal.md", "foo", false),
            vec!["Changes", "foo", "Proposal"]
        );
    }

    #[test]
    fn tab_design() {
        assert_eq!(
            tab_breadcrumbs("changes/foo/design.md", "foo", false),
            vec!["Changes", "foo", "Design"]
        );
    }

    #[test]
    fn tab_step() {
        assert_eq!(
            tab_breadcrumbs("changes/foo/steps/01-bar.md", "foo", false),
            vec!["Changes", "foo", "Steps", "01-bar.md"]
        );
    }

    #[test]
    fn tab_cap_nested() {
        assert_eq!(
            tab_breadcrumbs("changes/foo/caps/auth/session.md", "foo", false),
            vec!["Changes", "foo", "Capabilities", "auth", "session.md"]
        );
    }

    #[test]
    fn tab_cap_deeply_nested() {
        assert_eq!(
            tab_breadcrumbs("changes/foo/caps/a/b/c/d.md", "foo", false),
            vec!["Changes", "foo", "Capabilities", "a", "b", "c", "d.md"]
        );
    }

    #[test]
    fn tab_archive_proposal() {
        assert_eq!(
            tab_breadcrumbs(
                "archive/2026-04-20-01-foo/proposal.md",
                "2026-04-20-01-foo",
                true
            ),
            vec!["Archive", "2026-04-20-01-foo", "Proposal"]
        );
    }

    #[test]
    fn tab_vcs_active() {
        assert_eq!(
            tab_breadcrumbs("vcs:src/main.rs", "foo", false),
            vec!["Changes", "foo", "Changed files", "src/main.rs"]
        );
    }

    #[test]
    fn tab_vcs_archived() {
        assert_eq!(
            tab_breadcrumbs("vcs:src/main.rs", "2026-04-20-01-foo", true),
            vec![
                "Archive",
                "2026-04-20-01-foo",
                "Changed files",
                "src/main.rs"
            ]
        );
    }

    #[test]
    fn tab_file_finder() {
        assert_eq!(
            tab_breadcrumbs("file:Cargo.toml", "foo", false),
            vec!["Files", "Cargo.toml"]
        );
    }

    #[test]
    fn tab_unknown_falls_back() {
        assert_eq!(tab_breadcrumbs("weird-id", "foo", false), vec!["weird-id"]);
    }

    fn make_state(selected: &str, explorations: &[(&str, &str)]) -> State {
        State {
            selected_change: Some(selected.to_string()),
            expanded_sections: HashSet::new(),
            expanded_nodes: HashSet::new(),
            expanded_file_dirs: HashSet::new(),
            changed_files: vec![],
            changed_file_rows: vec![],
            explorer_tree: vec![],
            expanded_explorer_dirs: HashSet::new(),
            explorations: explorations
                .iter()
                .map(|(id, name)| Exploration {
                    id: (*id).to_string(),
                    display_name: (*name).to_string(),
                    idea_path: None,
                    archived_at: None,
                    session_count: 0,
                })
                .collect(),
            exploration_counter: 0,
            hovered_exploration: None,
            armed_remove_exploration: None,
            list_scroll: 0.0,
            known_file_dirs: HashSet::new(),
            pending_bindings: HashMap::new(),
            renaming_exploration: None,
            rename_draft: String::new(),
            sort_menu_open: false,
            hovered_queue_row: None,
        }
    }

    fn make_project(active: &[&str], archived: &[&str]) -> ProjectData {
        let mk = |name: &str, prefix_root: &str| ChangeData {
            name: name.to_string(),
            prefix: format!("{prefix_root}/{name}"),
            has_proposal: false,
            has_design: false,
            cap_tree: vec![],
            steps: vec![],
            reviews: vec![],
            shallow_mtime_nanos: None,
        };
        ProjectData {
            active_changes: active.iter().map(|n| mk(n, "changes")).collect(),
            archived_changes: archived.iter().map(|n| mk(n, "archive")).collect(),
            ..Default::default()
        }
    }

    #[test]
    fn exploration_root_after_selection() {
        let state = make_state("exploration-1000", &[("exploration-1000", "Exploration 1")]);
        let project = make_project(&[], &[]);
        let tabs = tab_bar::TabState::default();
        assert_eq!(
            breadcrumbs(&state, &project, &tabs),
            vec!["Explorations", "Exploration 1"]
        );
    }

    #[test]
    fn start_rename_opens_draft_for_exploration() {
        let mut state = make_state("exploration-1", &[("exploration-1", "Cloud agent options")]);
        let mut tabs = tab_bar::TabState::default();
        let mut interactions = HashMap::new();
        let project = make_project(&[], &[]);
        let highlighter = crate::highlight::SyntaxHighlighter::new();
        update(
            &mut state,
            &mut tabs,
            &mut interactions,
            Message::StartRenameExploration("exploration-1".into()),
            &project,
            &highlighter,
            false,
            1200.0,
            crate::config::VcsWorkflow::default(),
            crate::config::ViewerStyle::Classic,
        );
        assert_eq!(
            state.renaming_exploration.as_deref(),
            Some("exploration-1")
        );
        assert_eq!(state.rename_draft, "Cloud agent options");
        assert_eq!(state.selected_change.as_deref(), Some("exploration-1"));
    }

    #[test]
    fn second_select_on_exploration_opens_rename() {
        let mut state = make_state("exploration-1", &[("exploration-1", "Exploration 1")]);
        let mut tabs = tab_bar::TabState::default();
        let mut interactions = HashMap::new();
        let project = make_project(&[], &[]);
        let highlighter = crate::highlight::SyntaxHighlighter::new();
        update(
            &mut state,
            &mut tabs,
            &mut interactions,
            Message::SelectChange("exploration-1".into()),
            &project,
            &highlighter,
            false,
            1200.0,
            crate::config::VcsWorkflow::default(),
            crate::config::ViewerStyle::Classic,
        );
        assert_eq!(
            state.renaming_exploration.as_deref(),
            Some("exploration-1")
        );
        assert_eq!(state.rename_draft, "Exploration 1");
    }

    #[test]
    fn exploration_promoted_to_change_shows_changes_root() {
        let state = make_state("real-change", &[]);
        let project = make_project(&["real-change"], &[]);
        let tabs = tab_bar::TabState::default();
        assert_eq!(
            breadcrumbs(&state, &project, &tabs),
            vec!["Changes", "real-change"]
        );
    }

    #[test]
    fn change_archived_shows_archive_root() {
        let state = make_state("2026-04-20-01-foo", &[]);
        let project = make_project(&[], &["2026-04-20-01-foo"]);
        let tabs = tab_bar::TabState::default();
        assert_eq!(
            breadcrumbs(&state, &project, &tabs),
            vec!["Archive", "2026-04-20-01-foo"]
        );
    }

    fn tree_node(id: &str) -> crate::data::TreeNode {
        crate::data::TreeNode {
            id: id.into(),
            label: id.into(),
            children: vec![],
        }
    }

    fn step(done: bool) -> crate::data::StepInfo {
        crate::data::StepInfo {
            id: "changes/foo/steps/01-bar.md".into(),
            label: "01-bar.md".into(),
            completion: if done {
                StepCompletion::Done
            } else {
                StepCompletion::Partial(0, 1)
            },
        }
    }

    fn set_change(project: &mut ProjectData, name: &str, mutate: impl FnOnce(&mut ChangeData)) {
        let ch = project
            .active_changes
            .iter_mut()
            .find(|c| c.name == name)
            .expect("change exists");
        mutate(ch);
    }

    #[test]
    fn lifecycle_nothing_selected() {
        let state = State {
            selected_change: None,
            expanded_sections: HashSet::new(),
            expanded_nodes: HashSet::new(),
            expanded_file_dirs: HashSet::new(),
            changed_files: vec![],
            changed_file_rows: vec![],
            explorer_tree: vec![],
            expanded_explorer_dirs: HashSet::new(),
            explorations: vec![],
            exploration_counter: 0,
            hovered_exploration: None,
            armed_remove_exploration: None,
            list_scroll: 0.0,
            known_file_dirs: HashSet::new(),
            pending_bindings: HashMap::new(),
            renaming_exploration: None,
            rename_draft: String::new(),
            sort_menu_open: false,
            hovered_queue_row: None,
        };
        let project = make_project(&[], &[]);
        assert_eq!(compute_lifecycle_command(&state, &project), None);
    }

    #[test]
    fn lifecycle_exploration_always_explore() {
        let state = make_state("exploration-1000", &[("exploration-1000", "Exploration 1")]);
        let project = make_project(&[], &[]);
        assert_eq!(
            compute_lifecycle_command(&state, &project).as_deref(),
            Some("ds-explore")
        );
    }

    #[test]
    fn lifecycle_archived_is_none() {
        let state = make_state("2026-04-20-01-foo", &[]);
        let project = make_project(&[], &["2026-04-20-01-foo"]);
        assert_eq!(compute_lifecycle_command(&state, &project), None);
    }

    #[test]
    fn lifecycle_empty_change_suggests_propose() {
        let state = make_state("foo", &[]);
        let project = make_project(&["foo"], &[]);
        assert_eq!(
            compute_lifecycle_command(&state, &project).as_deref(),
            Some("ds-propose")
        );
    }

    #[test]
    fn lifecycle_with_proposal_suggests_design() {
        let state = make_state("foo", &[]);
        let mut project = make_project(&["foo"], &[]);
        set_change(&mut project, "foo", |c| c.has_proposal = true);
        assert_eq!(
            compute_lifecycle_command(&state, &project).as_deref(),
            Some("ds-design")
        );
    }

    #[test]
    fn lifecycle_with_design_suggests_spec() {
        let state = make_state("foo", &[]);
        let mut project = make_project(&["foo"], &[]);
        set_change(&mut project, "foo", |c| {
            c.has_proposal = true;
            c.has_design = true;
        });
        assert_eq!(
            compute_lifecycle_command(&state, &project).as_deref(),
            Some("ds-spec")
        );
    }

    #[test]
    fn lifecycle_feature_flow_with_caps_suggests_step() {
        let state = make_state("foo", &[]);
        let mut project = make_project(&["foo"], &[]);
        set_change(&mut project, "foo", |c| {
            c.has_proposal = true;
            c.has_design = true;
            c.cap_tree = vec![tree_node("caps/auth")];
        });
        assert_eq!(
            compute_lifecycle_command(&state, &project).as_deref(),
            Some("ds-step")
        );
    }

    #[test]
    fn lifecycle_caps_without_design_still_suggests_step() {
        // Design is optional: a feature change can go proposal → spec → step
        // with no design.md. Caps-but-no-steps must suggest `ds-step`, never
        // `ds-archive`, regardless of whether a design exists.
        let state = make_state("foo", &[]);
        let mut project = make_project(&["foo"], &[]);
        set_change(&mut project, "foo", |c| {
            c.has_proposal = true;
            c.cap_tree = vec![tree_node("caps/auth")];
        });
        assert_eq!(
            compute_lifecycle_command(&state, &project).as_deref(),
            Some("ds-step")
        );
    }

    #[test]
    fn lifecycle_steps_unfinished_suggests_apply() {
        let state = make_state("foo", &[]);
        let mut project = make_project(&["foo"], &[]);
        set_change(&mut project, "foo", |c| {
            c.has_proposal = true;
            c.has_design = true;
            c.cap_tree = vec![tree_node("caps/auth")];
            c.steps = vec![step(false), step(true)];
        });
        assert_eq!(
            compute_lifecycle_command(&state, &project).as_deref(),
            Some("ds-apply")
        );
    }

    // ── Phase display (shell/phase-pills) ──────────────────────────────────

    // @spec shell/phase-pills Derived stage display: Empty change yields empty short and propose send
    #[test]
    fn empty_change_yields_empty_short_and_propose_send() {
        // GIVEN an active change with no proposal, design, caps, steps, or reviews
        let project = make_project(&["foo"], &[]);
        // WHEN the phase-pill display is derived
        let d = phase_display_for_change("foo", &project, false).expect("active");
        // THEN the short stage is empty AND the lifecycle send is /ds-propose
        assert_eq!(d.short, PhaseShort::Empty);
        assert_eq!(d.short.label(), "empty");
        assert_eq!(d.lifecycle_send.as_deref(), Some("/ds-propose"));
    }

    // @spec shell/phase-pills Derived stage display: Proposal-only yields proposal short and design send
    #[test]
    fn proposal_only_yields_proposal_short_and_design_send() {
        // GIVEN an active change that has a proposal and no design, caps, steps, or reviews
        let mut project = make_project(&["foo"], &[]);
        set_change(&mut project, "foo", |c| c.has_proposal = true);
        // WHEN the phase-pill display is derived
        let d = phase_display_for_change("foo", &project, false).expect("active");
        // THEN the short stage is proposal AND the lifecycle send is /ds-design
        assert_eq!(d.short, PhaseShort::Proposal);
        assert_eq!(d.lifecycle_send.as_deref(), Some("/ds-design"));
    }

    // @spec shell/phase-pills Derived stage display: Open steps yield steps short and apply send
    #[test]
    fn open_steps_yield_steps_short_and_apply_send() {
        // GIVEN an active change that has at least one incomplete step
        let mut project = make_project(&["foo"], &[]);
        set_change(&mut project, "foo", |c| {
            c.has_proposal = true;
            c.has_design = true;
            c.cap_tree = vec![tree_node("caps/auth")];
            c.steps = vec![step(false)];
        });
        // WHEN the phase-pill display is derived
        let d = phase_display_for_change("foo", &project, false).expect("active");
        // THEN the short stage is steps AND the lifecycle send is /ds-apply
        assert_eq!(d.short, PhaseShort::Steps);
        assert_eq!(d.lifecycle_send.as_deref(), Some("/ds-apply"));
    }

    // @spec shell/phase-pills Derived stage display: Complete steps without review yield ready short and archive send
    #[test]
    fn complete_steps_without_review_yield_ready_short_and_archive_send() {
        // GIVEN an active change whose steps are all complete AND the change has no reviews
        let mut project = make_project(&["foo"], &[]);
        set_change(&mut project, "foo", |c| {
            c.has_proposal = true;
            c.cap_tree = vec![tree_node("caps/auth")];
            c.steps = vec![step(true)];
        });
        // WHEN the phase-pill display is derived
        let d = phase_display_for_change("foo", &project, false).expect("active");
        // THEN the short stage is ready AND the lifecycle send is /ds-archive
        assert_eq!(d.short, PhaseShort::Ready);
        assert_eq!(d.lifecycle_send.as_deref(), Some("/ds-archive"));
    }

    // @spec shell/phase-pills Derived stage display: No open steps with review yield review short and step send
    #[test]
    fn no_open_steps_with_review_yield_review_short_and_step_send() {
        // GIVEN an active change with no incomplete steps AND at least one review
        let mut project = make_project(&["foo"], &[]);
        set_change(&mut project, "foo", |c| {
            c.has_proposal = true;
            c.steps = vec![step(true)];
            c.reviews = vec!["01-review.md".into()];
        });
        // WHEN the phase-pill display is derived
        let d = phase_display_for_change("foo", &project, false).expect("active");
        // THEN the short stage is review AND the lifecycle send is /ds-step
        assert_eq!(d.short, PhaseShort::Review);
        assert_eq!(d.lifecycle_send.as_deref(), Some("/ds-step"));
    }

    // @spec shell/phase-pills Derived stage display: Archived yields archived short without lifecycle send
    #[test]
    fn archived_yields_archived_short_without_lifecycle_send() {
        // GIVEN an archived change
        // WHEN the phase-pill display is derived
        let d = phase_display_for_archived(false);
        // THEN the short stage is archived AND there is no lifecycle send
        assert_eq!(d.short, PhaseShort::Archived);
        assert!(d.lifecycle_send.is_none());
    }

    // @spec shell/phase-pills Derived stage display: Active-change long hover is the recognized phase description
    #[test]
    fn active_change_long_hover_is_the_recognized_phase_description() {
        // GIVEN an active change whose recognized long phase description is known
        let mut project = make_project(&["foo"], &[]);
        set_change(&mut project, "foo", |c| c.has_proposal = true);
        let facts = change_scope_facts("foo", &project).expect("facts");
        // WHEN the phase-pill display is derived
        let d = phase_display_for_change("foo", &project, false).expect("active");
        // THEN the lifecycle hover text is that long phase description
        assert_eq!(d.lifecycle_hover, facts.phase);
        assert_eq!(
            d.lifecycle_hover,
            "proposal drafted, design not yet written"
        );
    }

    // @spec shell/phase-pills Late-stage VCS pill: Ready with dirty tree is uncommitted with Commit send
    #[test]
    fn ready_with_dirty_tree_is_uncommitted_with_commit_send() {
        // GIVEN a phase-pill display whose short stage is ready
        // AND the working tree has pending changes versus HEAD
        let mut project = make_project(&["foo"], &[]);
        set_change(&mut project, "foo", |c| {
            c.steps = vec![step(true)];
        });
        // WHEN the VCS pill is derived
        let d = phase_display_for_change("foo", &project, true).expect("active");
        // THEN uncommitted, Commit send, repo-wide dirty hover
        assert_eq!(d.short, PhaseShort::Ready);
        assert_eq!(d.vcs, Some(VcsPill::Uncommitted));
        assert_eq!(d.vcs.unwrap().label(), "uncommitted");
        assert_eq!(d.vcs_send, Some("Commit"));
        assert_eq!(d.vcs_hover, Some(VCS_HOVER_DIRTY));
    }

    // @spec shell/phase-pills Late-stage VCS pill: Ready with clean tree is committed without send
    #[test]
    fn ready_with_clean_tree_is_committed_without_send() {
        // GIVEN ready + clean working tree
        let mut project = make_project(&["foo"], &[]);
        set_change(&mut project, "foo", |c| {
            c.steps = vec![step(true)];
        });
        // WHEN the VCS pill is derived
        let d = phase_display_for_change("foo", &project, false).expect("active");
        // THEN committed and no VCS send
        assert_eq!(d.vcs, Some(VcsPill::Committed));
        assert!(d.vcs_send.is_none());
        assert_eq!(d.vcs_hover, Some(VCS_HOVER_CLEAN));
    }

    // @spec shell/phase-pills Late-stage VCS pill: Pre-ready stage omits VCS pill
    #[test]
    fn pre_ready_stage_omits_vcs_pill() {
        // GIVEN a phase-pill display whose short stage is not ready and not archived
        let mut project = make_project(&["foo"], &[]);
        set_change(&mut project, "foo", |c| c.has_proposal = true);
        // WHEN the display is derived (even if tree dirty)
        let d = phase_display_for_change("foo", &project, true).expect("active");
        // THEN there is no VCS pill
        assert_eq!(d.short, PhaseShort::Proposal);
        assert!(d.vcs.is_none());
        assert!(d.vcs_send.is_none());
    }

    // @spec shell/phase-pills Late-stage VCS pill: Archived includes VCS pill from tree dirty state
    #[test]
    fn archived_includes_vcs_pill_from_tree_dirty_state() {
        // GIVEN an archived change AND dirty working tree
        // WHEN the phase-pill display is derived
        let d = phase_display_for_archived(true);
        // THEN VCS pill uncommitted with Commit send
        assert_eq!(d.short, PhaseShort::Archived);
        assert_eq!(d.vcs, Some(VcsPill::Uncommitted));
        assert_eq!(d.vcs_send, Some("Commit"));
    }

    // @spec shell/phase-pills Exploration display: Empty exploration offers explore short with explore send
    #[test]
    fn empty_exploration_offers_explore_short_with_explore_send() {
        // GIVEN an exploration whose chat session is empty
        // WHEN the phase-pill display is derived
        let d = phase_display_for_exploration(true);
        // THEN explore short and /ds-explore send
        assert_eq!(d.short, PhaseShort::Explore);
        assert_eq!(d.lifecycle_send.as_deref(), Some("/ds-explore"));
    }

    // @spec shell/phase-pills Exploration display: Non-empty exploration is explore short without send
    #[test]
    fn non_empty_exploration_is_explore_short_without_send() {
        // GIVEN an exploration whose chat session is non-empty
        // WHEN the phase-pill display is derived
        let d = phase_display_for_exploration(false);
        // THEN explore short without lifecycle send
        assert_eq!(d.short, PhaseShort::Explore);
        assert!(d.lifecycle_send.is_none());
    }

    // @spec shell/phase-pills Activation: Lifecycle activation submits empty-send next-stage text
    #[test]
    fn lifecycle_activation_submits_empty_send_next_stage_text() {
        // GIVEN a phase-pill display with lifecycle send /ds-design
        let display = PhaseDisplay {
            short: PhaseShort::Proposal,
            lifecycle_hover: "proposal drafted, design not yet written".into(),
            lifecycle_send: Some("/ds-design".into()),
            vcs: None,
            vcs_hover: None,
            vcs_send: None,
        };
        // WHEN the lifecycle pill is activated
        let sent = phase_pill_activation_send(&display, PhasePillKind::Lifecycle);
        // THEN the submitted prompt text is /ds-design
        assert_eq!(sent.as_deref(), Some("/ds-design"));
    }

    // @spec shell/phase-pills Activation: Uncommitted activation submits Commit
    #[test]
    fn uncommitted_activation_submits_commit() {
        // GIVEN a phase-pill display with VCS send Commit
        let display = PhaseDisplay {
            short: PhaseShort::Ready,
            lifecycle_hover: "all steps complete".into(),
            lifecycle_send: Some("/ds-archive".into()),
            vcs: Some(VcsPill::Uncommitted),
            vcs_hover: Some(VCS_HOVER_DIRTY),
            vcs_send: Some("Commit"),
        };
        // WHEN the VCS pill is activated
        let sent = phase_pill_activation_send(&display, PhasePillKind::Vcs);
        // THEN the submitted prompt text is Commit
        assert_eq!(sent.as_deref(), Some("Commit"));
    }

    fn make_tab(id: &str) -> crate::widget::tab_bar::Tab {
        crate::widget::tab_bar::Tab {
            id: id.into(),
            title: id.rsplit('/').next().unwrap_or(id).into(),
            view: crate::widget::tab_bar::TabView::Editor {
                editor: crate::widget::text_edit::EditorState::new(""),
                path: None,
            },
        }
    }

    #[test]
    fn rewrite_rewrites_artifact_preview_and_file_tabs() {
        let mut tabs = tab_bar::TabState {
            preview: Some(make_tab("changes/foo/proposal.md")),
            file_tabs: vec![
                make_tab("changes/foo/caps/auth/spec.md"),
                make_tab("changes/bar/proposal.md"),
                make_tab("file:Cargo.toml"),
            ],
            active: Default::default(),
        };

        rewrite_tab_ids_for_archive(&mut tabs, "foo", "2026-04-20-01-foo");

        assert_eq!(
            tabs.preview.as_ref().map(|t| t.id.as_str()),
            Some("archive/2026-04-20-01-foo/proposal.md"),
        );
        assert_eq!(
            tabs.file_tabs[0].id,
            "archive/2026-04-20-01-foo/caps/auth/spec.md"
        );
        assert_eq!(tabs.file_tabs[1].id, "changes/bar/proposal.md");
        assert_eq!(tabs.file_tabs[2].id, "file:Cargo.toml");
    }

    #[test]
    fn rewrite_rewrites_vcs_tab_ids() {
        let mut tabs = tab_bar::TabState {
            preview: Some(make_tab("vcs:duckspec/changes/foo/proposal.md")),
            file_tabs: vec![],
            active: Default::default(),
        };

        rewrite_tab_ids_for_archive(&mut tabs, "foo", "2026-04-20-01-foo");

        assert_eq!(
            tabs.preview.as_ref().map(|t| t.id.as_str()),
            Some("vcs:duckspec/archive/2026-04-20-01-foo/proposal.md"),
        );
    }

    #[test]
    fn rewrite_leaves_similar_but_different_names_alone() {
        let mut tabs = tab_bar::TabState {
            preview: Some(make_tab("changes/foo2/proposal.md")),
            file_tabs: vec![],
            active: Default::default(),
        };

        rewrite_tab_ids_for_archive(&mut tabs, "foo", "2026-04-20-01-foo");

        assert_eq!(
            tabs.preview.as_ref().map(|t| t.id.as_str()),
            Some("changes/foo2/proposal.md"),
        );
    }

    #[test]
    fn lifecycle_all_steps_done_suggests_archive() {
        let state = make_state("foo", &[]);
        let mut project = make_project(&["foo"], &[]);
        set_change(&mut project, "foo", |c| {
            c.has_proposal = true;
            c.has_design = true;
            c.cap_tree = vec![tree_node("caps/auth")];
            c.steps = vec![step(true), step(true)];
        });
        assert_eq!(
            compute_lifecycle_command(&state, &project).as_deref(),
            Some("ds-archive")
        );
    }

    // @spec chat/fast-response Population: Ordinary refresh leaves options empty when oneshot is ineligible
    #[test]
    fn ordinary_refresh_leaves_options_empty_when_oneshot_is_ineligible() {
        use crate::area::interaction::{AgentSession, InteractionState};
        use crate::scope::{Scope, ScopeKind};
        use std::collections::HashMap;

        // GIVEN not awaiting and oneshot ineligible (hints off / empty list)
        let mut project = make_project(&["foo"], &[]);
        set_change(&mut project, "foo", |c| {
            c.has_proposal = true;
            c.has_design = true;
            c.cap_tree = vec![tree_node("caps/auth")];
            c.steps = vec![step(false)];
        });
        let mut interactions = HashMap::new();
        let scope = Scope::Change("foo".into());
        let mut ix = InteractionState::default();
        let mut ax = AgentSession::new("foo".into(), ScopeKind::Change);
        // Non-empty session without trailing next → empty next-actions; still
        // ineligible without settled oneshot + hints.
        ax.session.messages.push(crate::chat_store::ChatMessage {
            role: crate::chat_store::Role::User,
            content: vec![crate::chat_store::ContentBlock::Text("hi".into())],
            timestamp: String::new(),
            is_priming: false,
        });
        ax.agent_default_prompts = vec!["would show if eligible".into()];
        ix.sessions.push(ax);
        interactions.insert(scope, ix);

        refresh_fast_response(&mut interactions, &project, false, false);

        let ax = interactions
            .get(&Scope::Change("foo".into()))
            .and_then(|i| i.sessions.first())
            .expect("session");
        assert!(ax.fast_response.options.is_empty());
    }

    // @spec chat/fast-response Population: Refresh does not clear options while awaiting a user choice
    #[test]
    fn refresh_does_not_clear_options_while_awaiting_a_user_choice() {
        use crate::area::interaction::{AgentSession, InteractionState};
        use crate::fast_response::{self, FastResponseSource};
        use crate::scope::{Scope, ScopeKind};
        use std::collections::HashMap;

        let mut project = make_project(&["foo"], &[]);
        set_change(&mut project, "foo", |c| {
            c.has_proposal = true;
            c.has_design = true;
            c.cap_tree = vec![tree_node("caps/auth")];
            c.steps = vec![step(false)];
        });
        let mut interactions = HashMap::new();
        let scope = Scope::Change("foo".into());
        let mut ix = InteractionState::default();
        let mut ax = AgentSession::new("foo".into(), ScopeKind::Change);
        ax.is_awaiting_user = true;
        ax.fast_response =
            fast_response::from_user_choice(99, None, [("a".into(), "Alpha".into())]);
        ix.sessions.push(ax);
        interactions.insert(scope, ix);

        refresh_fast_response(&mut interactions, &project, true, false);

        let ax = interactions
            .get(&Scope::Change("foo".into()))
            .and_then(|i| i.sessions.first())
            .expect("session");
        assert!(ax.is_awaiting_user);
        assert_eq!(ax.fast_response.options.len(), 1);
        assert!(matches!(
            ax.fast_response.source,
            FastResponseSource::UserChoice {
                correlation_id: 99,
                ..
            }
        ));
    }

    // @spec chat/fast-response Population: Refresh preserves oneshot fill when still eligible
    #[test]
    fn refresh_preserves_oneshot_fill_when_still_eligible() {
        use crate::area::interaction::{AgentSession, InteractionState};
        use crate::fast_response::{self, FastResponseSource};
        use crate::scope::{Scope, ScopeKind};
        use std::collections::HashMap;

        let project = make_project(&["foo"], &[]);
        let mut interactions = HashMap::new();
        let scope = Scope::Change("foo".into());
        let mut ix = InteractionState::default();
        let mut ax = AgentSession::new("foo".into(), ScopeKind::Change);
        // Non-empty session, no trailing next → empty next-actions.
        ax.session.messages.push(crate::chat_store::ChatMessage {
            role: crate::chat_store::Role::Assistant,
            content: vec![crate::chat_store::ContentBlock::Text("done".into())],
            timestamp: String::new(),
            is_priming: false,
        });
        ax.agent_default_prompts = vec!["most likely".into(), "alt".into()];
        ax.fast_response = fast_response::from_oneshot_hints(ax.agent_default_prompts.clone());
        ix.sessions.push(ax);
        interactions.insert(scope, ix);

        refresh_fast_response(&mut interactions, &project, true, false);

        let ax = interactions
            .get(&Scope::Change("foo".into()))
            .and_then(|i| i.sessions.first())
            .expect("session");
        assert_eq!(ax.fast_response.options.len(), 2);
        assert_eq!(ax.fast_response.options[0].label, "most likely");
        assert_eq!(ax.fast_response.options[1].label, "alt");
        assert!(matches!(
            ax.fast_response.source,
            FastResponseSource::OneshotHints
        ));
    }

    // @spec chat/fast-response Population: Settled eligible oneshot fills the option shell
    #[test]
    fn settled_eligible_oneshot_fills_the_option_shell() {
        use crate::area::interaction::{self, AgentSession};
        use crate::fast_response::FastResponseSource;
        use crate::scope::ScopeKind;

        let mut ax = AgentSession::new("foo".into(), ScopeKind::Change);
        ax.session.messages.push(crate::chat_store::ChatMessage {
            role: crate::chat_store::Role::Assistant,
            content: vec![crate::chat_store::ContentBlock::Text("done".into())],
            timestamp: String::new(),
            is_priming: false,
        });
        ax.agent_default_prompts = vec!["yes".into(), "no".into(), "maybe".into()];
        ax.refresh_next_actions(true);
        assert!(ax.next_actions.is_empty());

        interaction::sync_oneshot_chips(&mut ax, true);

        assert_eq!(ax.fast_response.options.len(), 3);
        assert_eq!(ax.fast_response.options[0].id, "yes");
        assert!(matches!(
            ax.fast_response.source,
            FastResponseSource::OneshotHints
        ));
    }

    // @spec chat/fast-response Population: Live user choice overwrites oneshot fill
    #[test]
    fn live_user_choice_overwrites_oneshot_fill() {
        use crate::area::interaction::{self, AgentSession};
        use crate::fast_response::{self, FastResponseSource};
        use crate::scope::ScopeKind;

        let mut ax = AgentSession::new("foo".into(), ScopeKind::Change);
        ax.fast_response =
            fast_response::from_oneshot_hints(vec!["oneshot a".into(), "oneshot b".into()]);
        assert!(matches!(
            ax.fast_response.source,
            FastResponseSource::OneshotHints
        ));

        interaction::apply_user_choice_request(
            &mut ax,
            7,
            None,
            vec![("opt-a".into(), "Alpha".into())],
            false,
        );

        assert_eq!(ax.fast_response.options.len(), 1);
        assert_eq!(ax.fast_response.options[0].id, "opt-a");
        assert!(matches!(
            ax.fast_response.source,
            FastResponseSource::UserChoice {
                correlation_id: 7,
                ..
            }
        ));
        assert!(ax.is_awaiting_user);
    }

    // @spec chat/fast-response Population: Oneshot settle does not replace a live user-choice fill
    #[test]
    fn oneshot_settle_does_not_replace_a_live_user_choice_fill() {
        use crate::area::interaction::{self, AgentSession};
        use crate::fast_response::{self, FastResponseSource};
        use crate::scope::ScopeKind;

        let mut ax = AgentSession::new("foo".into(), ScopeKind::Change);
        ax.is_awaiting_user = true;
        ax.fast_response =
            fast_response::from_user_choice(42, None, [("q1".into(), "Option one".into())]);
        ax.agent_default_prompts = vec!["would fill if not awaiting".into()];
        ax.next_actions.clear();

        interaction::sync_oneshot_chips(&mut ax, true);

        assert_eq!(ax.fast_response.options.len(), 1);
        assert_eq!(ax.fast_response.options[0].id, "q1");
        assert!(matches!(
            ax.fast_response.source,
            FastResponseSource::UserChoice {
                correlation_id: 42,
                ..
            }
        ));
    }

    /// @spec session/scope Lifecycle reflection: A change with unfinished steps reports remaining work and the apply next-stage
    #[test]
    fn facts_unfinished_steps_report_remaining_work_and_apply() {
        let mut project = make_project(&["foo"], &[]);
        set_change(&mut project, "foo", |c| {
            c.has_proposal = true;
            c.has_design = true;
            c.cap_tree = vec![tree_node("caps/auth")];
            c.steps = vec![step(false), step(true)];
        });
        let facts = change_scope_facts("foo", &project).expect("active change has facts");
        assert!(
            facts.steps_done < facts.step_count,
            "progress should not be complete"
        );
        assert_eq!(facts.next_command.as_deref(), Some("ds-apply"));
    }

    // @spec session/scope Lifecycle reflection: A change with all steps complete reports completion and the archive next-stage
    #[test]
    fn facts_all_steps_complete_report_completion_and_archive() {
        // GIVEN all steps complete AND no reviews
        let mut project = make_project(&["foo"], &[]);
        set_change(&mut project, "foo", |c| {
            c.has_proposal = true;
            c.has_design = true;
            c.cap_tree = vec![tree_node("caps/auth")];
            c.steps = vec![step(true), step(true)];
        });
        let facts = change_scope_facts("foo", &project).expect("active change has facts");
        assert_eq!(facts.steps_done, facts.step_count);
        assert!(facts.step_count > 0, "completion is over real steps");
        assert_eq!(facts.next_command.as_deref(), Some("ds-archive"));
    }

    // @spec session/scope Lifecycle reflection: All steps complete with a review suggests the step next-stage
    #[test]
    fn facts_all_steps_complete_with_review_suggests_step() {
        let mut project = make_project(&["foo"], &[]);
        set_change(&mut project, "foo", |c| {
            c.has_proposal = true;
            c.has_design = true;
            c.cap_tree = vec![tree_node("caps/auth")];
            c.steps = vec![step(true), step(true)];
            c.reviews = vec!["01-look.md".into()];
        });
        let facts = change_scope_facts("foo", &project).expect("active change has facts");
        assert_eq!(facts.next_command.as_deref(), Some("ds-step"));
    }

    /// @spec session/scope Lifecycle reflection: A change with only a proposal reports the design next-stage
    #[test]
    fn facts_proposal_only_reports_design() {
        let mut project = make_project(&["foo"], &[]);
        set_change(&mut project, "foo", |c| c.has_proposal = true);
        let facts = change_scope_facts("foo", &project).expect("active change has facts");
        assert_eq!(facts.next_command.as_deref(), Some("ds-design"));
    }

    /// Produce the first-turn orientation text for a change scope, exercising
    /// the full data → facts → render path the session hook uses in production.
    fn orientation_for(name: &str, project: &ProjectData) -> String {
        use duckchat::ContextHook;
        let scope = crate::scope::SessionScope {
            kind: crate::scope::ScopeKind::Change,
            scope_key: name.to_string(),
            change_facts: change_scope_facts(name, project),
            has_inputs_ledger: false,
        };
        crate::scope::CurrentScopeHook
            .compute(&scope)
            .expect("change scope always produces orientation")
            .text
    }

    /// @spec session/scope Current review in orientation: Orientation reports the highest-numbered review as the current review
    #[test]
    fn orientation_reports_highest_numbered_review() {
        let mut project = make_project(&["foo"], &[]);
        set_change(&mut project, "foo", |c| {
            c.has_proposal = true;
            c.reviews = vec!["01-initial.md".into(), "02-post-implementation.md".into()];
        });
        let facts = change_scope_facts("foo", &project).expect("active change has facts");
        assert_eq!(
            facts.current_review.as_deref(),
            Some("02-post-implementation.md")
        );

        let text = orientation_for("foo", &project);
        assert!(
            text.contains("duckspec/changes/foo/reviews/02-post-implementation.md"),
            "orientation must report the highest-numbered review at the full path: {text}"
        );
        assert!(
            !text.contains("reviews/01-initial.md"),
            "orientation must not report a lower-numbered review as current: {text}"
        );
    }

    // @spec session/scope Current review in orientation: A change with no reviews reports no current review
    #[test]
    fn orientation_with_no_reviews_reports_none() {
        let mut project = make_project(&["foo"], &[]);
        set_change(&mut project, "foo", |c| c.has_proposal = true);
        let facts = change_scope_facts("foo", &project).expect("active change has facts");
        assert_eq!(facts.current_review, None);

        let text = orientation_for("foo", &project);
        assert!(
            !text.contains("Current review:"),
            "orientation must not report a current review when none exist: {text}"
        );
    }

    // @spec session/scope Current review in orientation: Adding a review does not change reported step progress
    #[test]
    fn adding_a_review_does_not_change_reported_step_progress() {
        // Two changes with identical step completion; only `bar` has reviews.
        let mut project = make_project(&["foo", "bar"], &[]);
        for name in ["foo", "bar"] {
            set_change(&mut project, name, |c| {
                c.has_proposal = true;
                c.has_design = true;
                c.cap_tree = vec![tree_node("caps/auth")];
                c.steps = vec![step(true), step(false)];
            });
        }
        set_change(&mut project, "bar", |c| {
            c.reviews = vec!["01-a-look.md".into()];
        });

        let foo = change_scope_facts("foo", &project).expect("facts");
        let bar = change_scope_facts("bar", &project).expect("facts");
        assert_eq!(foo.steps_done, bar.steps_done);
        assert_eq!(foo.step_count, bar.step_count);
        assert_eq!(foo.steps_done, 1);
        assert_eq!(foo.step_count, 2);
    }

    // ── exploration archive / live lists ────────────────────────────────

    fn live_list_ids(exps: &[Exploration]) -> Vec<&str> {
        exps.iter()
            .filter(|e| e.is_on_live_list())
            .map(|e| e.id.as_str())
            .collect()
    }

    /// @spec exploration/archive Live list membership: Archived non–idea-owned exploration is absent from live lists
    #[test]
    fn archived_non_idea_owned_absent_from_live_lists() {
        let mut exp = Exploration::new(1);
        exp.mark_archived();
        assert!(!exp.is_on_live_list());
        assert!(live_list_ids(std::slice::from_ref(&exp)).is_empty());
    }

    /// @spec exploration/archive Live list membership: Live non–idea-owned exploration remains on live lists
    #[test]
    fn live_non_idea_owned_remains_on_live_lists() {
        let exp = Exploration::new(1);
        assert!(exp.is_on_live_list());
        assert_eq!(
            live_list_ids(std::slice::from_ref(&exp)),
            vec![exp.id.as_str()]
        );
    }

    /// @spec exploration/archive Hover control by state: Live exploration hover control archives
    #[test]
    fn live_exploration_hover_control_archives() {
        use crate::test_support::{FsTmp, with_home};

        let tmp = FsTmp::new();
        with_home(tmp.path(), || {
            let root = tmp.path().join("project");
            std::fs::create_dir_all(&root).unwrap();

            let exp = Exploration::new(1);
            let exp_id = exp.id.clone();
            let mut session = crate::chat_store::ChatSession::new(exp_id.clone());
            session.id = "1".into();
            crate::chat_store::save_session(&session, Some(&root)).unwrap();

            let mut state = make_state(&exp_id, &[]);
            state.explorations = vec![exp];
            let mut project = make_project(&[], &[]);
            project.project_root = Some(root.clone());
            let mut tabs = tab_bar::TabState::default();
            let mut interactions = HashMap::new();
            let hl = crate::highlight::SyntaxHighlighter::new();

            assert_eq!(
                exploration_hover_action(&state.explorations[0], false),
                ExplorationHoverAction::Archive(exp_id.clone())
            );

            update(
                &mut state,
                &mut tabs,
                &mut interactions,
                Message::ArchiveExploration(exp_id.clone()),
                &project,
                &hl,
                false,
                1200.0,
                crate::config::VcsWorkflow::default(),
            crate::config::ViewerStyle::Classic,
            );

            let exp = state.explorations.iter().find(|e| e.id == exp_id).unwrap();
            assert!(exp.is_archived());
            assert!(!exp.is_on_live_list());
            assert_eq!(crate::chat_store::count_sessions(&exp_id, Some(&root)), 1);
        });
    }

    /// @spec exploration/archive Hover control by state: Archived exploration hover control removes
    #[test]
    fn archived_exploration_hover_control_removes() {
        use crate::test_support::{FsTmp, with_home};

        let tmp = FsTmp::new();
        with_home(tmp.path(), || {
            let root = tmp.path().join("project");
            std::fs::create_dir_all(&root).unwrap();

            let mut exp = Exploration::new(1);
            let exp_id = exp.id.clone();
            exp.mark_archived();
            exp.session_count = 1;

            let mut session = crate::chat_store::ChatSession::new(exp_id.clone());
            session.id = "1".into();
            crate::chat_store::save_session(&session, Some(&root)).unwrap();

            assert!(matches!(
                exploration_hover_action(&exp, true),
                ExplorationHoverAction::Remove { armed: true, .. }
            ));

            let mut state = make_state(&exp_id, &[]);
            state.explorations = vec![exp];
            let mut project = make_project(&[], &[]);
            project.project_root = Some(root.clone());
            let mut tabs = tab_bar::TabState::default();
            let mut interactions = HashMap::new();
            let hl = crate::highlight::SyntaxHighlighter::new();

            update(
                &mut state,
                &mut tabs,
                &mut interactions,
                Message::RemoveExploration(exp_id.clone()),
                &project,
                &hl,
                false,
                1200.0,
                crate::config::VcsWorkflow::default(),
            crate::config::ViewerStyle::Classic,
            );

            assert!(state.explorations.iter().all(|e| e.id != exp_id));
            assert_eq!(crate::chat_store::count_sessions(&exp_id, Some(&root)), 0);
        });
    }

    /// @spec exploration/archive Hover control by state: Remove with sessions requires arm then commit
    #[test]
    fn remove_with_sessions_requires_arm_then_commit() {
        let mut exp = Exploration::new(1);
        exp.mark_archived();
        exp.session_count = 2;
        let id = exp.id.clone();

        // First activation only arms (control routes to ArmRemove).
        assert_eq!(
            exploration_hover_action(&exp, false),
            ExplorationHoverAction::ArmRemove(id.clone())
        );

        let mut state = make_state(&id, &[]);
        state.explorations = vec![exp.clone()];
        let project = make_project(&[], &[]);
        let mut tabs = tab_bar::TabState::default();
        let mut interactions = HashMap::new();
        let hl = crate::highlight::SyntaxHighlighter::new();

        update(
            &mut state,
            &mut tabs,
            &mut interactions,
            Message::ArmRemoveExploration(id.clone()),
            &project,
            &hl,
            false,
            1200.0,
            crate::config::VcsWorkflow::default(),
            crate::config::ViewerStyle::Classic,
        );
        assert_eq!(state.armed_remove_exploration.as_deref(), Some(id.as_str()));
        assert!(state.explorations.iter().any(|e| e.id == id));

        // Armed second activation routes to Remove.
        assert_eq!(
            exploration_hover_action(&exp, true),
            ExplorationHoverAction::Remove {
                id: id.clone(),
                armed: true,
            }
        );

        update(
            &mut state,
            &mut tabs,
            &mut interactions,
            Message::RemoveExploration(id.clone()),
            &project,
            &hl,
            false,
            1200.0,
            crate::config::VcsWorkflow::default(),
            crate::config::ViewerStyle::Classic,
        );
        assert!(state.explorations.iter().all(|e| e.id != id));
    }

    /// @spec exploration/archive Hover control by state: Remove with no sessions commits without arm
    #[test]
    fn remove_with_no_sessions_commits_without_arm() {
        let mut exp = Exploration::new(1);
        exp.mark_archived();
        exp.session_count = 0;
        let id = exp.id.clone();

        assert!(matches!(
            exploration_hover_action(&exp, false),
            ExplorationHoverAction::Remove { armed: false, .. }
        ));

        let mut state = make_state(&id, &[]);
        state.explorations = vec![exp];
        let project = make_project(&[], &[]);
        let mut tabs = tab_bar::TabState::default();
        let mut interactions = HashMap::new();
        let hl = crate::highlight::SyntaxHighlighter::new();

        update(
            &mut state,
            &mut tabs,
            &mut interactions,
            Message::RemoveExploration(id.clone()),
            &project,
            &hl,
            false,
            1200.0,
            crate::config::VcsWorkflow::default(),
            crate::config::ViewerStyle::Classic,
        );
        assert!(state.explorations.iter().all(|e| e.id != id));
    }

    // ── archive pending commit (predicate) ──────────────────────────────

    fn dirty_path(rel: &str) -> ChangedFile {
        ChangedFile {
            path: PathBuf::from(rel),
            status: FileStatus::Modified,
        }
    }

    /// @spec archive/pending-commit Pending predicate: Dirty path under the archive folder is pending
    #[test]
    fn pending_when_dirty_under_archive_folder() {
        let id = "2026-07-11-08-foo";
        let dirty = vec![dirty_path(&format!(
            "duckspec/archive/{id}/proposal.md"
        ))];
        assert!(archive_package_dirty(id, &dirty));
        let ch = ChangeData {
            name: id.into(),
            prefix: format!("archive/{id}"),
            has_proposal: true,
            has_design: false,
            cap_tree: vec![],
            steps: vec![],
            reviews: vec![],
            shallow_mtime_nanos: None,
        };
        assert!(is_pending_commit_archive(&ch, &dirty));
    }

    /// @spec archive/pending-commit Pending predicate: Dirt only outside the archive folder is not pending
    #[test]
    fn not_pending_when_dirt_only_outside_archive_folder() {
        let id = "2026-07-11-08-foo";
        let dirty = vec![
            dirty_path("duckspec/changes/other/proposal.md"),
            dirty_path("crates/duckboard/src/main.rs"),
            dirty_path("duckspec/archive/2026-07-10-01-other/proposal.md"),
        ];
        assert!(!archive_package_dirty(id, &dirty));
        let ch = ChangeData {
            name: id.into(),
            prefix: format!("archive/{id}"),
            has_proposal: true,
            has_design: false,
            cap_tree: vec![],
            steps: vec![],
            reviews: vec![],
            shallow_mtime_nanos: None,
        };
        assert!(!is_pending_commit_archive(&ch, &dirty));
    }

    // ── archive browse ──────────────────────────────────────────────────

    fn entry_ids(entries: &[ArchivedEntry<'_>]) -> Vec<String> {
        entries
            .iter()
            .map(|e| match e {
                ArchivedEntry::Change(c) => c.name.clone(),
                ArchivedEntry::Exploration(x) => x.id.clone(),
            })
            .collect()
    }

    /// @spec archive/browse Interleaved archived rows: Archived non–idea-owned explorations appear with archived changes
    #[test]
    fn archived_explorations_appear_with_archived_changes() {
        let project = make_project(&[], &["2026-07-01-01-done"]);
        let mut exp = Exploration::new(1);
        exp.mark_archived();
        let dirty: &[ChangedFile] = &[];
        let entries =
            archived_entries(&project.archived_changes, std::slice::from_ref(&exp), dirty);
        let ids = entry_ids(&entries);
        assert!(ids.iter().any(|id| id == "2026-07-01-01-done"));
        assert!(ids.iter().any(|id| id == &exp.id));
    }

    /// @spec archive/browse Interleaved archived rows: Mixed archive rows order by archive date descending
    #[test]
    fn mixed_archive_rows_order_by_date_descending() {
        let project = make_project(&[], &["2026-01-01-01-old", "2026-07-12-09-new"]);
        let mut older_exp = Exploration::new(1);
        older_exp.archived_at = Some("2026-03-15T10:00:00+00:00".into());
        let mut newer_exp = Exploration::new(2);
        newer_exp.archived_at = Some("2026-07-12T18:00:00+00:00".into());
        let exps = vec![older_exp.clone(), newer_exp.clone()];
        let dirty: &[ChangedFile] = &[];
        let entries = archived_entries(&project.archived_changes, &exps, dirty);
        let ids = entry_ids(&entries);
        // newest first: late-day exploration, then 09 change, then March exp, then Jan change
        assert_eq!(
            ids,
            vec![
                newer_exp.id,
                "2026-07-12-09-new".to_string(),
                older_exp.id,
                "2026-01-01-01-old".to_string(),
            ]
        );
    }

    /// @spec archive/browse Interleaved archived rows: Idea-owned archived explorations stay off Change and Dashboard archived lists
    #[test]
    fn idea_owned_archived_explorations_stay_off_archived_lists() {
        let project = make_project(&[], &["2026-07-01-01-done"]);
        let mut exp = Exploration::new(1);
        exp.idea_path = Some("/ideas/x.md".into());
        exp.mark_archived();
        let dirty: &[ChangedFile] = &[];
        let entries =
            archived_entries(&project.archived_changes, std::slice::from_ref(&exp), dirty);
        assert!(
            !entries
                .iter()
                .any(|e| matches!(e, ArchivedEntry::Exploration(_)))
        );
        assert_eq!(entries.len(), 1);
    }

    /// @spec archive/browse Interleaved archived rows: Pending archived package is omitted from Archived lists
    #[test]
    fn pending_archived_package_omitted_from_archived_lists() {
        let project = make_project(&[], &["2026-07-11-08-pending"]);
        let dirty = vec![dirty_path(
            "duckspec/archive/2026-07-11-08-pending/proposal.md",
        )];
        let entries = archived_entries(&project.archived_changes, &[], &dirty);
        assert!(entries.is_empty());
        assert!(!has_archived_section(
            &project.archived_changes,
            &[],
            &dirty
        ));
    }

    /// @spec archive/browse Archived section visibility: Archived section is empty only when both kinds are empty
    #[test]
    fn archived_section_present_with_only_exploration() {
        let project = make_project(&[], &[]);
        let mut exp = Exploration::new(1);
        exp.mark_archived();
        let dirty: &[ChangedFile] = &[];
        assert!(has_archived_section(
            &project.archived_changes,
            std::slice::from_ref(&exp),
            dirty,
        ));
        let entries =
            archived_entries(&project.archived_changes, std::slice::from_ref(&exp), dirty);
        assert_eq!(entries.len(), 1);
        assert!(matches!(entries[0], ArchivedEntry::Exploration(_)));
    }

    /// @spec archive/browse Archived section visibility: Change Archived section starts collapsed
    #[test]
    fn change_archived_section_starts_collapsed() {
        let state = State::new(None);
        assert!(!state.expanded_sections.contains("archived"));
        // Section still has rows to show when archives exist; collapsed by default.
        let project = make_project(&[], &["2026-07-01-01-done"]);
        assert!(has_archived_section(
            &project.archived_changes,
            &[],
            &[]
        ));
    }

    #[test]
    fn section_counts_match_list_membership() {
        let project = make_project(&["active-a", "active-b"], &["2026-07-01-01-done"]);
        let live = Exploration::new(1);
        let mut archived_exp = Exploration::new(2);
        archived_exp.mark_archived();
        let mut idea_owned = Exploration::new(3);
        idea_owned.idea_path = Some("/ideas/x.md".into());
        let mut idea_archived = Exploration::new(4);
        idea_archived.idea_path = Some("/ideas/y.md".into());
        idea_archived.mark_archived();
        let exps = vec![live, archived_exp, idea_owned, idea_archived];
        let dirty: &[ChangedFile] = &[];

        // Change: live free + idea-owned (not archived) explorations + two active changes.
        assert_eq!(
            change_section_count(&exps, &project.active_changes, 0),
            2 + 2
        );
        // Archived: one change + one non–idea-owned archived exploration.
        assert_eq!(
            archived_section_count(&project.archived_changes, &exps, dirty),
            2
        );
        // Only-exploration archive still counts.
        assert_eq!(
            archived_section_count(&[], std::slice::from_ref(&exps[1]), dirty),
            1
        );
    }

    /// Pure Change-section membership for pending archives (order + after actives).
    fn change_queue_ids(
        active: &[ChangeData],
        archived: &[ChangeData],
        dirty: &[ChangedFile],
    ) -> Vec<String> {
        let mut ids: Vec<String> = active.iter().map(|c| c.name.clone()).collect();
        for ch in pending_archives(archived, dirty) {
            ids.push(ch.name.clone());
        }
        ids
    }

    /// @spec archive/pending-commit Change list placement: Pending package appears in the Change section
    #[test]
    fn pending_package_appears_in_change_section_after_actives() {
        let project = make_project(
            &["active-wip"],
            &["2026-07-11-08-pending"],
        );
        let dirty = vec![dirty_path(
            "duckspec/archive/2026-07-11-08-pending/proposal.md",
        )];
        let ids = change_queue_ids(
            &project.active_changes,
            &project.archived_changes,
            &dirty,
        );
        assert_eq!(
            ids,
            vec![
                "active-wip".to_string(),
                "2026-07-11-08-pending".to_string(),
            ]
        );
        // Not on finished Archived list.
        let archived =
            archived_entries(&project.archived_changes, &[], &dirty);
        assert!(archived.is_empty());
    }

    /// @spec archive/pending-commit Change list placement: Multiple pending archives order newest-first after actives
    #[test]
    fn multiple_pending_archives_newest_first_after_actives() {
        let project = make_project(
            &["active-wip"],
            &["2026-01-01-01-old", "2026-07-12-09-new"],
        );
        let dirty = vec![
            dirty_path("duckspec/archive/2026-01-01-01-old/proposal.md"),
            dirty_path("duckspec/archive/2026-07-12-09-new/design.md"),
        ];
        let ids = change_queue_ids(
            &project.active_changes,
            &project.archived_changes,
            &dirty,
        );
        assert_eq!(
            ids,
            vec![
                "active-wip".to_string(),
                "2026-07-12-09-new".to_string(),
                "2026-01-01-01-old".to_string(),
            ]
        );
    }

    /// @spec archive/pending-commit Uncommitted chrome and Commit send: Pending row shows uncommitted chrome
    #[test]
    fn pending_row_shows_uncommitted_chrome() {
        let d = phase_display_for_pending_archive();
        assert_eq!(d.short, PhaseShort::Archived);
        assert_eq!(d.vcs, Some(VcsPill::Uncommitted));
        assert_eq!(d.vcs_send, Some("Commit"));
        assert_eq!(d.vcs_hover, Some(VCS_HOVER_ARCHIVE_PACKAGE));
        assert_eq!(PENDING_UNCOMMITTED_LABEL, "uncommitted");
    }

    /// @spec archive/pending-commit Uncommitted chrome and Commit send: Activating chrome sends Commit without committing
    #[test]
    fn activating_pending_chrome_sends_commit_without_vcs() {
        let msg = pending_uncommitted_activate("2026-07-11-08-foo");
        match msg {
            Message::PhasePillSend { target, text } => {
                assert_eq!(target, "2026-07-11-08-foo");
                assert_eq!(text, "Commit");
            }
            other => panic!("expected PhasePillSend, got {other:?}"),
        }
    }

    // ── pending list presentation identity ──────────────────────────────

    #[test]
    fn pending_archive_breadcrumb_uses_changes_root() {
        let id = "2026-07-11-08-pending";
        let mut state = make_state(id, &[]);
        state.changed_files = vec![dirty_path(&format!(
            "duckspec/archive/{id}/proposal.md"
        ))];
        let project = make_project(&[], &[id]);
        let tabs = tab_bar::TabState::default();
        assert_eq!(
            breadcrumbs(&state, &project, &tabs),
            vec!["Changes", id]
        );
    }

    #[test]
    fn pending_archive_tab_breadcrumb_uses_changes_root() {
        // Tab ids stay archive/…; list presentation root is still Changes.
        assert_eq!(
            tab_breadcrumbs(
                "archive/2026-07-11-08-pending/proposal.md",
                "2026-07-11-08-pending",
                false
            ),
            vec!["Changes", "2026-07-11-08-pending", "Proposal"]
        );
    }

    #[test]
    fn finished_archive_breadcrumb_uses_archive_root() {
        let id = "2026-04-20-01-foo";
        let state = make_state(id, &[]);
        let project = make_project(&[], &[id]);
        let tabs = tab_bar::TabState::default();
        assert_eq!(
            breadcrumbs(&state, &project, &tabs),
            vec!["Archive", id]
        );
    }

    #[test]
    fn select_pending_archive_expands_picker_not_archived() {
        let id = "2026-07-11-08-pending";
        let mut state = State::new(None);
        state.expanded_sections.clear();
        state.changed_files = vec![dirty_path(&format!(
            "duckspec/archive/{id}/proposal.md"
        ))];
        let project = make_project(&[], &[id]);
        let mut tabs = tab_bar::TabState::default();
        let mut interactions = HashMap::new();
        let highlighter = crate::highlight::SyntaxHighlighter::new();
        update(
            &mut state,
            &mut tabs,
            &mut interactions,
            Message::SelectChange(id.into()),
            &project,
            &highlighter,
            false,
            1200.0,
            crate::config::VcsWorkflow::default(),
            crate::config::ViewerStyle::Classic,
        );
        assert_eq!(state.selected_change.as_deref(), Some(id));
        assert!(state.expanded_sections.contains("picker"));
        assert!(!state.expanded_sections.contains("archived"));
    }

    #[test]
    fn select_finished_archive_expands_archived() {
        let id = "2026-04-20-01-foo";
        let mut state = State::new(None);
        state.expanded_sections.clear();
        let project = make_project(&[], &[id]);
        let mut tabs = tab_bar::TabState::default();
        let mut interactions = HashMap::new();
        let highlighter = crate::highlight::SyntaxHighlighter::new();
        update(
            &mut state,
            &mut tabs,
            &mut interactions,
            Message::SelectChange(id.into()),
            &project,
            &highlighter,
            false,
            1200.0,
            crate::config::VcsWorkflow::default(),
            crate::config::ViewerStyle::Classic,
        );
        assert_eq!(state.selected_change.as_deref(), Some(id));
        assert!(state.expanded_sections.contains("archived"));
        assert!(!state.expanded_sections.contains("picker"));
    }
}

#[cfg(test)]
mod explorer_tests {
    use super::*;

    fn paths(list: &[&str]) -> Vec<PathBuf> {
        list.iter().map(PathBuf::from).collect()
    }

    fn make_state() -> State {
        State::new(None)
    }

    #[test]
    fn tree_puts_sorted_dirs_before_sorted_files() {
        let mut dirs = HashSet::new();
        let tree = build_explorer_tree(
            &paths(&["zeta.rs", "src/b.rs", "src/a.rs", "Cargo.toml"]),
            &mut dirs,
        );
        let labels: Vec<&str> = tree.iter().map(|n| n.label.as_str()).collect();
        assert_eq!(labels, vec!["src", "Cargo.toml", "zeta.rs"]);
        let src_children: Vec<&str> = tree[0].children.iter().map(|n| n.id.as_str()).collect();
        assert_eq!(src_children, vec!["file:src/a.rs", "file:src/b.rs"]);
    }

    #[test]
    fn tree_ids_use_file_prefix_for_leaves_and_rel_paths_for_dirs() {
        let mut dirs = HashSet::new();
        let tree = build_explorer_tree(&paths(&["a/b/c.rs"]), &mut dirs);
        assert_eq!(tree[0].id, "a");
        assert_eq!(tree[0].children[0].id, "a/b");
        assert_eq!(tree[0].children[0].children[0].id, "file:a/b/c.rs");
        assert!(dirs.contains("a"));
        assert!(dirs.contains("a/b"));
    }

    #[test]
    fn set_project_files_prunes_stale_expanded_dirs() {
        let mut state = make_state();
        state.set_project_files(&paths(&["src/a.rs", "docs/b.md"]));
        state.expanded_explorer_dirs.insert("src".into());
        state.expanded_explorer_dirs.insert("docs".into());
        state.set_project_files(&paths(&["src/a.rs"]));
        assert!(state.expanded_explorer_dirs.contains("src"));
        assert!(!state.expanded_explorer_dirs.contains("docs"));
    }

    #[test]
    fn expand_ancestors_inserts_each_prefix() {
        let mut state = make_state();
        state.expand_explorer_ancestors("crates/duckboard/src/main.rs");
        assert!(state.expanded_explorer_dirs.contains("crates"));
        assert!(state.expanded_explorer_dirs.contains("crates/duckboard"));
        assert!(
            state
                .expanded_explorer_dirs
                .contains("crates/duckboard/src")
        );
        assert_eq!(state.expanded_explorer_dirs.len(), 3);
    }

    #[test]
    fn expand_ancestors_of_root_file_is_noop() {
        let mut state = make_state();
        state.expand_explorer_ancestors("Cargo.toml");
        assert!(state.expanded_explorer_dirs.is_empty());
    }

    #[test]
    fn vcs_tints_color_files_and_aggregate_dirs() {
        let cf = |path: &str, status: FileStatus| ChangedFile {
            path: PathBuf::from(path),
            status,
        };
        let tints = explorer_vcs_tints(&[
            cf("src/a.rs", FileStatus::Modified),
            cf("src/new/b.rs", FileStatus::Added),
            cf("docs/gone.md", FileStatus::Deleted),
        ]);

        let modified = theme::vcs_status_color(&FileStatus::Modified);
        let added = theme::vcs_status_color(&FileStatus::Added);

        assert_eq!(tints.get("file:src/a.rs"), Some(&modified));
        assert_eq!(tints.get("file:src/new/b.rs"), Some(&added));
        // Deleted files have no explorer row, but their dir is tinted with
        // the deletion color so the change is still discoverable.
        assert!(!tints.contains_key("file:docs/gone.md"));
        assert_eq!(
            tints.get("docs"),
            Some(&theme::vcs_status_color(&FileStatus::Deleted))
        );
        // Uniform dir takes its status color; mixed falls back to modified.
        assert_eq!(tints.get("src/new"), Some(&added));
        assert_eq!(tints.get("src"), Some(&modified));
    }

    #[test]
    fn flat_position_counts_only_visible_rows() {
        let mut state = make_state();
        state.set_project_files(&paths(&["src/a.rs", "src/b.rs", "Cargo.toml"]));
        // Collapsed: rows are [src, Cargo.toml].
        assert_eq!(
            state.explorer_flat_position("file:Cargo.toml"),
            Some((1, 2))
        );
        assert_eq!(state.explorer_flat_position("file:src/a.rs"), None);
        // Expanded: rows are [src, src/a.rs, src/b.rs, Cargo.toml].
        state.expanded_explorer_dirs.insert("src".into());
        assert_eq!(state.explorer_flat_position("file:src/a.rs"), Some((1, 4)));
        assert_eq!(
            state.explorer_flat_position("file:Cargo.toml"),
            Some((3, 4))
        );
    }
}

#[cfg(test)]
mod file_tree_tests {
    use super::*;

    fn cf(path: &str, status: FileStatus) -> ChangedFile {
        ChangedFile {
            path: PathBuf::from(path),
            status,
        }
    }

    #[test]
    fn root_file_lands_at_depth_zero() {
        let mut t = FileTree::new(PathBuf::new());
        t.insert(cf("main.rs", FileStatus::Modified));
        assert!(t.dirs.is_empty());
        assert_eq!(t.files.len(), 1);
    }

    #[test]
    fn nested_paths_create_directories() {
        let mut t = FileTree::new(PathBuf::new());
        t.insert(cf(".claude/foo.md", FileStatus::Added));
        t.insert(cf(".claude/bar/baz.md", FileStatus::Added));
        t.insert(cf("agents/x.md", FileStatus::Added));

        assert_eq!(t.dirs.len(), 2);
        let claude = t.dirs.get(".claude").expect("dir");
        assert_eq!(claude.files.len(), 1);
        assert_eq!(claude.dirs.len(), 1);
        assert_eq!(claude.path, PathBuf::from(".claude"));
        let bar = claude.dirs.get("bar").expect("subdir");
        assert_eq!(bar.path, PathBuf::from(".claude/bar"));
    }

    #[test]
    fn aggregate_status_uniform() {
        let mut t = FileTree::new(PathBuf::new());
        t.insert(cf(".claude/a.md", FileStatus::Added));
        t.insert(cf(".claude/b/c.md", FileStatus::Added));
        let claude = t.dirs.get(".claude").unwrap();
        assert_eq!(aggregate_status(claude), Some(FileStatus::Added));
    }

    #[test]
    fn aggregate_status_mixed_returns_none() {
        let mut t = FileTree::new(PathBuf::new());
        t.insert(cf(".claude/a.md", FileStatus::Added));
        t.insert(cf(".claude/b.md", FileStatus::Modified));
        let claude = t.dirs.get(".claude").unwrap();
        assert_eq!(aggregate_status(claude), None);
    }

    #[test]
    fn flatten_collapsed_hides_children() {
        let mut t = FileTree::new(PathBuf::new());
        t.insert(cf(".claude/a.md", FileStatus::Added));
        t.insert(cf(".claude/b.md", FileStatus::Added));
        t.insert(cf("main.rs", FileStatus::Modified));

        let expanded = HashSet::new();
        let mut rows = Vec::new();
        flatten_file_tree(&t, 0, &expanded, &mut rows);
        assert_eq!(rows.len(), 2);
        assert!(matches!(rows[0], ChangedFileRow::Dir { .. }));
        assert!(matches!(rows[1], ChangedFileRow::File { .. }));
    }

    #[test]
    fn flatten_expanded_reveals_children() {
        let mut t = FileTree::new(PathBuf::new());
        t.insert(cf(".claude/a.md", FileStatus::Added));
        t.insert(cf(".claude/b.md", FileStatus::Added));

        let mut expanded = HashSet::new();
        expanded.insert(".claude".to_string());
        let mut rows = Vec::new();
        flatten_file_tree(&t, 0, &expanded, &mut rows);
        assert_eq!(rows.len(), 3);
        match &rows[1] {
            ChangedFileRow::File { depth, .. } => assert_eq!(*depth, 1),
            _ => panic!("expected file row"),
        }
    }

    #[test]
    fn set_changed_files_rebuilds_row_cache() {
        let mut state = State::new(None);
        state.set_changed_files(vec![
            cf("src/a.rs", FileStatus::Modified),
            cf("src/b.rs", FileStatus::Modified),
        ]);
        // Auto-expand "src" → dir + two files.
        assert_eq!(state.changed_file_rows.len(), 3);
        assert!(matches!(
            &state.changed_file_rows[0],
            ChangedFileRow::Dir {
                key,
                is_expanded: true,
                ..
            } if key == "src"
        ));
        assert!(matches!(
            &state.changed_file_rows[1],
            ChangedFileRow::File { name, .. } if name == "a.rs"
        ));
        assert!(matches!(
            &state.changed_file_rows[2],
            ChangedFileRow::File { name, .. } if name == "b.rs"
        ));
    }

    #[test]
    fn toggle_file_dir_rebuilds_row_cache() {
        let mut state = State::new(None);
        state.set_changed_files(vec![
            cf("src/a.rs", FileStatus::Modified),
            cf("src/b.rs", FileStatus::Modified),
        ]);
        assert_eq!(state.changed_file_rows.len(), 3);

        // Collapse "src" via the same expand-set mutation path as ToggleFileDir.
        assert!(state.expanded_file_dirs.remove("src"));
        state.refresh_changed_file_rows();
        assert_eq!(state.changed_file_rows.len(), 1);
        assert!(matches!(
            &state.changed_file_rows[0],
            ChangedFileRow::Dir {
                key,
                is_expanded: false,
                ..
            } if key == "src"
        ));

        // Expand again.
        state.expanded_file_dirs.insert("src".into());
        state.refresh_changed_file_rows();
        assert_eq!(state.changed_file_rows.len(), 3);
    }

    #[test]
    fn rebuild_changed_file_rows_empty_when_no_files() {
        assert!(rebuild_changed_file_rows(&[], &HashSet::new()).is_empty());
    }
}
