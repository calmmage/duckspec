//! Session build pilot: arm mode, allowlist policy, TurnComplete decisions.
//!
//! See `chat/build-pilot`. Classification of `/build-*` submits lives in
//! `slash_commands`; this module owns armed state and safe auto-send rules.

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

    pub fn mode(self) -> Option<BuildPilotMode> {
        match self {
            Self::Off => None,
            Self::Armed(m) => Some(m),
        }
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

/// Allowlisted stage slash names (no leading `/`).
const STAGE_BOTH: &[&str] = &[
    "ds-explore",
    "ds-propose",
    "ds-design",
    "ds-spec",
    "ds-step",
    "ds-apply",
    "ds-followup",
];

/// True when `send` is safe to auto-submit for `mode`.
pub fn is_auto_safe(mode: BuildPilotMode, send: &str) -> bool {
    let token = send.trim();
    if token.is_empty() {
        return false;
    }
    if token == "confirm" {
        return true;
    }
    let name = token.strip_prefix('/').unwrap_or(token);
    if name == "ds-review" {
        return matches!(mode, BuildPilotMode::Auto);
    }
    if matches!(name, "ds-archive" | "ds-codex" | "ds-verify") {
        return false;
    }
    STAGE_BOTH.contains(&name)
}

/// Decide whether to auto-send rank-1 or disarm after a settled turn.
pub fn decide(mode: BuildPilotMode, next: &[NextAction]) -> PilotDecision {
    let Some(first) = next.first() else {
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
pub fn maybe_auto_send(pilot: &mut PilotState, next: &[NextAction]) -> Option<String> {
    let PilotState::Armed(mode) = *pilot else {
        return None;
    };
    match decide(mode, next) {
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
/// helper remains for pure kick+arm policy tests.
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
            decide(BuildPilotMode::Auto, &[]),
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
            decide(BuildPilotMode::Auto, &[na("/ds-spec")]),
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

    /// @spec chat/build-pilot Safe auto-send: Rank-1 confirm auto-sends while armed
    #[test]
    fn rank1_confirm_auto_sends_while_armed() {
        let mut pilot = PilotState::Armed(BuildPilotMode::Auto);
        let next = [na("confirm")];
        // WHEN that non-priming agent turn completes
        let send = maybe_auto_send(&mut pilot, &next);
        // THEN a new agent turn prompt is `confirm` AND pilot remains armed
        assert_eq!(send.as_deref(), Some("confirm"));
        assert_eq!(pilot, PilotState::Armed(BuildPilotMode::Auto));
    }

    /// @spec chat/build-pilot Safe auto-send: Rank-1 allowlisted stage slash auto-sends in auto mode
    #[test]
    fn rank1_allowlisted_stage_slash_auto_sends_in_auto_mode() {
        let mut pilot = PilotState::Armed(BuildPilotMode::Auto);
        let next = [na("/ds-spec")];
        let send = maybe_auto_send(&mut pilot, &next);
        assert_eq!(send.as_deref(), Some("/ds-spec"));
        assert!(pilot.is_armed());
    }

    /// @spec chat/build-pilot Safe auto-send: Rank-1 ds-review auto-sends only in auto mode
    #[test]
    fn rank1_ds_review_auto_sends_only_in_auto_mode() {
        let mut pilot = PilotState::Armed(BuildPilotMode::Auto);
        let next = [na("/ds-review")];
        let send = maybe_auto_send(&mut pilot, &next);
        assert_eq!(send.as_deref(), Some("/ds-review"));
        assert!(pilot.is_armed());
    }

    /// @spec chat/build-pilot Safe auto-send: Rank-1 ds-review disarms in fast mode without sending
    #[test]
    fn rank1_ds_review_disarms_in_fast_mode_without_sending() {
        let mut pilot = PilotState::Armed(BuildPilotMode::Fast);
        let next = [na("/ds-review")];
        let send = maybe_auto_send(&mut pilot, &next);
        assert!(send.is_none());
        assert_eq!(pilot, PilotState::Off);
    }

    /// @spec chat/build-pilot Safe auto-send: Rank-1 archive codex or verify disarms without sending
    #[test]
    fn rank1_archive_codex_or_verify_disarms_without_sending() {
        let mut pilot = PilotState::Armed(BuildPilotMode::Auto);
        let next = [na("/ds-archive")];
        let send = maybe_auto_send(&mut pilot, &next);
        assert!(send.is_none());
        assert_eq!(pilot, PilotState::Off);
    }

    /// @spec chat/build-pilot Safe auto-send: Missing or unsafe rank-1 disarms without sending
    #[test]
    fn missing_or_unsafe_rank1_disarms_without_sending() {
        let mut pilot = PilotState::Armed(BuildPilotMode::Auto);
        let send = maybe_auto_send(&mut pilot, &[]);
        assert!(send.is_none());
        assert_eq!(pilot, PilotState::Off);
    }

    /// @spec chat/build-pilot Disarm controls: Disarm stays off until build command relaunch
    #[test]
    fn disarm_stays_off_until_build_command_relaunch() {
        // GIVEN a session whose pilot was disarmed after an unsafe next action
        let mut pilot = PilotState::Armed(BuildPilotMode::Auto);
        let _ = maybe_auto_send(&mut pilot, &[]);
        assert_eq!(pilot, PilotState::Off);
        // WHEN an ordinary non-build turn completes with a would-be-safe next
        let send = maybe_auto_send(&mut pilot, &[na("confirm")]);
        // THEN pilot remains disarmed (no auto-send without re-launch)
        assert!(send.is_none());
        assert_eq!(pilot, PilotState::Off);
    }
}
