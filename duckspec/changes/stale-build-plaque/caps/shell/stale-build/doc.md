# Stale build plaque

When the open project is duckboard itself and the workspace package version on disk is
ahead of the running binary’s baked version, the shell surfaces a quiet Update plaque and
a manual install recipe — never an automatic rebuild or restart.

## When it appears

Stale-build evaluation runs only for a duckboard project tree: root `Cargo.toml` plus
`crates/duckboard` with package name `duckboard`. Other projects never show the plaque.

```
running binary version     disk workspace version
        (baked)            (root Cargo.toml)
            │                      │
            └──────── compare ─────┘
                       │
            disk > running ──► Update plaque
            else ───────────► nothing
```

Versions are plain `major.minor.patch` triples. Equal, lower disk, missing, or unparsable
versions produce no signal. Git SHA, dirty tree, and remote releases are out of scope.

## Status bar

When a signal is present, a compact **Update** chip sits in the status bar trailing
cluster (near the project folder label). Activating it opens the recipe panel only — no
shell-out.

## Recipe panel

The panel shows:

```
| Field | Content |
| --- | --- |
| Versions | running → disk |
| Steps | quit duckboard, then run the command in a terminal |
| Command | `cd '<project-root>' && just install` |
| Copy | puts that command on the clipboard |
```

There is no Run control. Dismiss (Esc / close) leaves the running process alone. If disk
and binary become aligned again while the panel is open, the panel closes.

## Refresh

The signal is re-evaluated on project open, manual refresh, and when the project root
`Cargo.toml` changes on disk.
