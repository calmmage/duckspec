# @ Exploration list labels

Manual rename and on-demand AI retitle for exploration rows in the CHANGE list, so labels
stay under user control after the first auto-title. The retitle (↻) control appears only
when refresh can use summarizable chat content.

## ~ What it owns

Exploration rows under CHANGE show a mutable `display_name`. That name is independent of
the stable exploration id used for chat directories. Two user actions update it:

```
| Action | Source | Writes |
| --- | --- | --- |
| Rename | Typed non-empty text | exploration `display_name` |
| Refresh | Title oneshot over the active session's current chat | session `title` + exploration `display_name` |
```

Last successful write wins. Blank rename commits and failed/empty refreshes do not clear
an existing label.

The ↻ control is part of exploration row hover/selected actions only when at least one
session for that exploration has summarizable content for title refresh. Otherwise the
control is omitted so it does not look clickable while no-op.

## @ Refresh input

Automatic first-turn titling still uses the opening user message. Refresh is different: it
builds a summarizer input from the active session's conversation so far so a retitle can
track topic drift. At minimum, later non-bare user turns participate when they exist.
Priming messages and bare slash commands do not count as summarizable content. The same
notion of summarizable content gates whether ↻ is shown on the list row.
