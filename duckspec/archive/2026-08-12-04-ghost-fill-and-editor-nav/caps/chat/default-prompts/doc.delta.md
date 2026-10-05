# @ Chat default prompts

Conversation-local empty-input defaults from a cheap-model oneshot: parse ordered `REPLY:`
suggestions (lifecycle heuristic passed only as a soft request hint), show and arm them
only after a non-empty parse settles, and drive empty Enter send plus fill and cycle from
the next-action list alone. The effective list is never seeded or filled from the
lifecycle heuristic.

Empty-composer next actions from an inherited donor list (empty change sessions after
new-session creation), lifecycle bootstrap, a trailing `next` meta card, or a trailing
A–B–C prose option run when no meta card is present, shown as ghost text with empty Enter
send, Tab/right-arrow fill, and Shift-Tab cycle; optional settings-gated oneshot reply
suggestions (up to three freeform `REPLY:` lines) that may fill fast-response chips only
when there is no next-action ghost.

## ~ Surfaces

Two empty-composer surfaces stay separate:

```
| Surface      | Source                                                                 | Empty-input chrome                         | Activation                                      |
|--------------|------------------------------------------------------------------------|--------------------------------------------|-------------------------------------------------|
| Next actions | Empty: inherited list, else lifecycle[0]; non-empty: trailing `next`, else A–B–C prose | Ghost + optional cycle marker before ghost | Enter (send) / Tab·→ (fill) / Shift-Tab (cycle) |
| Oneshot hint | Cheap-model `REPLY:` when agent input hints on                         | Fast-response chips (0–3) when eligible    | ⌘n / click (chip send)                          |
```

Empty-session lifecycle[0] is scope-derived: exploration → explore stage command; change →
first option of that change's artifact/step ladder; caps and codex → none. That bootstrap
does not use the agent input hints setting. A non-empty inherited next-action list on an
empty session outranks lifecycle bootstrap. Creating a new change chat session (session +
or ⌘N when it means new chat) may copy the prior active session's next-action list onto
the new empty session; inheritance lasts only while that session stays empty. Next actions
never list under the input and never fill chips. Oneshot never fills the next-action list
or ghost. After the first turn, missing both trailing `next` and a trailing A–B–C prose
option run means no next-action ghost — not a disk-phase fallback and not re-inheritance.

There is no under-input oneshot row, no oneshot loading strip, and no empty Cmd-Enter path
for oneshot suggestions. Empty Enter remains next-action only.

## ~ Next-action list

```
session empty + inherited non-empty
    │
    ▼
next-actions = inherited list   (ready immediately; not gated by agent input hints)

session empty + no inherited + lifecycle[0] (scope-derived)
    │
    ▼
next-actions = [lifecycle[0]]

session empty, no inherited, no lifecycle[0]
    │
    ▼
next-actions = []

session non-empty
    │
    ▼
trailing next actions from last assistant   (0–3 send tokens)
    │
    ├── has trailing next ──▶ next-actions = those tokens
    └── no trailing next ──▶ trailing A–B–C prose options (0 or 2–3)
            │
            └── none ──▶ next-actions = []
        (inherited list is not used; meta and prose never merge)
```

Empty Enter sends the active send token. When the composer is empty and a next action is
armed, bare Tab and bare right-arrow fill that token into the input without sending.
Shift-Tab advances the active index with wrap when there are two or more actions; cycling
never inserts text. A small cycle marker (Shift-Tab cue) appears **before** the ghost only
for multi-option empty idle. Ghost text is the active send token via the empty-input
placeholder (hidden while streaming). A newly created empty change session that inherited
a list starts at the first action.
