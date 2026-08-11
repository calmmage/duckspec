# Scaffold AGY ACP agent crate and binary discovery

Add workspace member `duckchat-agy-acp` with an ACP stdio shell and resolve its binary
from env, sibling of the running executable, or `PATH` — same pattern as Claude's agent.

## Tasks

- [x] 1. Add workspace member `crates/duckchat-agy-acp` (Cargo.toml + binary crate
         skeleton modeled on `duckchat-claude-acp`)

- [x] 2. Implement `main.rs` ACP stdio loop (initialize stub, method dispatch, JSON-RPC on
         stdout / logs on stderr)

- [x] 3. Advertise `loadSession: true` and a curated AGY model list on `initialize`

- [x] 4. Add `crates/duckchat/src/agy/agent_bin.rs` with `DUCKCHAT_AGY_ACP`,
         `duckchat-agy-acp` sibling/`PATH` resolution and `agy_acp_launch()`

- [x] 5. Export `agy` module plumbing from `duckchat` (`lib.rs`) enough for discovery
         tests

- [x] 6. @spec harness/agy Agent binary discovery: An explicit env override selects the agent binary

- [x] 7. @spec harness/agy Agent binary discovery: When env is unset, a sibling of the running executable is used if present

- [x] 8. @spec harness/agy Agent binary discovery: A missing agent binary fails the turn with a typed error
