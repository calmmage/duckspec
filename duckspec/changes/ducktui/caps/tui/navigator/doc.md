# Tui navigator

The left pane of the work screen. It turns existing duckspec, idea, and chat state into
one keyboard-navigable tree and binds the chat pane when a row is selected. It does not
own scope orientation text, exploration promotion rules, idea file storage, or chat
rendering.

## Tree shape

```
CHANGES
  ● build-pilot
  ○ session-work…     (3)
EXPLORATIONS
  · ducktui sketch
ARCHIVED              (collapsed)
────────────────
IDEAS                 (opens idea list in this pane)
CODEX
SETTINGS
```

```
| Section / entry | Source | Select action |
| --- | --- | --- |
| CHANGES rows | Active changes; phase glyph from change phase | Bind chat to `Scope::Change` |
| EXPLORATIONS rows | Non-archived explorations | Bind chat to `Scope::Exploration` |
| ARCHIVED | Archived changes/explorations; collapsed until expanded | Same bind rules when expanded and selected |
| Ideas | Bottom entry; idea list from the project's idea store | Open ideas navigation in the navigator only (no content browser). No fixed ideas session key. |
| Idea rows (under Ideas) | Project ideas | Change- or exploration-linked idea → bind that `Scope`; inbox-only idea → no chat scope |
| Codex | Fixed bottom entry | Bind chat to `Scope::Codex`; no content browser |
| Settings | Fixed bottom entry | Open settings screen via `tui/shell` |
```

There is no separate navigator database. Rows are derived from duckpond project state, the
idea store, and session counts from the shared session store.

## Selection and scope binding

Selection is scope binding, not content browsing. The shared `Scope` type and binding
semantics match duckboard so the same session directories and orientation hooks apply.
Exploration promotion remains owned by `exploration/promotion`; the navigator only selects
the exploration or the promoted change after that capability has done its work.

Ideas follow duckboard's idea-to-scope rule: an idea may point at a change name or an
exploration id; inbox-only ideas have no chat scope. Ducktui never invents a global
`"ideas"` session key. There is no middle column for idea file editing — ideas navigation
stays in the left pane (and quick capture may use an overlay owned by the shell).

Codex opens chat only. Settings is a shell transition, not a chat scope.

## Session count badges

```
| Persisted sessions | Badge |
| ------------------ | ----- |
| 0                  | none  |
| N > 0              | N     |
```

Counts come from the same session store both UIs share, so a conversation started in
duckboard updates the badge in ducktui after reload.

## Sibling ownership

```
| Concern | Owner |
| --- | --- |
| Orientation text for a bound scope | `session/scope` |
| Exploration → change promotion | `exploration/promotion` |
| Idea files and frontmatter links | `ideas/*` |
| Screens, focus, status bar | `tui/shell` |
| Transcript and composer | `tui/chat` |
| Cross-app session reload | `chat/session-sharing` |
```
