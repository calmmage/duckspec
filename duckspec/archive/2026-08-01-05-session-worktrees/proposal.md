# Session worktrees

Give each parallel idea its own working tree so Changed files, agent edits, and diffs no
longer collide when several chat sessions run against one project.

## Motivation

Duckboard already supports many explorations and chat sessions, but they all share one
project working tree. Changed files is a single global list; agents all use the same cwd.
Starting a second idea while the first has uncommitted edits either interleaves unrelated
diffs or forces the human to pause and clean up.

Why now: multi-session / multi-exploration use is already the normal path; without
filesystem isolation, more chat UX only multiplies the collision.

## Intent

- When starting or focusing parallel work while another session already has a dirty tree,
  duckboard can put that work on its own worktree instead of sharing the dirty main tree

- Switching chat/session focus switches which worktree is active for Changed files, diffs,
  file opens, and agent cwd

- Finishing an idea can merge that worktree back into the main line with as little
  ceremony as is safe — automatic when clean, never silent on conflict

- A session’s dirty state is attributable to that session in the UI (not a mystery global
  bag of files)

## Non-goals

- Multi-project or multi-repo worktree orchestration beyond the open project

- Replacing the user’s normal jj/git workflow outside duckboard

- Concurrent multi-agent “best-of-N” sandboxes as a separate product surface

- Auto-resolving merge conflicts without human review

- Redesigning the CHANGE list, exploration model, or chat tab chrome beyond what isolation
  requires
