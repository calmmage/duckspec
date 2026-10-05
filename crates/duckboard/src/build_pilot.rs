//! Session build pilot: arm mode, unattended auto-send policy, TurnComplete decisions.
//!
//! See `chat/build-pilot`. Classification of `/build-*` submits lives in
//! `slash_commands`; this module owns armed state and auto-send rules (N1 denylist,
//! R3 recommended promotion).

use crate::meta_card::NextAction;
use crate::slash_commands::BuildPilotMode;

/// Ephemeral per-session pilot arm state (not session-file persisted).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PilotState {
    #[default]
    Off,
    Armed(BuildPilotMode),
}

impl PilotState {
    pub fn is_armed(self) -> bool {
        matches!(self, Self::Armed(_))
    }

    /// Quiet composer plaque label when armed (`Build auto` / `Build fast`).
    /// `None` when disarmed — chrome only, never a transcript line.
    pub fn plaque_label(self) -> Option<&'static str> {
        match self {
            Self::Off => None,
            Self::Armed(BuildPilotMode::Auto) => Some("Build auto"),
            Self::Armed(BuildPilotMode::Fast) => Some("Build fast"),
        }
    }
}

/// After TurnComplete, given mode + ordered next actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PilotDecision {
    AutoSend(String),
    Disarm,
}

/// Hard-denylisted bare names (no leading `/`). Exact token after trim + slash strip.
const DENY_BOTH: &[&str] = &["reject", "revise", "ds-archive", "ds-codex", "ds-verify"];

/// Normalized command/token for denylist match: trim, optional leading `/` stripped.
fn token_name(send: &str) -> &str {
    send.trim().strip_prefix('/').unwrap_or(send.trim())
}

/// True when `send` is hard-denylisted for `mode` (never auto-send).
pub fn is_hard_denylist(mode: BuildPilotMode, send: &str) -> bool {
    let name = token_name(send);
    if name.is_empty() {
        return true;
    }
    if DENY_BOTH.contains(&name) {
        return true;
    }
    // Fast mode never auto-sends review.
    if name == "ds-review" && matches!(mode, BuildPilotMode::Fast) {
        return true;
    }
    false
}

/// True when `send` may auto-submit for `mode` (N1: anything non-empty not denylisted).
pub fn is_auto_safe(mode: BuildPilotMode, send: &str) -> bool {
    let token = send.trim();
    if token.is_empty() {
        return false;
    }
    !is_hard_denylist(mode, token)
}

/// True when option send/label text is marked recommended.
pub fn is_recommended_mark(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("(recommended)") || lower.contains("recommended")
}

/// Pick mid-turn user-choice option for an armed pilot (M1): recommended label
/// if any, else the first option. `None` when `options` is empty.
pub fn pick_user_choice_option(options: &[(String, String)]) -> Option<&(String, String)> {
    if options.is_empty() {
        return None;
    }
    options
        .iter()
        .find(|(_, label)| is_recommended_mark(label))
        .or_else(|| options.first())
}

/// Trailing Recommendation letter (A–C) from assistant prose, if any.
///
/// Recognizes lines containing `Recommendation` then `:` then letter, with
/// optional markdown bold stars (e.g. `Recommendation: **B**`,
/// `**Recommendation:** B`). Last matching line wins.
pub fn recommendation_letter(source: &str) -> Option<char> {
    let mut last: Option<char> = None;
    for line in source.lines() {
        let lower = line.to_ascii_lowercase();
        let Some(idx) = lower.find("recommendation") else {
            continue;
        };
        let after = &line[idx + "recommendation".len()..];
        let after = after.trim_start_matches(|c: char| c == '*' || c.is_whitespace());
        let Some(after) = after.strip_prefix(':') else {
            continue;
        };
        let after = after.trim_start_matches(|c: char| c == '*' || c.is_whitespace());
        if let Some(c) = after.chars().next() {
            let u = c.to_ascii_uppercase();
            if matches!(u, 'A' | 'B' | 'C') {
                last = Some(u);
            }
        }
    }
    last
}

