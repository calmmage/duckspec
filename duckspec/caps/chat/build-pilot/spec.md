# Chat build pilot

Session pilot that arms from system `/build-auto` / `/build-fast`, kicks a rewritten
workflow turn, and keeps auto-advancing the session — rank-1 next actions, recommended or
first A–B–C options, and mid-turn choice chips — until a real stop, including while
another change or window is focused.

## Requirement: Kick and arm

Submitting `/build-auto` or `/build-fast` with optional free-text arguments on an
exploration or non-archived change session SHALL arm the pilot in the matching mode and
SHALL start one agent turn whose user-visible text and agent prompt are the rewritten kick
command — not the `/build-*` submit string. On exploration the kick command SHALL be
`/ds-explore` plus any arguments. On a change the kick command SHALL be the session's
lifecycle head (empty-send form with a leading `/`) plus any arguments. On a scope that
has no kick command (including caps, codex, and archived changes) the submit SHALL NOT arm
the pilot and SHALL NOT start an agent turn.

> test: code

### Scenario: Exploration kick rewrites to ds-explore with args

- **GIVEN** an exploration chat session ready to send

- **WHEN** the user submits `/build-auto add pilot that auto-sends next`

- **THEN** an agent turn is started whose prompt is
  `/ds-explore add pilot that auto-sends next`

> test: code
> - crates/duckboard/src/build_pilot.rs:308

### Scenario: Change kick uses lifecycle head with args

- **GIVEN** a non-archived change chat session whose lifecycle head is `ds-apply`
- **WHEN** the user submits `/build-fast fix the review`
- **THEN** an agent turn is started whose prompt is `/ds-apply fix the review`

> test: code
> - crates/duckboard/src/build_pilot.rs:327

### Scenario: User bubble shows rewritten kick not build command

- **GIVEN** an exploration chat session ready to send
- **WHEN** the user submits `/build-auto sketch the feature`
- **THEN** the user message recorded for that submit is `/ds-explore sketch the feature`
- **AND** that user message text is not `/build-auto sketch the feature`

> test: code
> - crates/duckboard/src/build_pilot.rs:343

### Scenario: Successful kick arms the requested mode

- **GIVEN** an exploration chat session with the pilot disarmed
- **WHEN** the user submits bare `/build-fast`
- **THEN** the session pilot mode is fast
- **AND** an agent turn is started

> test: code
> - crates/duckboard/src/build_pilot.rs:361

### Scenario: Unsupported scope does not arm or start agent turn

- **GIVEN** a caps-scope chat session ready to send
- **WHEN** the user submits `/build-auto`
- **THEN** the session pilot remains disarmed
- **AND** no agent turn is started for that submit

> test: code
> - crates/duckboard/src/build_pilot.rs:373

## Requirement: Unattended auto-send

While the pilot is armed, after a non-priming agent turn completes without a user cancel,
duckboard SHALL inspect the rank-1 next action for that turn (after recommended promotion
when applicable). When rank-1 is present and is not hard-denylisted for the armed mode,
duckboard SHALL start a new agent turn with that exact send text as both user-visible
message and agent prompt, and SHALL keep the pilot armed. Hard denylist in both modes:
`reject`, `revise`, `/ds-archive`, `/ds-codex`, and `/ds-verify` (exact token after trim;
optional leading `/` normalized). Hard denylist in fast mode only: `/ds-review`. Any other
non-empty rank-1 send text, including freeform decision tokens and prose option lines,
SHALL auto-send. When the next-action list has multiple A–B–C-style options, duckboard
SHALL promote a recommended option to rank-1 before decide when one is marked
(`recommended` / `(recommended)` in the option text, or a trailing Recommendation letter
line); otherwise rank-1 remains the first list entry. When there is no rank-1 next action,
or rank-1 is hard-denylisted for the mode, duckboard SHALL disarm the pilot and SHALL NOT
auto-send.

> test: code

### Scenario: Rank-1 confirm auto-sends while armed

- **GIVEN** a session armed in auto mode
- **AND** the completed turn's trailing next actions rank-1 send text is `confirm`
- **WHEN** that non-priming agent turn completes
- **THEN** a new agent turn is started whose prompt is `confirm`
- **AND** the pilot remains armed

> test: code
> - crates/duckboard/src/build_pilot.rs:385

### Scenario: Rank-1 allowlisted stage slash auto-sends in auto mode

- **GIVEN** a session armed in auto mode
- **AND** the completed turn's trailing next actions rank-1 send text is `/ds-spec`
- **WHEN** that non-priming agent turn completes
- **THEN** a new agent turn is started whose prompt is `/ds-spec`

> test: code
> - crates/duckboard/src/build_pilot.rs:397

### Scenario: Rank-1 ds-review auto-sends only in auto mode

- **GIVEN** a session armed in auto mode
- **AND** the completed turn's trailing next actions rank-1 send text is `/ds-review`
- **WHEN** that non-priming agent turn completes
- **THEN** a new agent turn is started whose prompt is `/ds-review`

> test: code
> - crates/duckboard/src/build_pilot.rs:407

