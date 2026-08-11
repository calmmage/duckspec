# @ Archive browse

How duckboard lists finished archived work on the Change list, Dashboard, and Ideas
surfaces: reverse chronology, mixed row kinds, and quiet defaults.

## ~ Interleaved rows

Change and Dashboard **Archived** sections combine:

```
| Row kind | Source | Sort key |
| --- | --- | --- |
| Finished archived change | `duckspec/archive/` package that is not pending | folder date-and-counter prefix |
| Archived exploration | duckboard soft archive (non–idea-owned) | exploration archive time |
```

A package is pending when it still has residual dirt under its archive folder; those
packages appear on the Change list instead, not here. Rows share one descending date order
so a freshly archived exploration sits among recent finished archived changes rather than
in a separate block.

Idea-owned explorations never appear here; the Ideas area owns that lifecycle.

## ~ Section defaults

```
Change list
  Change (active + live explorations + pending archives)   open by default
  Archived (finished only)                                 closed by default

Ideas list
  Inbox / Exploration / Change          open by default
  Archive                               closed by default
```

The Change Archived section is shown whenever there is at least one finished archived
change or one listable archived exploration. Selecting an archived change may expand its
section so the selection is visible; that is navigation feedback, not a change to the
default on a fresh list.
