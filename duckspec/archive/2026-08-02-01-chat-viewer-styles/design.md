# Chat viewer styles - Design

Duckboard Answer presentation is selected by a global `ViewerStyle` setting. Classic is
today's path and the default; Focus reuses Classic geometry around the trailing meta gate
with a hybrid paint model (open region = Classic TextEdit; foldable sections = plain
content-font for v1); Document is reserved for later and maps to Classic until
implemented.

## Style identity and config

```rust
// conceptual — duckboard config
pub enum ViewerStyle {
    Classic, // default
    Focus,
    Document,
}

// ~/.config/duckboard/config.toml
// [chat]
// viewer_style = "classic"
```

- Lives on existing `ChatConfig` as `viewer_style` (serde lowercase names).

- Missing / unknown / unimplemented effective values → **Classic** (forgiving prefs).

- **Global only**; no per-project or per-chat store in this change.

- **Settings** is the only v1 control: pick list of **implemented** styles only (`Classic`
  and, once Focus ships, `Focus`). Document is not listed until it has a real renderer.
  Chat chrome switcher is later and must share this key.

```
effective_style(config) =
  match config.chat.viewer_style {
    Classic => Classic,
    Focus if focus_shipped => Focus,
    Document | Focus | _ => Classic,
  }
```

## Answer presentation branch

```
agent_chat::view(..., style: ViewerStyle, ...)
        │
        ▼
  view_block(Assistant) ── Answer only
        │
        ├─ Classic (or live Answer, or no Focus layout)
        │       view_prose_block  // today's TextEdit recipe — identity path
        │
        └─ Focus (settled Answer + Sectioned layout)
                view_focus_answer
```

- Style applies to **Answer (Assistant) segments only**. User cards, System, Thinking, and
  Activity stay on existing paths.

- Caller threads `ViewerStyle` into `agent_chat::view` (no ambient config read inside the
  widget).

- Session storage and transcript segment model are unchanged; style is view-time (plus
  ephemeral Focus fold UI state).

### Classic default identity

- Effective Classic always uses the pre-change Answer body recipe (`view_prose_block`:
  single read-only TextEdit, markdown tables, highlights, meta-card tint, one last-answer
  band).

- Focus must not fork or dilute that path when style is Classic. Shared helpers are fine
  only if Classic remains a thin call into the same recipe; Classic does not go through
  Focus slice code.

- Product acceptance bar: with default config, Answer rendering matches pre-change.
  Regressions on default Classic block ship regardless of Focus quality.

## Focus geometry

Line-oriented, fence-aware, same family as `duckcore::meta_card` (not duckpond artifact
parse).

### Open region

Present only when a **trailing** `next` meta card exists (same trailing rule as
`trailing_next_actions`).

```
| Case | Open range |
| --- | --- |
| Trailing `next` only | `next.line_start ..= end of answer` |
| Trailing `next` + last `write` before it with **no other meta card** between | `write.line_start ..= next.line_end` (includes preview prose) |
| No trailing `next` | No open region → **Classic passthrough** for that Answer |
```

Earlier meta cards are never part of the open region.

### Sections

- Split body **before** the open region on ATX **H2/H3** outside fences (`^#{2,3}\s+`).
- **H1 is not** a fold boundary.
- Each H2/H3 starts a foldable section (body until next H2/H3 or open start).
- Lines before the first such heading form a **preamble** section.
- No headings but open region present → one foldable body section + open tail.
- Section ranges never include open-region lines.

```
## … / preamble     foldable
## …                foldable
…                   foldable
── open region ──  always shown
> **write** …
preview …
> **next** …
```

## Focus interaction and paint (Hybrid C)

- **Separate** from segment `chat_collapse` (Thinking/Activity/priming).

- Per Answer block: map of section keys → `{ collapsed, user_set }`.

- Default: all foldable sections **collapsed**; open region always expanded (no chevron on
  the gate).

- Toggle flips `collapsed` and sets `user_set`; auto defaults never override `user_set`
  for a still-present key.

- Section key: `(level, heading_text)` or preamble sentinel; on rematerialize, drop orphan
  keys; new keys get default collapsed.

- **Ephemeral** (not session-persisted). Clear all Focus fold state when effective style
  leaves Focus.

### Paint model

```
Focus Answer column
  section header (chevron + label)
  expanded section body  → plain content-font text (v1)
  …
  open region            → Classic TextEdit recipe
                           + last-answer band only here (when last Answer)
```

```
| Slice | Presentation |
| --- | --- |
| Collapsed section | Chevron + label only (heading text; preamble → `Preamble · N lines`) |
| Expanded foldable section | Plain content-font source text; no tables / meta tint / TextEdit |
| Open region | Same Classic body recipe as `view_prose_block`: read-only `TextEdit`, `md_tables`, word wrap, fit content, transparent bg; highlight ranges that fall in the open-region span |
```

- Prefer a slice-aware reuse of the Classic TextEdit setup for the open region rather than
  a second markdown engine. Expanded sections do not use that recipe in v1.

- Find and selection hit **visible** widgets only; open-region TextEdit participates like
  Classic; plain section text is weaker — accepted for v1.

- No expand-all / collapse-all in this change.

### Last-answer band under Focus

- `last_answer_band_target` still selects a single Answer index (unchanged landmarks
  rule).

- When that Answer renders Focus: apply `chat_last_answer_band` **only** to the
  open-region widget. Expanded section bodies never take the band.

- Classic path unchanged: one band on the single `view_prose_block`.

## Live / streaming

```
answer_presentation(style, answer_live):
  if style != Focus || answer_live → Classic
  else → focus_layout → Sectioned | PassthroughClassic
```

- **Live** Answer draft always Classic (stable headings/gate not available).

- Settled Answers may render Focus while a later turn streams.

- On settle under Focus: first-sight defaults (sections collapsed, open region on); no
  mid-stream fold carry-over.

- Style flip: settled Answers update immediately; live draft stays Classic until settle;
  Focus→Classic clears fold state.

Stream-ui apply/materialize cadence is unchanged; Focus does not add its own tick.

## Ship cut

```
| In this change | Later |
| --- | --- |
| Config + Settings + `effective_style` | Document renderer |
| Classic wire-through (identity-preserving) | Chat chrome style control |
| Focus geometry + hybrid paint + open-region band | Per-chat style, fold persistence |
| Document enum member only | Full classic paint on expanded sections, find-in-collapsed |
```

Implementation may slice as spine then Focus; **product intent of the change is both**.
Settings lists Classic | Focus once Focus exists.

## Compatibility and risk

- Existing sessions and markdown need no migration.

- Default Classic preserves pre-change look and behavior.

- Focus multi-slice presentation weakens whole-Answer find and section-body rich paint
  until a later pass — accepted for v1.

- Incomplete agent markdown without trailing `next` never gets aggressive collapse
  (passthrough Classic).

## Settled choices (concise)

- Names: `classic` / `focus` / `document`; field `chat.viewer_style`.

- Answer-only; thread style into view; Document hidden until real.

- Open region = trailing meta gate (policy A); H2/H3 only; no next → passthrough.

- Separate ephemeral fold state.

- **Hybrid C paint:** open region = Classic TextEdit; expanded sections = plain
  content-font for v1.

- **Last-answer band:** open region only under Focus; Classic keeps single full-body band.

- **Classic identity:** default/effective Classic is behavior-identical to pre-change.

- Focus only on non-live Answers.

- Ship spine + Focus; no chrome switcher / Document / full multi-slice TextEdit in scope.
