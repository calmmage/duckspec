# List digit switch

Shell `Ctrl+1/2/3` selects the nth digit-indexed row of the focused Change or Ideas queue,
using the list’s paint-order sequences for those rows, with no digit chrome on rows.

## Chord

```
| Chord  | Meaning                         |
| ------ | ------------------------------- |
| Ctrl+1 | Select digit-indexed row 1      |
| Ctrl+2 | Select digit-indexed row 2      |
| Ctrl+3 | Select digit-indexed row 3      |
```

Digits are 1-based over the digit-indexed paint sequence (not every decorative UI line).
There are no `⌃n` badges on rows; discovery is the chord itself.

## Eligibility

```
| Condition                                        | Digit chord      |
| ------------------------------------------------ | ---------------- |
| Project open, active area Change or Ideas        | Resolves         |
| Chat or content focused inside that area         | Still resolves   |
| Modal or exploration rename owns navigation keys | Does not resolve |
| Dashboard, Caps, Codex, Settings, or no project  | Does not resolve |
| Digit past the digit-indexed length              | Does not select  |
```

Area is what matters, not which column last held focus. While a modal or rename owns the
keyboard, those surfaces keep the keys.

## Index sequences

```
Change live body                    Ideas idea walk
────────────────                    ───────────────
explorations on live list           expanded sections only
+ active changes                    Inbox → Exploration → Change → Archive
after shared queue sort             selectable idea rows only
and star pins                       (nested idea paths under tags)
then pending-commit archives        tag-folder chrome not indexed
when shown after live WIP
```

Shared queue ordering (star pin prefix and sort key) is the same model as the CHANGE and
Ideas queues (`ideas/queue-list`). Digit switch does not invent a second order; it
addresses the list’s sequences for those rows (plus Ideas section expand and tag
collapse).

The separate **Archived** section and the files explorer are not part of the Change digit
index. Tag-folder chevrons and other non-idea chrome are not part of the Ideas digit
index.

## Selection

A resolved digit selects the target change/exploration or idea and leaves the user in the
same area. Hitting the digit for the row that is already selected does nothing useful and
does not open exploration rename (double-click rename on an exploration stays a pointer
affordance only).

## Related surfaces

```
| Surface                      | Relationship                                    |
| ---------------------------- | ----------------------------------------------- |
| ideas/queue-list             | Supplies pin/sort order for queue bodies        |
| chat option ⌘-number chips   | Different chord and surface; not list selection |
| Caps / Codex lists           | Not digit-indexed by this capability            |
| Multi-project / project tabs | Out of scope                                    |
```
