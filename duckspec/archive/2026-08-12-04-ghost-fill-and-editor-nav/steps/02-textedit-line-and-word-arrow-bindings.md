# TextEdit line and word arrow bindings

Remap focused TextEdit Cmd/Option horizontal arrows to macOS line and word navigation.

## Tasks

- [x] 1. In `text_edit/render.rs`, map Cmd+←/→ (no Option) to `MoveHome` / `MoveEnd`

- [x] 2. Map Option+←/→ (no Cmd) to `MoveWordLeft` / `MoveWordRight`; leave Cmd+Option
         unbound

- [x] 3. @spec editor/line-word-nav Line and word arrow navigation: Cmd+arrow moves to line start or end

- [x] 4. @spec editor/line-word-nav Line and word arrow navigation: Option+arrow moves by word

- [x] 5. @spec editor/line-word-nav Line and word arrow navigation: Shift with Cmd or Option arrow extends selection
