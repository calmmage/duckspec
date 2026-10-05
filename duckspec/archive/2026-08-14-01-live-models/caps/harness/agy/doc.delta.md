# @ AGY harness

## ~ Models

The harness lists models from live `agy models` output when that command returns a
non-empty catalog. Each row’s display name is both the model id and the picker label — the
same string print mode accepts as `--model`. When live discovery is empty or fails, the
harness falls back to a curated table of those display names so the AGY slice is not wiped
on a missing binary. Every listed model is tagged with the AGY harness identity.
