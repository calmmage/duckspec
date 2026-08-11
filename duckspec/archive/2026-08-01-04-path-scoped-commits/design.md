# Path-scoped commits - Design

Path-scope post-archive `` `commit` `` and standing VCS priming so agents commit only this
change’s dirty paths — never the whole working tree by default — with a global git / jj /
worktrees workflow for tool choice.

## Approach

```
archive success
      │
      ▼
archive.md Handoff
  • brief outcome
  • commit message (project convention when known)
  • path set for THIS change (dirty ∩ change membership)
  • `next` → `commit` only when include set nonempty
      │
 user `commit`
      ▼
agent: path-scoped VCS (git pathspec / jj fileset)
  • unrelated dirty left out; empty set → no invent
```

Priming injects `VcsWorkflow::standing_instructions` (Settings → `[vcs].workflow`) so
mid-lifecycle commits inherit the same rule.

No duckboard commit executor. No `/ds-commit` stage. No `ds paths` CLI in this cut.

## Archive template handoff

File: `crates/duckspec/content/templates/archive.md` (Handoff only).

Membership (dirty ∩ change-owned):

- **duckspec:** landed archive dir for this change, top-level `caps/` paths this archive
  applied, removals under `changes/<name>/` if still dirty

- **code / other:** paths this change produced that are still dirty — listed explicitly

- **exclude** other active work / unknown WIP

- **ambiguous** → ask before include; never whole-tree default

## VCS workflow + standing instructions

```rust
// crates/duckboard/src/config.rs
pub enum VcsWorkflow { Git, Jj, Worktrees } // global Config.vcs.workflow
// standing_instructions(): tool rules + path-scoped change commit bullets
```

First-turn priming body includes VCS standing text via `assemble_priming_body`. Settings
pick_list persists workflow.

## Decisions

- **Template + priming** — no `ds paths` CLI this cut
- **Archive dir** — use the landed / orientation-named archive id, not fuzzy multi-match
- **Mid-lifecycle** — standing instructions in priming (not a third codex doc)
- **One `` `commit` `` confirmation** — path set is information; re-ask only on ambiguity

## Impact

- `archive.md` handoff, `VcsWorkflow` standing text, Settings picker, unit tests on
  strings

- Caps below document the behavioral contract (prior archive claimed “no caps”; this
  change records the contract in duckspec)

## Open questions

*(none — resolved: no CLI; full archive id; priming + archive only)*
