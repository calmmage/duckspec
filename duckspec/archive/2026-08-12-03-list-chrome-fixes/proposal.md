# List chrome fixes

Polish CHANGE (and Ideas) list chrome so marks, list width, and retitle controls are
honest, single-purpose, and usable — matching the behavior already shipped in duckboard.

Exploration surfaced three related UI defects on the CHANGE queue and adjacent Ideas list.
They share one product surface (row chrome and left layout) and were fixed together in
code; this package records that unit so caps can catch up and the work can archive
cleanly.

```
| Friction | Intended contract |
| --- | --- |
| Star painted twice (title + clickable control) | One interactive mark: outline on hover when unmarked; always when marked; title stays plain |
| List column looked draggable but was fixed | List column width is user-draggable (session memory); free-space chat split uses live list width |
| ↻ looked clickable but often no-op | Show retitle only when summarizable chat exists |

```

**In scope**

- Single mark presentation on CHANGE live queue and Ideas rows (mint-on-click cycle kept
  as today).

- Draggable list column between list and content, with sane min/max clamps and rebalance
  of uncustomized chat width from the new free space.

- Exploration-row ↻ visibility gated on retitle-able session content.

**Out of scope**

- Persist list width across app restarts.

- Chat door chevrons / interaction-column customize modes (already owned elsewhere).

- Mark data model, pin sort, and oneshot retitle algorithms beyond the visibility/chrome
  contracts above.

- Unrelated dirty WIP or other active changes.

**Rejected**

- Leaving the list fixed and only “educating” the user — the grip must work.

- Prefixing the title with the mark glyph in addition to a control — double-paint is a
  defect, not a feature.

**Done in code; this change captures the contract** for propose → design/spec → archive.
Spec lag known: content/chat layout still documents a fixed list column; mark and
list-label caps under-specify row chrome and refresh visibility.
