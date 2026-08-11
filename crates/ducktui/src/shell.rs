//! Application shell: screens, pane focus, overlays, global keys, status bar.
//!
//! Pure state machine — no terminal I/O. The event loop and ratatui views call
//! into this module.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use duckchat::{ModelInfo, ModelRef, SlashCommand};
use duckcore::chat_store;
use duckcore::session_sharing::DriveRole;
use duckcore::slash_commands::{self, system_registry};

use crate::chat_pane::ChatPane;
use crate::config::{self, SharedConfig};
use crate::navigator::{BoundChat, Navigator, SelectResult};

/// Full-screen surface (exactly three).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    ProjectPicker,
    Work,
    Settings,
}

/// Focused pane on the work screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneFocus {
    Navigator,
    Chat,
}

/// Modal overlays on the work screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    ModelPicker,
    SlashPalette,
    SessionSwitcher,
    QuickIdea,
    Help,
}

/// Shared list UI for catalog-backed overlays (slash / model / sessions).
#[derive(Debug, Clone, Default)]
pub struct OverlayListState {
    pub cursor: usize,
    /// Display rows (label + optional description).
    pub rows: Vec<String>,
    /// Slash command names aligned with `rows` (slash palette only).
    pub slash_names: Vec<String>,
    /// Models aligned with `rows` (model picker only).
    pub models: Vec<ModelRef>,
    /// Session ids aligned with `rows`; `None` means “new session” (session switcher).
    pub session_ids: Vec<Option<String>>,
}

impl OverlayListState {
    fn clamp_cursor(&mut self) {
        if self.rows.is_empty() {
            self.cursor = 0;
        } else if self.cursor >= self.rows.len() {
            self.cursor = self.rows.len() - 1;
        }
    }

    fn move_cursor(&mut self, delta: isize) {
        if self.rows.is_empty() {
            return;
        }
        let n = self.rows.len() as isize;
        self.cursor = ((self.cursor as isize + delta).rem_euclid(n)) as usize;
    }
}

/// Turn phase shown in the status bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TurnState {
    #[default]
    Idle,
    Streaming,
    AwaitingChoice,
}

/// Context meter numerator / denominator for the status bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ContextFill {
    pub used: usize,
    pub window: Option<usize>,
}

/// Observable status bar fields (every screen).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusBar {
    /// Bound scope label when a project is bound and a scope is selected.
    pub scope: Option<String>,
    pub model: Option<ModelRef>,
    pub context: ContextFill,
    pub turn: TurnState,
}

/// Work-screen layout: two panes only (no middle content column).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkLayout {
    pub has_navigator: bool,
    pub has_chat: bool,
    pub has_middle_content: bool,
    pub focus: PaneFocus,
}

/// Result of dispatching a key through shell layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyEffect {
    /// App should exit.
    Quit,
    /// Key handled by the shell (screen, overlay, focus).
    Handled,
    /// Key was offered to the focused pane (no overlay).
    Pane(PaneFocus),
    /// Key was offered to the open overlay.
    Overlay(Overlay),
    /// Nothing applied (unknown key).
    Ignored,
    /// Project was bound; host should start the file watcher if needed.
    ProjectBound,
}

/// Focus target on the project picker screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerFocus {
    Path,
    Recent(usize),
}

/// Project picker path entry + shared-config recents.
#[derive(Debug, Clone)]
pub struct ProjectPicker {
    pub path_input: String,
    pub recents: Vec<PathBuf>,
    pub focus: PickerFocus,
}

impl Default for ProjectPicker {
    fn default() -> Self {
        Self {
            path_input: String::new(),
            recents: Vec::new(),
            focus: PickerFocus::Path,
        }
    }
}

impl ProjectPicker {
    pub fn from_recents(recents: Vec<PathBuf>) -> Self {
        let focus = if recents.is_empty() {
            PickerFocus::Path
        } else {
            PickerFocus::Recent(0)
        };
        Self {
            path_input: String::new(),
            recents,
            focus,
        }
    }
}

/// Editable shared prefs on the settings screen (theme stays terminal-local).
#[derive(Debug, Clone, Default)]
pub struct SettingsEditor {
    /// Index into [`SettingsEditor::fields`].
    pub cursor: usize,
    pub default_model: Option<ModelRef>,
    pub agent_input_hints: bool,
    pub oneshot_models: HashMap<String, String>,
    /// Process catalog snapshot for cycling models.
    pub catalog: Vec<ModelInfo>,
    /// Ordered field ids: `"default_model"`, `"agent_input_hints"`, `"oneshot:<harness>"`.
    pub fields: Vec<String>,
}

impl SettingsEditor {
    pub fn from_config(cfg: &SharedConfig, catalog: Vec<ModelInfo>) -> Self {
        let mut harnesses: Vec<String> = Vec::new();
        for m in &catalog {
            if !harnesses.iter().any(|h| h == &m.harness) {
                harnesses.push(m.harness.clone());
            }
        }
        let mut fields = vec!["default_model".into(), "agent_input_hints".into()];
        for h in &harnesses {
            fields.push(format!("oneshot:{h}"));
        }
        Self {
            cursor: 0,
            default_model: cfg.default_model.clone(),
            agent_input_hints: cfg.chat.agent_input_hints,
            oneshot_models: cfg.chat.oneshot_models.clone(),
            catalog,
            fields,
        }
    }

    fn field(&self) -> Option<&str> {
        self.fields.get(self.cursor).map(String::as_str)
    }

    fn move_cursor(&mut self, delta: isize) {
        if self.fields.is_empty() {
            return;
        }
        let n = self.fields.len() as isize;
        let cur = self.cursor as isize + delta;
        self.cursor = ((cur % n) + n) as usize % n as usize;
    }

    fn models_for(&self, harness: &str) -> Vec<&ModelInfo> {
        self.catalog
            .iter()
            .filter(|m| m.harness == harness)
            .collect()
    }

