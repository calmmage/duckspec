# Render inputs markdown

Implement pure path + render helpers for the change inputs ledger (filter and order only).

## Tasks

- [x] 1. Add `inputs_path` and pure `render_inputs_markdown` in duckboard (e.g.
         `chat_store` or `inputs_ledger.rs`) for `duckspec/changes/<name>/inputs.md`,
         user-only non-priming text, session-creation then message order

- [x] 2. @spec chat/inputs-ledger Package path: Ledger path is under the change directory

- [x] 3. @spec chat/inputs-ledger Verbatim user-only content: Non-priming user text is present verbatim

- [x] 4. @spec chat/inputs-ledger Verbatim user-only content: Priming user messages are omitted

- [x] 5. @spec chat/inputs-ledger Verbatim user-only content: Non-user roles are omitted

- [x] 6. @spec chat/inputs-ledger Verbatim user-only content: Sessions appear in creation order
