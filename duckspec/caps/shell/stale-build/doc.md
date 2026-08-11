# Stale build plaque

When the open project is duckboard itself and the workspace package version on disk is
ahead of the running binary’s baked version, the shell surfaces a quiet Update plaque and
a manual install recipe — never an automatic rebuild or restart.

When the open project is duckboard itself and the workspace is ahead of the running binary
— by package version or by source fingerprint — the shell surfaces a quiet Update plaque
and a manual install recipe — never an automatic rebuild or restart.

## When it appears

Stale-build evaluation runs only for a duckboard project tree: root `Cargo.toml` plus
`crates/duckboard` with package name `duckboard`. Other projects never show the plaque.

```
running binary                 open project tree
  package version (baked)        package version (Cargo.toml)
  source id (baked)              source id (jj / git + dirty)
            │                              │
            └──────── compare ─────────────┘
                       │
        disk version > running  ──► Update plaque
        OR source ids differ    ──► Update plaque
        else ───────────────────► nothing
```

Package versions are plain `major.minor.patch` triples. Source fingerprints prefer the jj
working-copy change id, else a short git commit, with a `+` dirty suffix when the tree has
local changes. Equal versions with matching fingerprints produce no signal. Missing source
ids skip source comparison (version-only). Remote releases are out of scope.

## Status bar

When a signal is present, a compact **Update** chip sits in the status bar trailing
cluster (near the project folder label). Activating it opens the recipe panel only — no
shell-out.

## Recipe panel

The panel shows:

```
| Field | Content |
| --- | --- |
| Display | running → disk (versions; source ids when that drove the signal) |
| Steps | quit duckboard, then run the command in a terminal |
| Command | `cd '<project-root>' && just install` |
| Copy | puts that command on the clipboard |
```

`just install` refreshes cargo bins and `/Applications/Duckboard.app`. There is no Run
control. Dismiss (Esc / close) leaves the running process alone. If the tree and binary
align again while the panel is open, the panel closes.

## Refresh

The signal is re-evaluated on project open, manual refresh, and when the project root
`Cargo.toml` changes on disk.
