# Uncommitted chrome and Commit send

Always-on uncommitted plaque on pending Change rows; activate selects the change and sends
`Commit` into chat without running VCS.

## Prerequisites

- [x] @step list-membership

## Tasks

- [x] 1. Show trailing `uncommitted` chrome on pending Change-section archive rows;
         compose with phase pills when that surface is on (archived + package-scoped
         uncommitted)

- [x] 2. On chrome activation, select the change if needed and submit `Commit` via the
         existing list phase-pill send path (no native VCS commit)

- [x] 3. @spec archive/pending-commit Uncommitted chrome and Commit send: Pending row shows uncommitted chrome

- [x] 4. @spec archive/pending-commit Uncommitted chrome and Commit send: Activating chrome sends Commit without committing
