# Files explorer uses active work root

Walk and refresh the Files explorer from the focused scope’s work root so the tree matches
open/diff/agent cwd.

## Prerequisites

- [x] @step active-root-plumbing

## Context

From review finding 2: `refresh_project_files` always walks `project_root` while Changed
files and file open already use `active_work_root`.

## Tasks

- [x] 1. Point `refresh_project_files` (and reveal-if-needed path assumptions) at
         `active_work_root` instead of always `project_root`

- [x] 2. On scope switch, refresh the explorer when the Files section is expanded

- [x] 3. @spec worktree/active-root Focused root authority: File open resolves under the focused root
