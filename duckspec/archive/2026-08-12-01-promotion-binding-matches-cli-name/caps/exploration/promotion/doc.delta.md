# @ Exploration promotion

## ~ Bindings

A binding is the authoritative record that a specific exploration created a specific
change. It is established at the causal moment: when an exploration session's agent runs
`ds create change <name>`, duckboard records `<name> → <exploration>` using the argument
**verbatim** (the same string the CLI uses as the folder under `changes/`, not a slug).
Promotion consults only bindings.

```
| Situation | Binding? | Outcome |
| --- | --- | --- |
| Exploration's agent ran `ds create change foo` | yes | `foo` adopts that exploration; chat input focused |
| Exploration's agent ran `ds create change "Add sidebar"` | yes | `Add sidebar` adopts that exploration; chat input focused |
| Change `foo` created out-of-band | no | `foo` left standalone; no forced chat focus |
| Working-copy restored `foo`'s directory | no | `foo` left standalone; no forced chat focus |
| `foo` unarchived | no | `foo` left standalone; no forced chat focus |
```

When a change is left standalone, nothing is lost: any exploration the user was viewing
keeps its own chat scope, and the change simply has no adopted chat.