    /// Cycle default model through the catalog (or clear when empty).
    fn cycle_default_model(&mut self, forward: bool) {
        if self.catalog.is_empty() {
            self.default_model = None;
            return;
        }
        let cur = self.default_model.as_ref().and_then(|sel| {
            self.catalog
                .iter()
                .position(|m| m.harness == sel.harness && m.id == sel.model)
        });
        let next = match (cur, forward) {
            (Some(i), true) => (i + 1) % self.catalog.len(),
            (Some(i), false) => {
                if i == 0 {
                    self.catalog.len() - 1
                } else {
                    i - 1
                }
            }
            (None, _) => 0,
        };
        let m = &self.catalog[next];
        self.default_model = Some(ModelRef::new(&m.harness, &m.id));
    }

    fn cycle_oneshot(&mut self, harness: &str, forward: bool) {
        let models: Vec<String> = self
            .models_for(harness)
            .into_iter()
            .map(|m| m.id.clone())
            .collect();
        if models.is_empty() {
            return;
        }
        let cur = self
            .oneshot_models
            .get(harness)
            .and_then(|id| models.iter().position(|m| m == id));
        let next = match (cur, forward) {
            (Some(i), true) => (i + 1) % models.len(),
            (Some(i), false) => {
                if i == 0 {
                    models.len() - 1
                } else {
                    i - 1
                }
            }
            (None, _) => 0,
        };
        self.oneshot_models
            .insert(harness.to_string(), models[next].clone());
    }

    /// Write editor state into shared config and persist.
    pub fn persist(&self) -> SharedConfig {
        let mut cfg = config::load();
        cfg.default_model = self.default_model.clone();
        cfg.chat.agent_input_hints = self.agent_input_hints;
        cfg.chat.oneshot_models = self.oneshot_models.clone();
        if let Err(e) = config::save(&cfg) {
            tracing::warn!("failed to save settings: {e}");
        }
        cfg
    }
}

/// Shell application state.
#[derive(Debug, Clone)]
pub struct Shell {
    pub screen: Screen,
    pub project_root: Option<PathBuf>,
    pub focus: PaneFocus,
    pub overlay: Option<Overlay>,
    pub scope_label: Option<String>,
    pub model: Option<ModelRef>,
    pub context: ContextFill,
    pub turn: TurnState,
    /// Set when quit is requested via the global key layer.
    pub quit_requested: bool,
    /// Left-pane scope tree (work screen).
    pub navigator: Navigator,
    /// Right-pane chat presentation.
    pub chat: ChatPane,
    /// Project picker path entry and shared recents.
    pub picker: ProjectPicker,
    /// Settings screen editor (shared prefs only).
    pub settings: SettingsEditor,
    /// In-memory shared config snapshot (oneshot prefs, etc.).
    pub shared_config: SharedConfig,
    /// List content for the open overlay (slash / model / sessions).
    pub overlay_list: OverlayListState,
    /// Composer buffer for the quick-idea overlay.
    pub quick_idea_text: String,
    /// Status line after quick-idea save / errors (cleared on next open).
    pub overlay_status: Option<String>,
    /// Harness-discovered slash commands (merged with system registry for the palette).
    pub discovered_commands: Vec<SlashCommand>,
}

impl Default for Shell {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Shell {
    /// Start the app. No bound project → project picker.
    pub fn new(project_root: Option<PathBuf>) -> Self {
        let shared_config = config::load();
        let screen = if project_root.is_some() {
            Screen::Work
        } else {
            Screen::ProjectPicker
        };
        let navigator = match &project_root {
            Some(root) => Navigator::from_project(root),
            None => Navigator::default(),
        };
        let picker = ProjectPicker::from_recents(shared_config.projects.recent.clone());
        let model = shared_config.default_model.clone();
        Self {
            screen,
            project_root,
            focus: PaneFocus::Navigator,
            overlay: None,
            scope_label: None,
            model,
            context: ContextFill::default(),
            turn: TurnState::Idle,
            quit_requested: false,
            navigator,
            chat: ChatPane::new("chat"),
            picker,
            settings: SettingsEditor::default(),
            shared_config,
            overlay_list: OverlayListState::default(),
            quick_idea_text: String::new(),
            overlay_status: None,
            discovered_commands: Vec::new(),
        }
    }

    /// Reload recents into the picker from shared config.
    pub fn refresh_picker_recents(&mut self) {
        self.shared_config = config::load();
        let prev = self.picker.path_input.clone();
        self.picker = ProjectPicker::from_recents(self.shared_config.projects.recent.clone());
        self.picker.path_input = prev;
    }

    /// Bind a project path and open the work screen (no config write).
    pub fn bind_project(&mut self, path: impl Into<PathBuf>) {
        let path = path.into();
        self.navigator = Navigator::from_project(&path);
        self.project_root = Some(path);
        self.screen = Screen::Work;
        self.focus = PaneFocus::Navigator;
        self.overlay = None;
        if self.model.is_none() {
            self.model = self.shared_config.default_model.clone();
        }
    }

    /// Bind a project and promote it in the shared recents list (same file as duckboard).
    pub fn bind_project_and_promote(&mut self, path: impl Into<PathBuf>) {
        let path = path.into();
        self.bind_project(path.clone());
        self.shared_config = config::promote_recent(&path);
        self.picker.recents = self.shared_config.projects.recent.clone();
        if self.model.is_none() {
            self.model = self.shared_config.default_model.clone();
        }
    }

    /// Confirm the current picker selection (path entry or highlighted recent).
    ///
    /// Returns `ProjectBound` when a path was bound.
    pub fn confirm_picker(&mut self) -> KeyEffect {
        let path = match self.picker.focus {
            PickerFocus::Path => {
                let raw = self.picker.path_input.trim();
                if raw.is_empty() {
                    return KeyEffect::Ignored;
                }
                PathBuf::from(raw)
            }
            PickerFocus::Recent(i) => match self.picker.recents.get(i) {
                Some(p) => p.clone(),
                None => return KeyEffect::Ignored,
            },
        };
        self.bind_project_and_promote(path);
        KeyEffect::ProjectBound
    }

