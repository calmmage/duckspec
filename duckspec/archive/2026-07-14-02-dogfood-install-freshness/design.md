# Dogfood install freshness - Design

Two dogfood loops: `just install` deploys the macOS app where Dock launches, and
stale-build compares a baked source fingerprint against the open tree so equal semver
still warns.

## Approach

```
just install
  ├─ cargo install → ~/.cargo/bin/{ds,duckboard,duckchat-claude-acp}
  ├─ just bundle   → dist/Duckboard.app (release + acp sibling + ad-hoc sign)
  └─ replace       → /Applications/Duckboard.app

build.rs (duckboard)
  └─ bake DUCKBOARD_SOURCE_ID  (jj change id or git commit, + if dirty)

open self-project
  └─ evaluate()
        disk_semver > running_semver  ──► stale (version)
        OR source_id_disk ≠ source_id_baked ──► stale (source)
        else ───────────────────────────────► None
```

Install stays a **manual** terminal recipe. Fingerprint is dogfood-only; other projects
still never evaluate.

## Local install recipe

Extend root `justfile` `install` after the existing `cargo install` lines:

1. Invoke `just bundle` (unchanged assembly of `dist/Duckboard.app`)

2. Replace `/Applications/Duckboard.app` with the freshly built bundle (`rm -rf` then
   `cp -R`; re-`codesign` if copy invalidates the ad-hoc signature)

3. Print a short note: quit any running Duckboard and reopen from Applications

Do not change `install_command()` string (`cd '<root>' && just install`) — the plaque
already points at that entrypoint; expanding what install *does* is enough.

Non-macOS: bundle recipes already assume macOS tools (`sips`, `iconutil`, `codesign`).
Keep the same assumption; no portable install target in this change.

## Source fingerprint

`crates/duckboard/build.rs` writes env values for the binary:

```rust
// cargo:rustc-env=DUCKBOARD_SOURCE_ID=<id>
// optional dirty suffix: <id>+
```

Resolution order at **build** and **disk read** (same function shape):

1. Prefer `jj log -r @ --no-graph -T 'change_id.short()'` when `jj` works in that tree
2. Else `git rev-parse --short HEAD`
3. Else empty / unavailable

Dirty bit: append `+` when the working copy has non-empty status (`jj status` working-copy
changes, or `git status --porcelain` non-empty). Missing tooling → no source id (skip
source comparison).

```rust
pub struct StaleBuildInfo {
    pub running: String, // display: version and/or source
    pub disk: String,    // display: version and/or source
    pub project_root: PathBuf,
}

const RUNNING_VERSION: &str = env!("CARGO_PKG_VERSION");
const RUNNING_SOURCE: &str = env!("DUCKBOARD_SOURCE_ID"); // may be ""

pub fn evaluate(project_root: &Path) -> Option<StaleBuildInfo>;
```

**Stale when** self-project and either:

```
| Condition | Display `running` → `disk` |
| --- | --- |
| Parseable versions and disk triple > running triple | `0.1.0` → `0.2.0` (source optional) |
| Both source ids non-empty and unequal | `0.1.0 (abc)` → `0.1.0 (def+)` |
```

If source id is missing on either side, source comparison does not fire (no false plaque
from bare crates.io builds). Version-ahead still applies when versions are readable.

Inject evaluation helpers with explicit running version + running source for unit tests
(no real `jj` required). Disk source reader takes project root and optional status
overrides in tests via writing a fake `.git` / calling a pure
`format_source_id(id, dirty)`.

Refresh triggers unchanged: project open, manual refresh, root `Cargo.toml` watch. Also
re-evaluate is enough on open after install; no need to watch `.git` for this pass (plaque
after reopen is the install loop).

## Recipe panel

Keep chrome: title, `running → disk` line, quit + run steps, copyable install command.
When the signal is source-driven, the version line already carries fingerprints in the
parenthesized form above — no new panel fields.

## Impact

- `justfile` — `install` deploys Applications after bundle

- `crates/duckboard/build.rs` — bake source id

- `crates/duckboard/src/self_version.rs` — dual comparison + display strings

- `crates/duckboard/src/widget/stale_build.rs` — no structural change if display strings
  carry fingerprints

- Spec/doc: `shell/stale-build` deltas; new `shell/local-install`

## Decisions

- **jj change id preferred over git commit** - this repo’s daily VCS is jj; change id is
  stable across amend. Alternative: always git SHA (weaker under jj rewrites of commit
  id). Fallback to git when jj absent.

- **Dirty as `+` suffix, not content hash** - enough to flag “binary was clean, tree is
  dirty” and id moves; not a full dirty-content hash. Alternative: hash `jj diff`
  (heavier, flaky in tests).

- **Install invokes full `just bundle`** - reuses signed app assembly; accepts a second
  release build after cargo install. Alternative: share one binary path (more justfile
  coupling for little gain).

- **Hard-coded `/Applications/Duckboard.app`** - matches product launch path that failed
  in exploration. Alternative: env override (out of scope).

## Risks

- **Copy fails while app is running** → justfile errors visibly; user quits and re-runs
  install (recipe already says quit first).

- **build.rs shells out** → slightly slower clean builds; fail soft to empty source id.

- **False “fresh” while dirty-on-dirty** → accepted; id change or clean install clears it.
