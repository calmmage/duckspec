# Ducktui

A terminal UI companion over the same `duckspec/` directory and duckchat harnesses — a
keyboard-driven way to navigate scopes and drive coding agents from any terminal, without
duckboard's macOS-only shell.

Duckboard is an iced macOS app: icon sidebar, content column, chat column. That shell
doesn't travel — not over ssh, not into tmux, not onto Linux. The agent-driving core
underneath it (duckpond for `duckspec/` state, duckchat for harness turns) is UI-agnostic,
so a terminal front end can reuse it wholesale.

Ducktui deliberately drops duckboard's middle content column. In duckboard that column is
where artifacts are read and edited *and* where scope selection happens; ducktui keeps
only the second job and moves it into the sidebar. Reading `proposal.md` or specs stays in
your editor — ducktui is navigate + chat.

```
┌─ SCOPE ──────────┬─ CHAT ────────────────────────────────┐
│ CHANGES          │  transcript                           │
│  ● build-pilot   │   · thinking / activity (collapsed)   │
│  ○ session-work… │   · answer (markdown, meta cards)     │
│ EXPLORATIONS     │                                       │
│  · ducktui …     │ ┌───────────────────────────────────┐ │
│ IDEAS CODEX ⚙    │ │ composer  /slash  option chips    │ │
├──────────────────┴───────────────────────────────────────┤
│ status: scope · model · context fill · turn state        │
└──────────────────────────────────────────────────────────┘
```

## Screens

```
| Screen | Role |
| --- | --- |
| Project picker | Launcher when no project is bound: recent projects, path entry |
| Work screen | Scope navigator (left) + agent chat (right); the app lives here |
| Settings | Harness/model preferences, oneshot models, theme |
```

Everything else is an overlay on the work screen, not a screen: model picker,
slash-command palette, chat-session switcher, quick idea capture, help. A status bar
(scope, model, context fill, turn state) spans the bottom.

The left pane absorbs what duckboard splits between its icon rail and content lists:
active changes with phase state, explorations, archived collapsed, with
Ideas/Codex/Settings entries at the bottom. Selecting a row binds the chat scope, matching
duckboard's exploration/change binding.

## Boundaries

- No caps browser, artifact editors, diff view, or embedded terminal — the content
  column's reading/editing jobs are out of scope, permanently, not deferred.

- Chat sessions are shared with duckboard: the same session files, so a conversation
  started in one appears in the other.

- Harness behavior, meta cards, slash commands, and fast-response chips should match
  duckboard's semantics — same chat, different renderer.

## Open questions

- The chat/session logic ducktui needs (session store, agent turn driver, slash commands,
  meta cards, fast response, scope binding) currently lives inside the duckboard crate,
  not a library. Extract a shared crate versus duplicate is the central design decision.

- How much of duckboard's transcript presentation model (segment collapse, streaming
  cadence) transfers to a terminal renderer versus needs rethinking.
