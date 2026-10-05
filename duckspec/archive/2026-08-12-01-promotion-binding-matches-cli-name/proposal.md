# Promotion binding matches CLI folder name

Fix exploration→change promotion so multi-word `ds create change` titles bind and promote
correctly instead of leaving an orphan empty change beside the original exploration chat.

When an exploration's agent ran `ds create change "Add sidebar"`, duckboard recorded the
binding under the slug `add-sidebar` while the CLI created `changes/Add sidebar`. On
reload the new directory had no matching binding, so promotion never ran: the change
stayed empty and the exploration kept the chat. Focus-based adoption is intentionally
forbidden; only the binding path should attach the chat, and that path was broken for any
non-slug folder name.

```
agent tool: ds create change "Add sidebar"
        │
        ├─ binding key (buggy)  →  add-sidebar
        └─ CLI folder           →  changes/Add sidebar
                                        │
                              lookup binding["Add sidebar"]  →  miss
                                        │
                              standalone change + live exploration
```

The fix keeps the binding key identical to the create-change argument — the same string
the CLI uses as the directory name under `changes/`. No slugification or other rewrite on
the binding path. Kebab-only names already worked; multi-word and spaced titles now match
the same rule.

Out of scope: an explicit UI to attach an exploration to a change after the fact, and any
change to how `ds create change` names directories on disk.