    /// Handle a project-picker key (`code` or character injection via special codes).
    pub fn handle_picker_key(&mut self, code: &str) -> KeyEffect {
        match code {
            "enter" => return self.confirm_picker(),
            "up" | "k" => {
                match self.picker.focus {
                    PickerFocus::Path => {
                        if !self.picker.recents.is_empty() {
                            self.picker.focus = PickerFocus::Recent(self.picker.recents.len() - 1);
                        }
                    }
                    PickerFocus::Recent(0) => self.picker.focus = PickerFocus::Path,
                    PickerFocus::Recent(i) => self.picker.focus = PickerFocus::Recent(i - 1),
                }
                return KeyEffect::Handled;
            }
            "down" | "j" => {
                match self.picker.focus {
                    PickerFocus::Path => {
                        if !self.picker.recents.is_empty() {
                            self.picker.focus = PickerFocus::Recent(0);
                        }
                    }
                    PickerFocus::Recent(i) if i + 1 < self.picker.recents.len() => {
                        self.picker.focus = PickerFocus::Recent(i + 1);
                    }
                    PickerFocus::Recent(_) => self.picker.focus = PickerFocus::Path,
                }
                return KeyEffect::Handled;
            }
            "tab" => {
                self.picker.focus = match self.picker.focus {
                    PickerFocus::Path if !self.picker.recents.is_empty() => PickerFocus::Recent(0),
                    _ => PickerFocus::Path,
                };
                return KeyEffect::Handled;
            }
            "backspace" => {
                self.picker.focus = PickerFocus::Path;
                self.picker.path_input.pop();
                return KeyEffect::Handled;
            }
            c if c.starts_with("char:") => {
                if let Some(ch) = c.strip_prefix("char:").and_then(|s| s.chars().next()) {
                    self.picker.focus = PickerFocus::Path;
                    self.picker.path_input.push(ch);
                }
                return KeyEffect::Handled;
            }
            _ => {}
        }
        KeyEffect::Ignored
    }

    /// Handle a settings-screen key.
    pub fn handle_settings_key(&mut self, code: &str) -> KeyEffect {
        match code {
            "up" | "k" => {
                self.settings.move_cursor(-1);
                KeyEffect::Handled
            }
            "down" | "j" => {
                self.settings.move_cursor(1);
                KeyEffect::Handled
            }
            "left" | "h" => {
                self.settings_adjust(false);
                KeyEffect::Handled
            }
            "right" | "l" | "enter" | " " => {
                self.settings_adjust(true);
                KeyEffect::Handled
            }
            _ => KeyEffect::Ignored,
        }
    }

    fn settings_adjust(&mut self, forward: bool) {
        let Some(field) = self.settings.field().map(str::to_string) else {
            return;
        };
        match field.as_str() {
            "default_model" => {
                self.settings.cycle_default_model(forward);
                self.model = self.settings.default_model.clone();
            }
            "agent_input_hints" => {
                self.settings.agent_input_hints = !self.settings.agent_input_hints;
            }
            other if other.starts_with("oneshot:") => {
                let harness = &other["oneshot:".len()..];
                self.settings.cycle_oneshot(harness, forward);
            }
            _ => return,
        }
        self.shared_config = self.settings.persist();
    }

    /// Apply a navigator selection (bind chat or open settings).
    pub fn apply_nav_select(&mut self, result: SelectResult) {
        match result {
            SelectResult::Bound(bound) => {
                self.open_bound_chat(bound);
            }
            SelectResult::Unbound => {
                self.scope_label = None;
                // Keep chat pane but clear scope association; no invented ideas key.
            }
            SelectResult::OpenedIdeasNav => {
                // Ideas list in navigator; do not invent a fixed ideas chat scope.
                // Tree gained idea rows — keep Ideas highlighted.
                self.navigator.ensure_default_selection();
            }
            SelectResult::OpenSettings => {
                self.open_settings();
            }
            SelectResult::ToggledArchived => {
                self.navigator.ensure_default_selection();
            }
            SelectResult::None => {}
        }
    }

    /// Bind the chat pane to a navigator scope and load the latest shared session
    /// when one exists. Displayed-only — no write on bind.
    pub fn open_bound_chat(&mut self, bound: BoundChat) {
        self.scope_label = Some(bound.label());
        let key = bound.chat_key();
        self.chat = open_chat_for_scope(&key, self.project_root.as_deref());
    }

    /// Rebuild the navigator tree from the bound project, preserving selection
    /// (and ideas-nav / bound chat) when still valid.
    pub fn refresh_navigator(&mut self) {
        let Some(root) = self.project_root.clone() else {
            return;
        };
        let selected = self.navigator.selected.clone();
        let ideas_open = self.navigator.ideas_nav_open;
        let bound = self.navigator.bound.clone();
        self.navigator = Navigator::from_project(&root);
        self.navigator.ideas_nav_open = ideas_open;
        self.navigator.bound = bound;
        if let Some(sel) = selected {
            self.navigator.selected = Some(sel);
        }
        self.navigator.ensure_default_selection();
    }

    /// Keys while the work-screen navigator pane is focused.
    pub fn handle_navigator_key(&mut self, code: &str) -> KeyEffect {
        match code {
            "up" | "k" => {
                self.navigator.move_selection(-1);
                KeyEffect::Handled
            }
            "down" | "j" => {
                self.navigator.move_selection(1);
                KeyEffect::Handled
            }
            "enter" => {
                let result = self.navigator.activate_selected();
                self.apply_nav_select(result);
                KeyEffect::Handled
            }
            "r" | "refresh-nav" => {
                self.refresh_navigator();
                KeyEffect::Handled
            }
            _ => KeyEffect::Pane(PaneFocus::Navigator),
        }
    }

