# Chat inputs ledger

A derived markdown ledger of verbatim user chat under each change package — refreshed from
local sessions while the change is active, and kept with the package on archive.

## Requirement: Package path

The inputs ledger for a change named `<name>` SHALL be the file
`duckspec/changes/<name>/inputs.md` under the project root. Because the file lives inside
the change directory, archiving the change SHALL keep the ledger with that directory.

> test: code

### Scenario: Ledger path is under the change directory

- **GIVEN** a project with a change directory for a named change
- **WHEN** the inputs ledger for that change is written
- **THEN** the file path is `duckspec/changes/<name>/inputs.md` under the project root

> test: code
> - crates/duckcore/src/inputs_ledger.rs:214

## Requirement: Verbatim user-only content

The ledger SHALL include only messages with the user role that are not priming. For each
included message, the ledger SHALL reproduce the message's user text content verbatim —
exact phrasing, punctuation, and structure as stored — without paraphrase, summary, or
added prose. The ledger SHALL NOT include assistant messages, system messages, tool use or
tool results, reasoning blocks, or priming user messages. Sessions SHALL appear in
session-creation order; within a session, included messages SHALL appear in session
message order.

> test: code

### Scenario: Non-priming user text is present verbatim

- **GIVEN** a change-scoped session that holds a non-priming user message with known text
- **WHEN** the inputs ledger for that change is rendered
- **THEN** the ledger contains that text exactly as stored

> test: code
> - crates/duckcore/src/inputs_ledger.rs:225

### Scenario: Priming user messages are omitted

- **GIVEN** a change-scoped session that holds a priming user message
- **WHEN** the inputs ledger for that change is rendered
- **THEN** the ledger does not contain that priming message's text

> test: code
> - crates/duckcore/src/inputs_ledger.rs:237

### Scenario: Non-user roles are omitted

- **GIVEN** a change-scoped session that holds assistant, system, or tool content and a
  non-priming user message

- **WHEN** the inputs ledger for that change is rendered

- **THEN** the ledger contains the user text

- **AND** the ledger does not contain the non-user content

> test: code
> - crates/duckcore/src/inputs_ledger.rs:252

### Scenario: Sessions appear in creation order

- **GIVEN** a change scope with two sessions that each have non-priming user text
- **AND** the sessions have different creation times
- **WHEN** the inputs ledger for that change is rendered
- **THEN** the earlier-created session's user text appears before the later session's

> test: code
> - crates/duckcore/src/inputs_ledger.rs:292

## Requirement: Empty and missing change

When a change has no qualifying user messages, the inputs ledger file SHALL be absent (not
created, or removed if a prior export left a file that would now be empty). When the
change directory does not exist, export SHALL NOT create the change directory and SHALL
NOT write an inputs ledger for that name under another path.

> test: code

### Scenario: No qualifying messages leaves no ledger file

- **GIVEN** a change directory for a named change
- **AND** that change's sessions have no non-priming user messages
- **WHEN** inputs export runs for that change
- **THEN** `duckspec/changes/<name>/inputs.md` does not exist

> test: code
> - crates/duckcore/src/inputs_ledger.rs:313

### Scenario: Missing change directory skips export

- **GIVEN** sessions scoped to a name that has no change directory under
  `duckspec/changes/`

- **WHEN** inputs export runs for that name

- **THEN** no `inputs.md` is written under `duckspec/changes/` for that name

- **AND** no new change directory is created for that name

> test: code
> - crates/duckcore/src/inputs_ledger.rs:335

## Requirement: Fresh full rebuild

After a durable save of a session scoped to a change whose change directory exists, the
inputs ledger for that change SHALL be a full rebuild from all sessions currently in that
scope — not an append of only the latest message. When the rebuilt content is identical to
the file already on disk, the export SHALL NOT rewrite the file.

> test: code

### Scenario: Save refreshes the ledger from all sessions

- **GIVEN** a change directory and two change-scoped sessions with distinct non-priming
  user text

- **WHEN** one of those sessions is durably saved

- **THEN** the inputs ledger contains the user text from both sessions

> test: code
> - crates/duckcore/src/inputs_ledger.rs:352

### Scenario: Unchanged content does not rewrite the file

- **GIVEN** a change whose inputs ledger already matches a full rebuild of its sessions
- **WHEN** a durable save triggers export again without any change to qualifying messages
- **THEN** the ledger file's modification time is unchanged

> test: code
> - crates/duckcore/src/inputs_ledger.rs:372

## Requirement: Promotion export

After an exploration scope's sessions are migrated into a change scope, the inputs ledger
for that change SHALL include the non-priming user text from the migrated sessions.

> test: code

### Scenario: Post-promotion export includes migrated user text

- **GIVEN** an exploration session with non-priming user text

- **AND** a change directory for the target change

- **WHEN** that exploration's sessions are migrated into the change scope and inputs
  export runs for the change

- **THEN** the change's inputs ledger contains that user text

> test: code
> - crates/duckcore/src/inputs_ledger.rs:423

## Requirement: Non-change scopes

Durable saves for exploration, caps, or codex scopes SHALL NOT write
`duckspec/changes/<…>/inputs.md` as a result of those saves alone.

> test: code

### Scenario: Non-change scope save does not create inputs.md under changes

- **GIVEN** a session scoped to exploration, caps, or codex
- **AND** no change-scoped export is otherwise triggered
- **WHEN** that session is durably saved
- **THEN** no new `inputs.md` appears under `duckspec/changes/`

> test: code
> - crates/duckcore/src/inputs_ledger.rs:394
