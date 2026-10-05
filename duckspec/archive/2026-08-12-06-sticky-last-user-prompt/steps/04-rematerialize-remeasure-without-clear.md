# Rematerialize remeasure without clear

After transcript rematerialize, refresh sticky content top when a source exists even if
idx/body did not change; only clear (hide until measure) when source changes or is absent
— so auto-collapse of earlier Thinking/Activity does not leave a permanent stale top.

## Prerequisites

- [x] @step selective-sticky-content-top-invalidation

## Context

Second-review residual: `sync_collapse_states` can auto-collapse Thinking/Activity on
rematerialize without changing last-user idx/body. Step 03 keeps the content-top cache in
that case, but `maybe_measure_sticky_last_user` skips when the cache is `Some`, so the top
never refreshes. Fix: schedule remeasure after materialize without clearing unless source
changed.

## Tasks

- [x] 1. Ensure post-materialize path can request a sticky content-top measure even when
         `sticky_last_user_content_top` is already `Some` (today
         `maybe_measure_sticky_last_user` no-ops when cache is set)

- [x] 2. Keep selective clear: only set content top to `None` when last-user idx/body
         changed or no source (`sync_sticky_last_user` policy from step 03)

- [x] 3. On rematerialize with unchanged source, schedule remeasure so the next layout op
         overwrites the cached top without force-hiding the sticky

- [x] 4. Smoke: rematerialize that auto-collapses earlier Thinking/Activity with unchanged
         last user still ends with an up-to-date content top (sticky remains shown when
         offset still qualifies)

## Outcomes

`sticky_last_user_remeasure` is set on materialize when a source exists; measure runs even
if a top is cached and clears the flag on `StickyLastUserLayout`. Source change still
clears the top (brief hide until measure); unchanged source keeps the top while remeasure
overwrites it.
