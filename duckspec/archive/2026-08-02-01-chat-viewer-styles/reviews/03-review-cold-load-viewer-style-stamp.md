# Review: Cold load viewer style stamp

Accepted one residual gap after style-flip rematerialize: session load materializes under
default Classic stamp while the view can already use stored Focus, so Hybrid C editor
content can be wrong on first paint. Mid-session Settings / interaction flips remain
fixed. Default Classic cold load is unaffected.

## Scope

Full change through step 07: proposal, Hybrid C design, caps `chat/viewer-style` and
`chat/focus-answer`, steps 01–07, Focus geometry, Answer presentation, Settings,
`apply_viewer_style` / Settings rematerialize, load path `ensure_sessions_with_label`.
Prior reviews 01–02. Mechanical: `ds check` / `ds audit` clean; Focus/viewer-style tests
green including style-flip rebuild.

## Summary

```
| # | finding | resolution | → next |
| --- | --- | --- | --- |
| 1 | Cold load materializes without effective style stamp | Stamp effective style before first materialize on load | `/ds-step` |
```

## Findings

### 1. Cold load materializes without effective style stamp

**Where:** `ensure_sessions_with_label` in `crates/duckboard/src/area/interaction.rs`
(load loop calls `materialize_chat_ui` after `AgentSession::from_session`);
`update_with_side_effects` order (apply then ensure); external reload materialize; Hybrid
C `answer_editor_desired_lines` depends on `ax.viewer_style`.

**Evidence:** Loaded sessions default `viewer_style` to Classic and materialize
immediately. View layout reads `config.chat.effective_viewer_style()` each frame. With
stored Focus, first paint can use Focus layout while editors still hold full-body lines,
so the open-region TextEdit can show the whole Answer until a later `apply_viewer_style`
runs. Mid-session Settings and interaction stamp paths already rematerialize on change
(step 07). Default Classic load stamp matches effective style.

**Impact:** Focus cold start / first open of a scope can show incorrect open-region
content. Classic default identity on cold load remains correct. Same class of desync as
review 02, different entry point.

**Discussion:** (A) Thread effective style into load paths and stamp before first
materialize (or apply after load) — smallest complete fix. (B) Accept lag until first
interaction — Focus-only residual. (C) Separate open-region editors — larger redesign.

**Resolution:** A — stamp effective viewer style before first materialize on load and
equivalent paths so Hybrid C desired editor lines match the view from first paint.

**Next:** `/ds-step` — pass effective style into `ensure_sessions_with_label` (and any
load/reload materialize that assumes stamp already set); stamp then materialize; keep
existing apply-on-change paths.

## Resolved concerns

- **Review 01 Hybrid C / band / Classic identity:** implemented; not reopened.

- **Review 02 mid-session style flip:** Settings + `apply_viewer_style` covered; unit test
  green; not reopened.

- **Paint/band pure-helper tests:** still accepted for v1.

- **Working-copy noise:** not a product finding for this change.

## Outcome

Not ready to archive. Primary route is **`/ds-step`**: stamp effective viewer style on
cold load / first materialize so Focus Hybrid C editor content is correct on first paint.
