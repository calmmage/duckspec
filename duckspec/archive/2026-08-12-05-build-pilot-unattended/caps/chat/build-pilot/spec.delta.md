# @ Chat build pilot

Session pilot that arms from system `/build-auto` / `/build-fast`, kicks a rewritten
workflow turn, and keeps auto-advancing the session — rank-1 next actions, recommended or
first A–B–C options, and mid-turn choice chips — until a real stop, including while
another change or window is focused.

## = Requirement: Safe auto-send

Requirement: Unattended auto-send

## ~ Requirement: Unattended auto-send

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

### Scenario: Missing rank-1 disarms without sending

- **GIVEN** a session armed in auto mode
- **AND** the completed turn has no trailing next actions
- **WHEN** that non-priming agent turn completes
- **THEN** no auto-sent agent turn is started for that completion
- **AND** the session pilot is disarmed

> test: code

### Scenario: Rank-1 freeform decision token auto-sends while armed

- **GIVEN** a session armed in auto mode

- **AND** the completed turn's trailing next actions rank-1 send text is
  `confirm proposal`

- **WHEN** that non-priming agent turn completes

- **THEN** a new agent turn is started whose prompt is `confirm proposal`

- **AND** the pilot remains armed

> test: code

### Scenario: Rank-1 prose option line auto-sends while armed

- **GIVEN** a session armed in auto mode

- **AND** the completed turn's trailing next actions rank-1 send text is
  `A — harness-scoped resume`

- **WHEN** that non-priming agent turn completes

- **THEN** a new agent turn is started whose prompt is `A — harness-scoped resume`

- **AND** the pilot remains armed

> test: code

### Scenario: Recommended option is auto-sent over earlier non-recommended

- **GIVEN** a session armed in auto mode

- **AND** the completed turn exposes next actions for options A then B with B marked
  recommended

- **WHEN** that non-priming agent turn completes

- **THEN** a new agent turn is started whose prompt is the recommended option B send text

- **AND** the pilot remains armed

> test: code

## + Requirement: Mid-turn choice auto-answer

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

### Scenario: Armed pilot auto-selects recommended-marked chip over first

- **GIVEN** a session armed in auto mode

- **AND** a live mid-turn user-choice request whose second option label is marked
  recommended

- **WHEN** that user-choice request is applied

- **THEN** the recommended option is answered in-band

- **AND** the session pilot remains armed

> test: code

## + Requirement: Focus independence

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

### Scenario: Hop after TurnComplete does not require active session

- **GIVEN** a session armed in auto mode that is not the active UI selection
- **AND** the completed turn's rank-1 next action is auto-sendable
- **WHEN** that non-priming agent turn completes
- **THEN** a new agent turn is started on that session with the rank-1 send text
- **AND** the pilot remains armed

> test: code

## + Requirement: Failed hop disarms

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
