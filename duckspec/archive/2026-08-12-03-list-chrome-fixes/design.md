# List chrome fixes - Design

Three deltas on existing duckboard layout/list chrome: resizable list column, single
interactive mark presentation, and gated retitle control. Implementation already matches
this direction; design records ownership and geometry so specs can catch up.

## Capability ownership

```
| Concern | Owner | Kind |
| --- | --- | --- |
| List width + free-space formula + list grip | `layout/content-chat-split` | delta (was fixed list) |
| Mark data, cycle, mint, pin time | `ideas/marks` | keep; optional presentation note |
| Row mark paint (one control, plain title) | list paint in CHANGE + Ideas (`queue_list::mark_glyph` + `after_icon`) | contract under marks / queue presentation |
| Retitle oneshot + refresh semantics | `exploration/list-labels` | keep |
| ↻ visibility when retitle is possible | `exploration/list-labels` | delta |

```

No new capability. No shared “list-chrome” package cap.

## List column resize

```
three-column row
┌────────┬──────────┬──────┬─────────┬──────┬──────┐
│sidebar │  list W  │ grip │ content │ door │ chat │
│  48    │ session  │  6px │  fill?  │  16  │  …   │
└────────┴──────────┴──────┴─────────┴──────┴──────┘
```

- **State:** `State.list_column_width` on the app shell (session memory). Default
  `theme::LIST_COLUMN_WIDTH` (260).

- **Grip:** `widget::list_resize` — drag-only strip between list and content (not the chat
  door, not a 1px decorative divider).

- **Message:** `Message::ListColumnWidth(f32)`; clamp to `[MIN_LIST_WIDTH, max]` where max
  is min of soft max (480) and window room (sidebar + grips + min chat reserved).

- **Free space** for content↔chat:

```
free = window
     − SIDEBAR − 1
     − list_w
     − list_resize::HANDLE_WIDTH
     − interaction_toggle::HANDLE_WIDTH
```

Helpers: `free_content_chat_width_for` / `equal_interaction_width_for` /
`rebalance_uncustomized_for` take live `list_w`. Default-list wrappers remain for tests
that assume stock width.

- **Coupling:** list width change and window resize rebalance *uncustomized* interaction
  panels only; customized chat width stays absolute.

- **Out of scope:** persist list width in config.

## Mark row presentation

```
row: [leading?] [type icon?] [mark control?] [plain title] [trailing…]
                      │
                      └─ after_icon only
                         unmarked: ☆ on hover, absent when not hovered
                         marked: always show glyph; click cycles
```

- **Title:** always plain display name / idea title — never `title_with_mark` in list
  paint.

- **Control:** `mark_glyph(mark, hovered)` drives the single `after_icon` button;
  `CycleQueueMark` / `CycleIdeaMark` keep mint-on-click behavior.

- **Surfaces:** CHANGE live queue (explorations + changes) and Ideas tree rows.

- **Rejected:** glyph-in-title *and* separate control (double-paint).

## Retitle (↻) visibility

- **When shown:** exploration row is hovered or selected, and at least one loaded session
  for that exploration has `title_refresh_target` (non-priming, non-bare-slash user
  content).

- **When hidden:** no summarizable content → no ↻ (avoids silent no-op).

- **On activate:** existing `RefreshExplorationTitle` / `start_exploration_title_refresh`
  path unchanged (streaming / cold handle / empty summary still leave labels alone).

- **Evidence for the gate:** disk sessions via `load_sessions_for` at list paint time
  (same store the retitle path reads after ensure).

## Impact and compatibility

- Spec drift fixed by deltas: `content-chat-split` loses “list is fixed”; `list-labels`
  gains show/hide rule; mark presentation may sit as a thin marks or paint requirement.

- Existing tests for free-space half/floor/rebalance keep default list width helpers;
  list-resize clamps are implementation constants.

- No migration; session-only list width resets each process.

## Settled choices

- Deltas on three existing owners; no new cap.
- List grip is peer chrome to the chat door, not the same widget.
- Session-only list width (no `[ui]` persistence).
- Hide ↻ rather than disable-with-tooltip for empty explorations.
- Mark mint-on-click remains; presentation only dedupes the glyph.