    /// Transcript scroll / expand keys while chat is focused (composer typing is
    /// handled separately in the event loop).
    pub fn handle_chat_viewport_key(&mut self, code: &str) -> KeyEffect {
        match code {
            "page-up" => {
                let n = self.chat.page_lines();
                self.chat.scroll_up(n);
                KeyEffect::Handled
            }
            "page-down" => {
                let n = self.chat.page_lines();
                self.chat.scroll_down(n);
                KeyEffect::Handled
            }
            "scroll-up" => {
                self.chat.scroll_up(1);
                KeyEffect::Handled
            }
            "scroll-down" => {
                self.chat.scroll_down(1);
                KeyEffect::Handled
            }
            "expand" => {
                self.chat.toggle_next_collapsible();
                KeyEffect::Handled
            }
            _ => KeyEffect::Pane(PaneFocus::Chat),
        }
    }

    pub fn open_settings(&mut self) {
        self.overlay = None;
        // Prefer live catalog when providers have been refreshed; empty is fine.
        let catalog = duckcore::agent::available_models();
        self.shared_config = config::load();
        self.settings = SettingsEditor::from_config(&self.shared_config, catalog);
        self.screen = Screen::Settings;
    }

    /// Leave settings → work if bound, else project picker.
    ///
    /// Theme is not stored in shared config (terminal-local only).
    pub fn leave_settings(&mut self) {
        self.model = self
            .settings
            .default_model
            .clone()
            .or_else(|| self.shared_config.default_model.clone());
        self.screen = if self.project_root.is_some() {
            Screen::Work
        } else {
            Screen::ProjectPicker
            // Refresh recents when returning unbound (settings may have been
            // opened without rebinding).
        };
        if self.screen == Screen::ProjectPicker {
            self.refresh_picker_recents();
        }
    }

    pub fn open_overlay(&mut self, overlay: Overlay) {
        if self.screen != Screen::Work {
            return;
        }
        self.overlay = Some(overlay);
        self.overlay_status = None;
        match overlay {
            Overlay::SlashPalette => self.refresh_slash_list(),
            Overlay::ModelPicker => self.refresh_model_list(),
            Overlay::SessionSwitcher => self.refresh_session_list(),
            Overlay::QuickIdea => {
                self.overlay_list = OverlayListState::default();
                self.quick_idea_text.clear();
            }
            Overlay::Help => {
                self.overlay_list = OverlayListState::default();
            }
        }
    }

    pub fn close_overlay(&mut self) {
        self.overlay = None;
        self.overlay_list = OverlayListState::default();
        self.chat.slash_palette_open = false;
        self.quick_idea_text.clear();
        self.overlay_status = None;
    }

    /// Merged system + discovered completion catalog (same rules as duckboard).
    pub fn slash_catalog(&self) -> Vec<SlashCommand> {
        slash_commands::build_completion_catalog(
            system_registry(),
            self.discovered_commands.clone(),
        )
    }

    /// Apply harness discovery and rebuild an open slash palette.
    pub fn apply_discovered_commands(&mut self, commands: Vec<SlashCommand>) {
        self.discovered_commands = commands;
        if self.overlay == Some(Overlay::SlashPalette) {
            self.refresh_slash_list();
        }
    }

    /// Rebuild slash palette rows from the shared catalog filtered by composer prefix.
    pub fn refresh_slash_list(&mut self) {
        let catalog = self.slash_catalog();
        let prefix = slash_filter_prefix(&self.chat.composer);
        let filtered: Vec<SlashCommand> = catalog
            .into_iter()
            .filter(|c| prefix.is_empty() || c.name.starts_with(prefix))
            .collect();
        let mut list = OverlayListState::default();
        for cmd in filtered {
            let tag = slash_commands::slash_kind_row_tag(cmd.kind)
                .map(|t| format!("[{t}] "))
                .unwrap_or_default();
            list.rows
                .push(format!("/{name}  {tag}{desc}", name = cmd.name, desc = cmd.description));
            list.slash_names.push(cmd.name);
        }
        list.clamp_cursor();
        self.overlay_list = list;
        self.chat.slash_palette_open = true;
    }

    /// Rebuild model picker from the process catalog.
    pub fn refresh_model_list(&mut self) {
        let catalog = duckcore::agent::available_models();
        let mut list = OverlayListState::default();
        // Prefer currently selected model as cursor.
        let mut selected_idx = 0usize;
        for (i, m) in catalog.iter().enumerate() {
            let mark = if self.model.as_ref().is_some_and(|sel| {
                sel.harness == m.harness && sel.model == m.id
            }) {
                selected_idx = i;
                "▸ "
            } else {
                "  "
            };
            list.rows.push(format!(
                "{mark}{}/{}  {}",
                m.harness, m.id, m.display
            ));
            list.models.push(ModelRef::new(&m.harness, &m.id));
        }
        list.cursor = selected_idx.min(list.rows.len().saturating_sub(1));
        self.overlay_list = list;
    }

    /// List sessions for the active chat scope (+ new session row).
    pub fn refresh_session_list(&mut self) {
        let scope = self.chat.session.scope.clone();
        let root = self.project_root.as_deref();
        let sessions = chat_store::load_sessions_for(&scope, root);
        let mut list = OverlayListState::default();
        list.rows.push("+ New session".into());
        list.session_ids.push(None);
        let mut current = 0usize;
        for (i, s) in sessions.iter().enumerate() {
            let title = s
                .title
                .as_deref()
                .filter(|t| !t.is_empty())
                .unwrap_or(s.display_name.as_str());
            let title = if title.is_empty() { "(untitled)" } else { title };
            let mark = if s.id == self.chat.session.id {
                current = i + 1;
                "▸ "
            } else {
                "  "
            };
            list.rows
                .push(format!("{mark}{title}  ·  {}", short_id(&s.id)));
            list.session_ids.push(Some(s.id.clone()));
        }
        list.cursor = current.min(list.rows.len().saturating_sub(1));
        self.overlay_list = list;
    }

