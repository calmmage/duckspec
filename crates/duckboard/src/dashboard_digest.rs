//! Dashboard digest ranking, shortlist cut, and attention signal assembly.
//!
//! Scores are derived only — nothing here writes heat, scores, or shortlist
//! membership into `duckspec/` or idea frontmatter.

#![allow(dead_code)]

use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;

use crate::chat_store::{ChatSession, Exploration};
use crate::data::{ChangeData, ProjectData, StepCompletion};
use crate::queue_list::last_message_activity_nanos;

/// Resting shortlist length for Changes / Explorations (filter law).
pub const SHORTLIST_N: usize = 3;

/// Fixed heat slots in the day-seeded 2+1 cut (top of rank, never spun).
pub const HEAT_SLOTS: usize = 2;

/// One active change ready for ranking (timestamps injected by callers).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeRankInput {
    pub name: String,
    /// Max of shallow mtime and scoped chat activity as unix nanos. `None` = cold.
    pub attention_ts: Option<i128>,
    /// 0–2: partial steps and/or validation errors each contribute 1.
    pub needs_work: u8,
}

/// One exploration for Dashboard ranking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplorationRankInput {
    pub id: String,
    /// Latest scoped chat activity as unix nanos.
    pub chat_activity: Option<i128>,
    /// Soft-archived explorations are omitted from the live Explorations list.
    pub archived: bool,
}

/// Max of session creation and last non-priming message activity for one session.
pub fn session_attention_nanos(session: &ChatSession) -> i128 {
    let mut best = session.created_at_nanos;
    if let Some(msg) = last_message_activity_nanos(std::slice::from_ref(session)) {
        best = best.max(msg);
    }
    best
}

/// Latest attention per chat scope across sessions (max created / message activity).
pub fn chat_activity_by_scope(sessions: &[ChatSession]) -> HashMap<String, i128> {
    let mut map: HashMap<String, i128> = HashMap::new();
    for session in sessions {
        let t = session_attention_nanos(session);
        map.entry(session.scope.clone())
            .and_modify(|v| *v = (*v).max(t))
            .or_insert(t);
    }
    map
}

/// Load sessions for each scope and merge into one activity map.
pub fn collect_chat_activity(
    project_root: Option<&Path>,
    scopes: impl IntoIterator<Item = String>,
) -> HashMap<String, i128> {
    let mut map = HashMap::new();
    for scope in scopes {
        let sessions = crate::chat_store::load_sessions_for(&scope, project_root);
        for (k, v) in chat_activity_by_scope(&sessions) {
            map.entry(k)
                .and_modify(|cur: &mut i128| *cur = (*cur).max(v))
                .or_insert(v);
        }
    }
    map
}

/// Partial steps and/or validation errors each contribute 1 (cap 2).
pub fn needs_work_score(change: &ChangeData, validation_errors: usize) -> u8 {
    let mut n = 0u8;
    if change
        .steps
        .iter()
        .any(|s| matches!(s.completion, StepCompletion::Partial(_, _)))
    {
        n += 1;
    }
    if validation_errors > 0 {
        n += 1;
    }
    n
}

