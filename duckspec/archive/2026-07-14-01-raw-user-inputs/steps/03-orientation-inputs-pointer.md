# Orientation inputs pointer

Point change-scope orientation at `inputs.md` when the ledger file is present and
non-empty.

## Prerequisites

- [x] @step export-writes-and-triggers

## Tasks

- [x] 1. In change-scope orientation (`scope.rs` / `CurrentScopeHook`), when
         `duckspec/changes/{name}/inputs.md` exists and is non-empty, name that path and
         direct re-grounding (prefer raw inputs; do not invent motivation; do not rewrite
         the file)

- [x] 2. @spec session/scope Inputs ledger pointer: Present inputs.md is named in orientation

- [x] 3. @spec session/scope Inputs ledger pointer: Absent inputs.md yields no inputs pointer
