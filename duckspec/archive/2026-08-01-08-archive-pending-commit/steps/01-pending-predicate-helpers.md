# Pending predicate helpers

Pure helpers that decide whether an archived package is pending from dirty paths under
`duckspec/archive/<id>/`.

## Tasks

- [x] 1. Add pure helpers that treat an archive id as pending when any `ChangedFile` path
         is under `duckspec/archive/<id>/` (folder path or descendant, including deletes)

- [x] 2. @spec archive/pending-commit Pending predicate: Dirty path under the archive folder is pending

- [x] 3. @spec archive/pending-commit Pending predicate: Dirt only outside the archive folder is not pending
