# @ Chat slash commands

## @ Requirement: Kind cues in completion

Each completion row SHALL paint the command name token with a color determined by that
entry's kind (System, Workflow, and Agent are pairwise distinct). System rows SHALL
include a short `sys` tag. When two entries have equal fuzzy match scores, the completion
list SHALL order System before Workflow before Agent. When two Workflow entries have equal
fuzzy match scores, the list SHALL order them by ascending frontmatter order key when both
have a key; Workflow entries without an order key SHALL appear after all Workflow entries
that have a key; equal order keys SHALL break ties by name ascending.

### + Scenario: Equal scores order Workflow by order key then name

- **GIVEN** two Workflow catalog entries with equal fuzzy scores for the current query

- **AND** the first has a higher order key than the second

- **WHEN** the filtered completion list is built

- **THEN** the entry with the lower order key appears before the entry with the higher
  order key

> test: code

### + Scenario: Workflow without order key sorts after ordered Workflow

- **GIVEN** two Workflow catalog entries with equal fuzzy scores for the current query
- **AND** one entry has an order key and the other has no order key
- **WHEN** the filtered completion list is built
- **THEN** the entry with an order key appears before the entry without an order key

> test: code

## @ Requirement: Local system submit

Submitting a bare system command SHALL be handled by duckboard without starting an agent
turn. For bare `/help`, the session SHALL record a user message with the submitted text
followed by a system message; the session SHALL NOT enter a streaming agent turn; pending
selection attachments SHALL remain available for a later agent turn. The system message
SHALL begin with a fixed prefix that names the system command and teaches the `//help`
escape, then list non-empty sections of the live catalog grouped by kind. Within the
Workflow section, entries SHALL appear by ascending order key when present; entries
without an order key SHALL appear after ordered Workflow entries; equal keys SHALL break
ties by name ascending.

### + Scenario: Help Workflow section lists by order key then name

- **GIVEN** a completion catalog with at least two Workflow entries that have different
  order keys

- **WHEN** the `/help` system message body is built

- **THEN** the Workflow section lists those entries in ascending order-key order

> test: code

## + Requirement: Frontmatter order and description

When a filesystem command or skill file is discovered for the slash catalog, its YAML
frontmatter `description` field SHALL become the entry's description (empty when absent or
blank). When frontmatter includes an `order` field, the entry SHALL carry an order key
derived only from accepted forms: a non-negative integer, or an integer with exactly one
decimal digit (e.g. `1`, `3.1`). The order key SHALL be that value in tenths (e.g. `1` →
10, `3.1` → 31). Any other `order` text, or a missing `order` field, SHALL yield no order
key.

> test: code

### Scenario: Description from frontmatter is on the discovered command

- **GIVEN** a command markdown file whose frontmatter sets a non-empty `description`
- **WHEN** slash commands are discovered from that file's directory
- **THEN** the catalog entry for that command name has that description text

> test: code

### Scenario: Integer order maps to tenths key

- **GIVEN** a command markdown file whose frontmatter sets `order` to an integer form such
  as `1`

- **WHEN** slash commands are discovered from that file's directory

- **THEN** the catalog entry for that command has order key 10 for `order: 1` (value in
  tenths)

> test: code

### Scenario: One-decimal order maps to tenths key

- **GIVEN** a command markdown file whose frontmatter sets `order` to a one-decimal form
  such as `3.1`

- **WHEN** slash commands are discovered from that file's directory

- **THEN** the catalog entry for that command has order key 31 for `order: 3.1`

> test: code

### Scenario: Invalid order forms yield no order key

- **GIVEN** a command markdown file whose frontmatter sets `order` to a rejected form such
  as `3.10`, `1.`, or `.5`

- **WHEN** slash commands are discovered from that file's directory

- **THEN** the catalog entry for that command has no order key

> test: code

### Scenario: Missing order yields no order key

- **GIVEN** a command markdown file whose frontmatter has no `order` field
- **WHEN** slash commands are discovered from that file's directory
- **THEN** the catalog entry for that command has no order key

> test: code
