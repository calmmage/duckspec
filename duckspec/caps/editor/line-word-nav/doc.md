# Editor line and word nav

Shared TextEdit horizontal navigation: Cmd moves by line ends, Option moves by word.

## Bindings

```
| Binding              | Action                          |
| -------------------- | ------------------------------- |
| Cmd+← / → (no Option)| Line start / line end           |
| Option+← / → (no Cmd)| Word left / word right          |
| + Shift              | Extend selection while moving   |
| Home / End           | Line start / line end           |
| Cmd+Option+← / →     | Unbound (no line or word jump)  |
```

These bindings live in the shared TextEdit keyboard handler, so every surface that uses
that widget gets the same behavior: file tabs, chat composer, queue editor, and read-only
editors that still allow caret navigation.

App-level shortcuts that own Cmd+arrows when the editor is not the focus target (for
example chat answer landmarks) are separate and do not redefine these in-editor bindings.
