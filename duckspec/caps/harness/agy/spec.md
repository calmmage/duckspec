# AGY harness

The AGY harness drives Antigravity CLI (`agy`) through an owned ACP agent child over
headless print mode: the `agy` process starts on the first user prompt (not at open),
sessions bind durable AGY conversation ids for resume, each main turn runs a cold print
invocation that auto-approves tools, and a successful turn surfaces only a final answer as
profile content for the shared ACP client.

## Requirement: Owned ACP agent over headless AGY

An AGY turn SHALL be driven by the shared ACP client against the owned AGY ACP agent
process, not by an in-host print-mode client. That agent SHALL use headless `agy` print
mode as its backend and SHALL auto-approve tool permission prompts so the host never
blocks on AGY permission UI.

> test: code

### Scenario: An AGY turn is driven through the owned ACP agent process

- **GIVEN** a turn whose model names the AGY harness
- **WHEN** the turn runs
- **THEN** the host ACP client speaks to the owned AGY ACP agent process
- **AND** the host does not drive AGY via an in-host print-mode client

> test: code
> - crates/duckchat/src/agy.rs:251

### Scenario: The agent runs headless agy print mode with auto-approved tools

- **GIVEN** the owned AGY ACP agent handling a turn
- **WHEN** it executes the turn against AGY
- **THEN** the backend is headless `agy` print mode
- **AND** tool permission prompts are auto-approved without host UI

> test: code
> - crates/duckchat-agy-acp/src/agent.rs:577

## Requirement: Session lifecycle and durable conversation ids

Opening a new AGY conversation without a prior session id SHALL NOT start the `agy`
process before the first user prompt is submitted. Completing a turn that opened without a
prior session id SHALL surface a durable AGY conversation id for the host to persist.
Running a turn with a prior AGY conversation id SHALL resume that same id. A load or
resume that fails because the conversation is gone or no longer usable SHALL surface
session-not-found so the host can clear the stored id.

> test: code

### Scenario: Opening a new session does not start agy before the first user prompt

- **GIVEN** an AGY conversation with no prior session id
- **WHEN** the harness opens a new session without submitting user content
- **THEN** the open completes without starting the `agy` process

> test: code
> - crates/duckchat-agy-acp/src/agent.rs:452

### Scenario: A turn without a prior session surfaces a durable AGY conversation id

- **GIVEN** an AGY turn request carrying no session id
- **WHEN** the harness runs the turn successfully
- **THEN** it surfaces a durable AGY conversation id for the host to persist

> test: code
> - crates/duckchat-agy-acp/src/agent.rs:479

### Scenario: A turn with a prior AGY conversation id resumes that id

- **GIVEN** an AGY turn request carrying a previously assigned AGY conversation id
- **WHEN** the harness runs the turn
- **THEN** it opens the session by resuming that same id

> test: code
> - crates/duckchat-agy-acp/src/agent.rs:514

### Scenario: A failed resume of a dead AGY conversation surfaces session-not-found

- **GIVEN** an AGY turn request carrying a conversation id that is no longer loadable
- **WHEN** the harness attempts to resume that id
- **THEN** the failure surfaces as session-not-found rather than an untyped panic

> test: code
> - crates/duckchat-agy-acp/src/agent.rs:547

## Requirement: Batch final-answer emission

A successful AGY main turn SHALL surface the assistant's final answer as profile content
events the shared ACP client maps to the answer channel. The harness SHALL NOT require
progressive tool-use, reasoning, or usage events on the wire for a successful turn.

> test: code

### Scenario: A successful turn surfaces the final assistant answer as content

- **GIVEN** a successful AGY main turn that produced a final assistant answer
- **WHEN** the turn completes
- **THEN** that answer is delivered as profile content for the host transcript

> test: code
> - crates/duckchat-agy-acp/src/agent.rs:638

### Scenario: A successful turn does not require tool or reasoning events on the wire

- **GIVEN** a successful AGY main turn

- **WHEN** the turn completes

- **THEN** the turn is considered complete without tool-use or reasoning events on the ACP
  wire

> test: code
> - crates/duckchat-agy-acp/src/agent.rs:673

## Requirement: Main-only title and reply suggestions

For the AGY harness, a title summary SHALL be produced without spawning `agy`. Reply
suggestions for the AGY harness SHALL return an empty list without spawning `agy`.

> test: code

### Scenario: Title summary does not spawn agy

- **GIVEN** an AGY chat handle
- **WHEN** a title summary is requested
- **THEN** the result is a plain-text title string
- **AND** the request does not spawn `agy`

> test: code
> - crates/duckchat/src/agy.rs:299

### Scenario: Reply suggestions are empty without spawning agy

- **GIVEN** an AGY chat handle
- **WHEN** reply suggestions are requested
- **THEN** the result is an empty list
- **AND** the request does not spawn `agy`

> test: code
> - crates/duckchat/src/agy.rs:330

## Requirement: Agent binary discovery

Resolving the AGY ACP agent binary SHALL prefer an explicit environment override, then a
binary sibling of the running executable when present, then the process `PATH`. When no
agent binary can be launched, running an AGY turn SHALL fail with a typed error rather
than panicking.

> test: code

### Scenario: An explicit env override selects the agent binary

- **GIVEN** an environment override naming an AGY ACP agent binary
- **WHEN** the AGY harness resolves the agent to spawn
- **THEN** it selects that override path

> test: code
> - crates/duckchat/src/agy/agent_bin.rs:135

### Scenario: When env is unset, a sibling of the running executable is used if present

- **GIVEN** no environment override for the AGY ACP agent
- **AND** an AGY ACP agent binary next to the running executable
- **WHEN** the AGY harness resolves the agent to spawn
- **THEN** it selects that sibling binary

> test: code
> - crates/duckchat/src/agy/agent_bin.rs:149

### Scenario: A missing agent binary fails the turn with a typed error

- **GIVEN** no resolvable AGY ACP agent binary
- **WHEN** an AGY turn is run
- **THEN** the turn fails with a typed error rather than panicking

> test: code
> - crates/duckchat/src/agy/agent_bin.rs:166

## Requirement: Offered AGY models

Models offered by the AGY harness SHALL be tagged with the AGY harness identity so the
picker and dispatch can distinguish them from other harnesses' models.

> test: code

### Scenario: Offered AGY models are tagged with the agy harness

- **GIVEN** the AGY harness is registered and offers models
- **WHEN** its models are listed
- **THEN** each offered model names the AGY harness

> test: code
> - crates/duckchat/src/agy.rs:351
