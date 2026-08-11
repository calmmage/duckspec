//! Application configuration stored at `~/.config/duckboard/config.toml`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use duckchat::ModelRef;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub ui: UiConfig,
    pub content: FontConfig,
    pub projects: ProjectsConfig,
    /// Global main-chat default model (harness-tagged `ModelRef`). `None` until
    /// seeded after catalog refresh (or when no catalog model is choosable).
    /// Legacy configs omit the field and deserialize as `None`.
    pub default_model: Option<ModelRef>,
    /// Project override of the global default, keyed by `project_hash`. Absent
    /// means use the global default. Legacy bare-string values load as the
    /// `claude-code` harness via `ModelRef`'s deserialize shim.
    pub model_defaults: HashMap<String, ModelRef>,
    /// Chat affordances: optional oneshot reply chips after a turn.
    pub chat: ChatConfig,
    /// Global version-control workflow for agent standing instructions.
    pub vcs: VcsConfig,
    /// Shared CHANGE / Ideas queue sort and pillow visibility prefs.
    pub list: crate::queue_list::ListConfig,
}

/// Answer body presentation mode for chat (global setting).
///
/// Wire names are lowercase in config.toml (`classic` / `focus` / `document`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ViewerStyle {
    /// Today's full-body Answer path (default).
    #[default]
    Classic,
    /// Classic paint + section folds around the trailing meta gate.
    Focus,
    /// Reserved rich document surface; effective Classic until implemented.
    Document,
}

impl ViewerStyle {
    /// Config / wire name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Focus => "focus",
            Self::Document => "document",
        }
    }

    /// Short label for Settings pickers.
    pub fn label(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::Focus => "Focus",
            Self::Document => "Document",
        }
    }
}

impl std::fmt::Display for ViewerStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// Whether Focus Answer presentation is shipped for effective style + Settings.
pub const FOCUS_ANSWER_IMPLEMENTED: bool = true;

/// Styles offered in Settings (implemented Answer presentation only).
pub fn implemented_viewer_styles() -> &'static [ViewerStyle] {
    if FOCUS_ANSWER_IMPLEMENTED {
        &[ViewerStyle::Classic, ViewerStyle::Focus]
    } else {
        &[ViewerStyle::Classic]
    }
}

/// Resolve presentation style from a stored preference.
///
/// Unknown is not representable on the enum; unimplemented stored values
/// (Document always; Focus while [`FOCUS_ANSWER_IMPLEMENTED`] is false) map to
/// Classic.
pub fn effective_viewer_style(stored: ViewerStyle) -> ViewerStyle {
    match stored {
        ViewerStyle::Classic => ViewerStyle::Classic,
        ViewerStyle::Focus if FOCUS_ANSWER_IMPLEMENTED => ViewerStyle::Focus,
        ViewerStyle::Focus | ViewerStyle::Document => ViewerStyle::Classic,
    }
}

/// Global chat UI flags (all projects / instances).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ChatConfig {
    /// After a turn, run a cheap oneshot for freeform reply chips when eligible.
    /// Default off (model cost).
    pub agent_input_hints: bool,
    /// Preferred oneshot model id per harness (`claude-code`, `grok`, …).
    /// Global (not per-project). Absent key → string-match default from catalog.
    pub oneshot_models: HashMap<String, String>,
    /// When true, future: re-arm pilot after crash/restart/error. Default off.
    /// v1: loaded/saved and shown in Settings; runtime ignore (no reactivate).
    pub pilot_reactivate_on_error: bool,
    /// Answer viewer presentation mode. Default classic.
    pub viewer_style: ViewerStyle,
}

impl ChatConfig {
    /// Configured oneshot model id for `harness`, if any.
    pub fn oneshot_model(&self, harness: &str) -> Option<&str> {
        self.oneshot_models.get(harness).map(String::as_str)
    }

