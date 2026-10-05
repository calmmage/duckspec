# @ Chat answer landmarks

## + Keyboard ownership

```
| Chord | Chat input focused | Chat open, input not focused |
| --- | --- | --- |
| Cmd+← / → | line start/end in composer | previous / next Answer |
| Cmd+↑ / ↓ | history top / bottom | history top / bottom |
| bare ← / → | caret (or empty-composer fill on bare → when ghost armed) | — |
```

Base landmark gate: chat tab + active session, terminal not focused, no navigation-owning
modal. Horizontal answer jumps add “composer not focused” so typing in the AI chat input
never double-handles Cmd+← / → as both line nav and answer jump.
