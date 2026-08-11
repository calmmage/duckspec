# Tui shell

Ducktui's application chrome: three full screens, a two-pane work surface, layered input
dispatch, and a status bar. Navigator contents and chat rendering live in sibling
capabilities; the shell only owns screens, focus, overlays, and status.

## Screen map

```
              no project
     ┌──────────────────────┐
     │   Project picker     │
     └──────────┬───────────┘
                │ bind project
                ▼
     ┌──────────────────────┐     open settings
     │     Work screen      │ ──────────────────► Settings
     └──────────────────────┘ ◄──────────────────
                ▲                 leave settings
                │ unbind / no project
                └── (settings leaves to picker)
```

```
| Screen | When shown | Role |
| --- | --- | --- |
| Project picker | No project bound | Recent projects and path entry |
| Work screen | Project bound | Navigator + chat; the app lives here |
| Settings | User opens settings | Harness/model prefs, oneshot models, theme |
```

## Work layout

The work screen is two panes only — no middle content column.

```
┌─ navigator ─┬─ chat ──────────────────────────┐
│             │                                 │
│             │                                 │
├─────────────┴─────────────────────────────────┤
│ status bar                                    │
└───────────────────────────────────────────────┘
```

Exactly one pane is focused. Tab toggles focus between navigator and chat. Reading and
editing artifacts stays outside ducktui (the user's editor); the shell never opens a
content browser pane.

## Input layers

Keys resolve in a fixed order:

```
global keys ──► open overlay (if any) ──► focused pane
```

```
| Layer | Handles | Examples |
| --- | --- | --- |
| Global | Always first | Quit, help, screen switches |
| Overlay | All remaining input while open | Model picker, slash palette, session switcher, quick idea, help |
| Pane | Remainder when no overlay | Navigator selection, composer typing |
```

An overlay that owns a binding while open (for example Escape to dismiss) documents that
ownership; otherwise globals still win.

## Status bar

The status bar spans every screen. Fields:

```
| Field | Bound project | Unbound |
| --- | --- | --- |
| Scope identity | Active scope label | Absent |
| Model | Selected model id/name | Selected model id/name |
| Context fill | Last-known usage vs window | Zero / empty meter |
| Turn state | Idle, streaming, awaiting choice, … | Idle |
```

## Sibling ownership

```
| Concern | Owner |
| --- | --- |
| Navigator tree and scope binding | `tui/navigator` |
| Transcript, composer, chips | `tui/chat` |
| Shared session write/reload rules | `chat/session-sharing` |
| Chat segment model and slash semantics | existing `chat/*` capabilities |
```