    /// Set or clear the global oneshot model preference for `harness`.
    pub fn set_oneshot_model(&mut self, harness: &str, model: Option<String>) {
        match model {
            Some(m) => {
                self.oneshot_models.insert(harness.to_string(), m);
            }
            None => {
                self.oneshot_models.remove(harness);
            }
        }
    }

    /// Effective Answer presentation style for this config.
    pub fn effective_viewer_style(&self) -> ViewerStyle {
        effective_viewer_style(self.viewer_style)
    }
}

/// Global VCS preferences (all projects / instances).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct VcsConfig {
    /// How the agent should treat version control. Injected into first-turn
    /// priming. Default: plain git.
    pub workflow: VcsWorkflow,
}

/// Logical VCS workflow the agent is told to follow.
///
/// `Worktrees` is agent-facing guidance only in this cut — multi-worktree
/// session plumbing (per-change cwd) is not implemented yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VcsWorkflow {
    /// Plain git (branch / commit / push).
    #[default]
    Git,
    /// Jujutsu (`jj`) with a git backend.
    Jj,
    /// Git with one worktree per parallel change (logical only for now).
    Worktrees,
}

impl VcsWorkflow {
    pub const ALL: [VcsWorkflow; 3] = [Self::Git, Self::Jj, Self::Worktrees];

    /// Short label for Settings pickers.
    pub fn label(self) -> &'static str {
        match self {
            Self::Git => "Git",
            Self::Jj => "Jujutsu (jj)",
            Self::Worktrees => "Git worktrees",
        }
    }

    /// Standing instructions injected into the first-turn priming body.
    pub fn standing_instructions(self) -> &'static str {
        match self {
            Self::Git => {
                "Version control: plain git.\n\
                 - Use `git` commands for all version control operations\n\
                 - Do NOT use `jj` commands\n\
                 - NEVER commit automatically — always show the suggested commit \
                 message and wait for explicit user confirmation before running \
                 `git commit`\n\
                 - When committing work for a duckspec change (including \
                 post-archive `commit`), commit **only** paths that belong to \
                 that change (`git add` those paths, then commit). Do **not** \
                 commit the entire dirty tree by default; leave unrelated dirty \
                 paths uncommitted. If nothing dirty belongs to the change, \
                 report that and do not invent a commit\n\
                 - Do NOT run destructive git commands (force push, hard reset, \
                 checkout that discards work) without explicit confirmation"
            }
            Self::Jj => {
                "Version control: jujutsu (jj).\n\
                 - Use `jj` commands for all version control operations\n\
                 - Do NOT use `git` commands\n\
                 - NEVER commit automatically — always show the suggested commit \
                 message and wait for explicit user confirmation before running \
                 `jj commit`\n\
                 - When committing work for a duckspec change (including \
                 post-archive `commit`), commit **only** paths that belong to \
                 that change (`jj commit` with those filesets). Do **not** \
                 commit the entire dirty tree by default; leave unrelated dirty \
                 paths uncommitted. If nothing dirty belongs to the change, \
                 report that and do not invent a commit\n\
                 - Do NOT run destructive jj commands (like `jj abandon`, \
                 `jj squash --force`) without explicit confirmation\n\
                 - Duckboard may create jj workspaces named `duck-<scope_key>` under \
                 `.duckboard/worktrees/duck-<scope_key>/` for parallel scopes; prefer \
                 those workspaces when placement is Worktree/Auto, and leave Main-pinned \
                 scopes on the primary working copy"
            }
            Self::Worktrees => {
                "Version control: git with worktrees (one worktree per parallel \
                 change).\n\
                 - Prefer a dedicated git worktree for each change when working \
                 in parallel; do not thrash the primary tree with unrelated work\n\
                 - Duckboard creates worktrees at `.duckboard/worktrees/duck-<scope_key>/` \
                 with branch `duck/<scope_key>` — names match `git worktree list` outside \
                 the app\n\
                 - Use plain `git` and `git worktree` — not `jj`\n\
                 - NEVER commit automatically — always show the suggested commit \
                 message and wait for explicit user confirmation before running \
                 `git commit`\n\
                 - When committing work for a duckspec change (including \
                 post-archive `commit`), commit **only** paths that belong to \
                 that change (`git add` those paths, then commit). Do **not** \
                 commit the entire dirty tree by default; leave unrelated dirty \
                 paths uncommitted. If nothing dirty belongs to the change, \
                 report that and do not invent a commit\n\
                 - Do NOT run destructive git commands without explicit confirmation"
            }
        }
    }
}