/// Leading A–C letter on an option send line (`B — …`, `A - …`).
fn option_send_letter(send: &str) -> Option<char> {
    let c = send.trim().chars().next()?;
    let u = c.to_ascii_uppercase();
    matches!(u, 'A' | 'B' | 'C').then_some(u)
}

fn move_index_to_front(out: &mut Vec<NextAction>, i: usize) {
    if i > 0 {
        let item = out.remove(i);
        out.insert(0, item);
    }
}

/// Promote a recommended option to rank-1; otherwise keep list order.
///
/// 1. Option `send` marked `recommended` / `(recommended)` (first such wins).
/// 2. Else trailing Recommendation letter in `assistant` matching an option letter.
///
/// Single-item and empty lists are returned unchanged.
pub fn promote_recommended(next: &[NextAction], assistant: Option<&str>) -> Vec<NextAction> {
    if next.len() < 2 {
        return next.to_vec();
    }
    let mut out = next.to_vec();
    if let Some(i) = out.iter().position(|a| is_recommended_mark(&a.send)) {
        move_index_to_front(&mut out, i);
        return out;
    }
    if let Some(letter) = assistant.and_then(recommendation_letter)
        && let Some(i) = out
            .iter()
            .position(|a| option_send_letter(&a.send) == Some(letter))
    {
        move_index_to_front(&mut out, i);
    }
    out
}

/// Decide whether to auto-send rank-1 or disarm after a settled turn.
///
/// Applies recommended promotion (option marks + optional assistant Recommendation
/// letter), then N1 denylist policy.
pub fn decide(mode: BuildPilotMode, next: &[NextAction], assistant: Option<&str>) -> PilotDecision {
    let ordered = promote_recommended(next, assistant);
    let Some(first) = ordered.first() else {
        return PilotDecision::Disarm;
    };
    let send = first.send.trim();
    if is_auto_safe(mode, send) {
        PilotDecision::AutoSend(first.send.clone())
    } else {
        PilotDecision::Disarm
    }
}

/// Apply decide to mutable pilot state; returns text to send, if any.
///
/// `assistant` is the last non-priming assistant text (for Recommendation letter).
pub fn maybe_auto_send(
    pilot: &mut PilotState,
    next: &[NextAction],
    assistant: Option<&str>,
) -> Option<String> {
    let PilotState::Armed(mode) = *pilot else {
        return None;
    };
    match decide(mode, next, assistant) {
        PilotDecision::AutoSend(text) => Some(text),
        PilotDecision::Disarm => {
            *pilot = PilotState::Off;
            None
        }
    }
}

/// Join a bare lifecycle head (`ds-explore`) with optional free-text args into
/// empty-send form (`/ds-explore …`).
pub fn join_slash_args(head: &str, args: &str) -> String {
    let name = head.trim().trim_start_matches('/');
    let args = args.trim();
    if args.is_empty() {
        format!("/{name}")
    } else {
        format!("/{name} {args}")
    }
}

/// Rewritten kick prompt when `kick_head` is present (bare lifecycle name).
/// `None` for unsupported scopes (no kick). Does not arm the pilot.
pub fn kick_text(kick_head: Option<&str>, args: &str) -> Option<String> {
    let head = kick_head.filter(|h| !h.trim().is_empty())?;
    Some(join_slash_args(head, args))
}

