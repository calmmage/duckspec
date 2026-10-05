# Sticky last-user prompt

Keep the latest real user ask visible at the top of duckboard chat while its original
paper card is scrolled off-screen.

Long agent turns push the last user card out of the transcript viewport. Stick-to-bottom
and answer landmarks help you follow the reply, but they do not keep the ask itself in
view — re-checking what you said means scrolling up and losing place.

```
┌─────────────────────────────┐
│  sticky last-user (when     │  ← top of chat pane; only while original
│  original is above viewport)│     is off the top of the scroll
├─────────────────────────────┤
│  scrollable transcript      │
│  … original user card …     │  ← remains in history; panel only mirrors
│  think / activity / answer  │
├─────────────────────────────┤
│  composer stack             │  ← unchanged
└─────────────────────────────┘
```

**What shows**

- Source: the last non-priming user message (same family of rule as inputs ledger and
  reply oneshots). Priming Setup never owns the sticky.

- Content: a read-only mirror of that text — not a second composer, not an edit surface.

- Long asks: max-height with internal scroll (exact height left to design).

**When it shows**

- Sticky only while the original last-user card is scrolled above the top of the
  transcript viewport (“stick to the top on scroll”).

- Hide again when that card is back in view — no permanent dual copy of the card.

- No sticky when there is no qualifying last user (empty session, only priming).

- When a new user message is sent, the sticky tracks that new last ask.

**Boundaries**

```
| In | Out |
| --- | --- |
| Top of chat pane | Above-composer / composer-stack placement |
| Off-screen smart sticky | Always-on permanent panel |
| Last non-priming user ask | Sticky latest Answer (already has last-answer band) |
| Read-only mirror | Edit/resend in the panel |
| | Empty-composer lifecycle chips (`obvious-bubble`) |
```

Click-to-scroll to the original block is optional later; not required for the core
outcome.
