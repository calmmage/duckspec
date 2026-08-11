# Cold load viewer style stamp

Stamp effective viewer style before first materialize on session load so Hybrid C editor
content matches the view on first paint.

## Prerequisites

- [x] @step style-flip-rematerialize

## Context

Review 03: `ensure_sessions_with_label` materializes loaded sessions while `viewer_style`
is still default Classic. With stored Focus, first paint can show full body in the
open-region TextEdit until a later apply. Mid-session flip is already fixed.

## Tasks

- [x] 1. Thread effective `ViewerStyle` into `ensure_sessions_with_label` (and callers)

- [x] 2. On load: stamp each session's `viewer_style` (clear folds when not Focus) before
         first `materialize_chat_ui`

- [x] 3. Stamp empty newly created sessions with the effective style as well

- [x] 4. Smoke: load a Focus-layout Answer under stored Focus → open-region editor holds
         gate lines only without waiting for a later interaction update

## Outcomes

- `ensure_sessions_with_label` takes effective `viewer_style` and stamps before first
  materialize via `stamp_viewer_style_for_load`.

- Call sites: change, ideas (`open_idea` + interaction), main (phase pill / exploration
  title), and `update_with_side_effects`.
