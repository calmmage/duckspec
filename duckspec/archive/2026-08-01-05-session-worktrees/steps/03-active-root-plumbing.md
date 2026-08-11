# Active root plumbing

Focused scope’s work root drives Changed files, file open, and agent cwd; cold rebind on
focus switch; no mid-stream root swap; caps/codex always on main.

## Prerequisites

- [x] @step worktree-manager-and-placement-policy

## Tasks

- [x] 1. Derive `active_work_root` from the focused exploration/change binding; refresh
         Changed files, diffs, and explorer from that root only

- [x] 2. Per-scope `agent_subscription` working directory (fold path into subscription
         identity); force project main for caps and codex

- [x] 3. Resolve project-relative file open under the active root; watch main plus every
         live sidecar

- [x] 4. @spec worktree/active-root Focused root authority: Changed files reflect the focused scope’s root only

- [x] 5. @spec worktree/active-root Focused root authority: Agent working directory matches the focused scope’s root

- [x] 6. @spec worktree/active-root Focused root authority: File open resolves under the focused root

- [x] 7. @spec worktree/active-root Focus switch: Switching scope rebinds Changed files to the new root

- [x] 8. @spec worktree/active-root Focus switch: Cold agent rebinds to the new root on next use

- [x] 9. @spec worktree/active-root No mid-stream rebind: Streaming session keeps its root until the turn ends

- [x] 10. @spec worktree/active-root Caps and codex on Main: Caps and codex scopes always use the project main root
