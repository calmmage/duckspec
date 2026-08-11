# Archive pending commit

After duckspec archive, packages with residual dirt under their archive folder stay on the
Change list with uncommitted chrome until that dirt is gone.

## Pending

```
pending(id) ≔ any dirty path under duckspec/archive/<id>/
              (folder path, descendants, including deletes)
```

`id` is the full archive folder name. Pending is recomputed from disk packages plus the
current dirty list — no sidecar flag.

## List placement

```
Change section
  explorations + active changes
  + pending archives (after actives, newest-first among them)

Archived section (finished packages only)
  non-pending archives + listable archived explorations
```

Disk still loads packages only from `duckspec/archive/`. Placement is a view split.
Selecting a pending row uses the same archived identity, tabs, and chat scope as selecting
that package under Archived.

## Chrome

```
pending row → trailing "uncommitted"
activate    → select change if needed + send "Commit" into chat
```

Does not run `jj`/`git` commit. Path-scoped commit remains chat/agent with standing VCS
rules. When phase-pill list is on, the row may also show `archived` + package-scoped
`uncommitted` with the same send.
