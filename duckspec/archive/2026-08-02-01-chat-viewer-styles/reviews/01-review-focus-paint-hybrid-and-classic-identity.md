# Review: Focus paint hybrid and Classic identity

Accepted: Hybrid Focus paint (open region = Classic TextEdit; sections may be plain),
last-answer band only on the open region, and Classic default must stay behavior-identical
to pre-change. Spec and step work follow; design amendment is the earliest layer.

## Scope

Proposal, design, caps `chat/viewer-style` and `chat/focus-answer`, five steps, Focus
geometry module, Answer presentation / Settings wiring, and the classic-paint stub test.
Mechanical: `ds check` / `ds audit` were clean when the change was considered complete;
Focus-related unit tests were green at that point.

## Summary

```
| # | finding | resolution | → next |
| --- | --- | --- | --- |
| 1 | Focus “classic paint” overclaimed | Hybrid C: open region TextEdit; expanded sections plain content-font OK | `/ds-design` |
| 2 | Last-answer band on every Focus slice | Band only on the open region for the last Answer | `/ds-step` |
| 3 | Classic-paint scenario is a stub | Retarget for Hybrid C; real tests open-region classic vs section plain | `/ds-spec` |
```

## Findings

### 1. Focus “classic paint” overclaimed

**Where:** `design.md` Focus interaction; `chat/focus-answer` Classic paint requirement;
`classic_answer_slice_view` (plain `text()` for all Focus slices); step 04 Outcomes
(explicit deferral).

**Evidence:** Design called for Classic paint via TextEdit slices. Spec requires classic
Answer body paint for expanded sections **and** open region. Implementation used
content-font `text()` for both; tables, highlight, and meta-card tint do not apply.

**Impact:** Write-gate previews under Focus read worse than Classic; contract overclaims
shipping behavior.

**Discussion:** Full multi-slice TextEdit (B) matches original design but is heavy.
Accepting plain text everywhere (A) is honest but weak for the gate. Hybrid (C): open
region uses real Classic TextEdit; foldable section bodies stay plain source text for v1.

**Resolution:** Hybrid C.

**Next:** `/ds-design` — amend Focus chrome so classic paint applies to the open region as
TextEdit; expanded sections may use simpler content-font presentation. Then `/ds-spec` and
`/ds-step` for contract and implementation.

### 2. Last-answer band on every Focus slice

**Where:** `view_focus_answer` / `classic_answer_slice_view` passing `is_last_answer` into
each slice; contrast with `chat/answer-landmarks` (single band target).

**Evidence:** Classic applies the band once to the Answer. Focus applied
`chat_last_answer_band` to every expanded section plus the open region when the Answer was
last.

**Impact:** Multiple contrast strips on one reply; harder to scan; inconsistent with
landmarks.

**Discussion:** Outer single band (B) vs open-region-only (A). Open-region-only matches
Hybrid C (gate is the emphasized primary).

**Resolution:** Band only on the open region for the last Answer (A).

**Next:** `/ds-step` — only the open-region slice (or its TextEdit) gets last-answer band
styling.

### 3. Classic-paint scenario is a stub; Classic default is the real bar

**Where:** `focus_body_uses_classic_paint() -> true` and the linked paint scenario;
product priority from review discussion.

**Evidence:** Paint scenario always passes. Product bar: **default Classic must work
exactly as before** (full TextEdit Answer path, one band, tables/tint/find). Focus is
opt-in only.

**Impact:** Focus paint drift is untested; more importantly, any change must not alter
default Classic behavior.

**Discussion:** Recommend retargeting paint scenarios for Hybrid C (open-region classic
path vs section plain path) and replacing the stub with real assertions. Classic identity
remains the acceptance criterion for default settings.

**Resolution:** Amend `chat/focus-answer` Classic-paint for Hybrid C; replace stub with
tests distinguishing open-region classic path vs section plain path. Treat Classic default
identity as non-negotiable for the change.

**Next:** `/ds-spec` — rewrite Classic paint requirement/scenarios for Hybrid C; keep /
strengthen Classic default identity scenarios under `chat/viewer-style`. Then `/ds-step`
for tests and open-region TextEdit + band fix.

## Resolved concerns

- **Working-copy noise** (half-applied tree, other concurrent work): acknowledged; not
  treated as a separate product finding for this review record.

- **Unknown TOML `viewer_style` failing whole config load:** left as optional hardening;
  not accepted as a required finding.

## Outcome

Not ready to freeze/archive. Earliest route is **`/ds-design`** (Hybrid C paint + band
policy on the open region). Then **`/ds-spec`** (Hybrid paint contracts + Classic
identity), then **`/ds-step`** (open-region TextEdit, open-region-only band, real tests).
Default Classic must remain behavior-identical to pre-change.