impl std::fmt::Display for VcsWorkflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectsConfig {
    /// Most-recently-opened first. Capped at RECENT_CAP.
    pub recent: Vec<PathBuf>,
}

/// Maximum number of entries kept in `projects.recent`.
const RECENT_CAP: usize = 12;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FontConfig {
    pub font_family: String,
    pub font_size: f32,
}

/// Global UI chrome preferences (fonts + surface flags).
///
/// Lives under `[ui]` in config.toml. Legacy keys are only `font_family` /
/// `font_size`; phase-pill flags default on when omitted.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub font_family: String,
    pub font_size: f32,
    /// Phase pills on change-list rows. Default true.
    pub phase_pill_list: bool,
    /// Phase pills above the chat composer. Default true.
    pub phase_pill_chat: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ui: UiConfig::default(),
            content: FontConfig {
                font_family: String::from("monospace"),
                font_size: 13.0,
            },
            projects: ProjectsConfig::default(),
            default_model: None,
            model_defaults: HashMap::new(),
            chat: ChatConfig::default(),
            vcs: VcsConfig::default(),
            list: crate::queue_list::ListConfig::default(),
        }
    }
}

impl Config {
    /// The global main-chat default model, if set (or seeded).
    pub fn global_model_default(&self) -> Option<&ModelRef> {
        self.default_model.as_ref()
    }

    /// Set (or, with `None`, clear) the global main-chat default model.
    pub fn set_global_model_default(&mut self, model: Option<ModelRef>) {
        self.default_model = model;
    }

    /// The project override for `project_root`, if one is set.
    pub fn project_model_default(&self, project_root: &Path) -> Option<ModelRef> {
        self.model_defaults
            .get(&project_hash(project_root))
            .cloned()
    }

    /// Set (or, with `None`, clear) the project override for `project_root`.
    pub fn set_project_model_default(&mut self, project_root: &Path, model: Option<ModelRef>) {
        let key = project_hash(project_root);
        match model {
            Some(m) => {
                self.model_defaults.insert(key, m);
            }
            None => {
                self.model_defaults.remove(&key);
            }
        }
    }
}

impl ProjectsConfig {
    /// Promote `path` to the head of the recent list, deduping by canonical
    /// form when available and capping the list length. No-op if `path` is
    /// empty.
    pub fn touch(&mut self, path: &Path) {
        if path.as_os_str().is_empty() {
            return;
        }
        let canonical = path.canonicalize().ok();
        let target = canonical.as_deref().unwrap_or(path);
        self.recent.retain(|p| {
            let pc = p.canonicalize().ok();
            pc.as_deref().unwrap_or(p.as_path()) != target
        });
        self.recent.insert(0, target.to_path_buf());
        if self.recent.len() > RECENT_CAP {
            self.recent.truncate(RECENT_CAP);
        }
    }
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            font_family: String::new(),
            font_size: 13.0,
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            font_family: String::new(),
            font_size: 13.0,
            phase_pill_list: true,
            phase_pill_chat: true,
        }
    }
}

// Shared on-disk layout with ducktui (same ~/.config/duckboard paths).
pub use duckcore::paths::{config_dir, config_path, data_dir, project_hash, set_config_dir_override};

