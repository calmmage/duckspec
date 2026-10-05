# Chat sticky last-user

Smart sticky strip at the top of duckboard chat that mirrors the last real user ask only
while that ask’s in-transcript card has scrolled above the viewport top.

## Role next to other chat chrome

```
| Surface | Job |
| --- | --- |
| In-transcript User card | Canonical ask in history (scrolls away) |
| Sticky last-user | Temporary top mirror while that card is above the viewport |
| Last Answer band | Contrast on the latest reply body |
| Session scroll | Open/switch latest, area restore, stick-to-bottom |
| Composer stack | Phase pills, pilot, queue, input, footer |
```

The sticky does not replace the transcript card, does not band the Answer, and does not
live in the composer stack.

## Layout

```
column
  ├── sticky last-user slot   ← outside scroll; empty when hidden
  ├── chat scrollable
  ├── completion
  ├── input divider
  └── composer
```

The first child stays mounted when the sticky hides (empty / zero-height slot) so show and
hide do not remount the composer. The original User card remains in the scrollable
transcript; the sticky only mirrors it.

## Source and visibility

```
blocks ──► last non-priming User ──► body text
                │
                └─► content top (cached) ──┐
                                           ├─► show?
last scroll offset ────────────────────────┘
```

- **Source:** last `User` segment that is not priming. Priming Setup, system lines, tools,
  answers, and user-choice chips never own the sticky.

- **Show:** source exists and that segment’s content top is above the viewport top (same
  top-edge slack as answer reply jumps). Missing offset counts as zero.

- **Hide:** no source, unknown content top, or the original card’s top is still at or
  below the viewport top.

- Content top is remeasured when the transcript layout changes; scroll only rechecks the
  pure visibility rule against the cached top.

A new user send moves the source to the new last non-priming User ask.

## Panel

- **Chrome:** paper user-card surface (same family as the in-transcript User card).

- **Body:** read-only full text of the source ask — not a second composer, no edit or
  send.

- **Height:** max six content line heights; longer asks scroll inside the panel.

- **Interaction:** no click-to-scroll-to-original in this capability.
