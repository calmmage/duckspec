# @ Exploration list labels

Manual rename and on-demand AI retitle for exploration rows in the CHANGE list, so labels
stay under user control after the first auto-title. The retitle control is shown only when
refresh can do work.

## + Requirement: Refresh control visibility

On a CHANGE-list exploration row that shows hover or selected row actions, the
title-refresh control SHALL be shown only when at least one session for that exploration
has summarizable content for title refresh (non-priming, non-bare-slash user text, as for
`title_refresh_target`). When no such session content exists, the control SHALL be absent.

> test: code

### Scenario: Summarizable content shows the refresh control

- **GIVEN** an exploration with a session that has non-priming non-bare-slash user content
- **WHEN** refresh-control visibility is resolved for that exploration's sessions
- **THEN** the refresh control is considered available

> test: code

### Scenario: No summarizable content hides the refresh control

- **GIVEN** an exploration whose sessions have no non-priming non-bare-slash user content
- **WHEN** refresh-control visibility is resolved for that exploration's sessions
- **THEN** the refresh control is not available

> test: code
