//! Shared queue ordering for CHANGE and Ideas lists: star pin ladder, sort
//! keys, and last-message activity.

use std::cmp::Ordering;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::chat_store::{ChatMessage, ChatSession};
use crate::idea_store::IdeaMark;

/// How many starred rows pin above the ordinary segment.
pub const FAV_PIN_LIMIT: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QueueKey {
    Exploration(String),
    Change(String),
    Idea(PathBuf),
}

/// Sort key for the ordinary (non-pin) segment. Default: last message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SortKey {
    #[default]
    LastMessage,
    Phase,
    Created,
}

impl SortKey {
    pub const ALL: [SortKey; 3] = [Self::LastMessage, Self::Phase, Self::Created];

    pub fn label(self) -> &'static str {
        match self {
            SortKey::LastMessage => "Last message",
            SortKey::Phase => "Phase",
            SortKey::Created => "Created",
        }
    }
}

/// Shared list preferences (Change + Ideas).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ListConfig {
    pub sort_key: SortKey,
    pub show_type_pillows: bool,
    pub show_phase_pillows: bool,
}

impl Default for ListConfig {
    fn default() -> Self {
        Self {
            sort_key: SortKey::LastMessage,
            show_type_pillows: true,
            show_phase_pillows: true,
        }
    }
}

/// One row in a sortable queue body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueRowMeta {
    pub key: QueueKey,
    pub title: String,
    pub mark: IdeaMark,
    /// RFC3339 (or comparable) pin time when mark is star.
    pub favored_at: Option<String>,
    pub type_tags: Vec<String>,
    pub phase: Option<String>,
    /// Latest non-priming activity (unix nanos); `None` sorts last under last-message.
    pub last_message_at: Option<i128>,
    /// Creation time string (idea `created` or similar); used for Created key.
    pub created_at: Option<String>,
}

/// Max graphemes for a type-tag pillow label before ellipsis.
pub const TYPE_TAG_DISPLAY_MAX: usize = 16;

/// Default character budget for title + pillows on a steady row before overflow.
pub const DEFAULT_ROW_CHAR_BUDGET: usize = 36;

/// Type/phase pillows projected for a queue row under prefs.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RowPillows {
    pub type_tags: Vec<String>,
    pub phase: Option<String>,
}

impl RowPillows {
    pub fn is_empty(&self) -> bool {
        self.type_tags.is_empty() && self.phase.is_none()
    }
}

/// Secondary tags only — primary (index 0) is tree/filing, never a pillow.
pub fn type_tags_from_idea_tags(tags: &[String]) -> Vec<String> {
    tags.iter().skip(1).cloned().collect()
}

/// Project type + phase pillows under list prefs.
pub fn project_row_pillows(
    tags: &[String],
    phase: Option<&str>,
    prefs: &ListConfig,
) -> RowPillows {
    let type_tags = if prefs.show_type_pillows {
        type_tags_from_idea_tags(tags)
    } else {
        Vec::new()
    };
    let phase = if prefs.show_phase_pillows {
        phase
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    } else {
        None
    };
    RowPillows { type_tags, phase }
}

/// Truncate a tag for dense pillow display.
pub fn truncate_tag_display(tag: &str, max_chars: usize) -> String {
    let count = tag.chars().count();
    if count <= max_chars {
        return tag.to_string();
    }
    let keep = max_chars.saturating_sub(1);
    tag.chars().take(keep).collect::<String>() + "…"
}

fn pillow_char_cost(pillows: &RowPillows) -> usize {
    let tags: usize = pillows
        .type_tags
        .iter()
        .map(|t| truncate_tag_display(t, TYPE_TAG_DISPLAY_MAX).chars().count() + 3)
        .sum();
    let phase = pillows
        .phase
        .as_ref()
        .map(|p| p.chars().count() + 3)
        .unwrap_or(0);
    tags + phase
}

/// Whether title + enabled pillows exceed the row character budget.
pub fn pillows_overflow(title: &str, pillows: &RowPillows, char_budget: usize) -> bool {
    if pillows.is_empty() {
        return false;
    }
    title.chars().count() + pillow_char_cost(pillows) > char_budget
}

/// Pillows to paint on the steady row. When over budget, hide until hover.
pub fn steady_row_pillows<'a>(
    title: &str,
    pillows: &'a RowPillows,
    char_budget: usize,
    hovered: bool,
) -> Option<&'a RowPillows> {
    if pillows.is_empty() {
        return None;
    }
    if pillows_overflow(title, pillows, char_budget) && !hovered {
        None
    } else {
        Some(pillows)
    }
}