    fn activate_overlay_selection(&mut self) {
        match self.overlay {
            Some(Overlay::SlashPalette) => {
                let Some(name) = self.overlay_list.slash_names.get(self.overlay_list.cursor).cloned()
                else {
                    self.close_overlay();
                    return;
                };
                // Replace the current line's `/prefix` with `/name `.
                self.chat.composer = replace_slash_token(&self.chat.composer, &name);
                self.chat.slash_palette_open = false;
                self.close_overlay();
            }
            Some(Overlay::ModelPicker) => {
                let Some(model) = self.overlay_list.models.get(self.overlay_list.cursor).cloned()
                else {
                    self.close_overlay();
                    return;
                };
                self.model = Some(model.clone());
                self.shared_config.default_model = Some(model);
                if let Err(e) = config::save(&self.shared_config) {
                    tracing::warn!("failed to persist default model: {e}");
                }
                // Also refresh context window from catalog when known.
                if let Some(ref m) = self.model {
                    self.context.window = duckcore::agent::model_context_window(m);
                }
                self.close_overlay();
            }
            Some(Overlay::SessionSwitcher) => {
                let scope = self.chat.session.scope.clone();
                let choice = self
                    .overlay_list
                    .session_ids
                    .get(self.overlay_list.cursor)
                    .cloned()
                    .unwrap_or(None);
                match choice {
                    None => {
                        // New empty session (displayed until driven).
                        let mut pane = ChatPane::new(scope);
                        pane.drive_role = DriveRole::Displayed;
                        pane.scroll_to_bottom();
                        self.chat = pane;
                    }
                    Some(id) => {
                        let root = self.project_root.as_deref();
                        let sessions = chat_store::load_sessions_for(&scope, root);
                        if let Some(session) = sessions.into_iter().find(|s| s.id == id) {
                            let mut pane = ChatPane::new(scope);
                            pane.load_from_session(session);
                            pane.drive_role = DriveRole::Displayed;
                            pane.scroll_to_bottom();
                            self.chat = pane;
                        }
                    }
                }
                self.close_overlay();
            }
            Some(Overlay::QuickIdea) => {
                self.save_quick_idea();
            }
            _ => self.close_overlay(),
        }
    }

    /// Write a new inbox idea under the shared duckboard ideas path.
    fn save_quick_idea(&mut self) {
        let body = self.quick_idea_text.trim().to_string();
        if body.is_empty() {
            self.overlay_status = Some("idea body is empty".into());
            return;
        }
        match write_inbox_idea(&body, self.project_root.as_deref()) {
            Ok(path) => {
                self.overlay_status = Some(format!("saved {}", path.display()));
                // Refresh navigator idea list if project is bound.
                if let Some(root) = self.project_root.clone() {
                    let selected = self.navigator.selected.clone();
                    self.navigator = Navigator::from_project(&root);
                    if let Some(sel) = selected {
                        self.navigator.selected = Some(sel);
                        self.navigator.ensure_default_selection();
                    }
                }
                self.quick_idea_text.clear();
                self.close_overlay();
            }
            Err(e) => {
                self.overlay_status = Some(format!("save failed: {e}"));
            }
        }
    }

    /// Keys while an overlay is open (after globals / Escape).
    pub fn handle_overlay_key(&mut self, code: &str) -> KeyEffect {
        let Some(kind) = self.overlay else {
            return KeyEffect::Ignored;
        };
        match code {
            "up" | "k"
                if matches!(
                    kind,
                    Overlay::SlashPalette
                        | Overlay::ModelPicker
                        | Overlay::SessionSwitcher
                ) =>
            {
                self.overlay_list.move_cursor(-1);
                KeyEffect::Handled
            }
            "down" | "j"
                if matches!(
                    kind,
                    Overlay::SlashPalette
                        | Overlay::ModelPicker
                        | Overlay::SessionSwitcher
                ) =>
            {
                self.overlay_list.move_cursor(1);
                KeyEffect::Handled
            }
            "enter" => {
                self.activate_overlay_selection();
                KeyEffect::Handled
            }
            "backspace" if kind == Overlay::SlashPalette => {
                self.chat.composer.pop();
                let line = self.chat.composer.rsplit('\n').next().unwrap_or("");
                if !line.starts_with('/') || line.starts_with("//") {
                    self.close_overlay();
                } else {
                    self.refresh_slash_list();
                }
                KeyEffect::Handled
            }
            "backspace" if kind == Overlay::QuickIdea => {
                self.quick_idea_text.pop();
                KeyEffect::Handled
            }
            c if c.starts_with("char:") && kind == Overlay::SlashPalette => {
                if let Some(ch) = c.strip_prefix("char:").and_then(|s| s.chars().next()) {
                    self.chat.composer.push(ch);
                    self.refresh_slash_list();
                }
                KeyEffect::Handled
            }
            c if c.starts_with("char:") && kind == Overlay::QuickIdea => {
                if let Some(ch) = c.strip_prefix("char:").and_then(|s| s.chars().next()) {
                    self.quick_idea_text.push(ch);
                }
                KeyEffect::Handled
            }
            _ => KeyEffect::Overlay(kind),
        }
    }

    pub fn toggle_pane_focus(&mut self) {
        if self.screen != Screen::Work || self.overlay.is_some() {
            return;
        }
        self.focus = match self.focus {
            PaneFocus::Navigator => PaneFocus::Chat,
            PaneFocus::Chat => PaneFocus::Navigator,
        };
    }

    pub fn work_layout(&self) -> Option<WorkLayout> {
        if self.screen != Screen::Work {
            return None;
        }
        Some(WorkLayout {
            has_navigator: true,
            has_chat: true,
            has_middle_content: false,
            focus: self.focus,
        })
    }

    pub fn status_bar(&self) -> StatusBar {
        let scope = if self.project_root.is_some() {
            self.scope_label.clone()
        } else {
            None
        };
        StatusBar {
            scope,
            model: self.model.clone(),
            context: self.context,
            turn: self.turn,
        }
    }

