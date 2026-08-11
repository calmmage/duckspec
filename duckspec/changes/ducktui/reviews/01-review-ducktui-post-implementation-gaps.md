# Review: Ducktui post-implementation gaps

Audit and step checkboxes are green, but the interactive product and a few contracts still
lag the design. Five agreed follow-ups, all keeping design direction **A** (finish
implementation) rather than narrowing the vision; Ideas needs a short contract correction
first.

## Scope

Reviewed change `ducktui`: proposal, design, caps (`chat/session-sharing`, `tui/shell`,
`tui/navigator`, `tui/chat`), all five steps (checked), `crates/duckcore`,
`crates/ducktui`, duckboard rewire surfaces, `@spec` backlinks, `ds check` /
`ds audit ducktui` (clean). Stack note: TUI is **ratatui + crossterm + tokio**, not
Textual.

## Summary

```
| # | finding | resolution | → next |
| --- | --- | --- | --- |
| 1 | Agent + file event loop not live | Keep design; wire agent, watcher, stream apply in the app | /ds-step |
| 2 | Transcript segments not in duckcore | Extract shared segment builder as design already requires | /ds-step |
| 3 | Drive-only writes incomplete in duckboard | Route all persists through drive-aware gate | /ds-step |
| 4 | Settings / project picker shells only | Implement real picker + settings against shared config | /ds-step |
| 5 | Invented fixed Ideas scope | Align with duckboard idea→change/exploration binding; fix contract | /ds-spec |
```

## Findings

### 1. Agent + file event loop not live

**Where:** `crates/ducktui/src/main.rs` (`AppEvent::Agent` / `Files` stubs); submit path
in `dispatch_key` (persist only); design runtime and session-sharing sections; chat step
task 5.

**Evidence:** Agent and file variants are never produced. Enter sets
`agent_drive_requested` and may persist, but does not spawn `drive_harness` or apply
`AgentEvent`s to the transcript. No `spawn_watcher` in ducktui. Session-sharing helpers
exist and are tested; the TUI process does not run the watch/reload path.

**Impact:** The binary does not “drive agents” or live-share sessions as
proposed/designed, despite green unit specs.

**Discussion:** Narrowing design to a non-agent shell (B) was rejected. Document-only debt
(C) was rejected. Finishing the designed loop (A) preserves “same chat, different
renderer.”

**Resolution:** Implementation incomplete. Keep design; plan and implement full event loop
wiring.

**Next:** `/ds-step` — steps to: start watcher and forward `FileEvent`s; on submit spawn
`drive_harness` and merge `AgentEvent`s into session/`ChatPane`; apply stream-ui cadence
on tick.

### 2. Transcript segments not in duckcore

**Where:** `duckspec/changes/ducktui/design.md` (transcript rendering);
`crates/duckboard/src/widget/agent_chat.rs` (`TranscriptSeg`,
`build_transcript_segments`); `crates/ducktui/src/chat_pane.rs` (`PresentSeg`).

**Evidence:** Design requires duckcore to expose the shared Thinking/Activity/Answer
model. Construction remains duckboard-only; ducktui uses a parallel presentation model and
test-injected segments.

**Impact:** Drift vs `chat/transcript` and harder correct agent wiring.

**Discussion:** Amending design to allow dual models (B) was rejected. Extract (A) matches
settled design.

**Resolution:** Move UI-neutral segment construction (and collapse defaults as needed)
into duckcore; both UIs render from it.

**Next:** `/ds-step` — extract builder into duckcore; rewire duckboard; feed ducktui from
real sessions.

### 3. Drive-only writes incomplete in duckboard

**Where:** `chat/session-sharing` specs; `duckcore::session_sharing`; duckboard
`persist_session_snapshot` vs numerous `chat_store::save_session` call sites in
`area/interaction.rs`, `main.rs`, etc.

**Evidence:** Gated flush path exists; many turn and bookkeeping saves still call
`save_session` without `DriveRole`.

**Impact:** Displayed sessions can still be overwritten when coexisting with ducktui.

**Discussion:** Spec narrowing (B) rejected. Full gate (A) accepted.

**Resolution:** Every persist path marks driven when appropriate and uses `persist_driven`
(or equivalent).

**Next:** `/ds-step` — inventory and convert duckboard save sites; keep real turns
writing.

### 4. Settings / project picker shells only

**Where:** Design screens table; `crates/ducktui/src/ui.rs` (static picker/settings);
`crates/ducktui/src/config.rs` (read-only subset); shell specs (transitions only).

**Evidence:** Project bind in-app is not implemented (CLI/`bind_project` only). Settings
does not write shared harness/model/oneshot prefs. Shell specs pass without requiring
those behaviors.

**Impact:** App is hard to use as a standalone companion even after agent wiring.

**Discussion:** CLI-only bind (B) rejected. Implement as designed (A) accepted; may follow
agent wiring in step order.

**Resolution:** Real path entry + recents; settings edits shared config; theme may stay
app-local per design.

**Next:** `/ds-step` — picker and settings implementation steps (and extend shell/settings
contracts only if new requirements need `@spec` later).

### 5. Invented fixed Ideas scope

**Where:** `tui/navigator` selection requirement (fixed ideas scope); `BoundChat::Ideas` /
chat key `"ideas"`; duckboard `area/ideas.rs` `scope_for_path` → `Scope::Change` /
`Scope::Exploration` (inbox ideas have no chat scope).

**Evidence:** No `Scope::Ideas`. Ducktui invents a fixed `"ideas"` session key. Duckboard
binds chat per idea’s change or exploration, not a global ideas scope.

**Impact:** Wrong session files and orientation; dual-app mismatch.

**Discussion:** Adding `Scope::Ideas` (B) would invent store semantics. Documenting the
mismatch only (C) rejected. Align with duckboard (A) accepted.

**Resolution:** Ideas entry should match duckboard: e.g. open Ideas-oriented flow without
a fake global scope (list/select idea → bind change/exploration, or quick-idea path); drop
fixed `"ideas"` key unless duckboard gains one. Update navigator (and related) specs
before or with code.

**Next:** `/ds-spec` — rewrite Ideas selection contract to duckboard-aligned semantics;
then `/ds-step` to implement.

## Resolved concerns

- **Textual vs ratatui:** Building on Textual is possible only via Python rewrite or
  PyO3/RPC; not a drop-in. Stay on ratatui for this change.

- **Mechanical audit:** `ds audit` / `@spec` coverage are green; review findings are
  product and ownership gaps, not missing backlinks.

- **Overlay stubs:** Acceptable for shell MVP if full overlays ship with later steps; not
  a separate finding.

## Outcome

Not ready to archive. Design direction stands. Earliest corrective stage is `/ds-spec`
(Ideas binding). Then `/ds-step` for agent/watcher loop, transcript extract, drive-only
saves, picker/settings, and Ideas implementation.
