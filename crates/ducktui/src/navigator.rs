//! Scope navigator tree — left work-screen pane.
//!
//! Built only from existing project state (changes, explorations, ideas, session
//! counts). No separate navigator store. No fixed Ideas session scope.

use std::path::{Path, PathBuf};

use duckcore::chat_store;
use duckcore::paths;
use duckcore::scope::Scope;

/// Chat binding produced by a navigator selection (no content browser).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundChat {
    Change(String),
    Exploration(String),
    Codex,
}

impl BoundChat {
    pub fn label(&self) -> String {
        match self {
            Self::Change(name) => name.clone(),
            Self::Exploration(id) => id.clone(),
            Self::Codex => "codex".into(),
        }
    }

    /// Shared `Scope` for the bound chat.
    pub fn scope(&self) -> Scope {
        match self {
            Self::Change(name) => Scope::Change(name.clone()),
            Self::Exploration(id) => Scope::Exploration(id.clone()),
            Self::Codex => Scope::Codex,
        }
    }

    pub fn chat_key(&self) -> String {
        match self {
            Self::Change(name) => name.clone(),
            Self::Exploration(id) => id.clone(),
            Self::Codex => "codex".into(),
        }
    }
}

/// One idea row (duckboard-aligned frontmatter links).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdeaEntry {
    /// Stable identity for selection (absolute path string).
    pub id: String,
    pub title: String,
    pub change: Option<String>,
    pub exploration: Option<String>,
}

impl IdeaEntry {
    /// Same rule as duckboard `idea_scope`: change wins, else exploration, else none.
    pub fn scope(&self) -> Option<Scope> {
        if let Some(name) = self.change.as_deref() {
            Some(Scope::Change(name.to_string()))
        } else {
            self.exploration
                .as_deref()
                .map(|id| Scope::Exploration(id.to_string()))
        }
    }
}

/// Identity of a selectable navigator row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavId {
    Change(String),
    Exploration(String),
    ArchivedChange(String),
    ArchivedExploration(String),
    Ideas,
    /// Idea file identity (`IdeaEntry.id`).
    Idea(String),
    Codex,
    Settings,
    /// Header row — toggles expand/collapse.
    ArchivedSection,
}

/// One visible (or logical) row in the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavRow {
    pub id: NavId,
    pub label: String,
    /// Phase glyph for active changes (and archived when shown).
    pub phase: Option<String>,
    /// Session count badge; `None` when zero sessions.
    pub session_badge: Option<usize>,
    /// Depth / kind for rendering.
    pub kind: NavRowKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavRowKind {
    Section,
    Change,
    Exploration,
    Bottom,
    Idea,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeEntry {
    pub name: String,
    pub phase_indicator: String,
    pub session_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplorationEntry {
    pub id: String,
    pub display_name: String,
    pub session_count: usize,
    pub archived: bool,
}

/// Injected / scanned project state for the tree (no navigator-owned disk).
#[derive(Debug, Clone, Default)]
pub struct ProjectSnapshot {
    pub active_changes: Vec<ChangeEntry>,
    pub archived_changes: Vec<ChangeEntry>,
    pub explorations: Vec<ExplorationEntry>,
    pub ideas: Vec<IdeaEntry>,
}

impl ProjectSnapshot {
    /// Load from an existing project root via duckpond layout + duckcore stores.
    pub fn from_project(project_root: &Path) -> Self {
        let duckspec = project_root.join("duckspec");
        let changes_dir = duckspec.join("changes");
        let archive_dir = duckspec.join("archive");

        let active_changes = scan_changes(&changes_dir, Some(project_root));
        let archived_changes = scan_changes(&archive_dir, Some(project_root));

        let (exps, _) = chat_store::load_explorations(Some(project_root));
        let explorations = exps
            .into_iter()
            .map(|e| ExplorationEntry {
                session_count: chat_store::count_sessions(&e.id, Some(project_root)),
                archived: e.is_archived(),
                id: e.id,
                display_name: e.display_name,
            })
            .collect();

        let ideas = load_idea_entries(Some(project_root));

        Self {
            active_changes,
            archived_changes,
            explorations,
            ideas,
        }
    }
}

/// Lightweight idea scan under shared `<data>/ideas/` (YAML frontmatter links only).
pub fn load_idea_entries(project_root: Option<&Path>) -> Vec<IdeaEntry> {
    let root = paths::data_dir(project_root).join("ideas");
    let mut out = Vec::new();
    walk_md(&root, &mut out);
    out.sort_by(|a, b| a.title.cmp(&b.title));
    out
}

fn walk_md(dir: &Path, out: &mut Vec<IdeaEntry>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_md(&path, out);
        } else if path.extension().is_some_and(|e| e == "md")
            && let Some(idea) = parse_idea_entry(&path)
        {
            out.push(idea);
        }
    }
}

