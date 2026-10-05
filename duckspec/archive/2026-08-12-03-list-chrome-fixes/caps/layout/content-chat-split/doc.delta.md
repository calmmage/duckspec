# @ Content / chat column split

Default equal free-space width for the content and interaction columns until the user
customizes via the chat-door grip; absolute width after that; full remaining width for the
interaction column when content is hidden. The list column is session-resizable via its
own grip and is part of free-space geometry.

## ~ Free space

In three-column areas, free horizontal space for the content ↔ interaction split is the
window width minus:

- sidebar and its vertical divider
- the **current** list column width
- the list resize grip
- interaction handle (when the handle is shown)

The list column defaults to a stock width and may be changed for the process session.
Equal split applies only to content and interaction; both depend on free space after the
live list width.

## ~ Modes

```
┌─ uncustomized ─────────────────────────────────────────┐
│ content and interaction each get half of free space    │
│ free space uses live list width + list grip            │
│ resize, list-width change, door open, force-show       │
│ recompute half from the current free space             │
│ new panel with a known window starts at that half      │
└───────────────────────────┬────────────────────────────┘
                            │ first chat-door grip width change
                            ▼
┌─ customized ───────────────────────────────────────────┐
│ interaction width is absolute pixels                   │
│ window resize keeps that width; no equal rebalance     │
│ list-width change does not clear customized mode       │
└────────────────────────────────────────────────────────┘

┌─ content hidden ───────────────────────────────────────┐
│ no open tabs (any three-column area), or content       │
│ collapsed                                              │
│ interaction fills remaining width (not the half value) │
│ remaining width still subtracts live list + list grip  │
│ stored interaction width is only the restore target    │
│ opening a list item that creates a tab shows content   │
└────────────────────────────────────────────────────────┘
```

## ~ Grip vs other chrome

```
| Action | Effect on width mode |
| --- | --- |
| Chat-door middle-grip drag that changes width | Marks interaction customized; sets absolute interaction width |
| List resize grip drag | Sets session list column width; updates free space; does not customize the interaction panel |
| Top chevron open/close | Visibility only; does not customize; uncustomized open rebalances from current free space |
| Programmatic force-show | Visibility only; does not customize; uncustomized open rebalances from current free space |
| Bottom chevron collapse/restore content | Content visibility only; does not customize |
```

Chat-door drag width is clamped between the minimum panel width and free space so the row
does not invert. Uncustomized half-width is not limited by a fixed maximum below free
space. List width is clamped between a minimum list width and a soft maximum that also
leaves room for minimum interaction chrome.
