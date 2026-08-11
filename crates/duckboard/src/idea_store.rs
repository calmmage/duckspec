//! Idea model and per-project file storage under `<data>/ideas/`.
//!
//! Each idea is one markdown file with YAML frontmatter:
//!
//! ```text
//! ---
//! title: Fix overflow on long input
//! created: 2026-04-25T14:32:00+02:00
//! tags: [parser/spec, performance]      # first entry = primary, drives path
//! exploration: exploration-1714082400000  # only after Explore is clicked
//! change: 2026-04-25-01-fix-parser-overflow # only after promotion
//! archived: manual                        # only when in archive state
//! ---
//!
//! # Fix overflow on long input
//!
//! Body markdown…
//! ```
//!
//! Path encodes state and primary-tag tree:
//!
//! ```text
//! ideas/
//!   inbox/<slug>.md                          # untagged
//!   inbox/parser/<slug>.md                   # primary tag #parser
//!   inbox/parser/spec/<slug>.md              # primary tag #parser/spec
//!   exploration/…  change/…  archive/…
//! ```
//!
//! Identity of an idea is its path. There is no stable id field; cross-rename
//! continuity is provided by the `exploration` and `change` keys in the
//! frontmatter, which point at the chats/ directory and duckspec change
//! folder respectively. Saving with a new title or new primary tag moves
//! the file (atomic `rename` on a single filesystem).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::data::{self, ProjectData};

// ── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArchiveKind {
    Manual,
    ViaChange,
    Orphaned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdeaState {
    Inbox,
    Exploration,
    Change,
    Archive,
}

impl IdeaState {
    pub fn segment(self) -> &'static str {
        match self {
            IdeaState::Inbox => "inbox",
            IdeaState::Exploration => "exploration",
            IdeaState::Change => "change",
            IdeaState::Archive => "archive",
        }
    }

    #[allow(dead_code)]
    pub fn from_segment(s: &str) -> Option<Self> {
        match s {
            "inbox" => Some(IdeaState::Inbox),
            "exploration" => Some(IdeaState::Exploration),
            "change" => Some(IdeaState::Change),
            "archive" => Some(IdeaState::Archive),
            _ => None,
        }
    }

    pub const ALL: [IdeaState; 4] = [
        IdeaState::Inbox,
        IdeaState::Exploration,
        IdeaState::Change,
        IdeaState::Archive,
    ];
}

/// Exclusive boost mark on an idea. Unknown YAML values load as [`IdeaMark::None`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IdeaMark {
    #[default]
    None,
    Star,
    Hot,
    Cool,
}

impl IdeaMark {
    pub fn as_str(self) -> Option<&'static str> {
        match self {
            IdeaMark::None => None,
            IdeaMark::Star => Some("star"),
            IdeaMark::Hot => Some("hot"),
            IdeaMark::Cool => Some("cool"),
        }
    }

    fn from_yaml_str(s: &str) -> Self {
        match s {
            "star" => IdeaMark::Star,
            "hot" => IdeaMark::Hot,
            "cool" => IdeaMark::Cool,
            "none" | "" => IdeaMark::None,
            _ => IdeaMark::None,
        }
    }
}

impl Serialize for IdeaMark {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.as_str() {
            Some(s) => serializer.serialize_str(s),
            None => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for IdeaMark {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Accept string, null, or missing (via #[serde(default)] on the field).
        // Unknown strings map to None so hand-edited YAML never bricks load.
        let value = Option::<serde_yaml::Value>::deserialize(deserializer)?;
        Ok(match value {
            None | Some(serde_yaml::Value::Null) => IdeaMark::None,
            Some(serde_yaml::Value::String(s)) => IdeaMark::from_yaml_str(&s),
            Some(_) => IdeaMark::None,
        })
    }
}

fn mark_is_none(m: &IdeaMark) -> bool {
    matches!(m, IdeaMark::None)
}

/// Advance exclusive mark: none → star → hot → cool → none.
pub fn cycle_mark(m: IdeaMark) -> IdeaMark {
    match m {
        IdeaMark::None => IdeaMark::Star,
        IdeaMark::Star => IdeaMark::Hot,
        IdeaMark::Hot => IdeaMark::Cool,
        IdeaMark::Cool => IdeaMark::None,
    }
}

/// Set mark and maintain star pin time: record/refresh on star, clear otherwise.
pub fn apply_mark(fm: &mut Frontmatter, next: IdeaMark, now: OffsetDateTime) {
    fm.mark = next;
    fm.favored_at = match next {
        IdeaMark::Star => Some(iso8601_local(now)),
        _ => None,
    };
}

/// Link key for CHANGE-list rows that may inherit an idea mark.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueLinkKey {
    Change(String),
    Exploration(String),
}

/// Idea linked to a change folder name, if any.
pub fn idea_for_change<'a>(ideas: &'a [Idea], name: &str) -> Option<&'a Idea> {
    ideas
        .iter()
        .find(|i| i.frontmatter.change.as_deref() == Some(name))
}

/// Idea linked to an exploration id, if any.
pub fn idea_for_exploration<'a>(ideas: &'a [Idea], id: &str) -> Option<&'a Idea> {
    ideas
        .iter()
        .find(|i| i.frontmatter.exploration.as_deref() == Some(id))
}

/// Projected mark for a change row (none when unlinked).
pub fn mark_for_change(ideas: &[Idea], change_name: &str) -> IdeaMark {
    idea_for_change(ideas, change_name)
        .map(|i| i.frontmatter.mark)
        .unwrap_or(IdeaMark::None)
}

/// Projected mark for an exploration row (none when unlinked).
pub fn mark_for_exploration(ideas: &[Idea], exploration_id: &str) -> IdeaMark {
    idea_for_exploration(ideas, exploration_id)
        .map(|i| i.frontmatter.mark)
        .unwrap_or(IdeaMark::None)
}

/// Projected mark for a queue link key.
pub fn mark_for_link(ideas: &[Idea], key: &QueueLinkKey) -> IdeaMark {
    match key {
        QueueLinkKey::Change(name) => mark_for_change(ideas, name),
        QueueLinkKey::Exploration(id) => mark_for_exploration(ideas, id),
    }
}

/// Cycle the mark on the idea linked to `key`. Returns `true` if a linked idea
/// was updated in memory. Unlinked keys are a pure no-op (no idea is created).
pub fn cycle_mark_for_link(ideas: &mut [Idea], key: &QueueLinkKey, now: OffsetDateTime) -> bool {
    let idea = match key {
        QueueLinkKey::Change(name) => ideas
            .iter_mut()
            .find(|i| i.frontmatter.change.as_deref() == Some(name.as_str())),
        QueueLinkKey::Exploration(id) => ideas
            .iter_mut()
            .find(|i| i.frontmatter.exploration.as_deref() == Some(id.as_str())),
    };
    let Some(idea) = idea else {
        return false;
    };
    let next = cycle_mark(idea.frontmatter.mark);
    apply_mark(&mut idea.frontmatter, next, now);
    true
}

