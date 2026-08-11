# Extract duckcore

Create `crates/duckcore` and move UI-neutral chat, scope, agent-core, and watcher modules
out of duckboard; leave thin iced adapters behind so both UIs share one implementation.

## Tasks

- [x] 1. Add workspace crate `crates/duckcore` (deps on duckpond, duckchat, notify, tokio,
         serde as needed)

- [x] 2. Move `chat_store`, `scope`, `slash_commands`, `meta_card`, and `fast_response`
         into duckcore; rewire duckboard imports

- [x] 3. Split `agent.rs`: provider registry, turn driver, and `AgentEvent` over
         `tokio::mpsc` in duckcore; iced `Subscription` glue stays in duckboard

- [x] 4. Split `watcher.rs`: debouncer, gitignore filtering, and `FileEvent` in duckcore;
         iced subscription wrapper stays in duckboard

- [x] 5. Point duckboard at duckcore and keep existing duckboard tests green

- [x] 6. Update any `@spec` backlink paths that moved with the modules
