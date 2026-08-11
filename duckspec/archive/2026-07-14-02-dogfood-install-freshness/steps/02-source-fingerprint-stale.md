# Source fingerprint stale

Bake a source fingerprint into duckboard and treat unequal disk vs running fingerprints as
stale at equal package version.

## Prerequisites

- [x] @step local-install-applications

## Tasks

- [x] 1. Add `crates/duckboard/build.rs` that bakes `DUCKBOARD_SOURCE_ID` (jj/git + dirty)

- [x] 2. Extend `self_version::evaluate` for version-ahead or source-ahead; keep
         install_command unchanged

- [x] 3. @spec shell/stale-build Ahead comparison: Higher disk version is stale

- [x] 4. @spec shell/stale-build Ahead comparison: Equal versions are not stale

- [x] 5. @spec shell/stale-build Ahead comparison: Differing source fingerprints are stale at equal version

- [x] 6. @spec shell/stale-build Ahead comparison: Missing source fingerprint skips source comparison

- [x] 7. Ensure recipe panel still shows the signal display strings (running → disk)
