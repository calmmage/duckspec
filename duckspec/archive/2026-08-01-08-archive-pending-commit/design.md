# Archive pending commit - Design

After `ds archive`, keep the package in the Change list until `duckspec/archive/<id>/` has
no dirty paths; then show it only under Archived.

## Rule

```
pending(id) ≔ any changed_files path under duckspec/archive/<id>/
              (including the folder path itself and deletes under it)
```

`id` is the full archive folder name. No sidecar flag; recompute from disk + dirty list on
each refresh.

## List

View-only split (`ProjectData` loaders unchanged):

- **Change** = explorations + active changes + **pending** archives (pendings after
  actives, newest-first among them)

- **Archived** = non-pending archives only (+ archived explorations as today)

Shared helpers so other Archived UIs do not double-list pending packages.

## Identity

Pending rows are normal archived `ChangeData` (full id, `prefix = archive`). Selection,
tabs, overview, and chat match selecting that archive today. Breadcrumbs while pending in
Change: `Changes / <id>`. No lifecycle next-stage.

## Chrome and click

- Always-on trailing plaque: **`uncommitted`**

- Click plaque → select change if needed + send `Commit` into that change’s chat

- When phase pills are on: also `archived` + scoped `uncommitted` VCS pill (same click →
  `Commit`); hover means this archive package, not repo-wide

No native VCS commit, no auto-commit, no code-membership engine for `crates/`. Path-scoped
commit remains chat/agent + standing VCS rules.