/// If `kick_head` is present (bare lifecycle name), arm `pilot` and return the
/// rewritten kick text for user bubble + agent prompt. If absent (unsupported
/// scope), leave pilot unchanged and return `None`.
///
/// Product path arms only after send commits (`run_build_pilot_submit`); this
/// helper is for pure kick+arm policy unit tests.
#[cfg(test)]
pub fn try_arm_build_pilot(
    pilot: &mut PilotState,
    kick_head: Option<&str>,
    mode: BuildPilotMode,
    args: &str,
) -> Option<String> {
    let text = kick_text(kick_head, args)?;
    *pilot = PilotState::Armed(mode);
    Some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn na(send: &str) -> NextAction {
        NextAction {
            send: send.into(),
            reason: None,
        }
    }

    #[test]
    fn confirm_safe_both_modes() {
        assert!(is_auto_safe(BuildPilotMode::Auto, "confirm"));
        assert!(is_auto_safe(BuildPilotMode::Fast, "confirm"));
    }

    #[test]
    fn freeform_decision_safe_both_modes() {
        assert!(is_auto_safe(BuildPilotMode::Auto, "confirm proposal"));
        assert!(is_auto_safe(BuildPilotMode::Fast, "confirm proposal"));
    }

    #[test]
    fn review_only_auto() {
        assert!(is_auto_safe(BuildPilotMode::Auto, "/ds-review"));
        assert!(!is_auto_safe(BuildPilotMode::Fast, "/ds-review"));
    }

    #[test]
    fn archive_never_safe() {
        assert!(!is_auto_safe(BuildPilotMode::Auto, "/ds-archive"));
        assert!(!is_auto_safe(BuildPilotMode::Fast, "/ds-archive"));
    }

    #[test]
    fn decide_disarms_on_empty() {
        assert_eq!(
            decide(BuildPilotMode::Auto, &[], None),
            PilotDecision::Disarm
        );
    }

    /// @spec chat/build-pilot Mode plaque: Armed session shows mode plaque above input
    #[test]
    fn armed_session_shows_mode_plaque_above_input() {
        // GIVEN a session armed in auto mode
        let pilot = PilotState::Armed(BuildPilotMode::Auto);
        // WHEN composer chrome is evaluated
        // THEN a build-auto mode plaque is shown above the input
        assert_eq!(pilot.plaque_label(), Some("Build auto"));
        assert_eq!(
            PilotState::Armed(BuildPilotMode::Fast).plaque_label(),
            Some("Build fast")
        );
    }

    /// @spec chat/build-pilot Mode plaque: Disarmed session hides plaque
    #[test]
    fn disarmed_session_hides_plaque() {
        // GIVEN a session with the pilot disarmed
        let pilot = PilotState::Off;
        // WHEN composer chrome is evaluated
        // THEN no build pilot mode plaque is shown
        assert_eq!(pilot.plaque_label(), None);
    }

    #[test]
    fn decide_autosends_allowlisted() {
        assert_eq!(
            decide(BuildPilotMode::Auto, &[na("/ds-spec")], None),
            PilotDecision::AutoSend("/ds-spec".into())
        );
    }

    /// @spec chat/build-pilot Kick and arm: Exploration kick rewrites to ds-explore with args
    #[test]
    fn exploration_kick_rewrites_to_ds_explore_with_args() {
        // GIVEN an exploration chat session ready to send (kick head ds-explore)
        let mut pilot = PilotState::Off;
        // WHEN the user submits `/build-auto add pilot that auto-sends next`
        let kick = try_arm_build_pilot(
            &mut pilot,
            Some("ds-explore"),
            BuildPilotMode::Auto,
            "add pilot that auto-sends next",
        );
        // THEN agent turn prompt is `/ds-explore add pilot that auto-sends next`
        assert_eq!(
            kick.as_deref(),
            Some("/ds-explore add pilot that auto-sends next")
        );
    }

    /// @spec chat/build-pilot Kick and arm: Change kick uses lifecycle head with args
    #[test]
    fn change_kick_uses_lifecycle_head_with_args() {
        // GIVEN a non-archived change whose lifecycle head is `ds-apply`
        let mut pilot = PilotState::Off;
        // WHEN the user submits `/build-fast fix the review`
        let kick = try_arm_build_pilot(
            &mut pilot,
            Some("ds-apply"),
            BuildPilotMode::Fast,
            "fix the review",
        );
        // THEN agent turn prompt is `/ds-apply fix the review`
        assert_eq!(kick.as_deref(), Some("/ds-apply fix the review"));
    }

    /// @spec chat/build-pilot Kick and arm: User bubble shows rewritten kick not build command
    #[test]
    fn user_bubble_shows_rewritten_kick_not_build_command() {
        // GIVEN an exploration chat session ready to send
        let mut pilot = PilotState::Off;
        // WHEN the user submits `/build-auto sketch the feature`
        let kick = try_arm_build_pilot(
            &mut pilot,
            Some("ds-explore"),
            BuildPilotMode::Auto,
            "sketch the feature",
        )
        .expect("kick");
        // THEN the user message is the rewritten kick, not the /build-* string
        assert_eq!(kick, "/ds-explore sketch the feature");
        assert_ne!(kick, "/build-auto sketch the feature");
    }

    /// @spec chat/build-pilot Kick and arm: Successful kick arms the requested mode
    #[test]
    fn successful_kick_arms_the_requested_mode() {
        // GIVEN an exploration chat session with the pilot disarmed
        let mut pilot = PilotState::Off;
        // WHEN the user submits bare `/build-fast`
        let kick = try_arm_build_pilot(&mut pilot, Some("ds-explore"), BuildPilotMode::Fast, "");
        // THEN the session pilot mode is fast AND a kick turn is prepared
        assert_eq!(pilot, PilotState::Armed(BuildPilotMode::Fast));
        assert_eq!(kick.as_deref(), Some("/ds-explore"));
    }

    /// @spec chat/build-pilot Kick and arm: Unsupported scope does not arm or start agent turn
    #[test]
    fn unsupported_scope_does_not_arm_or_start_agent_turn() {
        // GIVEN a caps-scope chat session (no kick head)
        let mut pilot = PilotState::Off;
        // WHEN the user submits `/build-auto`
        let kick = try_arm_build_pilot(&mut pilot, None, BuildPilotMode::Auto, "");
        // THEN pilot remains disarmed AND no kick is prepared (no agent turn)
        assert!(kick.is_none());
        assert_eq!(pilot, PilotState::Off);
    }

    /// @spec chat/build-pilot Unattended auto-send: Rank-1 confirm auto-sends while armed
    #[test]
    fn rank1_confirm_auto_sends_while_armed() {
        let mut pilot = PilotState::Armed(BuildPilotMode::Auto);
        let next = [na("confirm")];
        // WHEN that non-priming agent turn completes
        let send = maybe_auto_send(&mut pilot, &next, None);
        // THEN a new agent turn prompt is `confirm` AND pilot remains armed
        assert_eq!(send.as_deref(), Some("confirm"));
        assert_eq!(pilot, PilotState::Armed(BuildPilotMode::Auto));
    }

    /// @spec chat/build-pilot Unattended auto-send: Rank-1 allowlisted stage slash auto-sends in auto mode
    #[test]
    fn rank1_allowlisted_stage_slash_auto_sends_in_auto_mode() {
        let mut pilot = PilotState::Armed(BuildPilotMode::Auto);
        let next = [na("/ds-spec")];
        let send = maybe_auto_send(&mut pilot, &next, None);
        assert_eq!(send.as_deref(), Some("/ds-spec"));
        assert!(pilot.is_armed());
    }

    /// @spec chat/build-pilot Unattended auto-send: Rank-1 ds-review auto-sends only in auto mode
    #[test]
    fn rank1_ds_review_auto_sends_only_in_auto_mode() {
        let mut pilot = PilotState::Armed(BuildPilotMode::Auto);
        let next = [na("/ds-review")];
        let send = maybe_auto_send(&mut pilot, &next, None);
        assert_eq!(send.as_deref(), Some("/ds-review"));
        assert!(pilot.is_armed());
    }

    /// @spec chat/build-pilot Unattended auto-send: Rank-1 ds-review disarms in fast mode without sending
    #[test]
    fn rank1_ds_review_disarms_in_fast_mode_without_sending() {
        let mut pilot = PilotState::Armed(BuildPilotMode::Fast);
        let next = [na("/ds-review")];
        let send = maybe_auto_send(&mut pilot, &next, None);
        assert!(send.is_none());
        assert_eq!(pilot, PilotState::Off);
    }

    /// @spec chat/build-pilot Unattended auto-send: Rank-1 archive codex or verify disarms without sending
    #[test]
    fn rank1_archive_codex_or_verify_disarms_without_sending() {
        let mut pilot = PilotState::Armed(BuildPilotMode::Auto);
        let next = [na("/ds-archive")];
        let send = maybe_auto_send(&mut pilot, &next, None);
        assert!(send.is_none());
        assert_eq!(pilot, PilotState::Off);
    }

    /// @spec chat/build-pilot Unattended auto-send: Missing rank-1 disarms without sending
    #[test]
    fn missing_rank1_disarms_without_sending() {
        let mut pilot = PilotState::Armed(BuildPilotMode::Auto);
        let send = maybe_auto_send(&mut pilot, &[], None);
        assert!(send.is_none());
        assert_eq!(pilot, PilotState::Off);
    }

    /// @spec chat/build-pilot Unattended auto-send: Rank-1 freeform decision token auto-sends while armed
    #[test]
    fn rank1_freeform_decision_token_auto_sends_while_armed() {
        let mut pilot = PilotState::Armed(BuildPilotMode::Auto);
        let next = [na("confirm proposal")];
        let send = maybe_auto_send(&mut pilot, &next, None);
        assert_eq!(send.as_deref(), Some("confirm proposal"));
        assert_eq!(pilot, PilotState::Armed(BuildPilotMode::Auto));
    }

    /// @spec chat/build-pilot Unattended auto-send: Rank-1 prose option line auto-sends while armed
    #[test]
    fn rank1_prose_option_line_auto_sends_while_armed() {
        let mut pilot = PilotState::Armed(BuildPilotMode::Auto);
        let next = [na("A — harness-scoped resume")];
        let send = maybe_auto_send(&mut pilot, &next, None);
        assert_eq!(send.as_deref(), Some("A — harness-scoped resume"));
        assert!(pilot.is_armed());
    }

    /// @spec chat/build-pilot Unattended auto-send: Recommended option is auto-sent over earlier non-recommended
    #[test]
    fn recommended_option_is_auto_sent_over_earlier_non_recommended() {
        // GIVEN options A then B with B marked recommended
        let mut pilot = PilotState::Armed(BuildPilotMode::Auto);
        let next = [na("A — first path"), na("B — second path (recommended)")];
        // WHEN that non-priming agent turn completes
        let send = maybe_auto_send(&mut pilot, &next, None);
        // THEN the recommended option B is sent
        assert_eq!(send.as_deref(), Some("B — second path (recommended)"));
        assert!(pilot.is_armed());

        // Letter-only: options lack the word recommended; trailing Recommendation: B
        let mut pilot = PilotState::Armed(BuildPilotMode::Auto);
        let next = [na("A — first path"), na("B — second path")];
        let assistant = "\
Pick:

- **A** — first path
- **B** — second path

**Recommendation:** **B**
";
        let send = maybe_auto_send(&mut pilot, &next, Some(assistant));
        assert_eq!(send.as_deref(), Some("B — second path"));
        assert!(pilot.is_armed());
    }

    #[test]
    fn recommendation_letter_parses_common_shapes() {
        assert_eq!(recommendation_letter("Recommendation: **B**\n"), Some('B'));
        assert_eq!(recommendation_letter("**Recommendation:** A\n"), Some('A'));
        assert_eq!(
            recommendation_letter("some text\nRecommendation: C\n"),
            Some('C')
        );
        assert_eq!(recommendation_letter("no rec here"), None);
    }

    #[test]
    fn pick_user_choice_prefers_recommended_then_first() {
        let opts = [
            ("a".into(), "Alpha".into()),
            ("b".into(), "Beta (recommended)".into()),
        ];
        let picked = pick_user_choice_option(&opts).expect("pick");
        assert_eq!(picked.0, "b");

        let no_mark = [("a".into(), "Alpha".into()), ("b".into(), "Beta".into())];
        let first = pick_user_choice_option(&no_mark).expect("pick");
        assert_eq!(first.0, "a");

        assert!(pick_user_choice_option(&[]).is_none());
    }

    /// @spec chat/build-pilot Disarm controls: Disarm stays off until build command relaunch
    #[test]
    fn disarm_stays_off_until_build_command_relaunch() {
        // GIVEN a session whose pilot was disarmed after a missing next action
        let mut pilot = PilotState::Armed(BuildPilotMode::Auto);
        let _ = maybe_auto_send(&mut pilot, &[], None);
        assert_eq!(pilot, PilotState::Off);
        // WHEN an ordinary non-build turn completes with a would-be-safe next
        let send = maybe_auto_send(&mut pilot, &[na("confirm")], None);
        // THEN pilot remains disarmed (no auto-send without re-launch)
        assert!(send.is_none());
        assert_eq!(pilot, PilotState::Off);
    }
}
