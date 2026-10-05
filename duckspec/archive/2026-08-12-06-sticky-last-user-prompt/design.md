# Sticky last-user prompt - Design

Smart sticky strip at the top of duckboard chat that mirrors the last real user ask only
while that ask’s transcript card has scrolled above the viewport.

## Placement

Outer chat column becomes:

```
column
  ├── sticky last-user slot   ← outside scroll; collapsed when hidden
  ├── chat scrollable         ← transcript unchanged
  ├── completion
  ├── input divider
  └── composer
```

The sticky never lives in the composer stack (phase pills, pilot, queue, input). Showing
or hiding it must not remount the composer: keep a **stable first child** (empty /
zero-height when hidden), same stability idea as the completion slot.

## Source identity

Pure helper over the painted transcript:

```
last_non_priming_user_idx(blocks) -> Option<usize>
  last i where blocks[i].kind == User && !blocks[i].is_priming
```

- Panel text = that block’s body (joined lines).

- Priming Setup, System, Thinking, Activity, Answer, and user-choice chips never own the
  sticky.

- No session message id field — re-resolve from `blocks` every evaluation so rematerialize
  stays correct.

## Visibility

Top-edge sticky (classic pin), not “fully off-screen”:

```
show = last_user.is_some()
    && content_top + VIEWPORT_TOP_EPS < offset_y
```

- `content_top` — layout content-y of the last-user block (same family as answer-landmark
  tops: `block.bounds.y - scrollable.bounds.y`).

- `offset_y` — `last_chat_offset_y` from `ChatScrolled` (treat missing as `0.0`).

- Reuse `VIEWPORT_TOP_EPS` (`1.0`) from answer landmarks.

- Unknown / unmeasured `content_top` (`None`) hides the sticky until a measure lands.

When stick-to-bottom follows a long turn, offset grows and the sticky shows without
special streaming cases. When the user scrolls the original card back under the top edge,
the sticky hides.

**Rejected:** fully-off threshold; always-on permanent panel.

## Measurement and state

Per active chat interaction (alongside `last_chat_offset_y` / stick-to-bottom):

```
cached last-user content_top: Option<f32>
  // None = unknown / no target → sticky hidden
```

**v1 clear vs remeasure** (keep these separate — clear force-hides until measure lands;
remeasure can overwrite a kept top without hiding):

```
| Event | Clear `content_top`? | Remeasure? |
| --- | --- | --- |
| `ChatScrolled` | No | No — pure show check only |
| Transcript rematerialize | **Only if** last-user idx or body **changed** (or no source); otherwise **keep** top (avoids stream flicker) | **Yes** whenever a source exists — overwrite top after auto-collapse / layout of earlier segments |
| Collapse toggle | Yes | Yes |
| Focus section fold | Yes | Yes |
| Session open / switch / clear | Yes (clear cache) | Yes when a source exists |
| No last user | Yes (`None`) | No |
```

**Out of scope for v1:** window / pane **width reflow** as an invalidation trigger (rare
stale top accepted).

Prefer **cache top + pure scroll check** over measuring every scroll. Measure via widget
operate on `chat-block-{idx}` (same family as answer landmarks).

Unit-test pure helpers: source selection and `show` boolean. Layout operate stays
integration-style.

## Panel presentation

```
| Concern | Direction |
| --- | --- |
| Chrome | Paper **user-card** surface (`chat_user_card` family) |
| Body | Read-only mirror (read-only `TextEdit` or equivalent); not a second composer |
| Max height | Cap ~**6 content line heights** |
| Overflow | Inner scroll inside the panel |
| Interaction | None required for v1 (no click-to-jump) |
```

## Data flow

```
blocks ──► last_non_priming_user_idx ──► text for panel
                │
                └─► measure content_top ──► cache (selective invalidation)
                                              │
last_chat_offset_y ───────────────────────────┴─► show?
                                                    │
view: sticky slot (text if show, empty if not) ─────┘
```

## Boundaries

```
| In | Out |
| --- | --- |
| Top-of-chat smart sticky | Above-composer placement |
| Last non-priming User block | Sticky Answer / last-answer band changes |
| Cached top + pure visibility | Always-on dual card |
| Max-height + inner scroll | Unbounded sticky height |
| Selective content-top invalidation (v1 set above) | Width-reflow remeasure (v1) |
| | Click-to-scroll-to-original (later optional) |
| | Edit/resend in panel |
```

## Impact

- **duckboard only** — `widget/agent_chat.rs`, interaction session cache, measure task
  wiring next to chat scroll handling.

- No duckpond / harness / persistence format change.

- Session scroll, stick-to-bottom, answer landmarks, and composer stack stay as they are;
  sticky is additive chrome.
