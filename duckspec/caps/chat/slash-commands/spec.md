# Chat slash commands

Kinded slash-command catalog for chat completion, local system handlers (including
`/help`), and a double-slash escape so colliding agent skills stay reachable.

Kinded slash-command catalog for chat completion, local system handlers (including `/help`
and build-pilot commands), and a double-slash escape so colliding agent skills stay
reachable.

## Requirement: Kinded completion catalog

Every entry in the slash completion catalog SHALL carry exactly one kind: System,
Workflow, or Agent. Duckboard system-registry names SHALL appear as System. Names
discovered from the agent harness that start with `ds-` SHALL appear as Workflow; other
discovered names SHALL appear as Agent. When the same name exists in the system registry
and in harness discovery, the catalog SHALL keep the System entry and SHALL NOT keep a
second entry for that name. The catalog SHALL NOT list Claude interactive builtins
(`clear`, `compact`, `cost`, `help`, `model`) as Agent entries from shared discovery.

> test: code

### Scenario: System registry entries are System

- **GIVEN** a system-registry command named `help`
- **WHEN** the completion catalog is built
- **THEN** the catalog includes an entry named `help`
- **AND** that entry's kind is System

> test: code
> - crates/duckcore/src/slash_commands.rs:283

### Scenario: Discovered ds-* names are Workflow

- **GIVEN** harness discovery returns a command named `ds-spec`
- **WHEN** the completion catalog is built
- **THEN** the catalog includes an entry named `ds-spec`
- **AND** that entry's kind is Workflow

> test: code
> - crates/duckcore/src/slash_commands.rs:295

### Scenario: Other discovered names are Agent

- **GIVEN** harness discovery returns a command named `review` that is not in the system
  registry

- **WHEN** the completion catalog is built

- **THEN** the catalog includes an entry named `review`

- **AND** that entry's kind is Agent

> test: code
> - crates/duckcore/src/slash_commands.rs:307

### Scenario: System name wins on collision with discovery

- **GIVEN** a system-registry command named `help`
- **AND** harness discovery also returns a command named `help`
- **WHEN** the completion catalog is built
- **THEN** the catalog has exactly one entry named `help`
- **AND** that entry's kind is System

> test: code
> - crates/duckcore/src/slash_commands.rs:319

### Scenario: Claude interactive builtins are not Agent catalog entries

- **GIVEN** a harness whose discovery uses the shared command scanner

- **WHEN** the completion catalog is built without a system override for those names

- **THEN** the catalog has no Agent entry named `clear`, `compact`, `cost`, `help`, or
  `model` that exists only as a Claude interactive builtin

> test: code
> - crates/duckcore/src/slash_commands.rs:334

### Scenario: build-auto and build-fast are System

- **GIVEN** the system registry includes commands named `build-auto` and `build-fast`
- **WHEN** the completion catalog is built
- **THEN** the catalog includes an entry named `build-auto` whose kind is System
- **AND** the catalog includes an entry named `build-fast` whose kind is System

> test: code
> - crates/duckcore/src/slash_commands.rs:363

## Requirement: Local system submit

Submitting a bare system command SHALL be handled by duckboard without starting an agent
turn. For bare `/help`, the session SHALL record a user message with the submitted text
followed by a system message; the session SHALL NOT enter a streaming agent turn; pending
selection attachments SHALL remain available for a later agent turn. The system message
SHALL begin with a fixed prefix that names the system command and teaches the `//help`
escape, then list non-empty sections of the live catalog grouped by kind. Within the
Workflow section, entries SHALL appear by ascending order key when present; entries
without an order key SHALL appear after ordered Workflow entries; equal keys SHALL break
ties by name ascending.

### Scenario: Bare /help does not start an agent turn

- **GIVEN** a chat session ready to send
- **WHEN** the user submits bare `/help`
- **THEN** no agent turn is started for that submit