fn parse_idea_entry(path: &Path) -> Option<IdeaEntry> {
    let text = std::fs::read_to_string(path).ok()?;
    let (title, change, exploration) = parse_frontmatter_links(&text);
    let title = if title.trim().is_empty() {
        path.file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "idea".into())
    } else {
        title
    };
    Some(IdeaEntry {
        id: path.to_string_lossy().into_owned(),
        title,
        change,
        exploration,
    })
}

/// Minimal frontmatter: `title`, `change`, `exploration` keys.
fn parse_frontmatter_links(text: &str) -> (String, Option<String>, Option<String>) {
    let mut title = String::new();
    let mut change = None;
    let mut exploration = None;
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        return (title, change, exploration);
    }
    for line in lines {
        let t = line.trim();
        if t == "---" {
            break;
        }
        if let Some(rest) = t.strip_prefix("title:") {
            title = unquote(rest.trim());
        } else if let Some(rest) = t.strip_prefix("change:") {
            let v = unquote(rest.trim());
            if !v.is_empty() {
                change = Some(v);
            }
        } else if let Some(rest) = t.strip_prefix("exploration:") {
            let v = unquote(rest.trim());
            if !v.is_empty() {
                exploration = Some(v);
            }
        }
    }
    (title, change, exploration)
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn scan_changes(dir: &Path, project_root: Option<&Path>) -> Vec<ChangeEntry> {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<PathBuf> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    names.sort();
    names
        .into_iter()
        .filter_map(|path| {
            let name = path.file_name()?.to_str()?.to_string();
            let has_proposal = path.join("proposal.md").exists();
            let has_design = path.join("design.md").exists();
            let has_caps = path.join("caps").is_dir()
                && std::fs::read_dir(path.join("caps"))
                    .map(|d| d.flatten().any(|e| e.path().is_dir() || e.path().is_file()))
                    .unwrap_or(false);
            let steps_dir = path.join("steps");
            let step_count = std::fs::read_dir(&steps_dir)
                .map(|d| {
                    d.flatten()
                        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
                        .count()
                })
                .unwrap_or(0);
            let phase_indicator =
                phase_glyph(has_proposal, has_design, has_caps, step_count, 0).to_string();
            let session_count = project_root
                .map(|root| chat_store::count_sessions(&name, Some(root)))
                .unwrap_or(0);
            Some(ChangeEntry {
                name,
                phase_indicator,
                session_count,
            })
        })
        .collect()
}

fn phase_glyph(
    has_proposal: bool,
    has_design: bool,
    has_caps: bool,
    step_count: usize,
    steps_done: usize,
) -> &'static str {
    if step_count > 0 && steps_done == step_count {
        "✓"
    } else if step_count > 0 {
        "●"
    } else if has_caps {
        "◐"
    } else if has_design {
        "◑"
    } else if has_proposal {
        "○"
    } else {
        "·"
    }
}

