# Dogfood install freshness

Install puts a current build where Dock and Finder launch, and the Update plaque notices
source drift without a package version bump.

## Motivation

Dogfood changes land continuously without bumping the workspace semver (still `0.1.0`).
`just install` only updates `~/.cargo/bin`, while the daily driver is
`/Applications/Duckboard.app` — so features like Setup collapse look “broken” when the old
app is still running. The stale-build plaque only compares package versions, so equal
`0.1.0` never warns that the open tree has moved past the running binary.

## Intent

- Local install refreshes both cargo bins and `/Applications/Duckboard.app` from the
  current tree

- After install and relaunch from Applications, the running UI matches the tree that was
  installed

- When this repo is the open project, the Update plaque appears if the disk source
  fingerprint differs from the fingerprint baked into the running binary — even when
  package versions are equal

- Semver-ahead detection still works for real releases

- The plaque still only offers a copyable `just install` recipe (no auto-rebuild)

## Non-goals

- Auto-restart or in-process hot reload
- Matching dirty-tree content beyond a dirty bit on the same change/commit id
- Changing public release versioning policy or DMG CI
- Installing for non-macOS hosts or custom app destinations
- Notarization / Developer ID signing changes