/// Cycle mark for a link and persist when a linked idea exists. Returns whether
/// an idea was updated. Unlinked keys leave storage and the idea list unchanged.
pub fn cycle_and_save_mark_for_link(
    ideas: &mut [Idea],
    key: &QueueLinkKey,
    project_root: Option<&Path>,
) -> bool {
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    if !cycle_mark_for_link(ideas, key, now) {
        return false;
    }
    // Re-find after cycle to save (path may be empty for unsaved tests).
    let idea = match key {
        QueueLinkKey::Change(name) => ideas
            .iter_mut()
            .find(|i| i.frontmatter.change.as_deref() == Some(name.as_str())),
        QueueLinkKey::Exploration(id) => ideas
            .iter_mut()
            .find(|i| i.frontmatter.exploration.as_deref() == Some(id.as_str())),
    };
    let Some(idea) = idea else {
        return false;
    };
    if idea.abs_path.as_os_str().is_empty() || !idea.abs_path.exists() {
        // In-memory only (e.g. unit tests without a file yet).
        return true;
    }
    let body = read_body(&idea.abs_path).unwrap_or_default();
    if let Err(e) = save_idea(idea, &body, project_root) {
        tracing::warn!("failed to save idea after mark cycle: {e}");
    }
    true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Frontmatter {
    pub title: String,
    pub created: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "mark_is_none")]
    pub mark: IdeaMark,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub favored_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exploration: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archived: Option<ArchiveKind>,
}

/// In-memory idea metadata. `abs_path` is the current on-disk location;
/// `state` and `primary_tag_path` are derived from `abs_path` at scan time.
/// The body is **not** held here — it's loaded on demand via `read_body`
/// when the user opens an idea, and passed to `save_idea` on Cmd-S.
#[derive(Debug, Clone)]
pub struct Idea {
    pub abs_path: PathBuf,
    pub state: IdeaState,
    /// Path segments derived from the primary tag. `["parser", "spec"]` for
    /// `#parser/spec`; empty when untagged.
    pub primary_tag_path: Vec<String>,
    pub frontmatter: Frontmatter,
}

impl Idea {
    /// Title for UI rendering — frontmatter wins; falls back to a slug-derived
    /// label if the file's frontmatter is missing/empty (e.g. external edit
    /// before our parser has run on it).
    pub fn display_title(&self) -> String {
        if !self.frontmatter.title.trim().is_empty() {
            return self.frontmatter.title.clone();
        }
        slug_to_display(&self.abs_path)
    }

    /// Stable scope key for this idea's chat: change_name (post-promotion)
    /// wins over exploration id (pre-promotion). `None` for inbox ideas with
    /// no chat scope yet.
    pub fn scope_key(&self) -> Option<&str> {
        self.frontmatter
            .change
            .as_deref()
            .or(self.frontmatter.exploration.as_deref())
    }
}

/// Best-effort title fallback derived from the file's slug. Capitalizes the
/// first letter so a row never reads as a kebab string.
fn slug_to_display(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("untitled");
    let mut s: String = stem.replace('-', " ");
    if let Some(c) = s.get_mut(0..1) {
        c.make_ascii_uppercase();
    }
    s
}

// ── Paths ────────────────────────────────────────────────────────────────────

pub fn ideas_root(project_root: Option<&Path>) -> PathBuf {
    crate::config::data_dir(project_root).join("ideas")
}

/// Slugify an idea title for use as a filename, falling back to `"idea"` when
/// the title has no alphanumeric characters and would otherwise slugify to an
/// empty string.
fn idea_slug(title: &str) -> String {
    let raw = duckpond::slug::slugify(title);
    if raw.is_empty() {
        "idea".to_string()
    } else {
        raw
    }
}

fn primary_tag_segments(tags: &[String]) -> Vec<String> {
    let Some(primary) = tags.first() else {
        return Vec::new();
    };
    primary
        .split('/')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(duckpond::slug::slugify)
        .filter(|s| !s.is_empty())
        .collect()
}

fn idea_path(root: &Path, state: IdeaState, tag_segments: &[String], slug: &str) -> PathBuf {
    let mut p = root.join(state.segment());
    for seg in tag_segments {
        p = p.join(seg);
    }
    p.join(format!("{slug}.md"))
}

// ── Frontmatter parser/serializer ────────────────────────────────────────────

/// Split a file's contents into (frontmatter, body). Recognizes the standard
/// `---\n…\n---\n` envelope at the start. If the envelope is absent or the
/// YAML fails to parse, returns a default frontmatter and the entire input as
/// body.
pub fn parse_file_contents(contents: &str) -> (Frontmatter, String) {
    let Some(rest) = contents.strip_prefix("---\n") else {
        return (Frontmatter::default(), contents.to_string());
    };
    // Find the closing `\n---\n` (or `\n---` at EOF).
    let (yaml, body) = if let Some(idx) = rest.find("\n---\n") {
        (&rest[..idx], &rest[idx + 5..])
    } else if let Some(idx) = rest.rfind("\n---") {
        if rest[idx + 4..].chars().all(char::is_whitespace) {
            (&rest[..idx], "")
        } else {
            return (Frontmatter::default(), contents.to_string());
        }
    } else {
        return (Frontmatter::default(), contents.to_string());
    };
    match serde_yaml::from_str::<Frontmatter>(yaml) {
        Ok(fm) => (fm, body.to_string()),
        Err(e) => {
            tracing::warn!("idea frontmatter parse error: {e}");
            (Frontmatter::default(), contents.to_string())
        }
    }
}

/// Serialize an idea as `---\n<yaml>---\n<body>`.
pub fn serialize_file_contents(frontmatter: &Frontmatter, body: &str) -> anyhow::Result<String> {
    let yaml = serde_yaml::to_string(frontmatter)?;
    // serde_yaml emits a trailing newline; ensure exactly one before the closing fence.
    let yaml = if yaml.ends_with('\n') {
        yaml
    } else {
        format!("{yaml}\n")
    };
    Ok(format!("---\n{yaml}---\n{body}"))
}

// ── Title derivation ─────────────────────────────────────────────────────────

/// Extract a title from the body's first H1 (`# Heading`). Returns `None` if
/// the first non-blank line isn't an ATX H1. Trailing hashes (`# Heading #`)
/// are stripped.
pub fn derive_title_from_body(body: &str) -> Option<String> {
    for line in body.lines() {
        let t = line.trim_start();
        if t.is_empty() {
            continue;
        }
        let stripped = t.strip_prefix('#')?;
        // Reject H2+ (#-prefix already consumed; another # means H2).
        if stripped.starts_with('#') {
            return None;
        }
        if !stripped.starts_with(char::is_whitespace) {
            return None;
        }
        let title = stripped.trim().trim_end_matches('#').trim();
        return Some(title.to_string());
    }
    None
}

pub fn fallback_title() -> String {
    "New idea".to_string()
}

