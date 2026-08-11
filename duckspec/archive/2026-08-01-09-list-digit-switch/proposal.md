# List digit switch

Hot-switch among the first few rows of the focused Change or Ideas list with `Ctrl+1/2/3`,
using the same order already shown on screen.

Selecting a change, exploration, or idea still means clicking the sidebar. When several
items are in play, that is slower than a fixed chord for “first / second / third in this
list.” Multi-project session switching was considered and set aside for a later change so
this one stays about list navigation inside one open project.

```
focused area          Ctrl+1 / 2 / 3
────────────          ─────────────────────────────
Change           ──►  select nth visible list row
                      (changes + explorations as shown)
Ideas            ──►  select nth visible idea row
other areas      ──►  no-op (v1)
```

Rules settled in exploration:

```
| Rule | Choice |
| --- | --- |
| Chord | `Ctrl+1`, `Ctrl+2`, `Ctrl+3` |
| Order | Visible list order (current sort / layout) |
| Context | Targets the focused area’s list only |
| On jump | Select that row; stay in the area |
| Missing nth row | No-op |
| v1 areas | Change and Ideas only |
```

**Not in this change:** warm multi-project sessions, project top-bar tabs, `Cmd+1..3`
project slots, named hotkeys for shell areas (Home / Caps / …), and digit jump in Caps or
Codex. Those remain separate product threads.