> test: code
> - crates/duckcore/src/slash_commands.rs:425
> - crates/ducktui/src/runtime.rs:642

### Scenario: Bare /help records user then system messages

- **GIVEN** a chat session ready to send
- **WHEN** the user submits bare `/help`
- **THEN** the transcript includes a user message whose text is `/help`
- **AND** a system message immediately after that user message

> test: code
> - crates/duckboard/src/area/interaction.rs:1283
> - crates/ducktui/src/runtime.rs:656

### Scenario: Local /help leaves selection attachments intact

- **GIVEN** a chat session with a pending selection attachment
- **WHEN** the user submits bare `/help`
- **THEN** the selection attachment is still pending for a later agent turn

> test: code
> - crates/duckboard/src/area/interaction.rs:1320

### Scenario: System reply prefix names the command and teaches //help

- **GIVEN** a chat session ready to send

- **WHEN** the user submits bare `/help`

- **THEN** the system message text includes a line stating that system command `/help` is
  running

- **AND** includes guidance to use `//help` for agent help

> test: code
> - crates/duckcore/src/slash_commands.rs:476

### Scenario: Help body lists non-empty kind sections from the live catalog

- **GIVEN** a completion catalog with at least one System entry and at least one Workflow
  entry and no Agent entries

- **WHEN** the user submits bare `/help`

- **THEN** the system message body includes a System section listing the System entries

- **AND** includes a Workflow section listing the Workflow entries

- **AND** does not include an Agent section

> test: code
> - crates/duckcore/src/slash_commands.rs:493

### Scenario: Help Workflow section lists by order key then name

- **GIVEN** a completion catalog with at least two Workflow entries that have different
  order keys

- **WHEN** the `/help` system message body is built

- **THEN** the Workflow section lists those entries in ascending order-key order

> test: code
> - crates/duckcore/src/slash_commands.rs:519

## Requirement: Double-slash agent escape

A bare double-slash command (`//name`) SHALL be submitted as an agent turn whose prompt is
the single-slash form (`/name`). The user-visible message text for that submit SHALL be
the typed double-slash form.

> test: code

### Scenario: Bare //help is an agent turn with prompt /help

- **GIVEN** a chat session ready to send
- **WHEN** the user submits bare `//help`
- **THEN** an agent turn is started
- **AND** the turn prompt is `/help`

> test: code
> - crates/duckcore/src/slash_commands.rs:436
> - crates/ducktui/src/runtime.rs:689

### Scenario: Escape keeps typed //help as the user message text

- **GIVEN** a chat session ready to send
- **WHEN** the user submits bare `//help`
- **THEN** the user message text recorded for that submit is `//help`

> test: code
> - crates/duckcore/src/slash_commands.rs:463
> - crates/ducktui/src/runtime.rs:708

## Requirement: Kind cues in completion

Each completion row SHALL paint the command name token with a color determined by that
entry's kind (System, Workflow, and Agent are pairwise distinct). System rows SHALL
include a short `sys` tag. When two entries have equal fuzzy match scores, the completion
list SHALL order System before Workflow before Agent. When two Workflow entries have equal
fuzzy match scores, the list SHALL order them by ascending frontmatter order key when both
have a key; Workflow entries without an order key SHALL appear after all Workflow entries
that have a key; equal order keys SHALL break ties by name ascending.

### Scenario: Name token color maps by kind

- **GIVEN** completion rows for System, Workflow, and Agent entries
- **WHEN** name-token colors are resolved
- **THEN** the three kinds resolve to three different colors

> test: code
> - crates/duckboard/src/widget/agent_chat.rs:1896

### Scenario: System rows include a sys tag

- **GIVEN** a System completion entry
- **WHEN** the completion row is rendered
- **THEN** the row includes a `sys` tag

> test: code
> - crates/duckboard/src/widget/agent_chat.rs:1910

### Scenario: Equal fuzzy scores order System, Workflow, Agent