/// Glyph for exclusive mark; outline star when unmarked and hovered.
pub fn mark_glyph(mark: IdeaMark, hovered: bool) -> Option<&'static str> {
    match mark {
        IdeaMark::Star => Some("⭐"),
        IdeaMark::Hot => Some("🔥"),
        IdeaMark::Cool => Some("❄"),
        IdeaMark::None if hovered => Some("☆"),
        IdeaMark::None => None,
    }
}

/// Display title with optional mark prefix (does not mutate storage).
///
/// Prefer the interactive `after_icon` mark control in list rows; do not also
/// prefix the label (that double-paints the star). Kept for tests / callers
/// that want a single-string rendering of title + mark.
pub fn title_with_mark(title: &str, mark: IdeaMark, hovered: bool) -> String {
    match mark_glyph(mark, hovered) {
        Some(g) => format!("{g} {title}"),
        None => title.to_string(),
    }
}

/// Stable rank for derived duckspec phase strings (lower = earlier lifecycle).
/// Unknown phases sort after known ones; missing phase after all phased rows.
pub fn phase_rank(phase: Option<&str>) -> u8 {
    match phase {
        Some("newly created, no artifacts yet") => 0,
        Some("proposal drafted, design not yet written") => 1,
        Some("design drafted, specs not yet written") => 2,
        Some("specs drafted, steps not yet written") => 3,
        Some("implementing steps") => 4,
        Some("all steps complete") => 5,
        Some("all steps complete, review on file") => 6,
        Some("review on file, no open steps") => 7,
        Some(_) => 50,
        None => 100,
    }
}

/// Max activity time across sessions for a chat scope: latest non-priming
/// message. Uses parsed message timestamps when present; otherwise the
/// owning session's `created_at_nanos` as a stand-in.
pub fn last_message_activity_nanos(sessions: &[ChatSession]) -> Option<i128> {
    let mut best: Option<i128> = None;
    for session in sessions {
        for msg in &session.messages {
            if msg.is_priming {
                continue;
            }
            // Count user/assistant/system content turns; priming already skipped.
            let t = parse_message_activity_nanos(msg).unwrap_or(session.created_at_nanos);
            best = Some(best.map_or(t, |b| b.max(t)));
        }
    }
    best
}

fn parse_message_activity_nanos(msg: &ChatMessage) -> Option<i128> {
    let ts = msg.timestamp.trim();
    if ts.is_empty() {
        return None;
    }
    // Prefer integer nanos if stored that way.
    if let Ok(n) = ts.parse::<i128>() {
        return Some(n);
    }
    // Fail open on non-numeric timestamps so session created_at is used.
    let _ = msg;
    None
}

/// Sort `rows` in place: up to [`FAV_PIN_LIMIT`] newest stars first (by
/// `favored_at` desc), then ordinary segment by `key`, title asc as tie-break.
pub fn sort_queue(rows: &mut [QueueRowMeta], key: SortKey) {
    if rows.is_empty() {
        return;
    }

    let mut star_idxs: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter(|(_, r)| r.mark == IdeaMark::Star)
        .map(|(i, _)| i)
        .collect();
    star_idxs.sort_by(|&a, &b| {
        cmp_favored_at_desc(&rows[a], &rows[b]).then_with(|| rows[a].title.cmp(&rows[b].title))
    });
    // Newest favored stars first; only the first FAV_PIN_LIMIT pin.
    let pin_order: Vec<QueueKey> = star_idxs
        .into_iter()
        .take(FAV_PIN_LIMIT)
        .map(|i| rows[i].key.clone())
        .collect();

    let mut indices: Vec<usize> = (0..rows.len()).collect();
    indices.sort_by(|&ia, &ib| {
        let a = &rows[ia];
        let b = &rows[ib];
        let a_pin = pin_order.iter().position(|k| k == &a.key);
        let b_pin = pin_order.iter().position(|k| k == &b.key);
        match (a_pin, b_pin) {
            (Some(i), Some(j)) => i.cmp(&j),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => ordinary_cmp(a, b, key),
        }
    });

    let sorted: Vec<QueueRowMeta> = indices.into_iter().map(|i| rows[i].clone()).collect();
    for (slot, row) in sorted.into_iter().enumerate() {
        rows[slot] = row;
    }
}

