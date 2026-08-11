# @ Session scope orientation

## + Requirement: Inputs ledger pointer

For a change scope, when the project has a non-empty file at
`duckspec/changes/{name}/inputs.md`, the orientation SHALL name that path and direct the
agent to use those raw human inputs for re-grounding — prefer them over inventing
motivation, and do not rewrite that file. When that file is absent or empty, the
orientation SHALL NOT invent an inputs-ledger pointer.

> test: code

### Scenario: Present inputs.md is named in orientation

- **GIVEN** a session scoped to a change

- **AND** `duckspec/changes/{name}/inputs.md` exists and is non-empty

- **WHEN** the orientation is produced

- **THEN** it names the path `duckspec/changes/{name}/inputs.md`

- **AND** it directs the agent to use those raw inputs for re-grounding rather than
  inventing motivation

- **AND** it directs the agent not to rewrite that file

> test: code

### Scenario: Absent inputs.md yields no inputs pointer

- **GIVEN** a session scoped to a change
- **AND** `duckspec/changes/{name}/inputs.md` does not exist
- **WHEN** the orientation is produced
- **THEN** the orientation does not name an inputs ledger path