/// Result of selecting a navigator row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectResult {
    Bound(BoundChat),
    /// Chat binding cleared (e.g. inbox-only idea).
    Unbound,
    /// Ideas navigation opened in the navigator; no fixed ideas chat scope.
    OpenedIdeasNav,
    OpenSettings,
    ToggledArchived,
    None,
}

#[derive(Debug, Clone)]
pub struct Navigator {
    pub snapshot: ProjectSnapshot,
    pub archived_expanded: bool,
    /// When true, idea rows are listed under the Ideas entry.
    pub ideas_nav_open: bool,
    pub selected: Option<NavId>,
    pub bound: Option<BoundChat>,
    /// Always false in ducktui — no middle content browser.
    pub content_browser: bool,
}

impl Default for Navigator {
    fn default() -> Self {
        Self::from_snapshot(ProjectSnapshot::default())
    }
}

impl Navigator {
    pub fn from_snapshot(snapshot: ProjectSnapshot) -> Self {
        let mut nav = Self {
            snapshot,
            archived_expanded: false,
            ideas_nav_open: false,
            selected: None,
            bound: None,
            content_browser: false,
        };
        nav.ensure_default_selection();
        nav
    }

    pub fn from_project(project_root: &Path) -> Self {
        Self::from_snapshot(ProjectSnapshot::from_project(project_root))
    }

    /// Index of the highlight in `visible_rows`, if the selected id is still listed.
    pub fn selection_index(&self) -> Option<usize> {
        let sel = self.selected.as_ref()?;
        self.visible_rows().iter().position(|r| &r.id == sel)
    }

    /// Seed highlight when none, or when the previous id left the tree.
    pub fn ensure_default_selection(&mut self) {
        if self.selection_index().is_some() {
            return;
        }
        let rows = self.visible_rows();
        // Prefer first non-section header (CHANGES / EXPLORATIONS labels).
        let pick = rows
            .iter()
            .find(|r| !is_inert_section_header(&r.id))
            .or_else(|| rows.first());
        self.selected = pick.map(|r| r.id.clone());
    }

    /// Move the highlight among currently visible rows (wraps).
    pub fn move_selection(&mut self, delta: isize) {
        let rows = self.visible_rows();
        if rows.is_empty() {
            self.selected = None;
            return;
        }
        let cur = self.selection_index().unwrap_or(0);
        let n = rows.len() as isize;
        let next = ((cur as isize + delta).rem_euclid(n)) as usize;
        self.selected = Some(rows[next].id.clone());
    }

    /// Activate the highlighted row (`select` on its id).
    pub fn activate_selected(&mut self) -> SelectResult {
        self.ensure_default_selection();
        let Some(id) = self.selected.clone() else {
            return SelectResult::None;
        };
        self.select(id)
    }

