# Self-version evaluation

Pure evaluation of whether the open project is a stale duckboard self-build, plus the
install recipe command string.

## Tasks

- [x] 1. Add `crates/duckboard/src/self_version.rs` with `StaleBuildInfo`, `evaluate`, and
         `install_command` per design

- [x] 2. Declare `mod self_version` from `main.rs`

- [x] 3. @spec shell/stale-build Self-project only: Non-duckboard project yields no stale signal

- [x] 4. @spec shell/stale-build Self-project only: Duckboard package tree is eligible for evaluation

- [x] 5. @spec shell/stale-build Ahead comparison: Higher disk version is stale

- [x] 6. @spec shell/stale-build Ahead comparison: Equal versions are not stale

- [x] 7. @spec shell/stale-build Ahead comparison: Lower disk version is not stale

- [x] 8. @spec shell/stale-build Ahead comparison: Unreadable disk version is not stale

- [x] 9. @spec shell/stale-build Install recipe command: Recipe uses just install under the project root

- [x] 10. @spec shell/stale-build Install recipe command: Recipe escapes single quotes in the path
