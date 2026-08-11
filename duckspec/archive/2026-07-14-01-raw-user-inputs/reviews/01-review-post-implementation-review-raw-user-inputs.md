# Post-implementation review: raw user inputs

Reviewed proposal → design → caps → code for the inputs ledger. Core export and package
placement work; agent re-grounding cues are thinner than design promised.

## Scope

`proposal.md`, `design.md`, `caps/chat/inputs-ledger`, `caps/session/scope` deltas, steps
01–03, `inputs_ledger.rs`, `chat_store` save/merge hooks, `scope` orientation, live
`inputs.md` dogfood. Post-implementation; full chain to code.

## Summary

```
| # | sev | lens | title | → next |
| --- | --- | --- | --- | --- |
| 1 | minor | fidelity | Design template mentions for inputs.md never shipped | ignore |
| 2 | minor | fidelity | Orientation pointer usually misses the first session | ignore |
| 3 | minor | quality | Message timestamps never set; design format is dead | ignore |
```

## Findings

### 1. Design template mentions for inputs.md never shipped - fidelity/minor

**Where:** `design.md` Agent re-grounding — Templates (light);
`crates/duckspec/content/templates/`

**Why:** Propose/design/explore were supposed to prefer quotes from `inputs.md`. Only
orientation carries that cue. Later stages that don’t re-prime can still invent motivation
without being told to open the ledger.

**Action:** Optional template one-liners, or accept orientation-only as v1 and drop the
design sentence on archive.

### 2. Orientation pointer usually misses the first session - fidelity/minor

**Where:** `crates/duckboard/src/area/interaction.rs` priming + `session_scope_for_ax`;
`crates/duckboard/src/scope.rs` `has_inputs_ledger`

**Why:** Orientation runs on first-turn prime, typically **before** the first real user
message is saved and `inputs.md` is written. Pointer is correct for **later** chats on the
same change; the long first session that *builds* the ledger rarely sees it.

**Action:** Optional follow-up: always name the conventional path for change scopes, or
re-check after first export. Not blocking if “later session” re-grounding is enough.

### 3. Message timestamps never set; design format is dead - quality/minor

**Where:** design ledger format (`<timestamp>`); production `timestamp: String::new()` on
user messages

**Why:** Render supports timestamps, but nothing stores them, so archaeology lacks
per-message time. Spec doesn’t require timestamps — only design shows them.

**Action:** Ignore for v1, or populate timestamps when creating user messages later.

## Verdict

**Archive-ready.** Export path, filter, package placement, promotion hook, and tests match
the proposal jobs (fresh ledger, human audit, archive ride-along). Remaining items are
optional re-grounding polish, not structural holes.