    /// Visible rows: archived children omitted while collapsed; ideas listed when open.
    pub fn visible_rows(&self) -> Vec<NavRow> {
        let mut rows = Vec::new();

        rows.push(section_header("CHANGES"));
        for c in &self.snapshot.active_changes {
            rows.push(NavRow {
                id: NavId::Change(c.name.clone()),
                label: c.name.clone(),
                phase: Some(c.phase_indicator.clone()),
                session_badge: badge(c.session_count),
                kind: NavRowKind::Change,
            });
        }

        rows.push(section_header("EXPLORATIONS"));
        for e in self.snapshot.explorations.iter().filter(|e| !e.archived) {
            rows.push(NavRow {
                id: NavId::Exploration(e.id.clone()),
                label: e.display_name.clone(),
                phase: None,
                session_badge: badge(e.session_count),
                kind: NavRowKind::Exploration,
            });
        }

        let has_archived = !self.snapshot.archived_changes.is_empty()
            || self.snapshot.explorations.iter().any(|e| e.archived);
        if has_archived {
            let label = if self.archived_expanded {
                "ARCHIVED ▼"
            } else {
                "ARCHIVED ▶"
            };
            rows.push(NavRow {
                id: NavId::ArchivedSection,
                label: label.into(),
                phase: None,
                session_badge: None,
                kind: NavRowKind::Section,
            });
            if self.archived_expanded {
                for c in &self.snapshot.archived_changes {
                    rows.push(NavRow {
                        id: NavId::ArchivedChange(c.name.clone()),
                        label: c.name.clone(),
                        phase: Some(c.phase_indicator.clone()),
                        session_badge: badge(c.session_count),
                        kind: NavRowKind::Change,
                    });
                }
                for e in self.snapshot.explorations.iter().filter(|e| e.archived) {
                    rows.push(NavRow {
                        id: NavId::ArchivedExploration(e.id.clone()),
                        label: e.display_name.clone(),
                        phase: None,
                        session_badge: badge(e.session_count),
                        kind: NavRowKind::Exploration,
                    });
                }
            }
        }

        let ideas_label = if self.ideas_nav_open {
            "IDEAS ▼"
        } else {
            "IDEAS"
        };
        rows.push(NavRow {
            id: NavId::Ideas,
            label: ideas_label.into(),
            phase: None,
            session_badge: None,
            kind: NavRowKind::Bottom,
        });
        if self.ideas_nav_open {
            for idea in &self.snapshot.ideas {
                rows.push(NavRow {
                    id: NavId::Idea(idea.id.clone()),
                    label: format!("  {}", idea.title),
                    phase: None,
                    session_badge: None,
                    kind: NavRowKind::Idea,
                });
            }
        }

        rows.push(NavRow {
            id: NavId::Codex,
            label: "CODEX".into(),
            phase: None,
            session_badge: None,
            kind: NavRowKind::Bottom,
        });
        rows.push(NavRow {
            id: NavId::Settings,
            label: "SETTINGS".into(),
            phase: None,
            session_badge: None,
            kind: NavRowKind::Bottom,
        });

        rows
    }

    pub fn select(&mut self, id: NavId) -> SelectResult {
        match id {
            NavId::ArchivedSection => {
                self.archived_expanded = !self.archived_expanded;
                self.selected = Some(NavId::ArchivedSection);
                SelectResult::ToggledArchived
            }
            NavId::Change(name) if name.starts_with("__section_") => SelectResult::None,
            NavId::Change(name) => {
                let bound = BoundChat::Change(name.clone());
                self.selected = Some(NavId::Change(name));
                self.bound = Some(bound.clone());
                self.content_browser = false;
                SelectResult::Bound(bound)
            }
            NavId::Exploration(id) => {
                let bound = BoundChat::Exploration(id.clone());
                self.selected = Some(NavId::Exploration(id));
                self.bound = Some(bound.clone());
                self.content_browser = false;
                SelectResult::Bound(bound)
            }
            NavId::ArchivedChange(name) => {
                let bound = BoundChat::Change(name.clone());
                self.selected = Some(NavId::ArchivedChange(name));
                self.bound = Some(bound.clone());
                self.content_browser = false;
                SelectResult::Bound(bound)
            }
            NavId::ArchivedExploration(id) => {
                let bound = BoundChat::Exploration(id.clone());
                self.selected = Some(NavId::ArchivedExploration(id));
                self.bound = Some(bound.clone());
                self.content_browser = false;
                SelectResult::Bound(bound)
            }
            NavId::Ideas => {
                self.ideas_nav_open = true;
                self.selected = Some(NavId::Ideas);
                self.content_browser = false;
                // Do not bind a fixed ideas session key; leave prior bind as-is
                // only for chat — contract: not bound to fixed ideas key.
                // Opening ideas nav does not invent a scope; clear fixed ideas bind if any.
                SelectResult::OpenedIdeasNav
            }
            NavId::Idea(idea_id) => {
                self.selected = Some(NavId::Idea(idea_id.clone()));
                self.content_browser = false;
                let idea = self.snapshot.ideas.iter().find(|i| i.id == idea_id);
                match idea.and_then(|i| i.scope()) {
                    Some(Scope::Change(name)) => {
                        let bound = BoundChat::Change(name);
                        self.bound = Some(bound.clone());
                        SelectResult::Bound(bound)
                    }
                    Some(Scope::Exploration(id)) => {
                        let bound = BoundChat::Exploration(id);
                        self.bound = Some(bound.clone());
                        SelectResult::Bound(bound)
                    }
                    Some(Scope::Codex) | Some(Scope::Caps) | None => {
                        self.bound = None;
                        SelectResult::Unbound
                    }
                }
            }
            NavId::Codex => {
                let bound = BoundChat::Codex;
                self.selected = Some(NavId::Codex);
                self.bound = Some(bound.clone());
                self.content_browser = false;
                SelectResult::Bound(bound)
            }
            NavId::Settings => {
                self.selected = Some(NavId::Settings);
                SelectResult::OpenSettings
            }
        }
    }
}

