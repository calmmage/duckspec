# Style flip rematerialize

When effective viewer style changes, stamp sessions and force `materialize_chat_ui` so
Hybrid C desired editor lines match the new style immediately.

## Prerequisites

- [x] @step hybrid-focus-paint-and-open-region-band

## Context

Review 02: Hybrid C stores open-region-only lines in `chat_editors` under Focus. View
layout flips from config each frame, but editor content only updates on materialize.
Settings save does not rematerialize. Leaving Focus can leave Classic showing only the
open-region slice until a later rebuild.

## Tasks

- [x] 1. Detect effective viewer-style change vs last stamped style (Settings path and/or
         interaction stamp path)

- [x] 2. On change: stamp `ax.viewer_style` to the new effective style for live sessions
         (all interaction states that hold chat sessions, or every active session path
         that already stamps style)

- [x] 3. Force `materialize_chat_ui` (or mark dirty + immediate rebuild) so
         `answer_editor_desired_lines` is applied before the next paint

- [x] 4. Keep existing leave-Focus fold clear; rebuild after stamp so Classic gets full
         body and Focus sectioned gets open-region-only editor content

- [x] 5. Smoke: Focus→Classic restores full Answer body in the editor without waiting for
         an unrelated rematerialize; Classic→Focus open-region editor holds gate lines
         only

## Outcomes

- `apply_viewer_style` / `apply_viewer_style_to_sessions` rematerialize only when style
  changes (no-op when already stamped).

- Settings `ViewerStyleSelected` rebuilds every interaction session; interaction side
  effects stamp all sessions in the panel on each update.
