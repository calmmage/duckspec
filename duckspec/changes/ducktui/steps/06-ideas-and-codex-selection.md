# Ideas and Codex selection

Align Ideas with duckboard (no fixed `"ideas"` session key); bind Codex; cover the updated
navigator selection scenarios.

## Prerequisites

- [x] @step navigator

## Context

Review finding 5 and the post-review `tui/navigator` contract: Ideas opens idea navigation
in the left pane only; linked ideas bind change/exploration scopes; inbox-only ideas do
not invent a chat scope. Codex binds `Scope::Codex`.

## Tasks

- [x] 1. Drop `BoundChat::Ideas` and any fixed `"ideas"` session key; Ideas entry opens
         the idea list in the navigator only (no content browser)

- [x] 2. Load the idea list from the same idea-store sources duckboard uses (or a thin
         shared helper)

- [x] 3. Selecting a change- or exploration-linked idea binds that shared `Scope`;
         selecting an inbox-only idea leaves chat unbound

- [x] 4. Codex entry binds `Scope::Codex` without a content browser

- [x] 5. Retarget or remove the obsolete Ideas unit test that expected a fixed ideas chat
         scope

- [x] 6. @spec tui/navigator Selection binds chat scope: Selecting Ideas opens navigator ideas list without a fixed ideas chat scope

- [x] 7. @spec tui/navigator Selection binds chat scope: Selecting a change-linked idea binds that change scope

- [x] 8. @spec tui/navigator Selection binds chat scope: Selecting an inbox-only idea does not bind a chat scope

- [x] 9. @spec tui/navigator Selection binds chat scope: Selecting Codex binds codex chat without a content browser