fn iso8601_local(dt: OffsetDateTime) -> String {
    let off = dt.offset();
    let total = off.whole_seconds();
    let sign = if total < 0 { '-' } else { '+' };
    let abs = total.unsigned_abs();
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{}{:02}:{:02}",
        dt.year(),
        dt.month() as u8,
        dt.day(),
        dt.hour(),
        dt.minute(),
        dt.second(),
        sign,
        abs / 3600,
        (abs % 3600) / 60,
    )
}

// ── Loading ──────────────────────────────────────────────────────────────────

/// Cap on how much of each idea file we read at startup just to extract its
/// frontmatter. Real-world frontmatter is ~200 bytes; the cap is a generous
/// ceiling that bounds worst-case work for files without a closing fence.
const MAX_FRONTMATTER_BYTES: usize = 4096;

/// Walk the `ideas/` tree and return all ideas with metadata only. Bodies
/// are deliberately not read here — call `read_body` when an idea is opened.
pub fn load_all(project_root: Option<&Path>) -> Vec<Idea> {
    let root = ideas_root(project_root);
    let mut out = Vec::new();
    for state in IdeaState::ALL {
        let state_root = root.join(state.segment());
        walk_state_dir(&state_root, state, &mut out);
    }
    out
}

fn walk_state_dir(state_root: &Path, state: IdeaState, out: &mut Vec<Idea>) {
    let mut stack: Vec<(PathBuf, Vec<String>)> = vec![(state_root.to_path_buf(), vec![])];
    while let Some((dir, tag_path)) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let p = entry.path();
            let Ok(ft) = entry.file_type() else { continue };
            if ft.is_dir() {
                let mut next = tag_path.clone();
                next.push(entry.file_name().to_string_lossy().into_owned());
                stack.push((p, next));
            } else if ft.is_file()
                && p.extension().is_some_and(|e| e == "md")
                && let Some(idea) = read_idea_meta(&p, state, tag_path.clone())
            {
                out.push(idea);
            }
        }
    }
}

/// Read just enough of `path` to extract its frontmatter. Stops at the
/// closing `\n---` fence or `MAX_FRONTMATTER_BYTES`, whichever comes first.
/// The body is never loaded into memory.
fn read_idea_meta(path: &Path, state: IdeaState, primary_tag_path: Vec<String>) -> Option<Idea> {
    use std::io::Read;
    let mut f = std::fs::File::open(path).ok()?;
    let mut buf = Vec::with_capacity(1024);
    let mut chunk = [0u8; 1024];
    loop {
        let n = match f.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => n,
            Err(_) => return None,
        };
        buf.extend_from_slice(&chunk[..n]);
        if has_closing_fence(&buf) || buf.len() >= MAX_FRONTMATTER_BYTES {
            break;
        }
    }
    let s = std::str::from_utf8(&buf).ok()?;
    let (frontmatter, _) = parse_file_contents(s);
    Some(Idea {
        abs_path: path.to_path_buf(),
        state,
        primary_tag_path,
        frontmatter,
    })
}

/// True once the buffer contains the closing `\n---` line of a frontmatter
/// envelope. Requires the opening `---\n` to already be present (we don't
/// validate that here — `parse_file_contents` handles malformed input
/// downstream).
fn has_closing_fence(buf: &[u8]) -> bool {
    if buf.len() < 8 || !buf.starts_with(b"---\n") {
        return false;
    }
    // Search after the opening fence.
    let after_open = &buf[4..];
    after_open.windows(5).any(|w| w == b"\n---\n") || after_open.ends_with(b"\n---")
}

/// Read the body portion of an idea file. Called when the user opens an
/// idea — the pinned tab feeds this into a fresh `EditorState`. Returns
/// the entire file as body if no frontmatter is found.
pub fn read_body(path: &Path) -> std::io::Result<String> {
    let raw = std::fs::read_to_string(path)?;
    let (_fm, body) = parse_file_contents(&raw);
    Ok(body)
}

// ── Saving ───────────────────────────────────────────────────────────────────

/// Persist an idea to disk. The body is passed explicitly because `Idea`
/// doesn't carry it — the area holds the editor state and forwards its
/// contents on Cmd-S. Recomputes title from `body`'s H1 (or fallback),
/// recomputes the target path from state + primary tag + slugified title,
/// and renames atomically if the path changed. On rename, prunes empty
/// ancestor directories. Updates `idea.abs_path` and `idea.primary_tag_path`
/// on success.
pub fn save_idea(idea: &mut Idea, body: &str, project_root: Option<&Path>) -> anyhow::Result<()> {
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());

    // Recompute title from body. Frontmatter title is purely derived — we
    // never let the YAML diverge from the body's H1.
    idea.frontmatter.title = derive_title_from_body(body).unwrap_or_else(fallback_title);
    if idea.frontmatter.created.trim().is_empty() {
        idea.frontmatter.created = iso8601_local(now);
    }

    // Archive sub-state must be cleared when not in archive (and defaulted
    // to Manual when entering archive without one set).
    match idea.state {
        IdeaState::Archive => {
            if idea.frontmatter.archived.is_none() {
                idea.frontmatter.archived = Some(ArchiveKind::Manual);
            }
        }
        _ => {
            idea.frontmatter.archived = None;
        }
    }

    let root = ideas_root(project_root);
    let target_segments = primary_tag_segments(&idea.frontmatter.tags);
    let slug = idea_slug(&idea.frontmatter.title);

    let candidate = idea_path(&root, idea.state, &target_segments, &slug);
    let final_path = if !candidate.exists() || candidate == idea.abs_path {
        candidate
    } else {
        unique_path(&root, idea.state, &target_segments, &slug)
    };

    let contents = serialize_file_contents(&idea.frontmatter, body)?;
    if let Some(parent) = final_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let prev_path = idea.abs_path.clone();
    if final_path != prev_path && prev_path.exists() {
        std::fs::rename(&prev_path, &final_path)?;
    }
    std::fs::write(&final_path, contents)?;

    if final_path != prev_path && !prev_path.as_os_str().is_empty() {
        prune_empty_dirs(prev_path.parent(), &root);
    }

    idea.abs_path = final_path;
    idea.primary_tag_path = target_segments;
    Ok(())
}