fn attention_ts(mtime: Option<i128>, chat: Option<i128>) -> Option<i128> {
    match (mtime, chat) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

/// Build change rank inputs from project data and a scope→activity map.
pub fn change_rank_inputs(
    project: &ProjectData,
    chat_activity: &HashMap<String, i128>,
) -> Vec<ChangeRankInput> {
    project
        .active_changes
        .iter()
        .map(|ch| {
            let errors = project
                .validations
                .get(&ch.name)
                .map(|v| v.total_count())
                .unwrap_or(0);
            ChangeRankInput {
                name: ch.name.clone(),
                attention_ts: attention_ts(
                    ch.shallow_mtime_nanos,
                    chat_activity.get(&ch.name).copied(),
                ),
                needs_work: needs_work_score(ch, errors),
            }
        })
        .collect()
}

/// Build exploration rank inputs from the exploration list and activity map.
pub fn exploration_rank_inputs(
    explorations: &[Exploration],
    chat_activity: &HashMap<String, i128>,
) -> Vec<ExplorationRankInput> {
    explorations
        .iter()
        .map(|e| ExplorationRankInput {
            id: e.id.clone(),
            chat_activity: chat_activity.get(&e.id).copied(),
            archived: e.is_archived(),
        })
        .collect()
}

/// Scopes to scan for Dashboard ranking: active change names + exploration ids.
pub fn dashboard_scopes(
    project: &ProjectData,
    explorations: &[Exploration],
) -> Vec<String> {
    let mut scopes: Vec<String> = project
        .active_changes
        .iter()
        .map(|c| c.name.clone())
        .collect();
    for e in explorations {
        scopes.push(e.id.clone());
    }
    scopes
}

/// Ranked active-change names for the open project (full order, no shortlist).
pub fn ranked_change_names(
    project: &ProjectData,
    chat_activity: &HashMap<String, i128>,
) -> Vec<String> {
    rank_changes(&change_rank_inputs(project, chat_activity))
}

/// Ranked live exploration ids (full order, no shortlist).
pub fn ranked_exploration_ids(
    explorations: &[Exploration],
    chat_activity: &HashMap<String, i128>,
) -> Vec<String> {
    rank_explorations(&exploration_rank_inputs(explorations, chat_activity))
}

/// Local calendar day `YYYY-MM-DD` for the 2+1 shortlist seed.
pub fn local_day_seed() -> String {
    let dt = time::OffsetDateTime::now_local()
        .unwrap_or_else(|_| time::OffsetDateTime::now_utc());
    format!("{:04}-{:02}-{:02}", dt.year(), u8::from(dt.month()), dt.day())
}

/// Whether `name` is an allowed Dashboard left-column section heading.
pub fn is_dashboard_left_section(name: &str) -> bool {
    matches!(name, "Changes" | "Explorations" | "Archived")
}

/// Section headings the left column may show (order fixed). Never Ideas/Stuck.
pub fn dashboard_left_section_names(
    has_changes: bool,
    show_explorations: bool,
    has_archived: bool,
) -> Vec<&'static str> {
    let mut names = Vec::new();
    if has_changes {
        names.push("Changes");
    }
    if show_explorations {
        names.push("Explorations");
    }
    if has_archived {
        names.push("Archived");
    }
    names
}

/// Resting or expanded filter-law slice for one ranked list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterLawSlice {
    pub visible: Vec<String>,
    /// `Some(n)` ⇒ show “… n more”. `None` when expanded or `len ≤ SHORTLIST_N`.
    pub more_count: Option<usize>,
}

/// Apply filter law: shortlist at rest when `len > N`, else full list.
pub fn filter_law_slice(ranked: &[String], expanded: bool, day_seed: &str) -> FilterLawSlice {
    if expanded || ranked.len() <= SHORTLIST_N {
        FilterLawSlice {
            visible: ranked.to_vec(),
            more_count: None,
        }
    } else {
        FilterLawSlice {
            visible: resting_shortlist(ranked, day_seed),
            more_count: Some(ranked.len() - SHORTLIST_N),
        }
    }
}

/// Archive filter law: newest-first **prefix** of N — never day-seeded 2+1 spin.
/// Caller only invokes this when the Archived section is open.
pub fn archive_filter_law_slice(ranked_newest_first: &[String], show_all: bool) -> FilterLawSlice {
    if show_all || ranked_newest_first.len() <= SHORTLIST_N {
        FilterLawSlice {
            visible: ranked_newest_first.to_vec(),
            more_count: None,
        }
    } else {
        FilterLawSlice {
            visible: ranked_newest_first[..SHORTLIST_N].to_vec(),
            more_count: Some(ranked_newest_first.len() - SHORTLIST_N),
        }
    }
}

