# AGY agent harness

Add Antigravity CLI (`agy`) as a first-class duckboard agent harness so Gemini-lineage
work can run from the same chat UI as Claude and Grok — with honest limits where the CLI
cannot stream tools.

## Motivation

Duckboard already multi-homes agent turns under harness-tagged models. Petr wants AGY in
that set so Gemini sessions live in the same product surface instead of a separate
terminal.

Why now: Cursor is treated as Grok (not a separate harness), Codex is deferred to the main
maintainer, and AGY was the remaining candidate after live probes. Headless AGY only
exposes batch print mode (final answer, no tool wire events, session id out of band).
Recording that product boundary before design avoids over-promising parity with Claude and
Grok.

## Intent

- Operators can pick AGY models in the harness-tagged model picker and send main-chat
  turns through AGY

- Headless turns auto-approve tools so the host never blocks on AGY permission prompts

- A successful turn yields the assistant's final answer in the transcript (same place as
  other harnesses)

- Conversation identity is stored per chat and reused on later turns when AGY still
  accepts it; a dead or failed resume is cleared and a fresh conversation can start

- Product capabilities match what the CLI actually exposes: no requirement for progressive
  tool/activity streaming or token-level answer deltas on the wire

- AGY stays a distinct harness identity — its session ids are not mixed with Claude or
  Grok resume

## Non-goals

- Full transcript parity with Claude/Grok (live tool rows, reasoning channel, usage meter
  from AGY telemetry)

- Cursor as a separate harness

- Codex support in this change

- Depending on interactive AGY TUI, language-server, or sidecar HTTP (`agentapi` /
  `ANTIGRAVITY_LS_ADDRESS`)

- Reverse-engineering conversation DBs or brain transcripts for progressive tool events
  (may be later work)

- Changing AGY install, auth, or Google account setup beyond using an already-working CLI

- Redesigning the model picker or chat UI beyond registering another harness
