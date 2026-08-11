# First-turn scope orientation

Inject duckcore scope orientation on the first agent turn for a session; skip when the
session already has a resumable agent id.

## Prerequisites

- [x] @step slash-submit-routing
- [x] @step agent-and-file-event-loop

## Context

Review finding 1: ducktui sends bare `TurnRequest` without `CurrentScopeHook` /
path-reference context. Duckboard primes or attaches orientation on first turn and omits
it when resume is possible. Reuse duckcore `SessionScope` + `CurrentScopeHook` (and
path-reference note); load change facts / inputs-ledger presence when practical so change
orientation is not empty. Prefer matching duckboard’s first-turn-vs-resume gate
(`agent_session_id` / brand-new session) rather than re-inventing a third model.

## Tasks

- [x] 1. Build a `SessionScope` for the bound chat (kind, scope key, change facts and
         inputs-ledger flag when available from project state)

- [x] 2. On agent submit when there is no resumable `agent_session_id`, attach first-turn
         orientation so the agent sees the scope blurb (hook output + path-reference note;
         AGENTS.md optional if easy via existing hook)

- [x] 3. When `agent_session_id` is set, do not re-send orientation on later turns

- [x] 4. Keep drive-only persist and existing stream apply behavior unchanged

- [x] 5. @spec session/scope Reliable first-turn delivery: The first turn's message body carries the scope orientation

- [x] 6. @spec session/scope Reliable first-turn delivery: A resumed session does not repeat the orientation
