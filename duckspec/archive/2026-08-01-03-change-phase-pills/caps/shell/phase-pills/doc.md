# Phase pills

Derived lifecycle (and late-stage VCS dirty) short pills on the change list and above
chat, with per-surface settings and click-to-send for the next stage command — never a
freeform status field.

## What it shows

Phase pills project recognized lifecycle state for humans. They do not store status.
Active-change shorts come from the same artifact and step facts that drive agent
orientation and empty-session next-stage bootstrap.

```
short labels
────────────
explore | empty | proposal | design | specs | steps | review | ready | archived
```

```
| Situation | Short | Lifecycle send |
| --- | --- | --- |
| exploration, empty session | explore | `/ds-explore` |
| exploration, non-empty session | explore | (none) |
| no artifacts | empty | `/ds-propose` |
| proposal only | proposal | `/ds-design` |
| design, no caps | design | `/ds-spec` |
| caps, no steps | specs | `/ds-step` |
| open steps | steps | `/ds-apply` |
| all steps done, no review | ready | `/ds-archive` |
| no open steps + review | review | `/ds-step` |
| archived | archived | (none) |
```

Long hover on the lifecycle pill for an active change is the existing long phase
description (e.g. “design drafted, specs not yet written”). Archived and explore use fixed
short explanations.

## VCS pill

A second pill appears only when short is `ready` or `archived`:

```
| Working tree vs HEAD | Face | Send |
| --- | --- | --- |
| dirty (any path, repo-wide) | uncommitted | `Commit` |
| clean | committed | (none) |
```

Hover states dirty honestly as repo-wide, not change-scoped.

## Surfaces

```
Change list row (list setting on)
  name … [proposal] [uncommitted?]

Chat composer (chat setting on; change/exploration only)
  [proposal] [uncommitted?]
  ─────────────────────────
  input / ghosts / footer
```

Caps and codex sessions never show the chat strip. Settings expose independent toggles for
list and chat pills; both default on.

### List phase pills vs Sort “Phase”

When **Change list** phase pills are on (Settings default), short pills own phase chrome
on the Change list:

- Sort menus (Change and Ideas) **omit** the Phase toggle for long phase text pillows
- Long phase density pillows do not show on change or idea rows in that mode
- Turning list phase pills **off** restores Sort Phase and long-phase pillows

Sort-by-phase ordering can still use lifecycle rank; only the long-phase face is
suppressed.

## Activation

Clicking a lifecycle pill with a send string submits that empty-send text into the target
session. List origin selects the change or exploration when needed, then sends — without
opening exploration rename if that row is already selected. Clicking uncommitted submits
`Commit`. There is no menu of arbitrary stages and no disk status write.
