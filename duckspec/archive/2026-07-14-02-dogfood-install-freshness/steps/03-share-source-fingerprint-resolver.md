# Share source fingerprint resolver

One jj/git+dirty implementation for bake-time and disk-time fingerprints so the plaque
cannot drift between `build.rs` and runtime.

## Prerequisites

- [x] @step source-fingerprint-stale

## Context

Review finding 1 (quality/major): `crates/duckboard/build.rs` and
`crates/duckboard/src/self_version.rs` each implement change id / commit / dirty rules.
Prefer a shared module both sides use (`include!` from `build.rs` is fine).

## Tasks

- [x] 1. Extract shared fingerprint resolver (jj change id, git short commit, dirty `+`)
         into one module both bake and runtime call

- [x] 2. Wire `build.rs` and `self_version` through the shared resolver; remove the
         duplicated helpers

- [x] 3. Re-run `cargo test -p duckboard self_version` (existing `@spec` coverage must
         stay green)