pub fn load() -> Config {
    let path = config_path();
    match std::fs::read_to_string(&path) {
        Ok(data) => match toml::from_str(&data) {
            Ok(config) => config,
            Err(e) => {
                tracing::warn!(path = %path.display(), "failed to parse config, using defaults: {e}");
                Config::default()
            }
        },
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            tracing::warn!(path = %path.display(), "failed to read config, using defaults: {e}");
            Config::default()
        }
        Err(_) => Config::default(),
    }
}

pub fn save(config: &Config) -> anyhow::Result<()> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    let data = toml::to_string_pretty(config)?;
    std::fs::write(config_path(), data)?;
    Ok(())
}

pub fn ui_font(config: &Config) -> iced::Font {
    if config.ui.font_family.is_empty() {
        iced::Font::DEFAULT
    } else {
        iced::Font::with_name(string_to_static(&config.ui.font_family))
    }
}

pub fn content_font(config: &Config) -> iced::Font {
    if config.content.font_family == "monospace" || config.content.font_family.is_empty() {
        iced::Font::MONOSPACE
    } else {
        iced::Font::with_name(string_to_static(&config.content.font_family))
    }
}

fn string_to_static(s: &str) -> &'static str {
    use std::collections::HashSet;
    use std::sync::OnceLock;
    static INTERNED: OnceLock<std::sync::Mutex<HashSet<&'static str>>> = OnceLock::new();
    let set = INTERNED.get_or_init(|| std::sync::Mutex::new(HashSet::new()));
    let mut guard = set.lock().unwrap();
    if let Some(&existing) = guard.get(s) {
        existing
    } else {
        let leaked: &'static str = Box::leak(s.to_string().into_boxed_str());
        guard.insert(leaked);
        leaked
    }
}

pub fn list_system_fonts() -> Vec<String> {
    let source = font_kit::source::SystemSource::new();
    let mut families: Vec<String> = source.all_families().unwrap_or_default();
    families.sort_unstable();
    families.dedup();
    families
}

#[cfg(test)]
mod tests {
    use super::*;

    // @spec chat/default-prompts Agent input hints gate: Default agent input hints setting is disabled
    #[test]
    fn default_agent_input_hints_setting_is_disabled() {
        // GIVEN application config defaults
        // WHEN the agent input hints setting is read
        // THEN it is disabled
        assert!(!Config::default().chat.agent_input_hints);
        assert!(!ChatConfig::default().agent_input_hints);
    }

    /// @spec chat/build-pilot Reactivate setting: Default pilot reactivate setting is off
    #[test]
    fn default_pilot_reactivate_setting_is_off() {
        // GIVEN a fresh default configuration
        // WHEN the pilot reactivate-on-error setting is read
        // THEN the setting is off
        assert!(!Config::default().chat.pilot_reactivate_on_error);
        assert!(!ChatConfig::default().pilot_reactivate_on_error);
    }

    /// @spec chat/build-pilot Reactivate setting: Setting can be enabled without reactivating pilot at runtime
    #[test]
    fn setting_enabled_does_not_arm_pilot_at_runtime() {
        // GIVEN a disarmed session and the pilot reactivate-on-error setting enabled
        let mut cfg = Config::default();
        cfg.chat.pilot_reactivate_on_error = true;
        // WHEN configuration is loaded for chat (setting is on; pilot state is separate)
        // THEN the session pilot remains disarmed — setting does not imply arm
        assert!(cfg.chat.pilot_reactivate_on_error);
        let pilot = crate::build_pilot::PilotState::default();
        assert!(!pilot.is_armed());
        assert_eq!(pilot, crate::build_pilot::PilotState::Off);
    }

    // @spec shell/phase-pills Surface settings: List and chat phase-pill settings default enabled
    #[test]
    fn list_and_chat_phase_pill_settings_default_enabled() {
        // GIVEN application config defaults
        // WHEN the phase-pill list and chat settings are read
        // THEN both are enabled
        assert!(Config::default().ui.phase_pill_list);
        assert!(Config::default().ui.phase_pill_chat);
        assert!(UiConfig::default().phase_pill_list);
        assert!(UiConfig::default().phase_pill_chat);
    }

