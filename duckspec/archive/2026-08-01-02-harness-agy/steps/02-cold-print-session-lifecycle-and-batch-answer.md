# Cold print session lifecycle and batch answer

Implement provisional open/load without spawning `agy`, cold `agy -p` per prompt with
auto-approve and 15m timeout, durable conversation rebind, SessionNotFound, cancel, and a
single final answer content update.

## Prerequisites

- [x] @step scaffold-agy-acp-agent-crate-and-binary-discovery

## Tasks

- [x] 1. Implement agent session table (`PendingOpen`, provisional ids, `session/new` and
         `session/load` without starting `agy`)

- [x] 2. Implement `session/prompt`: spawn cold
         `agy -p --dangerously-skip-permissions
                              --print-timeout 15m --log-file …`
         with optional `--model` / `--conversation`

- [x] 3. Parse durable conversation id from the print log (fallback:
         `last_conversations.json` for normalized cwd); rebind on prompt result when it
         differs from the open id

- [x] 4. On successful exit, emit one profile `agent_message_chunk` with stdout final text
         and `stopReason: end_turn` without requiring tool/reasoning updates

- [x] 5. Map dead resume / timeout-on-resume to SessionNotFound RPC shape; map other
         failures to process errors; kill in-flight `agy` on cancel

- [x] 6. Add scripted/`agy`-factory tests for open-without-spawn, rebind, resume, and
         session-not-found (mirror Claude agent test style where practical)

- [x] 7. @spec harness/agy Owned ACP agent over headless AGY: The agent runs headless agy print mode with auto-approved tools

- [x] 8. @spec harness/agy Session lifecycle and durable conversation ids: Opening a new session does not start agy before the first user prompt

- [x] 9. @spec harness/agy Session lifecycle and durable conversation ids: A turn without a prior session surfaces a durable AGY conversation id

- [x] 10. @spec harness/agy Session lifecycle and durable conversation ids: A turn with a prior AGY conversation id resumes that id

- [x] 11. @spec harness/agy Session lifecycle and durable conversation ids: A failed resume of a dead AGY conversation surfaces session-not-found

- [x] 12. @spec harness/agy Batch final-answer emission: A successful turn surfaces the final assistant answer as content

- [x] 13. @spec harness/agy Batch final-answer emission: A successful turn does not require tool or reasoning events on the wire
