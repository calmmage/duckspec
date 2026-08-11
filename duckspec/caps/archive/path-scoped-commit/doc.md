# Archive path-scoped commit

After a successful archive, the handoff proposes an optional commit of only this change's
dirty paths — never the whole working tree by default.

## Flow

```
archive success (check + sync + audit reported)
      │
      ▼
handoff
  • brief outcome
  • commit message (project convention when known)
  • include set = dirty ∩ this change
  • `next` → `commit` only when include set nonempty
      │
 user `commit`
      ▼
path-scoped VCS write for the include set only
  • report committed paths + remaining dirty
```

No auto-commit. Empty include set → say nothing owned is dirty; do not invent a commit;
omit `` `commit` ``.

## Include set

Membership is **dirty ∩ change-owned**, not "everything dirty under `duckspec/`".

```
| Kind        | Include when still dirty                                      |
| ----------- | ------------------------------------------------------------- |
| duckspec    | landed archive dir; top-level `caps/` this archive applied;   |
|             | remnants under former `changes/<name>/`                       |
| code/other  | paths this change actually produced                           |
| exclude     | other active changes, unknown WIP                             |
| ambiguous   | ask before include — never default to the whole dirty tree    |
```

The include set is information shown with the message before any VCS write. One
`` `commit` `` confirmation is enough unless membership is ambiguous.

## Path-scoped execute

On `` `commit` ``, only the include set is committed under the project's VCS rules
(path-limited `git` / `jj` — not whole-tree defaults). Unrelated dirty work stays out.
Standing VCS priming (see `shell/vcs-workflow`) repeats the same change-only rule for
mid-lifecycle commits.
