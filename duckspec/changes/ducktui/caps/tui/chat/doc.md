# Tui chat

How the terminal chat pane shows a transcript and accepts input. Segment construction,
slash catalog rules, fast-response policy, and meta-card meaning stay in the existing
`chat/*` capabilities; this capability is the terminal renderer and key bindings.

## Ownership boundary

```
| Concern | Owner |
| --- | --- |
| Thinking / Activity / Answer construction | `chat/transcript` |
| Slash catalog kinds and `//` escape | `chat/slash-commands` |
| Option chips and freeform custom answer | `chat/fast-response` |
| Meta-card parse and `next` tokens | `chat/meta-cards` |
| Session open/switch scroll intent | `chat/session-scroll` |
| Terminal collapse, scroll pin, numbered hints, composer keys | **this capability** |
```

## Segment display

Settled defaults match duckboard's collapse policy in a terminal:

```
| Segment   | Default render              |
| --------- | --------------------------- |
| Thinking  | One summary line; expandable |
| Activity  | One summary line; expandable |
| Answer    | Always fully expanded        |
```

Markdown and meta-card chrome render inside Answer (and expanded bodies). Tables fit to
pane width; narrow panes degrade to plain cell wrap.

## Viewport

```
stick engaged + streaming ──► pin to latest
manual scroll up          ──► release pin
return to bottom / open-switch ──► re-engage stick
```

Intentional session open or switch lands at latest and re-engages stick, matching
`chat/session-scroll`. Area-style mid-history restore is not a separate ducktui concern
beyond that shared intent.

## Numbered hints

Two sources share the same 1–9 activation surface when active:

```
| Source | When shown | Number key |
| --- | --- | --- |
| Trailing `next` meta-card tokens | After a turn that left a `next` card | Activates that token |
| Fast-response options | While chips are visible (awaiting or idle empty composer per `chat/fast-response`) | Activates that option |
```

Activation uses the existing semantic owners; the terminal only maps keys to those
actions.

## Composer keys

```
| Key | Effect |
| --- | --- |
| Enter | Submit current input |
| Alt+Enter | Insert newline (portable) |
| Shift+Enter | Newline only if the terminal keyboard protocol reports it |
| `/` at line start | Open slash palette from shared catalog |
| `//` at line start | Pass-through; not a single-slash catalog open |
```

The slash palette itself is an overlay owned by `tui/shell` input capture while open; this
capability defines when it opens from the composer.