### Scenario: Rank-1 ds-review disarms in fast mode without sending

- **GIVEN** a session armed in fast mode
- **AND** the completed turn's trailing next actions rank-1 send text is `/ds-review`
- **WHEN** that non-priming agent turn completes
- **THEN** no auto-sent agent turn is started for that completion
- **AND** the session pilot is disarmed

> test: code
> - crates/duckboard/src/build_pilot.rs:417

### Scenario: Rank-1 archive codex or verify disarms without sending

- **GIVEN** a session armed in auto mode
- **AND** the completed turn's trailing next actions rank-1 send text is `/ds-archive`
- **WHEN** that non-priming agent turn completes
- **THEN** no auto-sent agent turn is started for that completion
- **AND** the session pilot is disarmed

> test: code
> - crates/duckboard/src/build_pilot.rs:427

### Scenario: Missing rank-1 disarms without sending

- **GIVEN** a session armed in auto mode
- **AND** the completed turn has no trailing next actions
- **WHEN** that non-priming agent turn completes
- **THEN** no auto-sent agent turn is started for that completion
- **AND** the session pilot is disarmed

> test: code
> - crates/duckboard/src/build_pilot.rs:437

### Scenario: Rank-1 freeform decision token auto-sends while armed

- **GIVEN** a session armed in auto mode

- **AND** the completed turn's trailing next actions rank-1 send text is
  `confirm proposal`

- **WHEN** that non-priming agent turn completes

- **THEN** a new agent turn is started whose prompt is `confirm proposal`

- **AND** the pilot remains armed

> test: code
> - crates/duckboard/src/build_pilot.rs:446

### Scenario: Rank-1 prose option line auto-sends while armed

- **GIVEN** a session armed in auto mode

- **AND** the completed turn's trailing next actions rank-1 send text is
  `A — harness-scoped resume`

- **WHEN** that non-priming agent turn completes

- **THEN** a new agent turn is started whose prompt is `A — harness-scoped resume`

- **AND** the pilot remains armed

> test: code
> - crates/duckboard/src/build_pilot.rs:456

### Scenario: Recommended option is auto-sent over earlier non-recommended

- **GIVEN** a session armed in auto mode

- **AND** the completed turn exposes next actions for options A then B with B marked
  recommended

- **WHEN** that non-priming agent turn completes

- **THEN** a new agent turn is started whose prompt is the recommended option B send text

- **AND** the pilot remains armed

> test: code
> - crates/duckboard/src/build_pilot.rs:466

## Requirement: Disarm controls

