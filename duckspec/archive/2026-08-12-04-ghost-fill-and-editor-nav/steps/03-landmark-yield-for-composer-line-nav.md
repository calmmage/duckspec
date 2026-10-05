# Landmark yield for composer line nav

When the chat input is focused, Cmd+← / → only move the caret in the composer; answer
landmarks keep Cmd+↑ / ↓ and resume Cmd+← / → when the input is not focused.

## Tasks

- [x] 1. Split horizontal landmark eligibility: base gate AND NOT `chat_input_focused`

- [x] 2. In `main` Cmd+arrow dispatch, claim left/right only when horizontal-eligible

- [x] 3. @spec chat/answer-landmarks Landmark shortcut eligibility: Horizontal landmarks yield while chat input is focused

- [x] 4. @spec chat/answer-landmarks Landmark shortcut eligibility: Horizontal landmarks available when chat input is not focused
