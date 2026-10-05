# Ghost fill and editor line nav - Design

App-level next-action fill + remapped cycle keys; TextEdit Cmd/Option arrow bindings
aligned with macOS line/word nav.

## Components

```
| Piece | Responsibility |
| --- | --- |
| `FillNextAction` msg + handler | Insert active next-action `send` into `chat_input` without starting a turn |
| `handle_agent_chat_key` | Map Tab / → / Shift+Tab to fill vs cycle; keep slash completion first |
| `default_prompts` helpers | Pure eligibility: fill when list ≥ 1; cycle when list ≥ 2 |
| Ghost placeholder chrome | Multi-only Shift+Tab cue; body always active send |
| TextEdit keyboard match | Cmd = line ends; Option = word jumps |
```

## Ghost fill

```
empty + ghost armed + not streaming
        │
        ▼
Msg::FillNextAction
        │
        ▼
text = next_actions[next_action_idx].send
chat_input = EditorState::new(text)   // caret at end
rehighlight_input
// no dispatch_user_submit, no agent turn
```

- Pattern matches existing `completion_accept` (rebuild `EditorState`, caret at end).
- Eligibility: input empty (trim), not streaming, `next_actions.len() >= 1`.
- Display-only placeholder prefixes never enter fill/send text.
- After dispatch, `main` refocuses chat input (same batch pattern as `CycleNextAction`).

## Key routing

Order in `handle_agent_chat_key` after session is active:

```
1. slash completion visible
      Tab → CompletionAccept  (unchanged)

2. Shift+Tab + empty + can_cycle (≥2, not streaming)
      → CycleNextAction(+1) only   // forward wrap; no reverse

3. bare Tab or bare → + empty + can_fill (≥1, not streaming)
      + no command/alt/control
      → FillNextAction

4. existing fast-response Cmd+digit, Esc-Esc, …
```

```
| Key | Role |
| --- | --- |
| Tab (completion open) | Accept slash completion |
| Tab (empty + ghost) | Fill active option |
| → (empty + ghost, bare) | Fill active option |
| Shift+Tab (empty + multi) | Advance active index only |
| Enter | Send (unchanged; empty → active send) |
```

Active index is shared: after Shift+Tab lands on option 2, Tab fills option 2’s text.

## Affordance chrome

- Ghost body = active `send` always.

- Single option: placeholder is bare body (no prefix).

- Multi (`len > 1`, empty, not streaming): prefix display-only cycle cue `⇧⇥  {body}`.

- Rename/repurpose the existing multi marker helper from “Tab cycles” to “Shift+Tab
  cycles”; predicate stays empty + idle + multi.

## TextEdit arrow modifiers

In focused TextEdit keyboard handling (`text_edit/render.rs`):

```
| Binding | Action |
| --- | --- |
| Cmd+← / → (no alt) | `MoveHome` / `MoveEnd` |
| Option+← / → (no cmd) | `MoveWordLeft` / `MoveWordRight` |
| + Shift | selection extend (existing `bool`) |
| Home / End | unchanged |
| Cmd+Option+arrow | unbound |
```

No new `EditorAction` variants. Applies to every TextEdit (file tabs, chat input, queue,
nav in read-only).

## Cmd+arrow ownership (landmarks vs composer)

App-level chat landmarks and in-editor Cmd+line ends share Cmd+← / →. Ownership:

```
| Focus | Cmd+← / → | Cmd+↑ / ↓ |
| --- | --- | --- |
| Chat input focused (`chat_input_focused`) | line start/end in composer only (no answer jump) | history top/bottom landmarks (unchanged) |
| Chat open, input not focused | previous / next Answer landmarks | history top/bottom landmarks |
```

- `keybind_chat_landmarks` keeps the base gate (chat tab + session, not terminal, modals
  already excluded by caller).

- Horizontal landmarks additionally require **not** `chat_input_focused`, so TextEdit
  receives Cmd+← / → for line ends when typing in the AI chat input.

- Vertical landmarks stay available even while the composer is focused.

- Bare arrows remain composer caret moves (and empty-composer → fill for bare →).

## Out of scope

- Oneshot chips / fast-response activation
- Agent harness protocol

## Settled choices

- Dedicated `FillNextAction` over in-handler-only fill or overloading completion accept

- App router owns bare → fill (not TextEdit callbacks)

- Forward-only cycle on Shift+Tab

- Multi chrome shows Shift+Tab cue only; no fill glyph on single-ghost

- Cmd+Option left unbound

- Cmd+← / →: composer line nav when chat input focused; answer landmarks otherwise (while
  chat landmarks base gate holds)