Esc-Esc SHALL disarm an armed pilot. When a main agent turn is streaming, Esc-Esc SHALL
also cancel that turn. When the pilot is armed and no main turn is streaming, Esc-Esc
SHALL disarm without starting or cancelling an agent turn. Any user cancel of a streaming
main agent turn (including the composer cancel control and the same cancel path as
Esc-Esc's cancel) SHALL disarm the pilot. When that cancelled turn completes, the pilot
SHALL NOT auto-send a next action for that completion. After the pilot is disarmed for any
reason, it SHALL remain disarmed until the user successfully submits `/build-auto` or
`/build-fast` again; ordinary user messages SHALL NOT re-arm it.

> test: code

### Scenario: Esc-Esc while streaming cancels turn and disarms

- **GIVEN** a session armed in auto mode with a main agent turn streaming
- **WHEN** the user presses Escape twice
- **THEN** the streaming turn is cancelled
- **AND** the session pilot is disarmed

> test: code
> - crates/duckboard/src/area/interaction.rs:1170

### Scenario: Esc-Esc while idle and armed only disarms

- **GIVEN** a session armed in fast mode with no main agent turn streaming
- **WHEN** the user presses Escape twice
- **THEN** the session pilot is disarmed
- **AND** no agent turn is started or cancelled solely for that disarm

> test: code
> - crates/duckboard/src/area/interaction.rs:1201

### Scenario: User cancel of main turn disarms without pilot auto-send on that completion

- **GIVEN** a session armed in auto mode with a main agent turn streaming
- **AND** the completed turn would otherwise expose a safe rank-1 trailing next action
- **WHEN** the user cancels that main agent turn (composer cancel, not requiring Esc-Esc)
- **AND** that cancelled turn completes
- **THEN** the session pilot is disarmed
- **AND** no pilot auto-sent agent turn is started for that completion

> test: code
> - crates/duckboard/src/area/interaction.rs:1479

### Scenario: Disarm stays off until build command relaunch

- **GIVEN** a session whose pilot was disarmed after an unsafe next action
- **WHEN** the user submits an ordinary non-build message that completes an agent turn
- **THEN** the session pilot remains disarmed after that turn completes

> test: code
> - crates/duckboard/src/build_pilot.rs:521

## Requirement: Mode plaque

While the pilot is armed, the chat composer chrome SHALL show a mode plaque above the
input that identifies auto or fast accordingly. While the pilot is disarmed, that plaque
SHALL NOT be shown. The plaque is view chrome only and SHALL NOT be stored as a transcript
message.

> test: code

### Scenario: Armed session shows mode plaque above input

- **GIVEN** a session armed in auto mode
- **WHEN** composer chrome is evaluated
- **THEN** a build-auto mode plaque is shown above the input

> test: code
> - crates/duckboard/src/build_pilot.rs:276

### Scenario: Disarmed session hides plaque

- **GIVEN** a session with the pilot disarmed
- **WHEN** composer chrome is evaluated
- **THEN** no build pilot mode plaque is shown

> test: code
> - crates/duckboard/src/build_pilot.rs:290

## Requirement: Scope continuity

When an exploration session that holds an armed pilot is promoted into a change, the pilot
SHALL remain armed in the same mode on the resulting change session without requiring a
new `/build-*` submit.

> test: code

### Scenario: Armed pilot survives exploration promotion to change

- **GIVEN** an exploration session armed in auto mode
- **WHEN** that exploration is promoted into a change session
- **THEN** the change session pilot remains armed in auto mode

> test: code
> - crates/duckboard/src/chat_store_integration_tests.rs:69

## Requirement: Reactivate setting

Duckboard SHALL expose a global chat setting that controls whether the pilot may
reactivate after crash, restart, or error. The setting's default SHALL be off. In this
capability's product path, enabling the setting SHALL NOT by itself re-arm a pilot or
restore pilot mode after process restart; pilot arm state remains process-local and does
not persist in the session file.

> test: code

### Scenario: Default pilot reactivate setting is off

- **GIVEN** a fresh default configuration
- **WHEN** the pilot reactivate-on-error setting is read
- **THEN** the setting is off

> test: code
> - crates/duckboard/src/config.rs:451

### Scenario: Setting can be enabled without reactivating pilot at runtime

- **GIVEN** a disarmed session and the pilot reactivate-on-error setting enabled
- **WHEN** configuration is loaded for chat
- **THEN** the session pilot remains disarmed

> test: code
> - crates/duckboard/src/config.rs:461

## Requirement: Mid-turn choice auto-answer

While the pilot is armed and a live mid-turn user-choice request is presented, duckboard
SHALL automatically answer with the recommended option when one is marked in an option
label, otherwise with the first option, using the same in-band selected-option path as a
manual chip activation. The pilot SHALL remain armed. Auto-answer SHALL NOT require the
session to be the active UI selection or the window to be focused.

> test: code

### Scenario: Armed pilot auto-selects first chip when none marked recommended

- **GIVEN** a session armed in auto mode
- **AND** a live mid-turn user-choice request with two options and no recommended mark
- **WHEN** that user-choice request is applied
- **THEN** the first option is answered in-band
- **AND** the session pilot remains armed

> test: code
> - crates/duckboard/src/area/interaction.rs:1377

### Scenario: Armed pilot auto-selects recommended-marked chip over first

- **GIVEN** a session armed in auto mode

- **AND** a live mid-turn user-choice request whose second option label is marked
  recommended

- **WHEN** that user-choice request is applied

- **THEN** the recommended option is answered in-band

- **AND** the session pilot remains armed

> test: code
> - crates/duckboard/src/area/interaction.rs:1415

## Requirement: Focus independence

Selecting a different change, switching area, or leaving window focus SHALL NOT cancel an
in-flight turn on an armed piloted session, SHALL NOT disarm that session's pilot, and
SHALL NOT skip an otherwise-due pilot auto-send solely because that session is not the
active UI selection or the window is unfocused. Pilot hop routing SHALL use the completing
session's identity, not the currently selected change.

> test: code

### Scenario: Select other change leaves armed pilot intact

- **GIVEN** a session armed in auto mode on change A with a main agent turn streaming
- **WHEN** the user selects a different change B
- **THEN** the pilot on change A's session remains armed
- **AND** the streaming turn on change A is not cancelled solely by that selection

> test: code
> - crates/duckboard/src/area/change.rs:4610

### Scenario: Hop after TurnComplete does not require active session

- **GIVEN** a session armed in auto mode that is not the active UI selection
- **AND** the completed turn's rank-1 next action is auto-sendable
- **WHEN** that non-priming agent turn completes
- **THEN** a new agent turn is started on that session with the rank-1 send text
- **AND** the pilot remains armed

> test: code
> - crates/duckboard/src/area/interaction.rs:1279

## Requirement: Failed hop disarms

When the pilot decides to auto-send after a turn completes but the agent turn cannot be
started (including when no agent handle is available), duckboard SHALL disarm the pilot
and SHALL record a short system notice that the pilot stopped. The pilot SHALL NOT remain
armed indefinitely while unable to hop.

> test: code

### Scenario: Failed hop with no handle disarms and notices

- **GIVEN** a session armed in auto mode with no agent handle
- **AND** the completed turn's rank-1 next action is auto-sendable
- **WHEN** that non-priming agent turn completes
- **THEN** the session pilot is disarmed
- **AND** a system notice is recorded that the pilot stopped
- **AND** no pilot auto-sent agent turn is started for that completion

> test: code
> - crates/duckboard/src/area/interaction.rs:1350
