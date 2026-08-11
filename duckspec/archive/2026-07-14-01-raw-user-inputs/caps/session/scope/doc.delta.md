# @ Session scope orientation

## @ Scope kinds

```
kind          orientation content
────────────  ──────────────────────────────────────────────────────────────
change        the change name, project-root path under duckspec/changes/,
              its progress, its next stage, a statement that
              change-acting commands target this change by default, and —
              when present — a pointer at duckspec/changes/{name}/inputs.md
              for raw human inputs (re-ground; do not invent motivation;
              do not rewrite that file)
exploration   an early-stage brainstorming chat with no formal artifacts yet
caps          the project's capability tree — points at duckspec/caps/ and
              duckspec/project.md
codex         the project's codex — points at duckspec/codex/ and
              duckspec/project.md
```

Only the change kind carries progress and a next-stage suggestion. The other kinds
describe their scope and nothing more — they never report change progress or a change
next-stage.

## @ Change orientation

For a change scope the orientation is authoritative: it names the change, states that
change artifacts live under the project-root path `duckspec/changes/{name}/`, and tells
the agent that change-acting commands — such as archive and apply — act on that change by
default. The agent disambiguates only when the user names a different change, so a project
with several active changes never forces the agent to ask which one to act on.

The orientation also reports where the change sits in its lifecycle: the suggested next
stage and step progress.

When `duckspec/changes/{name}/inputs.md` exists and is non-empty, the orientation also
names that path and tells the agent to re-ground on those raw human inputs instead of
inventing motivation, and not to rewrite the file. When the file is missing or empty,
orientation says nothing about an inputs ledger.
