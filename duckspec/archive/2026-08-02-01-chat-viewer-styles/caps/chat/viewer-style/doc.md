# Chat viewer style

How duckboard chooses Answer body presentation via a global chat setting.

## Styles

```
| Stored   | When effective                                              |
| -------- | ----------------------------------------------------------- |
| classic  | Always available; default when unset                        |
| focus    | When Focus Answer presentation is implemented               |
| document | Reserved; effective classic until Document is implemented   |
```

Stored values may still include document (or future names) after hand-editing config.
Presentation always goes through **effective** style so unimplemented modes never paint a
half-wired Answer path.

## Settings

The Chat settings section exposes a viewer-style control listing only **implemented**
styles. Classic is always listed; Focus appears once Focus ships. Document stays out of
the list until it has a real renderer. The preference is global (all projects and
sessions), not per chat.

## Answer scope and Classic identity

Only Answer (assistant) bodies branch on effective style. Thinking, Activity, System, and
user cards keep their existing presentation paths.

When effective style is classic, Answers use the full-body classic path (single body
presentation — the same identity as before Focus). That path is the acceptance bar under
default settings; Focus is opt-in only.