- **GIVEN** three catalog entries of kinds System, Workflow, and Agent that all score
  equally for the current query

- **WHEN** the filtered completion list is built

- **THEN** those three appear in order System, then Workflow, then Agent

> test: code
> - crates/duckboard/src/widget/agent_chat.rs:1924

### Scenario: Equal scores order Workflow by order key then name

- **GIVEN** two Workflow catalog entries with equal fuzzy scores for the current query

- **AND** the first has a higher order key than the second

- **WHEN** the filtered completion list is built

- **THEN** the entry with the lower order key appears before the entry with the higher
  order key

> test: code
> - crates/duckboard/src/widget/agent_chat.rs:1952

### Scenario: Workflow without order key sorts after ordered Workflow

- **GIVEN** two Workflow catalog entries with equal fuzzy scores for the current query
- **AND** one entry has an order key and the other has no order key
- **WHEN** the filtered completion list is built
- **THEN** the entry with an order key appears before the entry without an order key

> test: code
> - crates/duckboard/src/widget/agent_chat.rs:1969

## Requirement: Frontmatter order and description

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
> - crates/duckchat/src/claude_code/discover.rs:235

### Scenario: Integer order maps to tenths key

- **GIVEN** a command markdown file whose frontmatter sets `order` to an integer form such
  as `1`

- **WHEN** slash commands are discovered from that file's directory

- **THEN** the catalog entry for that command has order key 10 for `order: 1` (value in
  tenths)

> test: code
> - crates/duckchat/src/claude_code/discover.rs:252

### Scenario: One-decimal order maps to tenths key

- **GIVEN** a command markdown file whose frontmatter sets `order` to a one-decimal form
  such as `3.1`

- **WHEN** slash commands are discovered from that file's directory

- **THEN** the catalog entry for that command has order key 31 for `order: 3.1`

> test: code
> - crates/duckchat/src/claude_code/discover.rs:269

### Scenario: Invalid order forms yield no order key

- **GIVEN** a command markdown file whose frontmatter sets `order` to a rejected form such
  as `3.10`, `1.`, or `.5`

- **WHEN** slash commands are discovered from that file's directory

- **THEN** the catalog entry for that command has no order key

> test: code
> - crates/duckchat/src/claude_code/discover.rs:286

### Scenario: Missing order yields no order key

- **GIVEN** a command markdown file whose frontmatter has no `order` field
- **WHEN** slash commands are discovered from that file's directory
- **THEN** the catalog entry for that command has no order key

> test: code
> - crates/duckchat/src/claude_code/discover.rs:305

## Requirement: Build pilot system classification

Submitting `/build-auto` or `/build-fast`, with or without trailing free-text arguments,
SHALL be classified as a local build-pilot system submit — not as an ordinary agent-only
submit of the raw `/build-*` text. Those names SHALL appear in the duckboard system
registry. What arming, kick rewrite, and auto-send do after classification is owned by the
build-pilot capability, not this one.

> test: code

### Scenario: build-auto with args classifies as local build pilot

- **GIVEN** composer submit text `/build-auto add pilot that auto-sends next`
- **WHEN** the submit is classified
- **THEN** the submit is a local build-pilot submit in auto mode
- **AND** the classified arguments are `add pilot that auto-sends next`

> test: code
> - crates/duckcore/src/slash_commands.rs:393

### Scenario: bare build-fast classifies as local build pilot

- **GIVEN** composer submit text `/build-fast`
- **WHEN** the submit is classified
- **THEN** the submit is a local build-pilot submit in fast mode
- **AND** the classified arguments are empty

> test: code
> - crates/duckcore/src/slash_commands.rs:409

### Scenario: build pilot names are system registry commands

- **GIVEN** the duckboard system command registry
- **WHEN** registry membership is checked for `build-auto` and `build-fast`
- **THEN** both names are system registry commands

> test: code
> - crates/duckcore/src/slash_commands.rs:383
