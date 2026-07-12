# Stale build plaque - Design

Read-only self-version check: when the open project is duckboard and its workspace semver
is ahead of `CARGO_PKG_VERSION`, show a status-bar Update chip that opens a manual
`just install` recipe panel with clipboard copy.

## Approach

```
project open / refresh / root Cargo.toml watch
              │
              ▼
     evaluate(project_root)
       │  not self / unreadable / disk ≤ running → None
       │  disk > running → Some(StaleBuildInfo)
       ▼
 State.stale: Option<StaleBuildInfo>
       │
       ▼
 status bar: [breadcrumbs …] [Update] [project]
       │ click
       ▼
 modal overlay (Esc closes)
   close duckboard, then:
   cd '<root>' && just install
   [Copy]
```

No process spawn for install. Pure compare + chrome. Pure helpers in a small module; UI
reuses the existing modal stack pattern in `main.rs` view.

## Self-version evaluation

New module `crates/duckboard/src/self_version.rs` (name flexible; keep out of `data.rs` /
watcher).

```rust
pub struct StaleBuildInfo {
    pub running: String,   // env!("CARGO_PKG_VERSION")
    pub disk: String,      // from root Cargo.toml
    pub project_root: PathBuf,
}

/// `None` = not self, unreadable, or not ahead.
pub fn evaluate(project_root: &Path) -> Option<StaleBuildInfo>;

pub fn install_command(project_root: &Path) -> String;
// → `cd '<escaped root>' && just install`
```

**Self project** (all required):

1. `project_root/Cargo.toml` exists and parses
2. `project_root/crates/duckboard/Cargo.toml` exists
3. That package table has `name = "duckboard"` (string)

**Disk version** (first match wins):

1. Root `[workspace.package].version` if present (this repo’s shape)
2. Else root `[package].version`
3. Else unreadable → `None` (no plaque)

**Running version:** `env!("CARGO_PKG_VERSION")` only — never re-read the binary.

**Ahead:** parse both as `major.minor.patch` (optional leading `v` stripped).
`disk > running` lexicographically on the triple. Missing/invalid segments → treat as
unreadable → no plaque. No prerelease/`semver` crate unless we later need it.

## When to re-evaluate

Call `evaluate` and store on `State`:

```
| Event | Why |
| --- | --- |
| Project open / switch (`set_project` path) | new root |
| Manual Refresh | same as other project reload |
| Watcher: root `Cargo.toml` modified/removed | version bump while open |
```

Do **not** re-parse on every duckspec artifact tick — only the root manifest (and project
switch/refresh).

If `evaluate` returns `None` while the modal is open, close the modal.

## Status-bar plaque

Extend `widget/status_bar` (or thin wrapper at call site in `view`):

- When `state.stale.is_some()`, render a compact clickable **Update** chip left of the
  project folder label (trailing cluster)

- Style: muted/warning accent, not error red — informational

- Optional tooltip/subtitle later; v1 label is just `Update`

- Click → `Message::OpenStaleBuildPanel` (exact name free)

Status bar today is non-interactive text only (`widget/status_bar.rs`); this is the first
chip there — keep API small (`Option` payload + `on_press`).

## Recipe panel

Small modal on the existing top-level `stack` in `view` (same family as project picker /
quick idea / new-file — always stacked for stable widget tree).

Contents:

1. Title: e.g. **Update available**
2. One line: running → disk versions (`0.1.0 → 0.2.0`)
3. Short steps: quit duckboard, then run the command in a terminal
4. Monospace command: `install_command(project_root)`
5. **Copy** button → `iced::clipboard::write` (already used in main for PTY)
6. Dismiss: Esc / click-away / explicit close — same conventions as other modals

No Run button. No shell out.

## Wiring in `State` / `Message`

```rust
// on State
stale: Option<self_version::StaleBuildInfo>,
stale_panel_open: bool,

// messages (sketch)
StaleBuildOpen,
StaleBuildClose,
StaleBuildCopy,
```

`StaleBuildCopy` returns `iced::clipboard::write(install_command(...))` and may flash
“Copied” in-panel if cheap; otherwise silent copy is enough for v1.

## Impact

- duckboard-only UI + tiny pure module; no duckpond/ds changes

- `toml` already a duckboard dep — parse root `Cargo.toml` with `toml::Table`

- No new runtime deps if triple compare stays hand-rolled

- Status bar gains an interactive affordance

- Watcher handler gains a root-`Cargo.toml` branch (or post-filter on existing
  `FileChanged` path)

## Decisions

- **Status bar chip, not sidebar** — project-scoped chrome already lives there (project
  name); sidebar stays navigation-only. Alternative: sidebar badge near refresh (rejected:
  wrong mental model, easier to miss).

- **Self-detect via `crates/duckboard` package name** — not “Cargo.toml mentions duckboard
  anywhere.” Avoids false positives on unrelated trees.

- **Workspace version at root** — matches `version.workspace = true` layout; ignore nested
  package version for this change.

- **Hand-rolled triple compare** — workspace versions are plain `x.y.z`; skip `semver`
  crate until prerelease matters.

- **`just install` only** — proposal non-goal forbids alternate deploy paths.

- **No auto actions** — panel is instructional + clipboard only.

## Risks

- **Version never bumps → plaque never shows** → acceptable; semver is the deliberate
  quiet signal (proposal non-goal: auto bump).

- **Forks relocate the crate** → self-detect fails closed (no plaque).

- **Path with spaces/quotes in copy string** → single-quote root and escape embedded `'`
  for POSIX `cd`.
