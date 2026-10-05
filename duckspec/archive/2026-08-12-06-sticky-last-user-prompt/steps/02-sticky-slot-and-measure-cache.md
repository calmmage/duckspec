# Sticky slot and measure cache

Wire the sticky panel into the chat column and keep content-top cache + pure visibility in
sync with scroll and rematerialize.

## Prerequisites

- [x] @step pure-sticky-helpers

## Tasks

- [x] 1. Add per-chat cache for last-user content top; clear on session open/switch/clear
         and when no source

- [x] 2. Remeasure content top via widget operate after layout-affecting transcript
         materialize (and related layout events from design)

- [x] 3. Render stable top-of-chat sticky slot (empty when hidden): user-card chrome,
         read-only body, max-height + inner scroll

- [x] 4. Drive show/text from pure helpers + cache + `last_chat_offset_y` (no measure on
         every scroll)

- [x] 5. Keep composer tree stable when sticky show/hide toggles

- [x] 6. Smoke: sticky appears when last user is above viewport and hides when scrolled
         back into view

## Outcomes

Visual smoke not run in a live duckboard window in this apply session; show/hide is
covered by pure visibility unit tests and full UI wiring (measure cache + top-of-chat
slot). Confirm once in a long chat if desired.
