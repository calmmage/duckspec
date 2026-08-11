# Chat inputs ledger

How duckboard keeps a precise-secretary markdown ledger of the user's raw chat under each
change package — for agent re-grounding, human audit, and archive archaeology.

## Where the ledger lives

```
duckspec/changes/<name>/
  proposal.md
  design.md
  inputs.md          ← derived user-input ledger
  caps/ …
  steps/ …
```

The ledger is an ordinary file inside the change directory. `ds archive` moves the whole
folder, so archaeology needs no separate path or archive step.

## Relationship to chat sessions

```
chat_store (local)                 change package (VCS)
chats/<change>/*.json              duckspec/changes/<name>/inputs.md
  full roles, tools, reasoning  ──export filter──►  user text only, markdown
```

Sessions remain the canonical chat record for the UI. The ledger is a **derived** view:
full rebuild from every session in the change scope, not an append-only event log and not
a second transcript of assistant or tool traffic.

## What is included

```
| Included | Excluded |
| --- | --- |
| `Role::User`, `is_priming == false` | Priming user messages |
| `ContentBlock::Text` as stored | Assistant, system, tool use/result, reasoning |
| Session creation order, then message order | Paraphrase, summaries, takeaways |
```

Bare confirms and slash-only user lines are included — audit purity over scannability.

## When export runs

```
durable save of change-scoped session
        │
        ▼
change dir exists? ──no──► skip (do not create the change)
        │ yes
        ▼
render full ledger from all sessions in scope
        │
        ├── no qualifying messages ──► leave inputs.md absent
        ├── same bytes as on disk ───► skip rewrite
        └── else ────────────────────► atomic write inputs.md
```

After exploration→change session migration (promotion), export runs once for the change so
pre-promotion user words appear without waiting for another user send.

Caps, codex, and exploration scopes do not write a package ledger on their own saves.

## Machine-owned file

Re-export overwrites `inputs.md`. Treat it as generated from chat; hand-edits do not
survive the next save. There is no separate schema type or `ds create inputs` in v1.

## Agent awareness

Change-scope orientation may name the ledger path when the file is present. That wording
is owned by session scope orientation; this capability only guarantees the file contract
above.
