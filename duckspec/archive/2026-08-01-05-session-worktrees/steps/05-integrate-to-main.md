# Integrate to Main

Shared integrate path for archive auto-merge and explicit merge: forget sidecar when
clean, stop on conflict, and update dependents after a base merges.

## Prerequisites

- [x] @step stack-base-and-require-merged-gate

## Tasks

- [x] 1. Implement `integrate_to_main` for jj and git backends; on clean success forget
         the sidecar and clear binding; on conflict surface paths without claiming success

- [x] 2. Hook archive success and explicit “merge to main” to the same integrate path;
         after a base merges cleanly, auto-update dependents onto Main when clean

- [x] 3. Update standing VCS instructions: document `duck-<scope_key>`; drop Worktrees
         “plumbing is not automatic yet”; add Jj workspace awareness

- [x] 4. @spec worktree/stack-and-merge Integrate to Main: Archive success auto-attempts integrate for a sidecar scope

- [x] 5. @spec worktree/stack-and-merge Integrate to Main: Clean integrate forgets the sidecar

- [x] 6. @spec worktree/stack-and-merge Integrate to Main: Conflict stops without claiming success

- [x] 7. @spec worktree/stack-and-merge Integrate to Main: Explicit merge to main uses the same integrate path

- [x] 8. @spec worktree/stack-and-merge Stack after parent merge: After base merges cleanly, dependent may rebase onto Main when clean