    #[test]
    fn legacy_ui_font_keys_load_with_phase_pill_defaults() {
        // GIVEN a config that only sets [ui] font fields (pre-phase-pills)
        let cfg: Config = toml::from_str(
            r#"
[ui]
font_family = "Inter"
font_size = 14.0
"#,
        )
        .expect("legacy ui table");
        // THEN fonts load and phase-pill flags default on
        assert_eq!(cfg.ui.font_family, "Inter");
        assert_eq!(cfg.ui.font_size, 14.0);
        assert!(cfg.ui.phase_pill_list);
        assert!(cfg.ui.phase_pill_chat);
    }

    #[test]
    fn missing_chat_table_deserializes_to_defaults() {
        let cfg: Config = toml::from_str("").expect("empty toml");
        assert!(!cfg.chat.agent_input_hints);
        assert!(cfg.chat.oneshot_models.is_empty());
        assert_eq!(cfg.chat.viewer_style, ViewerStyle::Classic);
        assert!(cfg.ui.phase_pill_list);
        assert!(cfg.ui.phase_pill_chat);
    }

    /// @spec chat/viewer-style Stored viewer style: Default stored style is classic
    #[test]
    fn default_stored_viewer_style_is_classic() {
        // GIVEN no chat viewer style has been stored
        // WHEN the stored viewer style is read
        // THEN the stored style is classic
        assert_eq!(Config::default().chat.viewer_style, ViewerStyle::Classic);
        assert_eq!(ChatConfig::default().viewer_style, ViewerStyle::Classic);
        let cfg: Config = toml::from_str("[chat]\nagent_input_hints = false\n")
            .expect("chat table without viewer_style");
        assert_eq!(cfg.chat.viewer_style, ViewerStyle::Classic);
    }

    /// @spec chat/viewer-style Stored viewer style: Chosen style round-trips through save and load
    #[test]
    fn chosen_viewer_style_round_trips_through_save_and_load() {
        // GIVEN the user selects a viewer style value among classic, focus, and document
        for style in [
            ViewerStyle::Classic,
            ViewerStyle::Focus,
            ViewerStyle::Document,
        ] {
            let mut cfg = Config::default();
            cfg.chat.viewer_style = style;
            // WHEN the setting is saved and loaded again
            let toml = toml::to_string(&cfg).expect("serialize config");
            let loaded: Config = toml::from_str(&toml).expect("deserialize config");
            // THEN the stored style is that selected value
            assert_eq!(loaded.chat.viewer_style, style);
            assert!(
                toml.contains(&format!("viewer_style = \"{}\"", style.as_str())),
                "toml should wire {style:?}: {toml}"
            );
        }
    }

    /// @spec chat/viewer-style Effective viewer style: Stored classic yields effective classic
    #[test]
    fn stored_classic_yields_effective_classic() {
        // GIVEN the stored viewer style is classic
        let stored = ViewerStyle::Classic;
        // WHEN the effective viewer style is resolved
        // THEN the effective style is classic
        assert_eq!(effective_viewer_style(stored), ViewerStyle::Classic);
        let mut cfg = Config::default();
        cfg.chat.viewer_style = ViewerStyle::Classic;
        assert_eq!(cfg.chat.effective_viewer_style(), ViewerStyle::Classic);
    }

    /// @spec chat/viewer-style Effective viewer style: Stored document yields effective classic while unimplemented
    #[test]
    fn stored_document_yields_effective_classic_while_unimplemented() {
        // GIVEN Document Answer presentation is not implemented
        // AND the stored viewer style is document
        assert!(!implemented_viewer_styles().contains(&ViewerStyle::Document));
        let stored = ViewerStyle::Document;
        // WHEN the effective viewer style is resolved
        // THEN the effective style is classic
        assert_eq!(effective_viewer_style(stored), ViewerStyle::Classic);
        let mut cfg = Config::default();
        cfg.chat.viewer_style = ViewerStyle::Document;
        assert_eq!(cfg.chat.effective_viewer_style(), ViewerStyle::Classic);
    }

