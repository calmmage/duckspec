# Idea queue list

Shared presentation for CHANGE and Ideas queues: pin up to three newest stars, sort the
rest by a chosen key, and show type and phase pillows with density and visibility prefs.

## Sort ladder

```
┌─────────────────────────────┐
│ pin prefix (≤3 stars)       │  newest pin time first
├─────────────────────────────┤
│ ordinary segment            │  active sort key
└─────────────────────────────┘
```

Star pin slots are computed per list body (Change active queue, or one Ideas section), not
globally across both areas.

```
| Sort key | Ordinary segment order |
| --- | --- |
| last-message (default) | Newest last non-priming chat message first; unknown activity last |
| phase | Derived duckspec lifecycle phase ladder; no-phase last; title tie-break |
| created | Creation time |
```

## Preferences

One shared preference set drives both lists:

```
| Pref | Default |
| --- | --- |
| sort key | last-message |
| show type pillows | on |
| show phase pillows | on |
```

The Change section header and each Ideas section header expose a sort menu that edits
these prefs.

## Pillows

```
| Kind | Source | Notes |
| --- | --- | --- |
| type | secondary tags (`tags[1..]`) | primary tag is tree-only, never a pillow |
| phase | derived change phase | only when the row is change-linked |
```

Density: if title + enabled pillows do not fit the row width, pillows leave the steady row
and appear only on hover. Mark and title stay. A disabled pillow kind never appears.
