# Recommendation letter promotion

Parse trailing Recommendation letter from assistant text; promote matching A/B/C option
before decide (review finding 1).

## Context

Review 01 finding 1: contract requires trailing `Recommendation: **X**` promotion; code
only scans option send text for `recommended`.

## Tasks

- [x] 1. Add pure helper: trailing Recommendation letter from assistant source (e.g.
         `Recommendation: **B**` / `**Recommendation:** B`)

- [x] 2. Wire letter into `promote_recommended` / `decide` (pass last-assistant text from
         TurnComplete path when available)

- [x] 3. Unit-test letter-only promotion when option lines lack the word “recommended”

- [x] 4. @spec chat/build-pilot Unattended auto-send: Recommended option is auto-sent over earlier non-recommended
