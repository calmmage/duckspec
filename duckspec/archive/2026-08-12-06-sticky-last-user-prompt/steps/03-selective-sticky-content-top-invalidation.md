# Selective sticky content-top invalidation

Stop wiping sticky content top on every rematerialize; clear and remeasure only when
last-user source idx or body changes (collapse/focus fold stay as today).

## Prerequisites

- [x] @step sticky-slot-and-measure-cache

## Context

Review finding 1 and amended design Measurement and state (v1 invalidation): rematerialize
must not clear `sticky_last_user_content_top` unless last-user block index or body text
changed. Width reflow remains out of scope.

## Tasks

- [x] 1. Track last sticky source identity (block index) and/or body used for cache
         validity

- [x] 2. Change `sync_sticky_last_user` so rematerialize re-resolves idx/body, updates the
         sticky editor when body changes, and clears `sticky_last_user_content_top`
         **only** when idx or body changed (or no source); otherwise keep the cached top

- [x] 3. Keep collapse toggle and focus-fold paths clearing content top (unchanged)

- [x] 4. Smoke: during a streaming rematerialize with unchanged last user, sticky stays
         visible when already shown (no forced hide from materialize alone)

## Outcomes

Smoke verified by code path: `sync_sticky_last_user` no longer clears content top when
`new_idx == sticky_last_user_source_idx` and body text matches the sticky editor;
streaming rematerialize therefore leaves a cached top (and sticky show) intact until
measure is needed for a real source change.