    /// Dispatch a key through global → overlay → pane layers.
    ///
    /// `code` is a stable string id used by tests and the terminal adapter
    /// (`"tab"`, `"quit"`, `"help"`, `"settings"`, `"escape"`, `"pane-char"`, …).
    pub fn handle_key(&mut self, code: &str) -> KeyEffect {
        // ── Global layer (always first) ───────────────────────────────────
        match code {
            "quit" => {
                self.quit_requested = true;
                return KeyEffect::Quit;
            }
            "help" if self.screen == Screen::Work => {
                self.open_overlay(Overlay::Help);
                return KeyEffect::Handled;
            }
            "model" if self.screen == Screen::Work && self.overlay.is_none() => {
                self.open_overlay(Overlay::ModelPicker);
                return KeyEffect::Handled;
            }
            "sessions" if self.screen == Screen::Work && self.overlay.is_none() => {
                self.open_overlay(Overlay::SessionSwitcher);
                return KeyEffect::Handled;
            }
            "quick-idea" if self.screen == Screen::Work && self.overlay.is_none() => {
                self.open_overlay(Overlay::QuickIdea);
                return KeyEffect::Handled;
            }
            "refresh-nav" if self.screen == Screen::Work && self.overlay.is_none() => {
                self.refresh_navigator();
                return KeyEffect::Handled;
            }
            "settings" if self.overlay.is_none() => {
                self.open_settings();
                return KeyEffect::Handled;
            }
            "leave-settings" if self.screen == Screen::Settings => {
                self.leave_settings();
                return KeyEffect::Handled;
            }
            _ => {}
        }

        // Overlay that owns Escape while open (documented conflict).
        if code == "escape" {
            if self.overlay.is_some() {
                self.close_overlay();
                return KeyEffect::Handled;
            }
            if self.screen == Screen::Settings {
                self.leave_settings();
                return KeyEffect::Handled;
            }
        }

        // ── Overlay layer ─────────────────────────────────────────────────
        if self.overlay.is_some() {
            return self.handle_overlay_key(code);
        }

        // ── Pane layer (work screen only) ─────────────────────────────────
        if self.screen == Screen::Work {
            if code == "tab" {
                self.toggle_pane_focus();
                return KeyEffect::Handled;
            }
            // Number keys activate chat hints when chat is focused.
            if self.focus == PaneFocus::Chat
                && let Some(d) = code.strip_prefix("digit-")
                && let Ok(n) = d.parse::<u8>()
            {
                let _ = self.chat.activate_number(n);
                return KeyEffect::Handled;
            }
            if self.focus == PaneFocus::Chat {
                return self.handle_chat_viewport_key(code);
            }
            if self.focus == PaneFocus::Navigator {
                return self.handle_navigator_key(code);
            }
            return KeyEffect::Pane(self.focus);
        }

        // ── Project picker ────────────────────────────────────────────────
        if self.screen == Screen::ProjectPicker {
            return self.handle_picker_key(code);
        }

        // ── Settings fields ───────────────────────────────────────────────
        if self.screen == Screen::Settings {
            return self.handle_settings_key(code);
        }

        KeyEffect::Ignored
    }
}

fn short_id(id: &str) -> &str {
    if id.len() <= 12 {
        id
    } else {
        &id[..12]
    }
}

/// Persist a new inbox idea next to duckboard's idea store layout.
fn write_inbox_idea(body: &str, project_root: Option<&Path>) -> anyhow::Result<PathBuf> {
    use duckcore::paths;
    use std::time::{SystemTime, UNIX_EPOCH};

    let title = body
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(|l| l.trim_start_matches('#').trim())
        .filter(|l| !l.is_empty())
        .unwrap_or("idea")
        .to_string();
    let slug = slugify(&title);
    let created = iso8601_now();
    let root = paths::data_dir(project_root).join("ideas").join("inbox");
    std::fs::create_dir_all(&root)?;
    let mut path = root.join(format!("{slug}.md"));
    if path.exists() {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        path = root.join(format!("{slug}-{n}.md"));
    }
    let contents = format!(
        "---\ntitle: {title:?}\ncreated: {created:?}\ntags: []\n---\n\n{body}\n"
    );
    std::fs::write(&path, contents)?;
    Ok(path)
}

fn slugify(title: &str) -> String {
    let mut s: String = title
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    while s.contains("--") {
        s = s.replace("--", "-");
    }
    s = s.trim_matches('-').to_string();
    if s.is_empty() {
        "idea".into()
    } else {
        s
    }
}

fn iso8601_now() -> String {
    // Simple local timestamp without extra deps (duckboard uses time crate).
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

/// Filter prefix after `/` on the current composer line (no leading slash).
fn slash_filter_prefix(composer: &str) -> &str {
    let line = composer.rsplit('\n').next().unwrap_or(composer);
    let rest = line.strip_prefix('/').unwrap_or("");
    if rest.starts_with('/') {
        return "";
    }
    rest.split_whitespace().next().unwrap_or(rest)
}

/// Replace `/partial` on the last line with `/name `.
fn replace_slash_token(composer: &str, name: &str) -> String {
    let replacement = format!("/{name} ");
    if composer.is_empty() {
        return replacement;
    }
    let (prefix, last) = match composer.rfind('\n') {
        Some(i) => (&composer[..=i], &composer[i + 1..]),
        None => ("", composer),
    };
    if let Some(idx) = last.rfind('/') {
        format!("{prefix}{}{replacement}", &last[..idx])
    } else {
        format!("{composer}{replacement}")
    }
}

/// Load the newest shared session for `scope` (duckboard-compatible order), or an
/// empty displayed pane when none exist. Never writes.
pub fn open_chat_for_scope(scope: &str, project_root: Option<&Path>) -> ChatPane {
    let mut pane = ChatPane::new(scope);
    pane.drive_role = DriveRole::Displayed;
    if let Some(session) = latest_session_for(scope, project_root) {
        pane.load_from_session(session);
        pane.drive_role = DriveRole::Displayed;
    }
    // Intentional open/switch: pin to latest content.
    pane.scroll_to_bottom();
    pane
}

/// Newest session for a scope from the shared store (`load_sessions_for` order).
pub fn latest_session_for(
    scope: &str,
    project_root: Option<&Path>,
) -> Option<duckcore::chat_store::ChatSession> {
    chat_store::load_sessions_for(scope, project_root)
        .into_iter()
        .next()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // @spec tui/shell Three screens: No bound project opens the project picker
    #[test]
    fn no_bound_project_opens_the_project_picker() {
        // GIVEN no project is bound
        // WHEN the app starts
        let shell = Shell::new(None);
        // THEN the project picker screen is shown
        assert_eq!(shell.screen, Screen::ProjectPicker);
    }

    // @spec tui/shell Three screens: Binding a project opens the work screen
    #[test]
    fn binding_a_project_opens_the_work_screen() {
        // GIVEN the project picker is shown
        let mut shell = Shell::new(None);
        assert_eq!(shell.screen, Screen::ProjectPicker);
        // WHEN a project path is bound
        shell.bind_project("/tmp/demo-project");
        // THEN the work screen is shown
        assert_eq!(shell.screen, Screen::Work);
        assert_eq!(
            shell.project_root.as_deref(),
            Some(Path::new("/tmp/demo-project"))
        );
    }

    // @spec tui/shell Three screens: Settings returns to the bound or unbound surface
    #[test]
    fn settings_returns_to_the_bound_or_unbound_surface() {
        // Bound path
        let mut bound = Shell::new(Some(PathBuf::from("/tmp/proj")));
        bound.open_settings();
        assert_eq!(bound.screen, Screen::Settings);
        bound.leave_settings();
        assert_eq!(bound.screen, Screen::Work);

        // Unbound path
        let mut unbound = Shell::new(None);
        unbound.open_settings();
        assert_eq!(unbound.screen, Screen::Settings);
        unbound.leave_settings();
        assert_eq!(unbound.screen, Screen::ProjectPicker);
    }

    // @spec tui/shell Work screen two panes: Work screen has navigator and chat only
    #[test]
    fn work_screen_has_navigator_and_chat_only() {
        // GIVEN a bound project
        let shell = Shell::new(Some(PathBuf::from("/tmp/proj")));
        // WHEN the work screen is shown
        assert_eq!(shell.screen, Screen::Work);
        let layout = shell.work_layout().expect("work layout");
        // THEN navigator + chat, no middle content column
        assert!(layout.has_navigator);
        assert!(layout.has_chat);
        assert!(!layout.has_middle_content);
    }

    // @spec tui/shell Work screen two panes: Tab toggles pane focus
    #[test]
    fn tab_toggles_pane_focus() {
        let mut shell = Shell::new(Some(PathBuf::from("/tmp/proj")));
        assert_eq!(shell.focus, PaneFocus::Navigator);
        // WHEN Tab
        assert_eq!(shell.handle_key("tab"), KeyEffect::Handled);
        assert_eq!(shell.focus, PaneFocus::Chat);
        // AND Tab again
        assert_eq!(shell.handle_key("tab"), KeyEffect::Handled);
        assert_eq!(shell.focus, PaneFocus::Navigator);
    }

    // @spec tui/shell Overlay input capture: Open overlay consumes pane-destined keys
    #[test]
    fn open_overlay_consumes_pane_destined_keys() {
        let mut shell = Shell::new(Some(PathBuf::from("/tmp/proj")));
        shell.open_overlay(Overlay::SlashPalette);
        // A key that would otherwise go to the focused pane
        let effect = shell.handle_key("pane-char");
        assert_eq!(effect, KeyEffect::Overlay(Overlay::SlashPalette));
        // Focus unchanged — pane did not receive it
        assert_eq!(shell.focus, PaneFocus::Navigator);
        // Tab also stays with overlay while open
        assert_eq!(
            shell.handle_key("tab"),
            KeyEffect::Overlay(Overlay::SlashPalette)
        );
        assert_eq!(shell.focus, PaneFocus::Navigator);
    }

    // @spec tui/shell Overlay input capture: Closing the overlay restores pane dispatch
    #[test]
    fn closing_the_overlay_restores_pane_dispatch() {
        let mut shell = Shell::new(Some(PathBuf::from("/tmp/proj")));
        shell.open_overlay(Overlay::Help);
        assert_eq!(shell.handle_key("escape"), KeyEffect::Handled);
        assert!(shell.overlay.is_none());
        // Subsequent non-global keys dispatch to the focused pane
        assert_eq!(
            shell.handle_key("pane-char"),
            KeyEffect::Pane(PaneFocus::Navigator)
        );
    }

    // @spec tui/shell Global keys before pane dispatch: Help opens from either focused pane
    #[test]
    fn help_opens_from_either_focused_pane() {
        let mut shell = Shell::new(Some(PathBuf::from("/tmp/proj")));
        assert_eq!(shell.focus, PaneFocus::Navigator);
        assert_eq!(shell.handle_key("help"), KeyEffect::Handled);
        assert_eq!(shell.overlay, Some(Overlay::Help));
        shell.close_overlay();

        shell.focus = PaneFocus::Chat;
        assert_eq!(shell.handle_key("help"), KeyEffect::Handled);
        assert_eq!(shell.overlay, Some(Overlay::Help));
    }

    // @spec tui/shell Global keys before pane dispatch: Quit is available without a pane handler
    #[test]
    fn quit_is_available_without_a_pane_handler() {
        let mut shell = Shell::new(Some(PathBuf::from("/tmp/proj")));
        // Quit does not require a focused-pane key handler
        assert_eq!(shell.handle_key("quit"), KeyEffect::Quit);
        assert!(shell.quit_requested);
    }

    // @spec tui/shell Status bar: Work screen status reports scope, model, context, and turn
    #[test]
    fn work_screen_status_reports_scope_model_context_and_turn() {
        let mut shell = Shell::new(Some(PathBuf::from("/tmp/proj")));
        shell.scope_label = Some("ducktui".into());
        shell.model = Some(ModelRef::new("grok", "grok-4.5"));
        shell.context = ContextFill {
            used: 12_000,
            window: Some(256_000),
        };
        shell.turn = TurnState::Streaming;
        let bar = shell.status_bar();
        assert_eq!(bar.scope.as_deref(), Some("ducktui"));
        assert_eq!(bar.model, Some(ModelRef::new("grok", "grok-4.5")));
        assert_eq!(bar.context.used, 12_000);
        assert_eq!(bar.context.window, Some(256_000));
        assert_eq!(bar.turn, TurnState::Streaming);
    }

    // @spec tui/shell Status bar: Project picker status has no scope
    #[test]
    fn project_picker_status_has_no_scope() {
        let mut shell = Shell::new(None);
        shell.model = Some(ModelRef::new("claude-code", "sonnet"));
        shell.scope_label = Some("should-not-show".into());
        let bar = shell.status_bar();
        assert!(bar.scope.is_none());
        assert_eq!(bar.model, Some(ModelRef::new("claude-code", "sonnet")));
        assert_eq!(bar.turn, TurnState::Idle);
    }

    #[test]
    fn navigator_settings_selection_opens_settings_screen() {
        use crate::navigator::{NavId, Navigator, ProjectSnapshot, change_entry};

        let mut shell = Shell::new(Some(PathBuf::from("/tmp/proj")));
        shell.navigator = Navigator::from_snapshot(ProjectSnapshot {
            active_changes: vec![change_entry("c", "●", 0)],
            ..Default::default()
        });
        let result = shell.navigator.select(NavId::Settings);
        shell.apply_nav_select(result);
        assert_eq!(shell.screen, Screen::Settings);
        assert!(shell.navigator.bound.is_none());
    }

    #[test]
    fn discovered_commands_appear_in_slash_palette_with_system() {
        use duckchat::SlashCommandKind;

        let mut shell = Shell::new(Some(PathBuf::from("/tmp/proj")));
        // System-only before discovery
        shell.open_overlay(Overlay::SlashPalette);
        assert!(
            shell.overlay_list.slash_names.iter().any(|n| n == "help"),
            "system help present: {:?}",
            shell.overlay_list.slash_names
        );
        assert!(
            !shell.overlay_list.slash_names.iter().any(|n| n == "ds-spec"),
            "no discovery yet"
        );

        shell.apply_discovered_commands(vec![
            SlashCommand {
                name: "ds-spec".into(),
                description: "Write specs".into(),
                kind: SlashCommandKind::Workflow,
                order_key: Some(20),
            },
            SlashCommand {
                name: "review".into(),
                description: "Agent skill".into(),
                kind: SlashCommandKind::Agent,
                order_key: None,
            },
        ]);
        assert_eq!(shell.overlay, Some(Overlay::SlashPalette));
        assert!(
            shell.overlay_list.slash_names.iter().any(|n| n == "help"),
            "system retained: {:?}",
            shell.overlay_list.slash_names
        );
        assert!(
            shell.overlay_list.slash_names.iter().any(|n| n == "ds-spec"),
            "workflow discovered: {:?}",
            shell.overlay_list.slash_names
        );
        assert!(
            shell.overlay_list.slash_names.iter().any(|n| n == "review"),
            "agent discovered: {:?}",
            shell.overlay_list.slash_names
        );
    }

    #[test]
    fn commands_available_event_merges_into_shell() {
        use crate::runtime::{self, AgentRuntime};
        use duckchat::SlashCommandKind;
        use duckcore::agent::AgentEvent;

        let mut shell = Shell::new(Some(PathBuf::from("/tmp/proj")));
        let mut rt = AgentRuntime::default();
        shell.open_overlay(Overlay::SlashPalette);
        runtime::apply_agent_event(
            &mut shell,
            &mut rt,
            AgentEvent::CommandsAvailable(vec![SlashCommand {
                name: "ds-apply".into(),
                description: "Apply".into(),
                kind: SlashCommandKind::Workflow,
                order_key: Some(10),
            }]),
        );
        assert_eq!(shell.discovered_commands.len(), 1);
        assert!(
            shell.overlay_list.slash_names.iter().any(|n| n == "ds-apply"),
            "open palette refreshed: {:?}",
            shell.overlay_list.slash_names
        );
    }

    #[test]
    fn navigator_refresh_picks_up_new_change_without_rebind() {
        use crate::navigator::NavId;
        use duckcore::test_support::{FsTmp, with_home};

        let tmp = FsTmp::new();
        with_home(tmp.path(), || {
            let root = tmp.path().join("proj");
            let change_a = root.join("duckspec/changes/alpha");
            std::fs::create_dir_all(&change_a).unwrap();
            std::fs::write(change_a.join("proposal.md"), "# alpha\n").unwrap();

            let mut shell = Shell::new(Some(root.clone()));
            let labels: Vec<String> = shell
                .navigator
                .visible_rows()
                .into_iter()
                .map(|r| r.label)
                .collect();
            assert!(
                labels.iter().any(|l| l.contains("alpha")),
                "initial tree has alpha: {labels:?}"
            );
            assert!(
                !labels.iter().any(|l| l.contains("beta")),
                "beta not present yet: {labels:?}"
            );

            // Select alpha so we can assert selection survives refresh.
            shell.navigator.selected = Some(NavId::Change("alpha".into()));

            let change_b = root.join("duckspec/changes/beta");
            std::fs::create_dir_all(&change_b).unwrap();
            std::fs::write(change_b.join("proposal.md"), "# beta\n").unwrap();

            // Without refresh the in-memory tree is still stale.
            let stale: Vec<String> = shell
                .navigator
                .visible_rows()
                .into_iter()
                .map(|r| r.label)
                .collect();
            assert!(
                !stale.iter().any(|l| l.contains("beta")),
                "stale until refresh: {stale:?}"
            );

            assert_eq!(shell.handle_key("refresh-nav"), KeyEffect::Handled);
            let fresh: Vec<String> = shell
                .navigator
                .visible_rows()
                .into_iter()
                .map(|r| r.label)
                .collect();
            assert!(
                fresh.iter().any(|l| l.contains("beta")),
                "refresh loads beta: {fresh:?}"
            );
            assert_eq!(
                shell.navigator.selected,
                Some(NavId::Change("alpha".into())),
                "selection preserved when still valid"
            );
        });
    }
}
