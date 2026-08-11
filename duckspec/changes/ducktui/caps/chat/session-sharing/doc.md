# Chat session sharing

Two UI processes can open the same project and share one set of chat session files.
Persistence still owns how a single process writes durably; this capability owns how a
process decides whether to write, reload, or ignore an external change.

## Drive vs display

At any moment a process treats each loaded session as either **driven** or **displayed**.

```
| Role      | Meaning                                              | Writes? |
| --------- | ---------------------------------------------------- | ------- |
| Driven    | This process owns the active or just-completed turn  | Yes     |
| Displayed | Loaded for viewing only; another process may write   | No      |
```

Only the driven role may persist. A process that merely shows a conversation never
overwrites it — including cosmetic UI changes that do not represent a local turn.

## Reload path

Each process watches the project tree for session-file events. When a change lands on a
session the process is **not** driving:

```
file event ──► classify as session path ──► idle? ──yes──► reload from disk
                                            │
                                            no (local in-flight)
                                            ▼
                                          ignore until turn settles
```

Reload replaces the in-memory session with the file contents so a turn completed in the
other app appears here without a manual refresh. Removal of the file drops the session
from the local list for that scope.

## In-flight shield

While a local turn is open, applying an external snapshot would clobber streamed messages
and the live agent subscription. Events for that session are held off until the turn
settles; after settlement the idle reload rule applies again.

## Conflict policy

Two processes driving the same session at once is unsupported and unprevented — the same
stance as two windows of one UI today. There are no lock files and no cross-process turn
claims. Each write is still atomic; when both finish, the later successful write is what
remains on disk. Callers that need exclusive control of a conversation should not send
turns from both apps into the same session concurrently.