    /// @spec chat/viewer-style Effective viewer style: Stored focus yields effective focus when Focus is implemented
    #[test]
    fn stored_focus_yields_effective_focus_when_focus_is_implemented() {
        // GIVEN Focus Answer presentation is implemented
        // AND the stored viewer style is focus
        assert!(FOCUS_ANSWER_IMPLEMENTED);
        assert!(implemented_viewer_styles().contains(&ViewerStyle::Focus));
        let stored = ViewerStyle::Focus;
        // WHEN the effective viewer style is resolved
        // THEN the effective style is focus
        assert_eq!(effective_viewer_style(stored), ViewerStyle::Focus);
        let mut cfg = Config::default();
        cfg.chat.viewer_style = ViewerStyle::Focus;
        assert_eq!(cfg.chat.effective_viewer_style(), ViewerStyle::Focus);
    }

    #[test]
    fn unknown_auto_messages_key_is_ignored() {
        // GIVEN legacy config that still lists auto_messages
        let cfg: Config = toml::from_str(
            r#"
[chat]
agent_input_hints = true
auto_messages = true
"#,
        )
        .expect("legacy chat table");
        // THEN load succeeds and agent_input_hints is honored
        assert!(cfg.chat.agent_input_hints);
    }
    /// @spec chat/oneshot-models Global per-harness oneshot preference: A configured oneshot model for a harness is stored globally
    #[test]
    fn configured_oneshot_model_for_a_harness_is_stored_globally() {
        // GIVEN a preferred oneshot model id for a harness
        let mut cfg = Config::default();

        // WHEN the oneshot model setting is saved
        cfg.chat
            .set_oneshot_model("claude-code", Some("haiku".into()));

        // THEN that preference is stored as a global application setting for that harness
        assert_eq!(cfg.chat.oneshot_model("claude-code"), Some("haiku"));
        let toml = toml::to_string(&cfg).unwrap();
        let loaded: Config = toml::from_str(&toml).unwrap();
        assert_eq!(loaded.chat.oneshot_model("claude-code"), Some("haiku"));
    }

    /// @spec chat/oneshot-models Global per-harness oneshot preference: Preferences are keyed by harness not by project
    #[test]
    fn preferences_are_keyed_by_harness_not_by_project() {
        // GIVEN a preferred oneshot model for a harness
        // AND more than one project
        let mut cfg = Config::default();
        cfg.chat
            .set_oneshot_model("grok", Some("grok-composer-2.5-fast".into()));

        // WHEN the oneshot model setting is read in either project
        // THEN the same global preference for that harness is returned
        // (oneshot_models live on chat config, not model_defaults / project hash)
        assert!(cfg.model_defaults.is_empty());
        assert_eq!(
            cfg.chat.oneshot_model("grok"),
            Some("grok-composer-2.5-fast")
        );
        assert_eq!(cfg.chat.oneshot_model("claude-code"), None);
    }

    /// @spec harness/selection Global default model setting: A configured global default is stored as an application setting
    #[test]
    fn configured_global_default_is_stored_as_an_application_setting() {
        // GIVEN a harness-tagged model choice for the global main-chat default
        let mut cfg = Config::default();
        let choice = ModelRef::new("claude-code", "sonnet");

        // WHEN the global default setting is saved
        cfg.set_global_model_default(Some(choice.clone()));

        // THEN that choice is stored as a global application setting
        assert_eq!(cfg.global_model_default(), Some(&choice));
        let toml = toml::to_string(&cfg).unwrap();
        let loaded: Config = toml::from_str(&toml).unwrap();
        assert_eq!(loaded.global_model_default(), Some(&choice));
        assert!(loaded.model_defaults.is_empty());
    }

