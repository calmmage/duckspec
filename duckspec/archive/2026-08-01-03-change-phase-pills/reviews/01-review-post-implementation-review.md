# Post-implementation review

Phase pills land as designed for derived short labels, Settings gates, VCS late-stage
pills, and click-to-send. Review found one real list-click bug (fixed in tree), one
product dual-pref issue (follow-up), and informal manual chrome checks (accepted for now).

## Scope

Change `change-phase-pills`: proposal, design, `shell/phase-pills` spec/doc, three steps,
duckboard config/settings, pure `PhaseDisplay`, list + composer chrome, activation
routing. Mechanical `ds check` / `ds audit` clean; 16/16 `test: code` scenarios linked.

## Summary

```
| # | finding | resolution | → next |
| --- | --- | --- | --- |
| 1 | Exploration list pill opened rename | Fixed: skip SelectChange when already selected | — |
| 2 | Dual phase visibility prefs on the list | Prefer short pills; hide/no-op Sort Phase when on | /ds-step |
```

## Findings

### 1. Exploration list pill opened rename

**Where:** `crates/duckboard/src/area/change.rs` (`SelectChange` second-click rename);
`crates/duckboard/src/main.rs` (`handle_list_phase_pill_send`)

**Evidence:** `SelectChange` on an already-selected exploration starts rename. List
phase-pill send always called `SelectChange` then submit, so a pill on the focused
exploration could open rename instead of only sending `/ds-explore` (or other next-stage
text).

**Impact:** Breaks the common “already looking at this exploration → click pill → send
stage command” path.

**Discussion:** Options: skip select when already selected (A); dedicated focus message
(B); leave as-is (C). Chose A so row second-click and pencil rename stay.

**Resolution:** Fixed: select only when target ≠ current selection; if already selected,
ensure session/panel and submit without `SelectChange`. Row rename and pencil unchanged.

**Next:** — (fixed in working tree; no further stage work for this finding)

### 2. Dual phase visibility prefs on the list

**Where:** `list.show_phase_pillows` (Change list Sort menu); `ui.phase_pill_list`
(Settings → Phase pills); list row trailing in `area/change.rs`

**Evidence:** Sort menu toggles long phase text pillows. Settings toggles short clickable
pills. When short pills are on, long phase is suppressed on active change rows, but two
knobs remain.

**Impact:** Confusing “turn Phase off / on” story; users may think list phase chrome is
one control.

**Discussion:** A leave both; B prefer short pills (ignore/hide Sort Phase when
`phase_pill_list`); C defer merge. Chose **B**.

**Resolution:** Accepted. When list phase pills are enabled, Sort “Phase” should not
control a parallel long-phase face (hide and/or no-op); short pills own list phase chrome.

**Next:** `/ds-step` - plan and implement: when `config.ui.phase_pill_list` is true, hide
or disable Sort “Phase” and ensure long-phase pillows do not reappear via
`show_phase_pillows` alone.

## Resolved concerns

- **Manual placement / toggle scenarios** — Spec marks list/composer chrome as `manual:`;
  step 03 spot-check was not a formal GUI pass. Accepted for this cut: pure display +
  config tests and audit cover the `test: code` contract; operator smoke-test when running
  duckboard remains recommended, not a blocking defect.

## Outcome

Core phase-pill behavior is ready after the rename fix. Primary remaining work is finding
2 (list pref consistency) via `/ds-step` before archive if one coherent phase control
story is required for this change.