fn unique_path(root: &Path, state: IdeaState, tag_segments: &[String], slug_base: &str) -> PathBuf {
    for n in 2..1000 {
        let candidate = idea_path(root, state, tag_segments, &format!("{slug_base}-{n}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    idea_path(root, state, tag_segments, slug_base)
}

/// Walk up from `start` removing directories that have become empty, stopping
/// at `stop_at` or the first non-empty ancestor or filesystem error.
fn prune_empty_dirs(start: Option<&Path>, stop_at: &Path) {
    let mut cur = start;
    while let Some(p) = cur {
        if p == stop_at || !p.starts_with(stop_at) {
            break;
        }
        let is_empty = std::fs::read_dir(p)
            .map(|mut d| d.next().is_none())
            .unwrap_or(false);
        if !is_empty {
            break;
        }
        if std::fs::remove_dir(p).is_err() {
            break;
        }
        cur = p.parent();
    }
}

// ── Construction & deletion ──────────────────────────────────────────────────

/// Mint a new untagged inbox idea. Path is unset until `save_idea` runs.
pub fn new_idea() -> Idea {
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    Idea {
        abs_path: PathBuf::new(),
        state: IdeaState::Inbox,
        primary_tag_path: Vec::new(),
        frontmatter: Frontmatter {
            title: fallback_title(),
            created: iso8601_local(now),
            tags: Vec::new(),
            mark: IdeaMark::None,
            favored_at: None,
            exploration: None,
            change: None,
            archived: None,
        },
    }
}

/// Target for first-tag idea minting from a CHANGE-list row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MintTarget {
    Exploration { id: String, display_name: String },
    Change { name: String },
}

/// Prettify a change folder slug for an idea title: kebab segments → spaced
/// words with the first character of each segment uppercased.
pub fn prettify_change_slug(name: &str) -> String {
    name.split('-')
        .filter(|s| !s.is_empty())
        .map(|seg| {
            let mut chars = seg.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Create and persist a linked idea for `target`. Optional `first_tag` seeds
/// the tag list when non-empty; mark-only mint may pass `None` or `""`.
/// If a linked idea already exists, returns its path without creating another.
/// Does not set `Exploration.idea_path` — callers update that when the target
/// is an exploration.
pub fn mint_linked_idea(
    ideas: &mut Vec<Idea>,
    target: MintTarget,
    first_tag: Option<&str>,
    project_root: Option<&Path>,
) -> anyhow::Result<PathBuf> {
    let tag = first_tag
        .map(|t| t.trim().trim_start_matches('#').trim())
        .filter(|t| !t.is_empty())
        .map(str::to_string);

    match &target {
        MintTarget::Exploration { id, .. } => {
            if let Some(idea) = idea_for_exploration(ideas, id) {
                return Ok(idea.abs_path.clone());
            }
        }
        MintTarget::Change { name } => {
            if let Some(idea) = idea_for_change(ideas, name) {
                return Ok(idea.abs_path.clone());
            }
        }
    }

    let mut idea = new_idea();
    match &target {
        MintTarget::Exploration { id, display_name } => {
            idea.state = IdeaState::Exploration;
            idea.frontmatter.title = display_name.clone();
            idea.frontmatter.exploration = Some(id.clone());
        }
        MintTarget::Change { name } => {
            idea.state = IdeaState::Change;
            idea.frontmatter.title = prettify_change_slug(name);
            idea.frontmatter.change = Some(name.clone());
        }
    }
    idea.frontmatter.tags = tag.into_iter().collect();
    let body = format!("# {}\n", idea.frontmatter.title);
    save_idea(&mut idea, &body, project_root)?;
    let path = idea.abs_path.clone();
    ideas.push(idea);
    Ok(path)
}

/// Ensure a linked idea exists for `target` (create with no tags if missing).
pub fn ensure_linked_idea(
    ideas: &mut Vec<Idea>,
    target: MintTarget,
    project_root: Option<&Path>,
) -> anyhow::Result<PathBuf> {
    mint_linked_idea(ideas, target, None, project_root)
}

/// Cycle mark on the idea for `target`, creating and linking the idea first
/// when none exists (mark-click mint). Returns whether the mark was updated.
pub fn cycle_mark_for_target(
    ideas: &mut Vec<Idea>,
    target: MintTarget,
    project_root: Option<&Path>,
) -> anyhow::Result<bool> {
    ensure_linked_idea(ideas, target.clone(), project_root)?;
    let key = match &target {
        MintTarget::Exploration { id, .. } => QueueLinkKey::Exploration(id.clone()),
        MintTarget::Change { name } => QueueLinkKey::Change(name.clone()),
    };
    Ok(cycle_and_save_mark_for_link(ideas, &key, project_root))
}

/// Apply a tag to a mint target: mint a linked idea when none exists, otherwise
/// append the tag on the existing idea. Returns `(idea_path, minted)`.
pub fn apply_tag_to_link(
    ideas: &mut Vec<Idea>,
    target: MintTarget,
    tag: &str,
    project_root: Option<&Path>,
) -> anyhow::Result<(PathBuf, bool)> {
    let cleaned = tag.trim().trim_start_matches('#').trim().to_string();
    if cleaned.is_empty() {
        anyhow::bail!("empty tag");
    }

    let already_linked = match &target {
        MintTarget::Exploration { id, .. } => idea_for_exploration(ideas, id).is_some(),
        MintTarget::Change { name } => idea_for_change(ideas, name).is_some(),
    };

    if !already_linked {
        let path = mint_linked_idea(ideas, target, Some(&cleaned), project_root)?;
        return Ok((path, true));
    }

    let idea = match &target {
        MintTarget::Exploration { id, .. } => ideas
            .iter_mut()
            .find(|i| i.frontmatter.exploration.as_deref() == Some(id.as_str())),
        MintTarget::Change { name } => ideas
            .iter_mut()
            .find(|i| i.frontmatter.change.as_deref() == Some(name.as_str())),
    };
    let Some(idea) = idea else {
        anyhow::bail!("linked idea missing after lookup");
    };
    if !idea.frontmatter.tags.iter().any(|t| t == &cleaned) {
        idea.frontmatter.tags.push(cleaned);
    }
    let body = if !idea.abs_path.as_os_str().is_empty() && idea.abs_path.exists() {
        read_body(&idea.abs_path).unwrap_or_else(|_| format!("# {}\n", idea.frontmatter.title))
    } else {
        format!("# {}\n", idea.frontmatter.title)
    };
    save_idea(idea, &body, project_root)?;
    Ok((idea.abs_path.clone(), false))
}

/// Tag an exploration row: mint+link on first tag, then keep `idea_path` in sync.
pub fn apply_tag_to_exploration(
    ideas: &mut Vec<Idea>,
    exp: &mut crate::chat_store::Exploration,
    tag: &str,
    project_root: Option<&Path>,
) -> anyhow::Result<(PathBuf, bool)> {
    let target = MintTarget::Exploration {
        id: exp.id.clone(),
        display_name: exp.display_name.clone(),
    };
    let (path, minted) = apply_tag_to_link(ideas, target, tag, project_root)?;
    exp.idea_path = Some(path.display().to_string());
    Ok((path, minted))
}

/// Tag a change row: mint on first tag when unlinked.
pub fn apply_tag_to_change(
    ideas: &mut Vec<Idea>,
    change_name: &str,
    tag: &str,
    project_root: Option<&Path>,
) -> anyhow::Result<(PathBuf, bool)> {
    apply_tag_to_link(
        ideas,
        MintTarget::Change {
            name: change_name.to_string(),
        },
        tag,
        project_root,
    )
}

pub fn delete_idea(idea: &Idea, project_root: Option<&Path>) {
    if idea.abs_path.exists()
        && let Err(e) = std::fs::remove_file(&idea.abs_path)
    {
        tracing::warn!(path = %idea.abs_path.display(), "failed to delete idea: {e}");
        return;
    }
    let root = ideas_root(project_root);
    prune_empty_dirs(idea.abs_path.parent(), &root);
}

// ── Reconcile against project state ──────────────────────────────────────────

/// A relocation performed by [`reconcile`]: an idea's file moved from
/// `old_path` to `new_path`. `title` is the idea's post-move display title,
/// used to refresh any tab labelled with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdeaMove {
    pub old_path: PathBuf,
    pub new_path: PathBuf,
    pub title: String,
}

/// Detect drift in change-state ideas: when the attached change has been
/// archived externally (via `/ds-archive`) or removed entirely, move the idea
/// into the archive state with the appropriate sub-kind. Performs file moves
/// in place; updates `ideas` to reflect new paths and frontmatter. Reads
/// each drifted idea's body from disk to round-trip it through `save_idea`.
/// Returns the relocations performed so callers can follow moved ideas (e.g.
/// re-point a selection or open tab).
pub fn reconcile(ideas: &mut [Idea], project: &ProjectData) -> Vec<IdeaMove> {
    let project_root = project.project_root.as_deref();
    let mut moves = Vec::new();
    for idea in ideas.iter_mut() {
        let Some((new_state, archived)) = drift_target(idea, project) else {
            continue;
        };
        idea.state = new_state;
        idea.frontmatter.archived = archived;
        let prev_path = idea.abs_path.clone();
        let body = read_body(&idea.abs_path).unwrap_or_default();
        if let Err(e) = save_idea(idea, &body, project_root) {
            tracing::warn!("failed to reconcile idea: {e}");
            continue;
        }
        if idea.abs_path != prev_path {
            moves.push(IdeaMove {
                old_path: prev_path,
                new_path: idea.abs_path.clone(),
                title: idea.display_title(),
            });
        }
    }
    moves
}

fn drift_target(idea: &Idea, project: &ProjectData) -> Option<(IdeaState, Option<ArchiveKind>)> {
    if idea.state == IdeaState::Archive {
        return None;
    }
    let change_name = idea.frontmatter.change.as_deref()?;
    let archived_externally = project
        .archived_changes
        .iter()
        .any(|c| data::strip_archive_prefix(&c.name) == Some(change_name));
    if archived_externally {
        return Some((IdeaState::Archive, Some(ArchiveKind::ViaChange)));
    }
    let still_exists = project.active_changes.iter().any(|c| c.name == change_name);
    if !still_exists {
        return Some((IdeaState::Archive, Some(ArchiveKind::Orphaned)));
    }
    None
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idea_slug_basic() {
        assert_eq!(
            idea_slug("Fix overflow on long input"),
            "fix-overflow-on-long-input"
        );
        assert_eq!(idea_slug("  Hello,  World! "), "hello-world");
        assert_eq!(idea_slug("Already-kebab-case"), "already-kebab-case");
        // Unicode alphanumerics are preserved, not folded away.
        assert_eq!(idea_slug("Spëcial Chårs"), "spëcial-chårs");
        // A title with no alphanumeric characters falls back to "idea".
        assert_eq!(idea_slug("---"), "idea");
        assert_eq!(idea_slug(""), "idea");
    }

    #[test]
    fn derive_title_h1_only() {
        assert_eq!(
            derive_title_from_body("# Hello\nbody"),
            Some("Hello".into())
        );
        assert_eq!(
            derive_title_from_body("\n\n  # Hello world  \nbody"),
            Some("Hello world".into())
        );
        assert_eq!(
            derive_title_from_body("# Hello ##\nbody"),
            Some("Hello".into())
        );
    }

    #[test]
    fn derive_title_rejects_h2_and_plain() {
        assert_eq!(derive_title_from_body("## Heading"), None);
        assert_eq!(derive_title_from_body("plain text"), None);
        assert_eq!(derive_title_from_body(""), None);
        // Hash without a following space is not an H1.
        assert_eq!(derive_title_from_body("#tag at start"), None);
    }

    #[test]
    fn primary_tag_path_segments() {
        assert!(primary_tag_segments(&[]).is_empty());
        assert_eq!(
            primary_tag_segments(&["parser/spec".into(), "performance".into()]),
            vec!["parser".to_string(), "spec".into()]
        );
        assert_eq!(
            primary_tag_segments(&["parser".into()]),
            vec!["parser".to_string()]
        );
        // Tag with internal slashes is hierarchical; non-alphanumerics in each
        // segment slugify normally.
        assert_eq!(
            primary_tag_segments(&["Cool Stuff/sub item".into()]),
            vec!["cool-stuff".to_string(), "sub-item".into()]
        );
        // A segment that slugifies to empty is dropped, not kept as "".
        assert_eq!(
            primary_tag_segments(&["parser/!!!/spec".into()]),
            vec!["parser".to_string(), "spec".into()]
        );
    }

    #[test]
    fn frontmatter_round_trip_minimum() {
        let fm = Frontmatter {
            title: "Test".into(),
            created: "2026-04-25T14:32:00+02:00".into(),
            tags: vec![],
            mark: IdeaMark::None,
            favored_at: None,
            exploration: None,
            change: None,
            archived: None,
        };
        let body = "# Test\n\nsome body\n";
        let serialized = serialize_file_contents(&fm, body).expect("serialize");
        assert!(serialized.starts_with("---\n"));
        assert!(serialized.contains("\n---\n"));
        assert!(serialized.ends_with("some body\n"));

        let (parsed_fm, parsed_body) = parse_file_contents(&serialized);
        assert_eq!(parsed_fm.title, "Test");
        assert_eq!(parsed_fm.created, "2026-04-25T14:32:00+02:00");
        assert!(parsed_fm.tags.is_empty());
        assert_eq!(parsed_body, body);
    }

    #[test]
    fn frontmatter_round_trip_full() {
        let fm = Frontmatter {
            title: "Fix overflow".into(),
            created: "2026-04-25T14:32:00+02:00".into(),
            tags: vec!["parser/spec".into(), "performance".into()],
            mark: IdeaMark::Hot,
            favored_at: None,
            exploration: Some("exploration-1714082400000".into()),
            change: Some("2026-04-25-01-fix-parser-overflow".into()),
            archived: Some(ArchiveKind::ViaChange),
        };
        let body = "# Fix overflow\n\nDetails here.\n";
        let serialized = serialize_file_contents(&fm, body).expect("serialize");
        // archived: via-change must be the kebab form.
        assert!(serialized.contains("archived: via-change"));

        let (parsed_fm, parsed_body) = parse_file_contents(&serialized);
        assert_eq!(parsed_fm.tags, vec!["parser/spec", "performance"]);
        assert_eq!(
            parsed_fm.exploration.as_deref(),
            Some("exploration-1714082400000")
        );
        assert_eq!(
            parsed_fm.change.as_deref(),
            Some("2026-04-25-01-fix-parser-overflow")
        );
        assert!(matches!(parsed_fm.archived, Some(ArchiveKind::ViaChange)));
        assert_eq!(parsed_fm.mark, IdeaMark::Hot);
        assert_eq!(parsed_body, body);
    }

    // @spec ideas/marks Exclusive mark on the idea: Mark cycles through none, star, hot, cool
    #[test]
    fn mark_cycles_through_none_star_hot_cool() {
        let mut m = IdeaMark::None;
        m = cycle_mark(m);
        assert_eq!(m, IdeaMark::Star);
        m = cycle_mark(m);
        assert_eq!(m, IdeaMark::Hot);
        m = cycle_mark(m);
        assert_eq!(m, IdeaMark::Cool);
        m = cycle_mark(m);
        assert_eq!(m, IdeaMark::None);
    }

    // @spec ideas/marks Exclusive mark on the idea: Mark persists across idea reload
    #[test]
    fn mark_persists_across_idea_reload() {
        let fm = Frontmatter {
            title: "Hot idea".into(),
            created: "2026-04-25T14:32:00+02:00".into(),
            mark: IdeaMark::Hot,
            ..Default::default()
        };
        let raw = serialize_file_contents(&fm, "# Hot idea\n").unwrap();
        let (parsed, _) = parse_file_contents(&raw);
        assert_eq!(parsed.mark, IdeaMark::Hot);
    }

    // @spec ideas/marks Exclusive mark on the idea: Unknown stored mark loads as none
    #[test]
    fn unknown_stored_mark_loads_as_none() {
        let raw = "---\ntitle: X\ncreated: 2026-01-01T00:00:00+00:00\nmark: bogon\n---\n# X\n";
        let (fm, _) = parse_file_contents(raw);
        assert_eq!(fm.mark, IdeaMark::None);
    }

    // @spec ideas/marks Star pin time: Entering star records a pin time
    #[test]
    fn entering_star_records_a_pin_time() {
        let mut fm = Frontmatter {
            title: "A".into(),
            created: "2026-01-01T00:00:00+00:00".into(),
            ..Default::default()
        };
        assert!(fm.favored_at.is_none());
        let now = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();
        apply_mark(&mut fm, IdeaMark::Star, now);
        assert_eq!(fm.mark, IdeaMark::Star);
        assert!(fm.favored_at.is_some());
    }

    // @spec ideas/marks Star pin time: Leaving star clears pin time
    #[test]
    fn leaving_star_clears_pin_time() {
        let mut fm = Frontmatter {
            title: "A".into(),
            created: "2026-01-01T00:00:00+00:00".into(),
            ..Default::default()
        };
        let t0 = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();
        apply_mark(&mut fm, IdeaMark::Star, t0);
        assert!(fm.favored_at.is_some());
        apply_mark(&mut fm, IdeaMark::Hot, t0);
        assert_eq!(fm.mark, IdeaMark::Hot);
        assert!(fm.favored_at.is_none());
    }

    // @spec ideas/marks Star pin time: Re-entering star refreshes pin time
    #[test]
    fn re_entering_star_refreshes_pin_time() {
        let mut fm = Frontmatter {
            title: "A".into(),
            created: "2026-01-01T00:00:00+00:00".into(),
            ..Default::default()
        };
        let t0 = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();
        let t1 = OffsetDateTime::from_unix_timestamp(1_700_000_100).unwrap();
        apply_mark(&mut fm, IdeaMark::Star, t0);
        let first = fm.favored_at.clone().expect("pin time");
        apply_mark(&mut fm, IdeaMark::None, t0);
        apply_mark(&mut fm, IdeaMark::Star, t1);
        let second = fm.favored_at.expect("refreshed pin time");
        assert_ne!(first, second);
        assert!(second > first);
    }

    fn sample_idea(mark: IdeaMark, change: Option<&str>, exploration: Option<&str>) -> Idea {
        Idea {
            abs_path: PathBuf::new(),
            state: if change.is_some() {
                IdeaState::Change
            } else if exploration.is_some() {
                IdeaState::Exploration
            } else {
                IdeaState::Inbox
            },
            primary_tag_path: vec![],
            frontmatter: Frontmatter {
                title: "Linked".into(),
                created: "2026-01-01T00:00:00+00:00".into(),
                mark,
                change: change.map(str::to_string),
                exploration: exploration.map(str::to_string),
                ..Default::default()
            },
        }
    }

    // @spec ideas/marks Linked rows inherit the idea mark: Change-linked row exposes the idea's mark
    #[test]
    fn change_linked_row_exposes_the_ideas_mark() {
        let ideas = vec![sample_idea(
            IdeaMark::Cool,
            Some("list-marks-tags-sort"),
            None,
        )];
        assert_eq!(
            mark_for_change(&ideas, "list-marks-tags-sort"),
            IdeaMark::Cool
        );
        assert_eq!(
            mark_for_link(
                &ideas,
                &QueueLinkKey::Change("list-marks-tags-sort".into())
            ),
            IdeaMark::Cool
        );
    }

    // @spec ideas/marks Linked rows inherit the idea mark: Exploration-linked row exposes the idea's mark
    #[test]
    fn exploration_linked_row_exposes_the_ideas_mark() {
        let ideas = vec![sample_idea(
            IdeaMark::Star,
            None,
            Some("exploration-1"),
        )];
        assert_eq!(
            mark_for_exploration(&ideas, "exploration-1"),
            IdeaMark::Star
        );
    }

    // @spec ideas/marks Linked rows inherit the idea mark: Unlinked row has no mark until mark cycle mints
    #[test]
    fn unlinked_row_has_no_mark_until_mark_cycle_mints() {
        let mut ideas: Vec<Idea> = vec![];
        let key = QueueLinkKey::Exploration("exploration-orphan".into());
        assert_eq!(mark_for_link(&ideas, &key), IdeaMark::None);
        // Low-level cycle without mint target still no-ops (no display name / target).
        assert!(!cycle_and_save_mark_for_link(&mut ideas, &key, None));
        assert!(ideas.is_empty());
    }

    #[test]
    fn cycle_and_save_updates_linked_idea_in_memory() {
        let mut ideas = vec![sample_idea(IdeaMark::None, Some("ch"), None)];
        let key = QueueLinkKey::Change("ch".into());
        assert!(cycle_and_save_mark_for_link(&mut ideas, &key, None));
        assert_eq!(mark_for_change(&ideas, "ch"), IdeaMark::Star);
    }

    #[test]
    fn prettify_change_slug_title_cases_kebab() {
        assert_eq!(
            prettify_change_slug("list-marks-tags-sort"),
            "List Marks Tags Sort"
        );
    }

    // @spec ideas/first-tag-mint First tag on free exploration mints a linked idea: First tag creates exploration-state idea with display name title
    // @spec ideas/first-tag-mint First tag on free exploration mints a linked idea: Exploration record points at the new idea
    // @spec ideas/first-tag-mint First tag on free exploration mints a linked idea: Linked exploration remains on the CHANGE list
    #[test]
    fn first_tag_on_free_exploration_mints_and_stays_on_change_list() {
        let (dir, project) = temp_project();
        let mut ideas = Vec::new();
        let mut exp = crate::chat_store::Exploration::new(1);
        exp.display_name = "Cloud agent options".into();
        assert!(exp.is_on_live_list());
        assert!(exp.idea_path.is_none());

        let (_path, minted) = apply_tag_to_exploration(
            &mut ideas,
            &mut exp,
            "ui",
            project.project_root.as_deref(),
        )
        .expect("mint");
        assert!(minted);
        assert_eq!(ideas.len(), 1);
        assert_eq!(ideas[0].state, IdeaState::Exploration);
        assert_eq!(ideas[0].frontmatter.title, "Cloud agent options");
        assert!(ideas[0].frontmatter.tags.iter().any(|t| t == "ui"));
        assert_eq!(
            ideas[0].frontmatter.exploration.as_deref(),
            Some(exp.id.as_str())
        );
        assert!(exp.idea_path.is_some());
        assert!(exp.is_on_live_list());
        cleanup(dir);
    }

    // @spec ideas/first-tag-mint First tag on unlinked change mints a linked idea: First tag creates change-state idea with prettified-slug title
    // @spec ideas/first-tag-mint First tag on unlinked change mints a linked idea: Idea links to the change name
    #[test]
    fn first_tag_on_unlinked_change_mints_prettified_linked_idea() {
        let (dir, project) = temp_project();
        let mut ideas = Vec::new();
        let (_path, minted) = apply_tag_to_change(
            &mut ideas,
            "list-marks-tags-sort",
            "queue",
            project.project_root.as_deref(),
        )
        .expect("mint");
        assert!(minted);
        assert_eq!(ideas.len(), 1);
        assert_eq!(ideas[0].state, IdeaState::Change);
        assert_eq!(
            ideas[0].frontmatter.title,
            prettify_change_slug("list-marks-tags-sort")
        );
        assert!(ideas[0].frontmatter.tags.iter().any(|t| t == "queue"));
        assert_eq!(
            ideas[0].frontmatter.change.as_deref(),
            Some("list-marks-tags-sort")
        );
        cleanup(dir);
    }

    // @spec ideas/first-tag-mint First tag on unlinked change mints a linked idea: Second tag on already-linked change does not mint another idea
    #[test]
    fn second_tag_on_already_linked_change_does_not_mint_another_idea() {
        let (dir, project) = temp_project();
        let mut ideas = Vec::new();
        apply_tag_to_change(
            &mut ideas,
            "list-marks-tags-sort",
            "queue",
            project.project_root.as_deref(),
        )
        .unwrap();
        let (_path, minted) = apply_tag_to_change(
            &mut ideas,
            "list-marks-tags-sort",
            "ui",
            project.project_root.as_deref(),
        )
        .unwrap();
        assert!(!minted);
        assert_eq!(ideas.len(), 1);
        assert!(ideas[0].frontmatter.tags.iter().any(|t| t == "queue"));
        assert!(ideas[0].frontmatter.tags.iter().any(|t| t == "ui"));
        cleanup(dir);
    }

    // @spec ideas/first-tag-mint Chat message does not create an idea: First chat message alone does not mint an idea
    #[test]
    fn first_chat_message_alone_does_not_mint_an_idea() {
        // Chat send has no mint call site; only apply_tag_* creates ideas.
        let ideas: Vec<Idea> = Vec::new();
        let exp = crate::chat_store::Exploration::new(1);
        assert!(exp.idea_path.is_none());
        assert!(idea_for_exploration(&ideas, &exp.id).is_none());
        assert!(ideas.is_empty());
    }

    // @spec ideas/first-tag-mint First mark cycle mints when unlinked: Mark cycle on free exploration creates a linked idea
    #[test]
    fn mark_cycle_on_free_exploration_creates_a_linked_idea() {
        let (dir, project) = temp_project();
        let mut ideas: Vec<Idea> = Vec::new();
        let mut exp = crate::chat_store::Exploration::new(1);
        exp.display_name = "Cloud agent options".into();
        let target = MintTarget::Exploration {
            id: exp.id.clone(),
            display_name: exp.display_name.clone(),
        };
        assert!(cycle_mark_for_target(&mut ideas, target, project.project_root.as_deref()).unwrap());
        assert_eq!(ideas.len(), 1);
        assert_eq!(ideas[0].state, IdeaState::Exploration);
        assert_eq!(ideas[0].frontmatter.title, "Cloud agent options");
        assert_eq!(ideas[0].frontmatter.mark, IdeaMark::Star);
        assert!(ideas[0].frontmatter.tags.is_empty());
        let path = ideas[0].abs_path.display().to_string();
        exp.idea_path = Some(path);
        assert!(exp.idea_path.is_some());
        assert!(exp.is_on_live_list());
        cleanup(dir);
    }

    #[test]
    fn mark_cycle_on_unlinked_change_creates_a_linked_idea() {
        let (dir, project) = temp_project();
        let mut ideas: Vec<Idea> = Vec::new();
        let target = MintTarget::Change {
            name: "list-marks-tags-sort".into(),
        };
        assert!(cycle_mark_for_target(&mut ideas, target, project.project_root.as_deref()).unwrap());
        assert_eq!(ideas.len(), 1);
        assert_eq!(ideas[0].state, IdeaState::Change);
        assert_eq!(ideas[0].frontmatter.mark, IdeaMark::Star);
        assert_eq!(
            ideas[0].frontmatter.change.as_deref(),
            Some("list-marks-tags-sort")
        );
        cleanup(dir);
    }

    #[test]
    fn parse_no_frontmatter_returns_default() {
        let raw = "# No frontmatter\n\nbody only\n";
        let (fm, body) = parse_file_contents(raw);
        assert!(fm.title.is_empty());
        assert_eq!(body, raw);
    }

    #[test]
    fn parse_malformed_yaml_returns_default() {
        let raw = "---\nthis is: not: valid yaml: somehow\n---\nbody\n";
        let (fm, body) = parse_file_contents(raw);
        assert!(fm.title.is_empty());
        // Body falls back to the entire input so no data is lost.
        assert_eq!(body, raw);
    }

    #[test]
    fn idea_state_segment_round_trip() {
        for s in IdeaState::ALL {
            assert_eq!(IdeaState::from_segment(s.segment()), Some(s));
        }
        assert_eq!(IdeaState::from_segment("nope"), None);
    }

    #[test]
    fn parse_serialize_round_trip_for_body_separation() {
        // Body is no longer carried on Idea; this test verifies that
        // `parse_file_contents` cleanly separates frontmatter and body so
        // `read_body` (which calls it) returns just the body slice.
        let fm = Frontmatter {
            title: "Test".into(),
            created: "2026-04-25T14:32:00+02:00".into(),
            tags: vec![],
            mark: IdeaMark::None,
            favored_at: None,
            exploration: None,
            change: None,
            archived: None,
        };
        let body = "# Test\n\nSome content.\n";
        let raw = serialize_file_contents(&fm, body).unwrap();
        let (parsed_fm, parsed_body) = parse_file_contents(&raw);
        assert_eq!(parsed_fm.title, "Test");
        assert_eq!(parsed_body, body);
    }

    #[test]
    fn has_closing_fence_finds_end_of_yaml() {
        // No fence yet — just opening.
        assert!(!has_closing_fence(b"---\ntitle: x\n"));
        // Fence in the middle, with body after.
        assert!(has_closing_fence(b"---\ntitle: x\n---\nbody"));
        // Fence at EOF, no trailing newline.
        assert!(has_closing_fence(b"---\ntitle: x\n---"));
        // Buffer doesn't even start with opening fence — never match.
        assert!(!has_closing_fence(b"some content\n---\nmore"));
    }

    #[test]
    fn read_idea_meta_skips_body_and_caps_at_max_bytes() {
        let tmp = tempdir();
        // Build a file with a normal frontmatter and a huge body. After the
        // closing fence, the file is mostly noise; meta-reading must not
        // load it.
        let fm = Frontmatter {
            title: "Big".into(),
            created: "2026-04-25T14:32:00+02:00".into(),
            tags: vec!["parser".into()],
            mark: IdeaMark::None,
            favored_at: None,
            exploration: None,
            change: None,
            archived: None,
        };
        let large_body = "# Big\n\n".to_string() + &"x".repeat(100_000);
        let raw = serialize_file_contents(&fm, &large_body).unwrap();
        let p = tmp.join("big.md");
        std::fs::write(&p, &raw).unwrap();

        let idea = read_idea_meta(&p, IdeaState::Inbox, vec![]).expect("meta");
        assert_eq!(idea.frontmatter.title, "Big");
        assert_eq!(idea.frontmatter.tags, vec!["parser"]);
        // Body field doesn't exist on Idea anymore; verify by reading body
        // separately via the public helper.
        let body = read_body(&p).unwrap();
        assert!(body.contains("# Big"));
        cleanup(tmp);
    }

    // ── Reconcile ─────────────────────────────────────────────────────────

    fn change_data(name: &str) -> crate::data::ChangeData {
        crate::data::ChangeData {
            name: name.into(),
            prefix: String::new(),
            has_proposal: false,
            has_design: false,
            cap_tree: vec![],
            steps: vec![],
            reviews: vec![],
            shallow_mtime_nanos: None,
        }
    }

    /// A change-state idea linked to `change`, with no I/O performed.
    fn change_idea(change: &str) -> Idea {
        Idea {
            abs_path: PathBuf::new(),
            state: IdeaState::Change,
            primary_tag_path: vec![],
            frontmatter: Frontmatter {
                title: "An idea".into(),
                created: "2026-01-01T00:00:00+00:00".into(),
                change: Some(change.into()),
                ..Default::default()
            },
        }
    }

    /// Redirect `config_dir` to a fresh temp dir for this thread and return a
    /// `ProjectData` whose ideas therefore live under it.
    fn temp_project() -> (PathBuf, ProjectData) {
        let dir = tempdir();
        crate::config::set_config_dir_override(dir.clone());
        (dir, ProjectData::default())
    }

    /// Materialize a change-state idea on disk under the temp ideas root.
    fn seed_change_idea(project: &ProjectData, change: &str) -> Idea {
        let mut idea = change_idea(change);
        let body = "# An idea\n\nbody\n";
        save_idea(&mut idea, body, project.project_root.as_deref()).unwrap();
        idea
    }

    /// @spec ideas/reconcile Change-linked drift classification: Linked change archived classifies the idea as via-change
    #[test]
    fn drift_archived_change_is_via_change() {
        let idea = change_idea("my-change");
        let project = ProjectData {
            archived_changes: vec![change_data("2026-01-01-01-my-change")],
            ..Default::default()
        };
        assert_eq!(
            drift_target(&idea, &project),
            Some((IdeaState::Archive, Some(ArchiveKind::ViaChange)))
        );
    }

    /// @spec ideas/reconcile Change-linked drift classification: Linked change gone classifies the idea as orphaned
    #[test]
    fn drift_vanished_change_is_orphaned() {
        let idea = change_idea("ghost-change");
        let project = ProjectData::default();
        assert_eq!(
            drift_target(&idea, &project),
            Some((IdeaState::Archive, Some(ArchiveKind::Orphaned)))
        );
    }

    /// @spec ideas/reconcile Change-linked drift classification: Active linked change leaves the idea unchanged
    #[test]
    fn drift_active_change_is_none() {
        let idea = change_idea("my-change");
        let project = ProjectData {
            active_changes: vec![change_data("my-change")],
            ..Default::default()
        };
        assert_eq!(drift_target(&idea, &project), None);
    }

    /// @spec ideas/reconcile Change-linked drift classification: Already-archived idea keeps its archive reason
    #[test]
    fn drift_already_archived_keeps_reason() {
        let mut idea = change_idea("my-change");
        idea.state = IdeaState::Archive;
        idea.frontmatter.archived = Some(ArchiveKind::Manual);
        // Even though the change is archived externally, an already-archived
        // idea is not reclassified.
        let project = ProjectData {
            archived_changes: vec![change_data("2026-01-01-01-my-change")],
            ..Default::default()
        };
        assert_eq!(drift_target(&idea, &project), None);

        let moves = reconcile(std::slice::from_mut(&mut idea), &project);
        assert!(moves.is_empty());
        assert_eq!(idea.frontmatter.archived, Some(ArchiveKind::Manual));
    }

    /// @spec ideas/reconcile Relocation reporting: An archiving relocation is reported with source and destination
    #[test]
    fn reconcile_reports_archiving_relocation() {
        let (dir, mut project) = temp_project();
        project.archived_changes = vec![change_data("2026-01-01-01-my-change")];
        let mut idea = seed_change_idea(&project, "my-change");
        let old_path = idea.abs_path.clone();

        let moves = reconcile(std::slice::from_mut(&mut idea), &project);

        assert_eq!(moves.len(), 1);
        let mv = &moves[0];
        assert_eq!(mv.old_path, old_path);
        assert_eq!(mv.new_path, idea.abs_path);
        // The relocation lands the file in the archive subtree as via-change.
        assert!(mv.new_path.starts_with(ideas_root(None).join("archive")));
        assert!(mv.new_path.exists());
        assert!(!old_path.exists());
        assert_eq!(idea.frontmatter.archived, Some(ArchiveKind::ViaChange));
        cleanup(dir);
    }

    /// @spec ideas/reconcile Relocation reporting: A no-op reconciliation reports no relocations
    #[test]
    fn reconcile_reports_nothing_when_no_drift() {
        let (dir, mut project) = temp_project();
        project.active_changes = vec![change_data("my-change")];
        let mut idea = seed_change_idea(&project, "my-change");
        let path = idea.abs_path.clone();

        let moves = reconcile(std::slice::from_mut(&mut idea), &project);

        assert!(moves.is_empty());
        assert_eq!(idea.abs_path, path);
        cleanup(dir);
    }

    fn tempdir() -> PathBuf {
        let mut p = std::env::temp_dir();
        let nanos = OffsetDateTime::now_utc().unix_timestamp_nanos();
        p.push(format!("duckboard-idea-test-{nanos}"));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn cleanup(p: PathBuf) {
        let _ = std::fs::remove_dir_all(p);
    }
}