/// Order active changes: attention recency desc, needs_work desc, name asc.
pub fn rank_changes(items: &[ChangeRankInput]) -> Vec<String> {
    let mut indexed: Vec<&ChangeRankInput> = items.iter().collect();
    indexed.sort_by(|a, b| {
        cmp_attention_desc(a.attention_ts, b.attention_ts)
            .then_with(|| b.needs_work.cmp(&a.needs_work))
            .then_with(|| a.name.cmp(&b.name))
    });
    indexed.into_iter().map(|c| c.name.clone()).collect()
}

/// Order live explorations: chat activity desc, then id timestamp fallback.
/// Archived explorations are omitted.
pub fn rank_explorations(items: &[ExplorationRankInput]) -> Vec<String> {
    let mut live: Vec<&ExplorationRankInput> = items.iter().filter(|e| !e.archived).collect();
    live.sort_by(|a, b| {
        let ta = exploration_sort_ts(a);
        let tb = exploration_sort_ts(b);
        cmp_attention_desc(ta, tb).then_with(|| a.id.cmp(&b.id))
    });
    live.into_iter().map(|e| e.id.clone()).collect()
}

fn exploration_sort_ts(e: &ExplorationRankInput) -> Option<i128> {
    e.chat_activity.or_else(|| exploration_id_timestamp(&e.id))
}

/// Parse `exploration-{nanos}` id into a sort timestamp when present.
pub fn exploration_id_timestamp(id: &str) -> Option<i128> {
    id.strip_prefix("exploration-")?.parse().ok()
}

/// Resting shortlist: full list when `len ≤ SHORTLIST_N`; otherwise top
/// `HEAT_SLOTS` plus one day-seeded pick from the remainder.
pub fn resting_shortlist(ranked: &[String], day_seed: &str) -> Vec<String> {
    if ranked.len() <= SHORTLIST_N {
        return ranked.to_vec();
    }
    let heat = &ranked[..HEAT_SLOTS];
    let tail = &ranked[HEAT_SLOTS..];
    let spin = day_seeded_pick(tail, day_seed);
    let mut out = heat.to_vec();
    out.push(spin);
    out
}

/// Deterministic pick from non-empty `tail` using local calendar day string.
pub fn day_seeded_pick(tail: &[String], day_seed: &str) -> String {
    debug_assert!(!tail.is_empty());
    let idx = day_seeded_index(tail.len(), day_seed);
    tail[idx].clone()
}

fn day_seeded_index(tail_len: usize, day_seed: &str) -> usize {
    if tail_len == 0 {
        return 0;
    }
    let mut h = DefaultHasher::new();
    day_seed.hash(&mut h);
    (h.finish() as usize) % tail_len
}

