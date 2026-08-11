# Review: Ducktui agent-send and slash fidelity

Prior interactive/rendering gaps are closed. Mechanical audit and unit specs are green.
The product still fails first-turn scope orientation, shared slash submit routing (help /
`//`), harness-discovered slash catalog merge, and mid-session navigator refresh — design
and contracts stay valid; implementation follows.

## Scope

Reviewed change `ducktui` after steps 11–16: proposal, design, caps
(`chat/session-sharing`, `tui/shell`, `tui/navigator`, `tui/chat`), all sixteen steps
(checked), prior reviews `01` and `02`, `crates/duckcore` (especially `scope.rs`,
`slash_commands.rs`), `crates/ducktui` (`runtime.rs`, `shell.rs`, `main.rs`,
chat/navigator/overlays), duckboard send path for comparison, `@spec` backlinks,
`ds check` / `ds audit ducktui` (clean). Stack remains ratatui + crossterm + tokio. Unit
tests: duckcore + ducktui green.

## Summary

```
| # | finding | resolution | → next |
| --- | --- | --- | --- |
| 1 | First-turn scope orientation not delivered | Wire duckcore orientation on ducktui send (first-turn vs resume) | /ds-step |
| 2 | Slash submit routing missing | Local `/help` + `//` rewrite; build pilot stays duckboard-only | /ds-step |
| 3 | Discovered slash catalog empty | Merge `CommandsAvailable` into palette via `build_completion_catalog` | /ds-step |
| 4 | Navigator stale after bind | Explicit refresh key; no full live watcher rebuild | /ds-step |
```

## Findings

### 1. First-turn scope orientation not delivered

**Where:** `session/scope` first-turn delivery; navigator doc (orientation hooks apply);
design shared `duckcore::scope`; `crates/ducktui/src/runtime.rs` `submit_user_prompt` /
`send_turn`; duckboard `send_agent_turn` priming / first-turn path in
`crates/duckboard/src/area/interaction.rs`.

**Evidence:** Ducktui sends `TurnRequest::new(prompt, root)` with optional `session_id`
and `model` only. It never builds `SessionScope`, runs `CurrentScopeHook`, or mirrors
duckboard first-turn priming / path-reference / related first-turn context. Hooks already
live in duckcore (`crates/duckcore/src/scope.rs`).

**Impact:** Agents driven from ducktui lack “default change is X / path is
`duckspec/changes/X/` …” context. That breaks the point of scope-bound chat and diverges
from duckboard on the same sessions.

**Discussion:** Narrowing product to no orientation (B) or document-only debt (C)
rejected. Implementing send-path orientation (A) matches design and existing contracts.

**Resolution:** Implement first-turn orientation in ducktui using duckcore hooks; follow
duckboard’s first-turn vs resume rules as far as the TUI needs (no design amend).

**Next:** `/ds-step` — on submit, assemble first-turn context when appropriate; keep
resume turns free of repeated orientation; keep drive-only persist behavior.

### 2. Slash submit routing missing

**Where:** Design / proposal “same chat” slash semantics; `chat/slash-commands`;
`crates/duckcore/src/slash_commands.rs` (`parse_submit_slash`); duckboard
`dispatch_user_submit`; ducktui submit always drives raw text.

**Evidence:** Palette open and `//` catalog non-open work. Submit never calls
`parse_submit_slash`. Bare `/help` becomes an agent turn; `//help` is not rewritten to
agent prompt `/help`; build-pilot locals are not special-cased.

**Impact:** System slash behavior forks across apps; `/help` and double-slash escape are
wrong in the TUI.

**Discussion:** Full parity including build pilot (A) is heavier (arm/kick state). Design
amend to freeform-only (C) rejected. Subset (B): local help + `//` agent escape; build
pilot remains duckboard-only for this change.

**Resolution:** Wire `parse_submit_slash` for `LocalHelp` (system help in transcript, no
agent turn) and `Agent { display, prompt }` including `//` rewrite. Do not implement TUI
build-pilot arm/kick in this change; if `LocalBuildPilot` appears, steps may treat it as a
clear no-op or freeform agent path without inventing pilot state.

**Next:** `/ds-step` — submit branch on shared parser; help body via existing duckcore
helpers; agent path uses display in session and prompt for the harness.

### 3. Discovered slash catalog empty

**Where:** Design slash palette fed by full catalog; duckboard merges `CommandsAvailable`;
`crates/ducktui/src/shell.rs` `refresh_slash_list` (`system_registry()` + `Vec::new()`);
`crates/ducktui/src/runtime.rs` `CommandsAvailable(_) => false`.

**Evidence:** Palette is system-only. After the worker advertises commands, ducktui drops
them. Typed `/ds-*` can still go to the agent as freeform, but completion/discoverability
is incomplete.

**Discussion:** Defer (B) and system-only design amend (C) rejected. Wiring the existing
event (A) is small and matches design.

**Resolution:** Store discovered commands; rebuild slash list with
`build_completion_catalog(system_registry(), discovered)`.

**Next:** `/ds-step` — runtime/shell catalog state, apply `CommandsAvailable`, refresh
open palette when catalog updates if useful.

### 4. Navigator stale after bind

**Where:** Design tree from project state; `Navigator::from_project` on bind and
quick-idea only; watcher applies active session file events only.

**Evidence:** New changes, phase moves, and session-count badges do not update until
rebind (or incidental rebuild). Live dual-app use leaves the left pane wrong.

**Discussion:** Full watcher-driven live refresh (A) is more work and selection thrash
risk. Pure defer (C) leaves no recovery short of rebind. Explicit refresh key (B) accepted
as the light fix.

**Resolution:** Add a documented navigator refresh key that rebuilds from project state
and preserves selection when possible. No requirement for continuous auto-refresh in this
change.

**Next:** `/ds-step` — key binding, rebuild helper, help overlay line.

## Resolved concerns

- **Prior reviews 01–02:** Agent/file loop, transcript extract, drive-only duckboard
  persists, picker/settings, Ideas binding, interactive navigator, session load on bind,
  expand/scroll, markdown, overlays — present in current code; not re-opened.

- **Model picker grouping:** Design says grouped by harness; flat list with `harness/id`
  labels accepted as enough for this change (no step, no design amend).

- **Mechanical audit:** Green `@spec` / `ds audit` do not imply product completeness; gaps
  above are agent-send and slash fidelity.

- **Build pilot in TUI:** Explicitly deferred with finding 2; not a separate open finding.

- **Textual:** Still out of scope; stay on ratatui.

## Outcome

Not ready to archive. Design and capability contracts stand. Earliest corrective stage is
`/ds-step` for first-turn orientation, slash submit subset (help + `//`), discovered slash
catalog merge, and navigator refresh key.