fn badge(n: usize) -> Option<usize> {
    (n > 0).then_some(n)
}

fn is_inert_section_header(id: &NavId) -> bool {
    matches!(id, NavId::Change(name) if name.starts_with("__section_"))
}

fn section_header(label: &str) -> NavRow {
    NavRow {
        id: NavId::Change(format!("__section_{label}")),
        label: label.into(),
        phase: None,
        session_badge: None,
        kind: NavRowKind::Section,
    }
}

/// Allow tests to inject session counts without filesystem chat dirs.
pub fn change_entry(name: &str, phase: &str, sessions: usize) -> ChangeEntry {
    ChangeEntry {
        name: name.into(),
        phase_indicator: phase.into(),
        session_count: sessions,
    }
}

pub fn exploration_entry(id: &str, name: &str, sessions: usize, archived: bool) -> ExplorationEntry {
    ExplorationEntry {
        id: id.into(),
        display_name: name.into(),
        session_count: sessions,
        archived,
    }
}

pub fn idea_entry(
    id: &str,
    title: &str,
    change: Option<&str>,
    exploration: Option<&str>,
) -> IdeaEntry {
    IdeaEntry {
        id: id.into(),
        title: title.into(),
        change: change.map(str::to_string),
        exploration: exploration.map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_snapshot() -> ProjectSnapshot {
        ProjectSnapshot {
            active_changes: vec![
                change_entry("build-pilot", "●", 2),
                change_entry("session-work", "○", 0),
            ],
            archived_changes: vec![change_entry("old-thing", "✓", 1)],
            explorations: vec![
                exploration_entry("exploration-1", "ducktui sketch", 0, false),
                exploration_entry("exploration-2", "archived exp", 0, true),
            ],
            ideas: vec![
                idea_entry("idea-linked", "Linked idea", Some("build-pilot"), None),
                idea_entry("idea-inbox", "Inbox only", None, None),
            ],
        }
    }

    // @spec tui/navigator Tree from existing state: Active changes appear under CHANGES with phase indicators
    #[test]
    fn active_changes_appear_under_changes_with_phase_indicators() {
        let nav = Navigator::from_snapshot(sample_snapshot());
        let rows = nav.visible_rows();
        let changes_idx = rows.iter().position(|r| r.label == "CHANGES").unwrap();
        let exp_idx = rows.iter().position(|r| r.label == "EXPLORATIONS").unwrap();
        let change_rows: Vec<_> = rows[changes_idx + 1..exp_idx]
            .iter()
            .filter(|r| r.kind == NavRowKind::Change)
            .collect();
        assert_eq!(change_rows.len(), 2);
        assert!(change_rows.iter().all(|r| r.phase.is_some()));
        assert_eq!(change_rows[0].label, "build-pilot");
        assert_eq!(change_rows[0].phase.as_deref(), Some("●"));
    }

    // @spec tui/navigator Tree from existing state: Explorations appear under EXPLORATIONS
    #[test]
    fn explorations_appear_under_explorations() {
        let nav = Navigator::from_snapshot(sample_snapshot());
        let rows = nav.visible_rows();
        let exp_idx = rows.iter().position(|r| r.label == "EXPLORATIONS").unwrap();
        let arch_idx = rows
            .iter()
            .position(|r| r.label.starts_with("ARCHIVED"))
            .unwrap();
        let exp_rows: Vec<_> = rows[exp_idx + 1..arch_idx]
            .iter()
            .filter(|r| r.kind == NavRowKind::Exploration)
            .collect();
        assert_eq!(exp_rows.len(), 1);
        assert_eq!(exp_rows[0].label, "ducktui sketch");
    }

    // @spec tui/navigator Tree from existing state: ARCHIVED is collapsed by default
    #[test]
    fn archived_is_collapsed_by_default() {
        let nav = Navigator::from_snapshot(sample_snapshot());
        assert!(!nav.archived_expanded);
        let rows = nav.visible_rows();
        assert!(rows.iter().any(|r| r.label.starts_with("ARCHIVED")));
        assert!(!rows.iter().any(|r| r.label == "old-thing"));
        assert!(!rows.iter().any(|r| r.label == "archived exp"));

        let mut nav = nav;
        nav.select(NavId::ArchivedSection);
        assert!(nav.archived_expanded);
        let rows = nav.visible_rows();
        assert!(rows.iter().any(|r| r.label == "old-thing"));
        assert!(rows.iter().any(|r| r.label == "archived exp"));
    }

    // @spec tui/navigator Tree from existing state: Ideas, Codex, and Settings appear as bottom entries
    #[test]
    fn ideas_codex_and_settings_appear_as_bottom_entries() {
        let nav = Navigator::from_snapshot(sample_snapshot());
        let rows = nav.visible_rows();
        let labels: Vec<_> = rows.iter().map(|r| r.label.as_str()).collect();
        let ideas = labels
            .iter()
            .position(|l| l.starts_with("IDEAS"))
            .unwrap();
        let codex = labels.iter().position(|l| *l == "CODEX").unwrap();
        let settings = labels.iter().position(|l| *l == "SETTINGS").unwrap();
        assert!(ideas < codex && codex < settings);
        assert!(ideas > labels.iter().position(|l| *l == "EXPLORATIONS").unwrap());
        assert!(rows[ideas].kind == NavRowKind::Bottom);
        assert!(rows[codex].kind == NavRowKind::Bottom);
        assert!(rows[settings].kind == NavRowKind::Bottom);
    }

    // @spec tui/navigator Selection binds chat scope: Selecting a change binds chat to that change scope
    #[test]
    fn selecting_a_change_binds_chat_to_that_change_scope() {
        let mut nav = Navigator::from_snapshot(sample_snapshot());
        let result = nav.select(NavId::Change("build-pilot".into()));
        assert_eq!(
            result,
            SelectResult::Bound(BoundChat::Change("build-pilot".into()))
        );
        assert_eq!(
            nav.bound.as_ref().map(|b| b.scope()),
            Some(Scope::Change("build-pilot".into()))
        );
    }

    // @spec tui/navigator Selection binds chat scope: Selecting an exploration binds chat to that exploration scope
    #[test]
    fn selecting_an_exploration_binds_chat_to_that_exploration_scope() {
        let mut nav = Navigator::from_snapshot(sample_snapshot());
        let result = nav.select(NavId::Exploration("exploration-1".into()));
        assert_eq!(
            result,
            SelectResult::Bound(BoundChat::Exploration("exploration-1".into()))
        );
        assert_eq!(
            nav.bound.as_ref().map(|b| b.scope()),
            Some(Scope::Exploration("exploration-1".into()))
        );
    }

    // @spec tui/navigator Selection binds chat scope: Selecting Ideas opens navigator ideas list without a fixed ideas chat scope
    #[test]
    fn selecting_ideas_opens_navigator_ideas_list_without_a_fixed_ideas_chat_scope() {
        let mut nav = Navigator::from_snapshot(sample_snapshot());
        let prior_bound = nav.bound.clone();
        let result = nav.select(NavId::Ideas);
        assert_eq!(result, SelectResult::OpenedIdeasNav);
        assert!(nav.ideas_nav_open);
        assert!(!nav.content_browser);
        // Not bound to a fixed ideas session key
        assert_eq!(nav.bound, prior_bound);
        assert!(nav.bound.as_ref().map(|b| b.chat_key()) != Some("ideas".into()));
        let rows = nav.visible_rows();
        assert!(rows.iter().any(|r| r.kind == NavRowKind::Idea));
        assert!(rows.iter().any(|r| r.label.contains("Linked idea")));
        assert!(rows.iter().any(|r| r.label.contains("Inbox only")));
    }

    // @spec tui/navigator Selection binds chat scope: Selecting a change-linked idea binds that change scope
    #[test]
    fn selecting_a_change_linked_idea_binds_that_change_scope() {
        let mut nav = Navigator::from_snapshot(sample_snapshot());
        nav.select(NavId::Ideas);
        let result = nav.select(NavId::Idea("idea-linked".into()));
        assert_eq!(
            result,
            SelectResult::Bound(BoundChat::Change("build-pilot".into()))
        );
        assert_eq!(
            nav.bound.as_ref().map(|b| b.scope()),
            Some(Scope::Change("build-pilot".into()))
        );
    }

    // @spec tui/navigator Selection binds chat scope: Selecting an inbox-only idea does not bind a chat scope
    #[test]
    fn selecting_an_inbox_only_idea_does_not_bind_a_chat_scope() {
        let mut nav = Navigator::from_snapshot(sample_snapshot());
        nav.select(NavId::Change("build-pilot".into()));
        nav.select(NavId::Ideas);
        let result = nav.select(NavId::Idea("idea-inbox".into()));
        assert_eq!(result, SelectResult::Unbound);
        assert!(nav.bound.is_none());
    }

    // @spec tui/navigator Selection binds chat scope: Selecting Codex binds codex chat without a content browser
    #[test]
    fn selecting_codex_binds_codex_chat_without_a_content_browser() {
        let mut nav = Navigator::from_snapshot(sample_snapshot());
        let result = nav.select(NavId::Codex);
        assert_eq!(result, SelectResult::Bound(BoundChat::Codex));
        assert_eq!(nav.bound.as_ref().map(|b| b.scope()), Some(Scope::Codex));
        assert!(!nav.content_browser);
    }

    // @spec tui/navigator Selection binds chat scope: Selecting Settings opens the settings screen
    #[test]
    fn selecting_settings_opens_the_settings_screen() {
        let mut nav = Navigator::from_snapshot(sample_snapshot());
        let prior = nav.bound.clone();
        let result = nav.select(NavId::Settings);
        assert_eq!(result, SelectResult::OpenSettings);
        assert_eq!(nav.bound, prior);
    }

    // @spec tui/navigator Session count badges: Scope with sessions shows the count badge
    #[test]
    fn scope_with_sessions_shows_the_count_badge() {
        let nav = Navigator::from_snapshot(sample_snapshot());
        let row = nav
            .visible_rows()
            .into_iter()
            .find(|r| r.label == "build-pilot")
            .unwrap();
        assert_eq!(row.session_badge, Some(2));
    }

    // @spec tui/navigator Session count badges: Scope with zero sessions shows no badge
    #[test]
    fn scope_with_zero_sessions_shows_no_badge() {
        let nav = Navigator::from_snapshot(sample_snapshot());
        let row = nav
            .visible_rows()
            .into_iter()
            .find(|r| r.label == "session-work")
            .unwrap();
        assert_eq!(row.session_badge, None);
    }
}