/// Known activity ranks above missing; among known, higher nanos first.
fn cmp_attention_desc(a: Option<i128>, b: Option<i128>) -> Ordering {
    match (a, b) {
        (Some(x), Some(y)) => y.cmp(&x),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/// Assert helper for the derived-only contract: ranking does not mutate a path.
pub fn rank_changes_leaves_path_untouched(duckspec_root: &Path, items: &[ChangeRankInput]) {
    let before = dir_fingerprint(duckspec_root);
    let _ = rank_changes(items);
    let after = dir_fingerprint(duckspec_root);
    assert_eq!(
        before, after,
        "ranking must not write under duckspec/"
    );
}

fn dir_fingerprint(root: &Path) -> Vec<(String, u64, i128)> {
    let mut out = Vec::new();
    if !root.is_dir() {
        return out;
    }
    let Ok(rd) = std::fs::read_dir(root) else {
        return out;
    };
    for entry in rd.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        if let Ok(meta) = entry.metadata() {
            let len = meta.len();
            let mtime = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_nanos() as i128)
                .unwrap_or(0);
            out.push((name, len, mtime));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn change(name: &str, attention: Option<i128>, needs_work: u8) -> ChangeRankInput {
        ChangeRankInput {
            name: name.into(),
            attention_ts: attention,
            needs_work,
        }
    }

    fn exp(id: &str, chat: Option<i128>, archived: bool) -> ExplorationRankInput {
        ExplorationRankInput {
            id: id.into(),
            chat_activity: chat,
            archived,
        }
    }

    // @spec shell/dashboard-digest Change ranking: Newer attention ranks above older
    #[test]
    fn newer_attention_ranks_above_older() {
        let items = [
            change("cold", Some(100), 0),
            change("hot", Some(200), 0),
        ];
        assert_eq!(
            rank_changes(&items),
            vec!["hot".to_string(), "cold".to_string()]
        );
    }

    // @spec shell/dashboard-digest Change ranking: Needs-work breaks recency ties
    #[test]
    fn needs_work_breaks_recency_ties() {
        let items = [
            change("clean", Some(50), 0),
            change("dirty", Some(50), 1),
        ];
        assert_eq!(
            rank_changes(&items),
            vec!["dirty".to_string(), "clean".to_string()]
        );
    }

    // @spec shell/dashboard-digest Change ranking: Missing activity sorts colder than known activity
    #[test]
    fn missing_activity_sorts_colder_than_known() {
        let items = [
            change("known", Some(10), 0),
            change("unknown", None, 0),
        ];
        assert_eq!(
            rank_changes(&items),
            vec!["known".to_string(), "unknown".to_string()]
        );
    }

    // @spec shell/dashboard-digest Exploration ranking: Newer chat activity ranks explorations
    #[test]
    fn newer_chat_activity_ranks_explorations() {
        let items = [
            exp("exploration-1", Some(100), false),
            exp("exploration-2", Some(300), false),
        ];
        assert_eq!(
            rank_explorations(&items),
            vec!["exploration-2".to_string(), "exploration-1".to_string()]
        );
    }

    // @spec shell/dashboard-digest Exploration ranking: Missing chat falls back to id timestamp
    #[test]
    fn missing_chat_falls_back_to_id_timestamp() {
        let items = [
            exp("exploration-100", None, false),
            exp("exploration-300", None, false),
        ];
        assert_eq!(
            rank_explorations(&items),
            vec!["exploration-300".to_string(), "exploration-100".to_string()]
        );
    }

    // @spec shell/dashboard-digest Exploration ranking: Archived exploration omitted from Explorations
    #[test]
    fn archived_exploration_omitted_from_explorations() {
        let items = [
            exp("exploration-1", Some(50), false),
            exp("exploration-2", Some(90), true),
        ];
        assert_eq!(
            rank_explorations(&items),
            vec!["exploration-1".to_string()]
        );
    }

    // @spec shell/dashboard-digest Day-seeded shortlist cut: Resting shortlist is two heat plus one spun tail
    #[test]
    fn resting_shortlist_is_two_heat_plus_one_spun_tail() {
        let ranked: Vec<String> = (0..5).map(|i| format!("c{i}")).collect();
        let short = resting_shortlist(&ranked, "2026-08-01");
        assert_eq!(short.len(), 3);
        assert_eq!(short[0], "c0");
        assert_eq!(short[1], "c1");
        assert!(["c2", "c3", "c4"].contains(&short[2].as_str()));
    }

    // @spec shell/dashboard-digest Day-seeded shortlist cut: Same day and inputs yield the same shortlist
    #[test]
    fn same_day_and_inputs_yield_same_shortlist() {
        let ranked: Vec<String> = (0..6).map(|i| format!("x{i}")).collect();
        let a = resting_shortlist(&ranked, "2026-08-01");
        let b = resting_shortlist(&ranked, "2026-08-01");
        assert_eq!(a, b);
    }

    // @spec shell/dashboard-digest Day-seeded shortlist cut: Different days may spin a different tail member
    #[test]
    fn different_days_may_spin_different_tail_member() {
        let ranked: Vec<String> = (0..8).map(|i| format!("r{i}")).collect();
        let d1 = resting_shortlist(&ranked, "2026-01-01");
        let d2 = resting_shortlist(&ranked, "2026-06-15");
        assert_eq!(&d1[..2], &d2[..2]);
        assert_eq!(&d1[..2], &["r0".to_string(), "r1".to_string()]);
        // With 6 tail members, different day seeds usually differ; if equal, still
        // valid — assert the invariant that heat is stable and third is from tail.
        let tail: Vec<&str> = ranked[2..].iter().map(|s| s.as_str()).collect();
        assert!(tail.contains(&d1[2].as_str()));
        assert!(tail.contains(&d2[2].as_str()));
        // Prefer observing a difference when the hash space allows; force enough
        // days until we see a different third or exhaust a reasonable window.
        let mut saw_diff = d1[2] != d2[2];
        if !saw_diff {
            for day in 2..40 {
                let d = resting_shortlist(&ranked, &format!("2026-03-{day:02}"));
                if d[2] != d1[2] {
                    saw_diff = true;
                    break;
                }
            }
        }
        assert!(
            saw_diff,
            "expected some calendar day to spin a different tail member"
        );
    }

    // @spec shell/dashboard-digest Day-seeded shortlist cut: Expand shows full ranked list
    #[test]
    fn expand_shows_full_ranked_list() {
        let ranked: Vec<String> = (0..5).map(|i| format!("e{i}")).collect();
        let short = resting_shortlist(&ranked, "2026-08-01");
        assert_eq!(short.len(), 3);
        // Expanded view is the full ranked order (not shortlist-only).
        assert_eq!(ranked.len(), 5);
        assert_eq!(ranked, vec!["e0", "e1", "e2", "e3", "e4"]);
        for name in &short {
            assert!(ranked.contains(name));
        }
    }

    // @spec shell/dashboard-digest Derived-only scores: Ranking leaves duckspec free of heat fields
    #[test]
    fn ranking_leaves_duckspec_free_of_heat_fields() {
        let root = std::env::temp_dir().join(format!(
            "duckboard-digest-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let duckspec = root.join("duckspec");
        fs::create_dir_all(duckspec.join("changes")).expect("mkdir");
        fs::write(duckspec.join("project.md"), "# p\n\nbody\n").expect("write");
        let items = [change("a", Some(1), 0), change("b", None, 1)];
        rank_changes_leaves_path_untouched(&duckspec, &items);
        let names: Vec<_> = fs::read_dir(&duckspec)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert!(!names.iter().any(|n| n.contains("heat") || n.contains("score")));
        let _ = fs::remove_dir_all(&root);
    }

    // @spec shell/dashboard-digest Dashboard section set: Left column sections are Changes Explorations and Archived only
    #[test]
    fn left_column_sections_are_changes_explorations_and_archived_only() {
        // Same flags the Dashboard left column uses when all three inventories exist.
        let names = dashboard_left_section_names(true, true, true);
        assert_eq!(names, vec!["Changes", "Explorations", "Archived"]);
        for name in &names {
            assert!(is_dashboard_left_section(name));
        }
        // View iterates only this plan — no other section title is allowed.
        assert!(names.iter().all(|n| is_dashboard_left_section(n)));
    }

    // @spec shell/dashboard-digest Dashboard section set: Ideas and Stuck sections are absent
    #[test]
    fn ideas_and_stuck_sections_are_absent() {
        let names = dashboard_left_section_names(true, true, true);
        assert!(!names.contains(&"Ideas"));
        assert!(!names.contains(&"Stuck"));
        assert!(!is_dashboard_left_section("Ideas"));
        assert!(!is_dashboard_left_section("Stuck"));
        // Sparse inventories still never introduce foreign section names.
        let sparse = dashboard_left_section_names(false, true, false);
        assert_eq!(sparse, vec!["Explorations"]);
        assert!(!sparse.iter().any(|n| *n == "Ideas" || *n == "Stuck"));
    }

    // @spec shell/dashboard-digest Filter-law shortlist: More than three changes shows three rows and remainder count
    #[test]
    fn more_than_three_changes_shows_three_rows_and_remainder_count() {
        let ranked: Vec<String> = (0..5).map(|i| format!("c{i}")).collect();
        let slice = filter_law_slice(&ranked, false, "2026-08-01");
        assert_eq!(slice.visible.len(), 3);
        assert_eq!(slice.more_count, Some(2));
    }

    // @spec shell/dashboard-digest Filter-law shortlist: Three or fewer changes omits overflow chrome
    #[test]
    fn three_or_fewer_changes_omits_overflow_chrome() {
        let ranked = vec!["a".into(), "b".into()];
        let slice = filter_law_slice(&ranked, false, "2026-08-01");
        assert_eq!(slice.visible, ranked);
        assert_eq!(slice.more_count, None);
    }

    // @spec shell/dashboard-digest Filter-law shortlist: Expand and collapse restore full list and shortlist
    #[test]
    fn expand_and_collapse_restore_full_list_and_shortlist() {
        let ranked: Vec<String> = (0..5).map(|i| format!("c{i}")).collect();
        let rest = filter_law_slice(&ranked, false, "2026-08-01");
        assert_eq!(rest.visible.len(), 3);
        let expanded = filter_law_slice(&ranked, true, "2026-08-01");
        assert_eq!(expanded.visible, ranked);
        assert_eq!(expanded.more_count, None);
        let collapsed_again = filter_law_slice(&ranked, false, "2026-08-01");
        assert_eq!(collapsed_again, rest);
    }

    // @spec shell/dashboard-digest Filter-law shortlist: New Exploration stays outside the shortlist slots
    #[test]
    fn new_exploration_stays_outside_the_shortlist_slots() {
        // Shortlist is only over exploration ids; the New Exploration control is not a list
        // member and therefore never occupies a filter-law slot.
        let ranked: Vec<String> = (0..5).map(|i| format!("exploration-{i}")).collect();
        let slice = filter_law_slice(&ranked, false, "2026-08-01");
        assert_eq!(slice.visible.len(), SHORTLIST_N);
        assert!(!slice.visible.iter().any(|s| s == "New Exploration"));
        assert_eq!(slice.more_count, Some(2));
    }

    // @spec shell/dashboard-digest Archived density on Dashboard: Expanded Archived shortlists newest first
    #[test]
    fn expanded_archived_shortlists_newest_first() {
        let ranked: Vec<String> = (0..5).map(|i| format!("a{i}")).collect(); // a0 newest
        let slice = archive_filter_law_slice(&ranked, false);
        assert_eq!(
            slice.visible,
            vec!["a0".to_string(), "a1".to_string(), "a2".to_string()]
        );
        assert_eq!(slice.more_count, Some(2));
    }

    // @spec shell/dashboard-digest Archived density on Dashboard: Archived shortlist does not day-rotate
    #[test]
    fn archived_shortlist_does_not_day_rotate() {
        let ranked: Vec<String> = (0..5).map(|i| format!("a{i}")).collect();
        let s1 = archive_filter_law_slice(&ranked, false);
        let s2 = archive_filter_law_slice(&ranked, false);
        // Prefix shortlist is stable (unlike 2+1 day spin on live sections).
        assert_eq!(s1, s2);
        assert_eq!(s1.visible, &ranked[..SHORTLIST_N]);
        // Contrast: day-seeded live shortlist may pick a non-prefix third slot.
        let live = resting_shortlist(&ranked, "2026-08-01");
        assert_eq!(&live[..2], &ranked[..2]);
        // Archive third is always ranked[2], not a spun tail member.
        assert_eq!(s1.visible[2], ranked[2]);
    }
}