fn cmp_favored_at_desc(a: &QueueRowMeta, b: &QueueRowMeta) -> Ordering {
    // Newest first: reverse string compare works for zero-padded ISO timestamps.
    b.favored_at.cmp(&a.favored_at)
}

fn ordinary_cmp(a: &QueueRowMeta, b: &QueueRowMeta, key: SortKey) -> Ordering {
    let primary = match key {
        SortKey::LastMessage => cmp_last_message_desc(a, b),
        SortKey::Phase => phase_rank(a.phase.as_deref()).cmp(&phase_rank(b.phase.as_deref())),
        SortKey::Created => cmp_created_desc(a, b),
    };
    primary.then_with(|| a.title.cmp(&b.title))
}

fn cmp_last_message_desc(a: &QueueRowMeta, b: &QueueRowMeta) -> Ordering {
    match (a.last_message_at, b.last_message_at) {
        (Some(x), Some(y)) => y.cmp(&x),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn cmp_created_desc(a: &QueueRowMeta, b: &QueueRowMeta) -> Ordering {
    match (&a.created_at, &b.created_at) {
        (Some(x), Some(y)) => y.cmp(x),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat_store::{ContentBlock, Role};
    use crate::idea_store::IdeaMark;

    fn row(
        key: QueueKey,
        title: &str,
        mark: IdeaMark,
        favored_at: Option<&str>,
        last: Option<i128>,
        phase: Option<&str>,
        created: Option<&str>,
    ) -> QueueRowMeta {
        QueueRowMeta {
            key,
            title: title.into(),
            mark,
            favored_at: favored_at.map(str::to_string),
            type_tags: vec![],
            phase: phase.map(str::to_string),
            last_message_at: last,
            created_at: created.map(str::to_string),
        }
    }

    fn ch(name: &str) -> QueueKey {
        QueueKey::Change(name.into())
    }

    fn exp(id: &str) -> QueueKey {
        QueueKey::Exploration(id.into())
    }

    fn key_str(k: &QueueKey) -> String {
        match k {
            QueueKey::Change(n) | QueueKey::Exploration(n) => n.clone(),
            QueueKey::Idea(p) => p.display().to_string(),
        }
    }

    #[test]
    fn idea_queue_key_round_trips_display() {
        let k = QueueKey::Idea(PathBuf::from("/ideas/inbox/x.md"));
        assert!(key_str(&k).contains("x.md"));
    }

    // @spec ideas/queue-list Star pin before ordinary sort: Up to three newest stars pin above non-pinned rows
    #[test]
    fn up_to_three_newest_stars_pin_above_non_pinned_rows() {
        let mut rows = vec![
            row(ch("a"), "A", IdeaMark::None, None, Some(100), None, None),
            row(ch("b"), "B", IdeaMark::Star, Some("2026-01-03"), Some(1), None, None),
            row(exp("c"), "C", IdeaMark::None, None, Some(200), None, None),
            row(ch("d"), "D", IdeaMark::Star, Some("2026-01-02"), Some(2), None, None),
            row(ch("e"), "E", IdeaMark::Star, Some("2026-01-01"), Some(3), None, None),
        ];
        sort_queue(&mut rows, SortKey::LastMessage);
        let keys: Vec<String> = rows.iter().map(|r| key_str(&r.key)).collect();
        // Three stars first (newest favored), then non-stars by last message.
        assert_eq!(&keys[..3], &["b".to_string(), "d".into(), "e".into()]);
        assert_eq!(keys[3], "c"); // newer last message (exploration row)
        assert_eq!(keys[4], "a");
    }

    // @spec ideas/queue-list Star pin before ordinary sort: A fourth star follows ordinary sort among non-pinned rows
    #[test]
    fn a_fourth_star_follows_ordinary_sort_among_non_pinned_rows() {
        let mut rows = vec![
            row(ch("s1"), "S1", IdeaMark::Star, Some("2026-01-04"), Some(10), None, None),
            row(ch("s2"), "S2", IdeaMark::Star, Some("2026-01-03"), Some(20), None, None),
            row(ch("s3"), "S3", IdeaMark::Star, Some("2026-01-02"), Some(30), None, None),
            row(ch("s4"), "S4", IdeaMark::Star, Some("2026-01-01"), Some(50), None, None),
            row(ch("n1"), "N1", IdeaMark::None, None, Some(40), None, None),
        ];
        sort_queue(&mut rows, SortKey::LastMessage);
        let pin_keys: Vec<String> = rows[..3].iter().map(|r| key_str(&r.key)).collect();
        assert_eq!(pin_keys, vec!["s1", "s2", "s3"]);
        // s4 is star but not pinned — ordinary segment by last message with n1
        let rest: Vec<String> = rows[3..].iter().map(|r| key_str(&r.key)).collect();
        assert_eq!(rest, vec!["s4", "n1"]); // 50 > 40
    }

    // @spec ideas/queue-list Star pin before ordinary sort: Pin ranking uses pin time newest-first
    #[test]
    fn pin_ranking_uses_pin_time_newest_first() {
        let mut rows = vec![
            row(ch("old"), "Old", IdeaMark::Star, Some("2026-01-01T00:00:00Z"), None, None, None),
            row(ch("new"), "New", IdeaMark::Star, Some("2026-06-01T00:00:00Z"), None, None, None),
            row(ch("mid"), "Mid", IdeaMark::Star, Some("2026-03-01T00:00:00Z"), None, None, None),
        ];
        sort_queue(&mut rows, SortKey::LastMessage);
        let titles: Vec<_> = rows.iter().map(|r| r.title.as_str()).collect();
        assert_eq!(titles, vec!["New", "Mid", "Old"]);
    }

    // @spec ideas/queue-list Ordinary sort keys: Default key orders by last non-priming message time newest-first
    #[test]
    fn default_key_orders_by_last_non_priming_message_time_newest_first() {
        let mut rows = vec![
            row(ch("old"), "Old", IdeaMark::None, None, Some(10), None, None),
            row(ch("new"), "New", IdeaMark::None, None, Some(99), None, None),
        ];
        sort_queue(&mut rows, SortKey::LastMessage);
        assert_eq!(rows[0].title, "New");
        assert_eq!(rows[1].title, "Old");
    }

    // @spec ideas/queue-list Ordinary sort keys: Missing activity sorts after rows with activity under last-message key
    #[test]
    fn missing_activity_sorts_after_rows_with_activity_under_last_message_key() {
        let mut rows = vec![
            row(ch("none"), "None", IdeaMark::None, None, None, None, None),
            row(ch("has"), "Has", IdeaMark::None, None, Some(1), None, None),
        ];
        sort_queue(&mut rows, SortKey::LastMessage);
        assert_eq!(rows[0].title, "Has");
        assert_eq!(rows[1].title, "None");
    }

    // @spec ideas/queue-list Ordinary sort keys: Phase key orders by lifecycle phase then title
    #[test]
    fn phase_key_orders_by_lifecycle_phase_then_title() {
        let mut rows = vec![
            row(
                ch("b"),
                "Beta",
                IdeaMark::None,
                None,
                None,
                Some("implementing steps"),
                None,
            ),
            row(
                ch("a"),
                "Alpha",
                IdeaMark::None,
                None,
                None,
                Some("proposal drafted, design not yet written"),
                None,
            ),
            row(
                ch("c"),
                "Charlie",
                IdeaMark::None,
                None,
                None,
                Some("proposal drafted, design not yet written"),
                None,
            ),
            row(ch("z"), "Zed", IdeaMark::None, None, None, None, None),
        ];
        sort_queue(&mut rows, SortKey::Phase);
        let titles: Vec<_> = rows.iter().map(|r| r.title.as_str()).collect();
        assert_eq!(titles, vec!["Alpha", "Charlie", "Beta", "Zed"]);
    }

    // @spec ideas/queue-list Ordinary sort keys: Created key orders by creation time
    #[test]
    fn created_key_orders_by_creation_time() {
        let mut rows = vec![
            row(ch("a"), "A", IdeaMark::None, None, None, None, Some("2026-01-01")),
            row(ch("b"), "B", IdeaMark::None, None, None, None, Some("2026-06-01")),
        ];
        sort_queue(&mut rows, SortKey::Created);
        assert_eq!(rows[0].title, "B");
        assert_eq!(rows[1].title, "A");
    }

    // @spec ideas/queue-list List preferences and sort menu: Sort key preference is shared by Change and Ideas lists
    #[test]
    fn sort_key_preference_is_shared_by_change_and_ideas_lists() {
        let mut prefs = ListConfig::default();
        assert_eq!(prefs.sort_key, SortKey::LastMessage);
        prefs.sort_key = SortKey::Phase;
        // Same prefs object drives both lists — one field, one value.
        let change_key = prefs.sort_key;
        let ideas_key = prefs.sort_key;
        assert_eq!(change_key, SortKey::Phase);
        assert_eq!(ideas_key, SortKey::Phase);
    }

    // @spec ideas/queue-list List preferences and sort menu: Type and phase pillow visibility prefs default on and are toggled from the menu
    #[test]
    fn type_and_phase_pillow_visibility_prefs_default_on_and_are_toggled() {
        let mut prefs = ListConfig::default();
        assert!(prefs.show_type_pillows);
        assert!(prefs.show_phase_pillows);
        prefs.show_type_pillows = false;
        assert!(!prefs.show_type_pillows);
        assert!(prefs.show_phase_pillows);
    }

    #[test]
    fn last_message_activity_skips_priming_and_uses_session_created() {
        let mut session = ChatSession::new("scope".into());
        session.created_at_nanos = 42;
        session.messages.push(ChatMessage {
            role: Role::User,
            content: vec![ContentBlock::Text("hi".into())],
            timestamp: String::new(),
            is_priming: true,
        });
        session.messages.push(ChatMessage {
            role: Role::User,
            content: vec![ContentBlock::Text("real".into())],
            timestamp: String::new(),
            is_priming: false,
        });
        assert_eq!(last_message_activity_nanos(&[session]), Some(42));
    }

    // @spec ideas/queue-list Type and phase pillows with density: Secondary tags render as type pillows; primary does not
    #[test]
    fn secondary_tags_render_as_type_pillows_primary_does_not() {
        let tags = vec!["parser".into(), "bugfix".into()];
        let prefs = ListConfig::default();
        let pillows = project_row_pillows(&tags, None, &prefs);
        assert_eq!(pillows.type_tags, vec!["bugfix".to_string()]);
        assert!(!pillows.type_tags.iter().any(|t| t == "parser"));
    }

    // @spec ideas/queue-list Type and phase pillows with density: Change-linked row can show derived phase pillow when pref is on
    #[test]
    fn change_linked_row_can_show_derived_phase_pillow_when_pref_is_on() {
        let prefs = ListConfig::default();
        assert!(prefs.show_phase_pillows);
        let pillows = project_row_pillows(
            &[],
            Some("proposal drafted, design not yet written"),
            &prefs,
        );
        assert_eq!(
            pillows.phase.as_deref(),
            Some("proposal drafted, design not yet written")
        );
        let mut off = prefs;
        off.show_phase_pillows = false;
        let hidden = project_row_pillows(
            &[],
            Some("proposal drafted, design not yet written"),
            &off,
        );
        assert!(hidden.phase.is_none());
    }

    // @spec ideas/queue-list Type and phase pillows with density: When title plus pillows overflow row width, pillows hide until row hover
    #[test]
    fn when_title_plus_pillows_overflow_row_width_pillows_hide_until_row_hover() {
        let pillows = RowPillows {
            type_tags: vec!["verylongsecondarytagname".into(), "another".into()],
            phase: Some("implementing steps".into()),
        };
        let title = "A fairly long queue title for overflow";
        assert!(pillows_overflow(title, &pillows, DEFAULT_ROW_CHAR_BUDGET));
        assert!(steady_row_pillows(title, &pillows, DEFAULT_ROW_CHAR_BUDGET, false).is_none());
        assert!(steady_row_pillows(title, &pillows, DEFAULT_ROW_CHAR_BUDGET, true).is_some());
    }

    // @spec ideas/queue-list Type and phase pillows with density: Hidden pillow prefs suppress those pillows even when space remains
    #[test]
    fn hidden_pillow_prefs_suppress_those_pillows_even_when_space_remains() {
        let tags = vec!["primary".into(), "feature".into()];
        let mut prefs = ListConfig::default();
        prefs.show_type_pillows = false;
        prefs.show_phase_pillows = true;
        let pillows = project_row_pillows(&tags, Some("implementing steps"), &prefs);
        assert!(pillows.type_tags.is_empty());
        assert_eq!(pillows.phase.as_deref(), Some("implementing steps"));
        // Plenty of budget — type still suppressed by pref.
        assert!(
            steady_row_pillows("Short", &pillows, 200, false)
                .unwrap()
                .type_tags
                .is_empty()
        );
    }

    #[test]
    fn mark_glyph_and_title_prefix() {
        assert_eq!(mark_glyph(IdeaMark::None, false), None);
        assert_eq!(mark_glyph(IdeaMark::None, true), Some("☆"));
        assert_eq!(mark_glyph(IdeaMark::Hot, false), Some("🔥"));
        assert_eq!(
            title_with_mark("Fix", IdeaMark::Star, false),
            "⭐ Fix"
        );
    }
}
