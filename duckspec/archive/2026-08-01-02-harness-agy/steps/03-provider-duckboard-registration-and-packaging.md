# Provider duckboard registration and packaging

Wire thin `AgyProvider` onto shared ACP runtimes, main-only title/reply, model tags,
duckboard harness dispatch, and build/release packaging for `duckchat-agy-acp`.

## Prerequisites

- [x] @step cold-print-session-lifecycle-and-batch-answer

## Tasks

- [x] 1. Implement `AgyProvider` (`crates/duckchat/src/agy.rs`): launch →
         `duckchat-agy-acp`, static AGY-tagged models, empty `list_commands`,
         `AcpMainRuntime` for main turns

- [x] 2. Title summary via local `clean_title` heuristic (no `agy` spawn); reply
         suggestions return empty without spawning `agy`

- [x] 3. Register `Harness::Agy` in `crates/duckboard/src/agent.rs` (dispatch,
         `available_models`, `agent_stream` match)

- [x] 4. Extend `Cargo.toml` / `justfile` / release packaging to build and ship
         `duckchat-agy-acp` next to `duckboard` (mirror Claude agent)

- [x] 5. @spec harness/agy Owned ACP agent over headless AGY: An AGY turn is driven through the owned ACP agent process

- [x] 6. @spec harness/agy Main-only title and reply suggestions: Title summary does not spawn agy

- [x] 7. @spec harness/agy Main-only title and reply suggestions: Reply suggestions are empty without spawning agy

- [x] 8. @spec harness/agy Offered AGY models: Offered AGY models are tagged with the agy harness

- [x] 9. Keep workspace tests/build green for the new harness registration path
