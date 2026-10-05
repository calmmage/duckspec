# Ghost fill and cycle keys

Wire empty-composer fill (Tab/→), Shift-Tab-only cycle, and multi cycle-marker chrome for
next actions.

## Tasks

- [x] 1. Add `can_fill_next_action` (and rename multi marker helper to cycle marker) in
         `default_prompts`

- [x] 2. Add `FillNextAction` message + handler: active send → `chat_input`, caret at end,
         rehighlight, no turn

- [x] 3. Remap `handle_agent_chat_key`: completion Tab first; Shift-Tab →
         `CycleNextAction(+1)`; bare Tab/→ → fill

- [x] 4. Refocus chat input after fill (same path as cycle)

- [x] 5. Update multi ghost placeholder prefix to Shift-Tab cycle cue; single option stays
         bare body

- [x] 6. @spec chat/default-prompts Next-action empty-input send, fill, and cycle: Empty submit sends the active next action

- [x] 7. @spec chat/default-prompts Next-action empty-input send, fill, and cycle: Empty submit is a no-op when the next-action list is empty

- [x] 8. @spec chat/default-prompts Next-action empty-input send, fill, and cycle: Shift-Tab cycles next actions with wrap

- [x] 9. @spec chat/default-prompts Next-action empty-input send, fill, and cycle: Tab or right-arrow fills the active next action

- [x] 10. @spec chat/default-prompts Next-action empty-input send, fill, and cycle: Multi next shows a cycle marker
