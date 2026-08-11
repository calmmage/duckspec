# Chat build pilot

Session pilot that arms from system `/build-auto` / `/build-fast`, kicks a rewritten
workflow turn, and auto-sends only safe trailing next tokens until disarmed.

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

### Scenario: Change kick uses lifecycle head with args

- **GIVEN** a non-archived change chat session whose lifecycle head is `ds-apply`
- **WHEN** the user submits `/build-fast fix the review`
- **THEN** an agent turn is started whose prompt is `/ds-apply fix the review`

> test: code

### Scenario: User bubble shows rewritten kick not build command

- **GIVEN** an exploration chat session ready to send
- **WHEN** the user submits `/build-auto sketch the feature`
- **THEN** the user message recorded for that submit is `/ds-explore sketch the feature`
- **AND** that user message text is not `/build-auto sketch the feature`

> test: code

### Scenario: Successful kick arms the requested mode

- **GIVEN** an exploration chat session with the pilot disarmed
- **WHEN** the user submits bare `/build-fast`
- **THEN** the session pilot mode is fast
- **AND** an agent turn is started

> test: code

### Scenario: Unsupported scope does not arm or start agent turn

- **GIVEN** a caps-scope chat session ready to send
- **WHEN** the user submits `/build-auto`
- **THEN** the session pilot remains disarmed
- **AND** no agent turn is started for that submit

> test: code

## Requirement: Safe auto-send

While the pilot is armed, after a non-priming agent turn completes, duckboard SHALL
inspect only the rank-1 trailing next action for that turn. When that send text is safe
for the armed mode, duckboard SHALL start a new agent turn with that exact send text as
both user-visible message and agent prompt. Safe in both modes: `confirm`, and allowlisted
workflow stage slashes `/ds-explore`, `/ds-propose`, `/ds-design`, `/ds-spec`, `/ds-step`,
`/ds-apply`, and `/ds-followup` (exact token after trim; optional leading `/` normalized).
Safe in auto mode only: `/ds-review`. Never safe in either mode: `/ds-archive`,
`/ds-codex`, `/ds-verify`, `reject`, `revise`, freeform text, and any other
non-allowlisted token. When there is no trailing next action, or rank-1 is not safe for
the mode, duckboard SHALL disarm the pilot and SHALL NOT auto-send.

> test: code

### Scenario: Rank-1 confirm auto-sends while armed

- **GIVEN** a session armed in auto mode
- **AND** the completed turn's trailing next actions rank-1 send text is `confirm`
- **WHEN** that non-priming agent turn completes
- **THEN** a new agent turn is started whose prompt is `confirm`
- **AND** the pilot remains armed

> test: code

### Scenario: Rank-1 allowlisted stage slash auto-sends in auto mode

- **GIVEN** a session armed in auto mode
- **AND** the completed turn's trailing next actions rank-1 send text is `/ds-spec`
- **WHEN** that non-priming agent turn completes
- **THEN** a new agent turn is started whose prompt is `/ds-spec`

> test: code

### Scenario: Rank-1 ds-review auto-sends only in auto mode

- **GIVEN** a session armed in auto mode
- **AND** the completed turn's trailing next actions rank-1 send text is `/ds-review`
- **WHEN** that non-priming agent turn completes
- **THEN** a new agent turn is started whose prompt is `/ds-review`

> test: code

### Scenario: Rank-1 ds-review disarms in fast mode without sending

- **GIVEN** a session armed in fast mode
- **AND** the completed turn's trailing next actions rank-1 send text is `/ds-review`
- **WHEN** that non-priming agent turn completes
- **THEN** no auto-sent agent turn is started for that completion
- **AND** the session pilot is disarmed

> test: code

### Scenario: Rank-1 archive codex or verify disarms without sending

- **GIVEN** a session armed in auto mode
- **AND** the completed turn's trailing next actions rank-1 send text is `/ds-archive`
- **WHEN** that non-priming agent turn completes
- **THEN** no auto-sent agent turn is started for that completion
- **AND** the session pilot is disarmed

> test: code

### Scenario: Missing or unsafe rank-1 disarms without sending

- **GIVEN** a session armed in auto mode
- **AND** the completed turn has no trailing next actions
- **WHEN** that non-priming agent turn completes
- **THEN** no auto-sent agent turn is started for that completion
- **AND** the session pilot is disarmed

> test: code

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

### Scenario: Esc-Esc while idle and armed only disarms

- **GIVEN** a session armed in fast mode with no main agent turn streaming
- **WHEN** the user presses Escape twice
- **THEN** the session pilot is disarmed
- **AND** no agent turn is started or cancelled solely for that disarm

> test: code

### Scenario: User cancel of main turn disarms without pilot auto-send on that completion

- **GIVEN** a session armed in auto mode with a main agent turn streaming
- **AND** the completed turn would otherwise expose a safe rank-1 trailing next action
- **WHEN** the user cancels that main agent turn (composer cancel, not requiring Esc-Esc)
- **AND** that cancelled turn completes
- **THEN** the session pilot is disarmed
- **AND** no pilot auto-sent agent turn is started for that completion

> test: code

### Scenario: Disarm stays off until build command relaunch

- **GIVEN** a session whose pilot was disarmed after an unsafe next action
- **WHEN** the user submits an ordinary non-build message that completes an agent turn
- **THEN** the session pilot remains disarmed after that turn completes

> test: code

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

### Scenario: Disarmed session hides plaque

- **GIVEN** a session with the pilot disarmed
- **WHEN** composer chrome is evaluated
- **THEN** no build pilot mode plaque is shown

> test: code

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

### Scenario: Setting can be enabled without reactivating pilot at runtime

- **GIVEN** a disarmed session and the pilot reactivate-on-error setting enabled
- **WHEN** configuration is loaded for chat
- **THEN** the session pilot remains disarmed

> test: code
