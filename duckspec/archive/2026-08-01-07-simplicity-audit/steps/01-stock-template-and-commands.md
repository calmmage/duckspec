# Stock template and commands

Add `simplicity-audit` template and three harness wrappers so `ds template` / `ds init`
expose `/ds-simplicity-audit`.

## Tasks

- [x] 1. Author `crates/duckspec/content/templates/simplicity-audit.md` (cut spine, hybrid
         write gate, handoffs per design)

- [x] 2. Add `commands/claude/ds-simplicity-audit.md` (`order: 1.5`, silent
         `ds template simplicity-audit`)

- [x] 3. Add `commands/opencode/ds-simplicity-audit.md` (same)

- [x] 4. Add `commands/codex/ds-simplicity-audit/SKILL.md` (name + same body)

- [x] 5. Rebuild/verify: `ds template simplicity-audit` prints the template; unknown-name
         path still clean
