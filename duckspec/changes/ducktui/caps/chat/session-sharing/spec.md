# Chat session sharing

How two UI processes coexist on the same per-scope session files: who may write, when a
displayed session reloads from disk, and what happens when both try to drive one session.

## Requirement: Drive-only writes

An app SHALL write a session file only when it is actively driving that session — its own
in-flight or completed turns. A session that is merely displayed MUST NOT be written by
the displaying app.

> test: code

### Scenario: Displayed-only session is never written

- **GIVEN** a session loaded for display only
- **AND** no local turn has been driven on that session
- **WHEN** the app updates other local UI state
- **THEN** the session file on disk is unchanged

### Scenario: Completing a local turn writes the session

- **GIVEN** a session the app is actively driving
- **WHEN** a local turn on that session completes
- **THEN** the session file on disk reflects the completed turn

## Requirement: External change reload

When a session file changes on disk and the local app is not driving that session, the app
SHALL reload the session from disk so the displayed transcript matches the latest
persisted content.

> test: code

### Scenario: External write reloads a displayed idle session

- **GIVEN** a session displayed locally with no in-flight turn
- **WHEN** another process writes that session file
- **THEN** the in-memory session matches the newly persisted content

### Scenario: External removal drops a displayed idle session

- **GIVEN** a session displayed locally with no in-flight turn
- **WHEN** another process removes that session file
- **THEN** the session is no longer present in the local session list for its scope

## Requirement: In-flight external shield

While a session has a local in-flight turn, the app SHALL ignore external file events for
that session until the turn settles. After settlement, subsequent external changes reload
again under the idle rule.

> test: code

### Scenario: External change during a local turn does not replace mid-turn state

- **GIVEN** a session with a local in-flight turn
- **WHEN** another process writes that session file before the turn settles
- **THEN** the in-memory session continues to reflect the local turn state
- **AND** the external file contents are not applied mid-turn

### Scenario: After local turn settlement, a later external change reloads

- **GIVEN** a session whose local turn has just settled
- **WHEN** another process writes that session file afterward
- **THEN** the in-memory session matches the newly persisted content

## Requirement: Concurrent drive is last-write-wins

Concurrent turns on the same session from two apps are unsupported. The system SHALL NOT
take multi-process locks on session files. When both apps write the same session, the last
successful atomic write is the persisted content.

> test: code

### Scenario: Later write is the persisted content

- **GIVEN** two apps each completing a write of the same session
- **WHEN** the second write finishes after the first
- **THEN** the session file on disk matches the second write