    // @spec shell/vcs-workflow Global workflow choice: Default workflow is plain git
    #[test]
    fn default_vcs_workflow_is_git() {
        assert_eq!(Config::default().vcs.workflow, VcsWorkflow::Git);
        assert_eq!(VcsConfig::default().workflow, VcsWorkflow::Git);
        let cfg: Config = toml::from_str("").expect("empty toml");
        assert_eq!(cfg.vcs.workflow, VcsWorkflow::Git);
    }

    // @spec shell/vcs-workflow Global workflow choice: Selected workflow persists in config
    #[test]
    fn selected_vcs_workflow_persists_in_config() {
        for workflow in [VcsWorkflow::Jj, VcsWorkflow::Worktrees] {
            let mut cfg = Config::default();
            cfg.vcs.workflow = workflow;
            let toml = toml::to_string(&cfg).expect("serialize");
            let loaded: Config = toml::from_str(&toml).expect("reload");
            assert_eq!(loaded.vcs.workflow, workflow);
        }
        assert_eq!(VcsWorkflow::ALL.len(), 3);
    }

    // @spec shell/vcs-workflow Standing instructions: Each workflow names its VCS tool
    #[test]
    fn standing_instructions_name_vcs_tool_for_each_workflow() {
        let git = VcsWorkflow::Git.standing_instructions();
        assert!(git.contains("`git`"), "git names git: {git}");
        assert!(
            git.contains("Do NOT use `jj`"),
            "git forbids jj: {git}"
        );

        let jj = VcsWorkflow::Jj.standing_instructions();
        assert!(jj.contains("`jj`"), "jj names jj: {jj}");
        assert!(
            jj.contains("Do NOT use `git`"),
            "jj forbids bare git: {jj}"
        );

        let wt = VcsWorkflow::Worktrees.standing_instructions();
        assert!(
            wt.contains("worktree"),
            "worktrees names worktrees: {wt}"
        );
        assert!(
            wt.contains("not `jj`") || wt.contains("— not `jj`"),
            "worktrees forbids jj: {wt}"
        );
    }

    // @spec shell/vcs-workflow Standing instructions: Each workflow requires path-scoped change-only commits
    #[test]
    fn standing_instructions_require_path_scoped_change_commit() {
        for workflow in VcsWorkflow::ALL {
            let text = workflow.standing_instructions();
            assert!(
                text.contains("only")
                    && text.contains("belong")
                    && text.to_lowercase().contains("path"),
                "{workflow:?}: expected path-scoped / change-only commit guidance"
            );
            assert!(
                text.contains("entire dirty tree") || text.contains("whole dirty"),
                "{workflow:?}: expected explicit ban on whole-tree commit default"
            );
        }
    }

    // @spec shell/vcs-workflow Standing instructions: Each workflow forbids auto-commit and inventing an empty commit
    #[test]
    fn standing_instructions_forbid_auto_commit_and_inventing_empty() {
        for workflow in VcsWorkflow::ALL {
            let text = workflow.standing_instructions();
            assert!(
                text.contains("NEVER commit automatically")
                    || text.contains("never commit automatically"),
                "{workflow:?}: expected auto-commit ban"
            );
            assert!(
                text.contains("explicit user confirmation")
                    || text.contains("explicit confirmation"),
                "{workflow:?}: expected confirmation before commit"
            );
            assert!(
                text.contains("do not invent a commit"),
                "{workflow:?}: expected empty-set / no-invent guidance"
            );
        }
    }

    // @spec shell/vcs-workflow Standing instructions: Worktrees workflow prefers a dedicated worktree per parallel change
    #[test]
    fn worktrees_standing_instructions_prefer_dedicated_worktree() {
        let text = VcsWorkflow::Worktrees.standing_instructions();
        assert!(
            text.contains("dedicated git worktree")
                || text.contains("dedicated") && text.contains("worktree"),
            "worktrees must prefer a dedicated worktree per parallel change: {text}"
        );
        assert!(
            text.contains("parallel"),
            "worktrees guidance must mention parallel work: {text}"
        );
    }
}
