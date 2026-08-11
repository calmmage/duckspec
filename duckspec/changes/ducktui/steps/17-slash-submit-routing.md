# Slash submit routing

Wire shared `parse_submit_slash` on ducktui submit: local `/help` without an agent turn,
and `//` display-versus-prompt split. Build pilot stays duckboard-only.

## Prerequisites

- [x] @step chat-pane
- [x] @step agent-and-file-event-loop

## Context

Review finding 2 (`03-review-ducktui-agent-send-and-slash-fidelity`): palette open works,
but submit always drives the raw composer string. Duckboard uses `parse_submit_slash` →
LocalHelp / LocalBuildPilot / Agent. This change implements local help and agent `//`
escape only; do not invent TUI build-pilot arm/kick state.

## Tasks

- [x] 1. Route every composer submit through a single path that calls
         `duckcore::slash_commands::parse_submit_slash` before starting an agent turn

- [x] 2. On `SubmitSlash::LocalHelp`: record user `/help` then a system message from
         `build_system_help_body` (live catalog); mark driven and persist; do not spawn or
         send an agent turn; do not leave the session streaming

- [x] 3. On `SubmitSlash::Agent { display, prompt }`: keep `display` as the user
         transcript text and send `prompt` to the harness (so bare `//help` displays as
         `//help` and prompts `/help`)

- [x] 4. On `SubmitSlash::LocalBuildPilot`: no pilot arm/kick — treat as freeform agent
         submit of the typed text (or a clear no-op without inventing pilot state)

- [x] 5. @spec chat/slash-commands Local system submit: Bare /help does not start an agent turn

- [x] 6. @spec chat/slash-commands Local system submit: Bare /help records user then system messages

- [x] 7. @spec chat/slash-commands Double-slash agent escape: Bare //help is an agent turn with prompt /help

- [x] 8. @spec chat/slash-commands Double-slash agent escape: Escape keeps typed //help as the user message text
