# Review: Ducktui interactive and rendering gaps

Prior review gaps (agent loop, transcript extract, drive-only persists, picker/settings,
Ideas binding) are closed in code. Mechanical audit is green. The product still lacks
interactive navigation, shared-session load on bind, live chat viewport keys, markdown
rendering, and real overlay bodies — all under settled design/specs.

## Scope

Reviewed change `ducktui` after steps 6–10: proposal, design, caps
(`chat/session-sharing`, `tui/shell`, `tui/navigator`, `tui/chat`), all ten steps
(checked), prior review `01-review-ducktui-post-implementation-gaps.md`,
`crates/duckcore`, `crates/ducktui` (especially `main.rs`, `shell.rs`, `navigator.rs`,
`chat_pane.rs`, `runtime.rs`, `ui.rs`, `config.rs`), duckboard persist call sites, `@spec`
backlinks, `ds check` / `ds audit
ducktui` (clean). Stack remains ratatui + crossterm +
tokio.

## Summary

```
| # | finding | resolution | → next |
| --- | --- | --- | --- |
| 1 | Navigator not interactively usable | Wire move + activate keys on nav focus | /ds-step |
| 2 | Scope bind does not load shared sessions | Load latest session for scope on bind | /ds-step |
| 3 | Expand / scroll not keyed | Wire expand + scroll in the event loop | /ds-step |
| 4 | Markdown / md_render missing | Implement design markdown renderer | /ds-step |
| 5 | Overlay bodies still stubs | Implement full overlays (slash, model, sessions, quick idea, help) | /ds-step |
```

## Findings

### 1. Navigator not interactively usable

**Where:** `crates/ducktui/src/navigator.rs` (`select`, tree); `shell.rs`
(`apply_nav_select`); `main.rs` work-screen dispatch; `tui/navigator` selection scenarios.

**Evidence:** Selection APIs and unit tests exist. With navigator focused, keys map to
`pane-char` / `KeyEffect::Pane(Navigator)` and stop — no move-cursor API and no Enter path
calling `select` / `apply_nav_select`. Settings via Ctrl+, works; tree Settings does not.

**Impact:** Core “navigate scopes + chat” flow is blocked in the live binary despite green
navigator specs.

**Discussion:** Narrowing to CLI-only scope (B) or treating selection as programmatic-only
(C) was rejected. Wiring keys (A) matches settled design and specs.

**Resolution:** Keep design/specs; implement interactive navigator (highlight move +
activate).

**Next:** `/ds-step` — j/k or arrows to move selection among `visible_rows`; Enter
activates `select` + `apply_nav_select`; optional first-row default selection.

### 2. Scope bind does not load shared sessions

**Where:** Design session-sharing / “same chats in both apps”; `shell.rs`
`apply_nav_select` (`ChatPane::new` only); duckcore `load_sessions_for`.

**Evidence:** Binding a scope creates an empty new session. Disk sessions for that scope
are not loaded. Watcher only reloads the active session id after external change.

**Impact:** Opening a change that already has duckboard history shows an empty chat;
continuity is accidental.

**Discussion:** Deferring until a session switcher (B) or always-new-session (C) rejected.
Load latest on bind (A) matches design; multi-session UI is finding 5.

**Resolution:** On scope bind, load the latest existing session for that scope (empty only
if none). Rule of thumb: most recent by store/mtime consistent with duckboard defaults.

**Next:** `/ds-step` — after bind, load via shared store into `ChatPane` with
`DriveRole::Displayed` until a local turn drives.

### 3. Expand / scroll not keyed

**Where:** `tui/chat` expand and autoscroll scenarios; `chat_pane.rs` (`expand_segment`,
`scroll_up`); `main.rs` (no bindings).

**Evidence:** Unit tests exercise APIs; the event loop never calls them. Streaming pin
logic exists for presentation updates only.

**Impact:** Users cannot inspect long transcripts or expand collapsed Thinking/Activity in
the product.

**Discussion:** Defer (B) or drop specs (C) rejected. Wire keys (A) so chat scenarios hold
in the binary.

**Resolution:** Bind scroll (e.g. PgUp/PgDn or arrows when appropriate) and expand for
collapsed segments when chat is focused.

**Next:** `/ds-step` — event-loop keys → viewport scroll and segment expand without
breaking composer typing.

### 4. Markdown / md_render missing

**Where:** `design.md` transcript rendering; `tui/chat` doc (markdown + tables);
presentation via plain `body_lines` / `Line::from` in `ui.rs`.

**Evidence:** No `md_render` module, no pulldown-cmark pipeline, no table fit-to-width.
Answers render as plain strings; meta-card chrome is not bordered blocks (next tokens only
as numbered hints).

**Impact:** Real agent output (code, structure, tables) is hard to read; design/doc claim
a renderer that is absent.

**Discussion:** Defer (B) and plain-text design amend (C) rejected. Implement (A).

**Resolution:** Add ducktui markdown rendering per design (styled lines; tables fit width;
narrow wrap).

**Next:** `/ds-step` — `md_render` + wire Answer/expanded bodies through it; keep segment
construction in duckcore.

### 5. Overlay bodies still stubs

**Where:** Design overlay list; `tui/shell` overlay capture specs; `ui.rs` overlay draw;
composer `OpenSlashPalette`.

**Evidence:** Capture/close works and is tested. Model, slash, session switcher, and quick
idea show stub text. Slash opens but has no catalog selection; model change is
settings-only.

**Impact:** Design-listed affordances are non-functional; multi-session use after finding
2 needs a switcher.

**Discussion:** Capture-only MVP (prior review / option D) and design drop (C) rejected.
Full implementation (A) accepted: slash, model, sessions, quick idea, and useful help.

**Resolution:** Implement real overlay UIs against shared catalog, slash registry, session
store, and idea capture paths as designed.

**Next:** `/ds-step` — steps for slash palette, model picker, session switcher, quick
idea, and help content; keep shell capture semantics.

## Resolved concerns

- **Prior review findings 1–5:** Agent/file event loop, duckcore transcript segments,
  duckboard drive-gated persists, project picker/settings writes, and Ideas without a
  fixed `"ideas"` session key are present in current code; not re-opened.

- **Mechanical audit:** Green `@spec` / `ds audit` do not imply product completeness; gaps
  above are interactive and rendering fidelity.

- **Textual:** Still out of scope; stay on ratatui.

## Outcome

Not ready to archive. Design and capability contracts stand. Earliest corrective stage is
`/ds-step` for interactive navigator, session load on bind, chat viewport keys, markdown
render, and full overlays (no design or spec amendment required by this discussion).
