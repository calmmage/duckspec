# Ducktui - Design

A new `crates/ducktui` terminal app (ratatui + crossterm + tokio) over a new shared crate
`crates/duckcore` that absorbs duckboard's UI-neutral chat and scope logic. Duckboard
keeps only thin iced adapters; ducktui adds equally thin terminal adapters. Session files,
config, and harness behavior are shared, so the same chats appear in both apps.

## Crate layout

```
duckpond ──────────► duckspec/ state (unchanged)
duckchat ──────────► harness turns (unchanged)
duckcore (NEW) ────► chat_store, scope, slash_commands, meta_card,
                     fast_response, agent core (provider registry,
                     turn driver, AgentEvent), watcher core
duckboard ─────────► iced views + iced adapters over duckcore
ducktui (NEW) ─────► ratatui views + tokio adapters over duckcore
```

Moved modules are already iced-free (`chat_store.rs`, `scope.rs`, `slash_commands.rs`,
`meta_card.rs`, `fast_response.rs` have zero iced imports). Two modules split:

- `agent.rs` — the provider registry, turn driver, and `AgentEvent` mapping move to
  duckcore over `tokio::mpsc`; the ~60 lines of `iced::Subscription` /
  `iced::stream::channel` glue stay in duckboard.

- `watcher.rs` — the notify-debouncer + gitignore core and `FileEvent` enum move to
  duckcore; the iced subscription wrapper stays in duckboard.

Rejected: folding these into duckchat (session persistence and duckspec scope binding are
app concerns, not harness concerns) and duplicating into ducktui (guaranteed drift against
"same chat, different renderer").

## Runtime and event architecture

Single tokio runtime; one app event loop selecting over three sources:

```
crossterm EventStream ──┐
duckcore AgentEvent  ───┼──► AppEvent ──► update(state) ──► draw(frame)
duckcore FileEvent   ───┘         ▲
                                  │ tick (streaming cadence)
```

- Terminal input, agent events, and file events merge into one `AppEvent` enum via
  `tokio::select!`.

- Streaming follows the `chat/stream-ui` contract's shape: agent text applies to session
  state immediately; the terminal redraws on a bounded tick (~30ms) plus structural
  immediacy (new segment, turn end).

- Redraw is full-frame (ratatui immediate mode); no dirty tracking needed at TUI scale.

## Session sharing and concurrency

Sessions stay in the existing per-scope directories as `{session_id}.json` with the
existing atomic write (`chat_store::write_atomic`), unchanged on disk — duckboard and
ducktui read and write the same files.

- **Ownership by turn:** an app only writes sessions it is actively driving (its own
  in-flight or completed turns). Neither app ever writes a session it merely displays.

- **Reload on external change:** each app watches the session directories via the shared
  watcher core and reloads sessions it is not driving when their files change. A session
  with a local in-flight turn ignores external file events until the turn settles.

- Concurrent turns on the *same* session from both apps are unsupported and unprevented
  (same as two duckboard windows today); last atomic write wins. No lock files.

Config is the existing shared `config_path()` file; ducktui reads the same
harness/model/oneshot preferences. Theme is app-local (terminal palette).

## Screens and shell

Three screens with a bottom status bar (scope, model, context fill, turn state) and modal
overlays on the work screen:

```
| Surface | Content |
| --- | --- |
| Project picker | Recent projects from shared config, path entry |
| Work screen | Scope navigator pane + chat pane |
| Settings | Harness/model prefs, oneshot models, written to shared config |
| Overlays | Model picker, slash palette, session switcher, quick idea, help |
```

Focus model: exactly one focused pane (navigator or chat) toggled with Tab; overlays
capture all input while open; global keys (quit, help, screen switches) live on a
leader-free top layer checked before pane dispatch.

## Scope navigator

Data joins three existing sources, all through duckcore/duckpond — no new state on disk:

- changes with phase state (duckpond, as the Dashboard area reads today)
- explorations and their chats (`chat_store::load_sessions_for`)
- session counts per scope for row badges

One flat tree: CHANGES section (phase pill as a colored glyph), EXPLORATIONS section,
collapsed ARCHIVED section, and bottom entries for Ideas, Codex, Settings. Selecting a row
binds the chat pane to that row's `Scope` (shared `scope.rs` type and semantics —
identical binding to duckboard, including exploration promotion). Ideas and Codex entries
open their scope's chat; they get no content browser.

## Transcript rendering

The segment model is reused, not redesigned: duckcore exposes the same Thinking / Activity
/ Answer transcript the `chat/transcript` capability defines, and ducktui renders it.

- Markdown renders via pulldown-cmark to owned styled lines (a ducktui `md_render`
  module); tables get the same fit-to-width treatment the `editor/md-table` capability
  defines, degraded to plain cell wrap when the pane is narrow.

- Thinking and Activity segments render collapsed to one summary line each, expandable per
  segment; the Answer is always expanded — matching the settled-collapse defaults of
  `chat/transcript`.

- Meta cards render as bordered blocks; a trailing `next` meta card's tokens become
  numbered pressable hints (1..9) consistent with fast-response chips.

- The transcript is a virtual list over rendered line blocks with a viewport; autoscroll
  pins to bottom during streaming and releases on manual scroll, matching
  `chat/session-scroll` intent.

## Composer and input

- Multiline editor: `tui-textarea` (or equivalent) embedded at the chat pane bottom; Enter
  sends, Alt+Enter inserts a newline (Shift+Enter only where the terminal's keyboard
  protocol reports it — kitty protocol enabled when available, Alt+Enter is the portable
  binding).

- `/` at line start opens the slash palette fed by duckcore `slash_commands`; `//` escape
  passes through, matching the `chat/slash-commands` contract.

- Fast-response options render as numbered chips above the composer; number keys activate
  while awaiting, matching `chat/fast-response` semantics (freeform submit completes the
  pending choice as a custom answer).

- Model picker overlay lists the duckchat model catalog grouped by harness
  (`harness/model-picker` semantics); context meter in the status bar uses the selected
  model's window.

## Settled choices

- New crate `duckcore` for shared logic; name is cosmetic and can change at implementation
  without design impact.

- ratatui + crossterm + tokio; no alternative TUI stack considered worth the trade
  (cursive's retained tree fights the streaming model).

- No middle content column, ever — reading and editing artifacts stays in the user's
  editor. The navigator is navigation only.

- Duckboard behavior is the reference implementation: where a chat semantic is already
  specified (`chat/*`, `harness/*` capabilities), ducktui conforms to the existing
  contract rather than forking it.
