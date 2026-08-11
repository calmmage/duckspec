# archive

## Before write

## Role

You finalize a change: validate, apply it into top-level `caps/`, and move the
change into the archive. Dry-run and show exactly what will land, wait for
confirmation, apply it, then verify the result.

## Context

1. Act on the change from session scope orientation; use `ds status` only to
   disambiguate when orientation is missing or the user names another change.
2. Load `duckspec/project.md` if present.
3. Load `ds schema style` if it is not already in context.
4. Skim the change (proposal, caps, steps) enough to explain what archive will
   apply.

## Instructions

1. **Dry run** - `ds archive <name> --dry`. Report the preview (table when many
   paths).
2. **Fix or stop** - if the dry run fails validation, work with the user until
   clean; do not archive.
3. **Gate** - `write` meta card + preview of the apply plan + `next` meta card
   (`confirm archive` / `reject archive`).
4. On `confirm archive` - `ds archive <name>`.
5. **Check** - `ds check` on affected paths under `caps/`.
6. **Sync** - `ds sync` so archived scenarios get `path:line` stamps on
   `test: code` markers (no-op when there are no code-linked scenarios).
7. **Audit** - `ds audit` (whole project) for post-merge integrity.

## Chat

Follow `style`. Dry-run and results are information (tables). Gate and handoff
use meta cards as in Write gate and Handoff.

## Write gate

**Confirm-then-archive.** After `confirm archive` only, run `ds archive <name>`.

```markdown
> **write**
>
> Archive change `<name>` into top-level caps and `duckspec/archive/`

| Capability | Apply |
| --- | --- |
| `<path>` | new (spec + doc) |
| `<path>` | delta (spec) |

Archive to: `duckspec/archive/YYYY-MM-DD-NN-<name>/`
From: `duckspec/changes/<name>/`

Irreversible outside version control.

> **next**
>
> `confirm archive`
> `reject archive`
```

Preview from the dry-run; real paths and apply kinds.

## Handoff

After a successful archive (check + sync + audit reported):

1. State the outcome briefly (archived; sync/audit clean, or note exceptions).
2. Propose a commit message in ordinary markdown (before any meta card). Use
   project conventions when they are known; otherwise a clear short message is
   enough - do not invent a convention regime.
3. **Build a path-scoped include set** — dirty working-tree paths that belong
   to **this** change only:
   - duckspec: archive dir just landed, top-level `caps/` paths this archive
     applied, and any still-dirty paths under the former `changes/<name>/`
   - code / other: paths this change actually produced that are still dirty
   - **exclude** dirty paths from other changes or unknown WIP
   - when membership is ambiguous, ask before including — never default to
     the whole dirty tree
4. Show the include set with the message (table or list) so the user sees
   what will be committed before any VCS write. Note excluded dirty paths
   briefly when useful.
5. Emit a `next` meta card with `` `commit` `` only when the include set is
   nonempty:

```markdown
> **next**
>
> `commit`
```

6. On user `` `commit` ``: run a **path-scoped** commit for that include set
   only (`git add <paths>` then `git commit`, or `jj commit` with those
   filesets — follow project VCS rules). Report which paths were committed
   and what dirty remains. Do **not** use whole-tree commit defaults
   (`git commit -a`, bare `jj commit` with no path limit) that would scoop
   unowned dirt.
7. If the include set is empty: say nothing owned is dirty; **do not invent a
   commit**; omit `` `commit` `` from the `next` meta card (offer only other
   actions if any).

Never auto-commit; wait for the user to choose `commit` (or another action).
Omit the entire `next` meta card only if there is truly nothing left to offer.

## After write
