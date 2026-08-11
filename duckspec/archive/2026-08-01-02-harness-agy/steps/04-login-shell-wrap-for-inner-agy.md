# Login-shell wrap for inner agy

Wrap production `agy` spawn through the user's login shell so Finder-launched Duckboard
still resolves `agy` on PATH (same pattern as Claude/Grok).

## Prerequisites

- [x] @step cold-print-session-lifecycle-and-batch-answer

## Context

Review finding 1: `default_spawn_factory` uses bare `Command::new("agy")`. Claude uses
`SHELL -ilc 'exec "$@"' duckchat-wrap claude` (see `duckchat-claude-acp` spawn). Scripted
tests must keep a direct binary override so peers do not require a real `agy`.

## Tasks

- [x] 1. Add production argv prefix for `agy` via login-interactive shell
         (`SHELL -ilc 'exec "$@"' … agy`), mirrored on Claude's spawn helper

- [x] 2. Support a test/direct override (env or factory path) that skips the shell wrap
         for scripted peers

- [x] 3. Rewire `default_spawn_factory` in `crates/duckchat-agy-acp/src/print.rs` to use
         the wrap for production flags (`-p`, permissions, timeout, log-file, model,
         conversation, prompt)

- [x] 4. Keep existing scripted-factory unit tests green under the override path

- [x] 5. Add a unit assertion that the production factory's program/args include the shell
         wrap (or `agy` after wrap), not only bare `agy` as argv0 when no override is set
