# Prose option next-actions - Design

Detect trailing A–B–C option runs in the last assistant message and seed the existing
next-action ghost / Tab / Enter path when no trailing `next` meta card is present. Shared
pure parse lives in duckcore; duckboard and ducktui both consume it.

## Responsibilities

```
| Piece | Owner |
| --- | --- |
| ABC trailing-run parse + send string + clip | `duckcore::prose_options` |
| Meta card then prose fallback | `duckcore::prose_options::next_actions_from_assistant` (or equivalent name) |
| Empty-session ladder (inherited / lifecycle) | unchanged in `duckboard` `next_action_list` |
| Non-empty session list | `next_action_list` calls `next_actions_from_assistant` |
| TUI numbered next hints | `ducktui` `recompute_next_hints` uses same fallback |
| Cap | extend `chat/default-prompts` |
```

`NextAction { send, reason: None }` is the only product type. Ghost and empty Enter keep
using `send` only.

## Priority

```
session empty?
  yes → inherited | lifecycle bootstrap   (no prose)
  no  → trailing_next_actions(src)
        if non-empty → that list
        else extract_trailing_abc_options(src)   // 0 or 2–3
```

Never merge meta tokens with prose options. Oneshot chips remain gated on an empty
next-action list. Mid-turn user-choice chips stay a separate path.

## Parse contract

- Letters **A–C** / **a–c** only; strict ascending order A→B→C…

- Trailing run: last contiguous option run near end of message; blank lines may sit
  between option lines; ≥2 options required; cap 3 (`MAX_NEXT_ACTIONS`)

- Skip fenced code interiors (same fence scan idea as meta cards)

- Line shapes: list markers (`-` `*` `•` / numbered list prefix), optional crude leading
  `|`, then letter (optional `**`), then separator `—` `-` `:` `|`, then body

- Single option alone → no list

- Numbered essay steps `1.` `2.` without letter form → no match

- Recommended-letter initial index deferred (always start at 0)

## Send / ghost string

One string for ghost and send:

```
{Letter} — {body}
```

Body is the rest of the line after the letter separator, lightly cleaned (strip wrapping
bold on the letter only). Char-budget clip (~96) with Unicode-aware ellipsis when over
budget. `reason` always `None`.

## Module sketch

```rust
// duckcore::prose_options
pub fn extract_trailing_abc_options(source: &str) -> Vec<NextAction>;
pub fn next_actions_from_assistant(source: &str) -> Vec<NextAction>;
// next_actions_from_assistant:
//   let meta = trailing_next_actions(source);
//   if !meta.is_empty() { meta } else { extract_trailing_abc_options(source) }
```

```rust
// duckboard default_prompts::next_action_list non-empty branch
Some(src) if !src.trim().is_empty() => next_actions_from_assistant(src),
```

```rust
// ducktui chat_pane::recompute_next_hints
let actions = next_actions_from_assistant(&answer_text);
```

## Surfaces

- **duckboard:** empty-composer ghost, Tab cycle, empty Enter (existing UI).

- **ducktui:** numbered next hints get the same meta-then-prose list so hosts do not
  diverge on shared sessions.

## Settled choices

- Product path is heuristic prose extract, not template-only meta cards.
- Explicit trailing `next` always wins.
- No new chip surface for prose options.
- No layout-width clipping; fixed char budget only.
- No 1–2–3 numeric option runs in this change.
