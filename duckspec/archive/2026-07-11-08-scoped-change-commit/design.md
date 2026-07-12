# Scoped change commit - Design

Path-scope post-archive `` `commit` `` (and standing VCS priming) so agents commit only
this change’s dirty paths — never the whole working tree by default.

## Approach

No new duckspec capability and no duckboard “run commit” executor. Commit stays agent +
VCS CLI. Two instruction surfaces already in the loop:

```
archive success
      │
      ▼
archive.md Handoff
  • brief outcome
  • commit message (project convention when known)
  • path set for THIS change (dirty ∩ change membership)
  • `next` → `commit`
      │
 user `commit`
      ▼
agent: show message + paths → path-scoped VCS
  • jj:  `jj commit <filesets> -m …`
  • git: `git add <paths> && git commit -m …`
  • unrelated dirty left out; empty set → no invent
```

Priming repeats the same rule for any mid-lifecycle commit, not only archive handoff.

## Archive template handoff

File: `crates/duckspec/content/templates/archive.md` (Handoff only; dry-run / gate /
archive body unchanged).

After check + sync + audit:

1. Brief outcome (as today).
2. Propose commit message (as today).
3. **Build the change path set** from dirty tree ∩ change membership:

   - **duckspec:** landed archive dir, top-level `caps/` paths this archive applied,
     removals under `changes/<name>/` if still dirty

   - **code / other:** paths this change actually produced (session work, design / steps /
     caps touch list) that are still dirty

   - **exclude** dirty paths belonging to other active work or unknown WIP

1. Show the path set in ordinary markdown (table or list) with the message — before any
   VCS write.

2. `next` meta card: `` `commit`  commit change `` (same token) only when the include set
   is nonempty.

3. On user `` `commit` ``: run path-scoped commit with that message and that path set;
   report included paths; note what dirty remains.

4. If the set is empty: say so; do not invent a commit; omit `` `commit` `` from the
   `next` meta card (or offer only non-commit actions).

5. If membership is ambiguous for some dirty paths: ask before including them — do not
   default to whole-tree.

Still never auto-commit. `` `commit` `` is the single confirmation (message was already
proposed; path set shown at handoff and/or on execute for transparency — no second gate
unless ambiguity).

## VCS standing instructions

File: `crates/duckboard/src/config.rs` — `VcsWorkflow::standing_instructions`.

Add one standing bullet to **Git**, **Jj**, and **Worktrees** (wording adapted to tool):

- When committing work for a duckspec change (including post-archive `` `commit` ``),
  commit **only** paths for that change (path-scoped `git add` / `jj commit` filesets). Do
  **not** commit the entire dirty tree by default.

- Unrelated dirty paths stay uncommitted.

- If nothing dirty belongs to the change, report that and do not invent a commit.

Keep existing: tool choice, never auto-commit without explicit confirmation, no
destructive ops without confirm. Worktrees arm keeps worktree guidance.

## Verification (no new caps)

Unit tests only — instruction text is the contract surface:

```
| Area | Assert |
| --- | --- |
| `standing_instructions` | Each workflow mentions path-scoped / change-only commit (not whole dirty tree) |
| archive template | Handoff text requires path set + scoped commit + empty-set / no invent |
```

Extend or sibling the existing `standing_instructions_mention_tool_for_each_workflow` test
in `config.rs`. For the archive template: assert handoff keywords from the embedded
template content in the duckspec crate if a natural test home exists; otherwise standing
tests + review of `archive.md`.

## Impact

- Agents after `ds init` / content path via `ds template archive` pick up new archive
  handoff; duckboard rebuild picks up priming.

- No schema, no `caps/`, no slash-command catalog change.

- No optional `ds paths` helper in this change.

## Decisions

- **No new capability / no `/ds-commit` stage** — archive handoff + priming only (proposal
  non-goals).

- **`` `commit` `` remains one confirmation** — path set is information; re-ask only on
  ambiguity or empty set.

- **Membership = dirty ∩ change-owned paths** — not “all dirty under duckspec/”.

- **Ambiguous paths → ask**, never whole-tree default.

- **No native UI commit button**.

- **Template + priming only** — no `ds paths` CLI in this change.

## Risks

- **Weak membership judgment** → agent still over/under-includes. Mitigation: require
  explicit path list in chat; ask on doubt.

- **Stale mental model** if only template updates and priming is ignored. Mitigation: both
  surfaces say the same rule.

- **jj fileset mistakes** leave partial commits. Mitigation: agent reports remaining dirty
  after commit.
