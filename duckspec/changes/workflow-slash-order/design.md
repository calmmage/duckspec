# Workflow slash order - Design

Carry optional workflow `order` from command frontmatter into `SlashCommand`, fill
descriptions the same way, and use order as the Workflow-aware sort key after fuzzy score
and kind.

## Approach

```
content/commands/{claude,opencode}/ds-*.md
        │  YAML frontmatter:
        │    description: "…"
        │    order: 1 | 3.1 | …   (see parse rules)
        ▼
ds init → .claude/commands/ (etc.)
        ▼
discover.rs  parse description + order_key
        ▼
SlashCommand { name, description, kind, order_key }
        │
        ▼
build_completion_catalog (kind tagging unchanged)
        │
        ├─► filter_commands   score ↓ → kind → order_key → name
        └─► /help Workflow    order_key → name
```

No UI stage numbers. Rows already show `cmd.description`; frontmatter fills the
empty-description gap.

**Cap dependency:** `chat/slash-commands` is still only under `system-slash-commands` (not
in main `caps/` yet). Specs for this change are a **delta** on that cap once mainline has
it, or sequenced after that change lands. Code assumes kinded catalog already exists.

## Frontmatter on installable commands

Canonical: `crates/duckspec/content/commands/claude/` and `…/opencode/` — same frontmatter
on both.

```markdown
---
description: Orient and brainstorm before opening a change
order: 1
---
Run `ds template explore` silently. …
```

### `order` parse rules (strict)

Accepted forms only:

```
| Form | Example | `order_key` (tenths) |
| --- | --- | --- |
| integer | `1`, `7` | `10`, `70` |
| one decimal digit | `0.5`, `3.1`, `6.2` | `5`, `31`, `62` |
```

Rejected → `order_key = None` (sort last with other unknowns):

- multi-digit fraction: `3.10`, `3.15`
- trailing/leading dot only: `1.`, `.5`
- extra junk / scientific / negatives: `1e2`, `-1`, `3.1.2`
- empty or non-numeric

No float arithmetic in the sort path: parse into `Option<u32>` tenths once.

Working table (relative order only; never shown in UI):

```
| order | name |
| --- | --- |
| 0.5 | `ds-backfill` |
| 1 | `ds-explore` |
| 2 | `ds-propose` |
| 3 | `ds-design` |
| 3.1 | `ds-verify` |
| 4 | `ds-spec` |
| 5 | `ds-step` |
| 6 | `ds-apply` |
| 6.1 | `ds-review` |
| 6.2 | `ds-followup` |
| 7 | `ds-archive` |
| 8 | `ds-codex` |
```

Missing `order` or failed parse → `None`. Missing `description` → `""`.

## SlashCommand order field

```rust
// crates/duckchat/src/provider.rs
pub struct SlashCommand {
    pub name: String,
    pub description: String,
    pub kind: SlashCommandKind,
    /// Tenths from frontmatter `order` (`3.1` → 31). `None` → after ordered peers.
    pub order_key: Option<u32>,
}
```

System registry and non-workflow skills: `order_key: None`.

## Discovery parse

```rust
// crates/duckchat/src/claude_code/discover.rs
struct FrontmatterMeta {
    description: String,
    order_key: Option<u32>,
}

fn parse_frontmatter_meta(path: &Path) -> FrontmatterMeta { /* … */ }

/// Accept only `digits` or `digits . digit` (exactly one fractional digit).
fn parse_order_key(raw: &str) -> Option<u32> {
    // "1" → Some(10), "3.1" → Some(31), "3.10" → None, "1." → None
}
```

Replace description-only helper. Discovery sort is **non-authoritative**; presentation
sort lives in duckboard (`filter_commands`, `/help`).

## Completion and help sort

```rust
// filter_commands (agent_chat.rs)
// 1. fuzzy score descending
// 2. kind: System → Workflow → Agent
// 3. order_key: Some ascending, None last
// 4. name ascending

// append_kind_section (slash_commands.rs)
// Workflow: order_key then name
// System / Agent: name (unchanged)
```

```rust
pub fn slash_order_rank(order_key: Option<u32>) -> (u8, u32) {
    match order_key {
        Some(k) => (0, k),
        None => (1, 0),
    }
}
```

## Impact

- All `SlashCommand { … }` construction sites gain `order_key`

- All `ds-*.md` under content/commands (claude + opencode) get frontmatter; local installs
  need reinstall/copy

- Spec work: delta on `chat/slash-commands` after that cap is on mainline

- No composer chrome, no harness protocol change

## Decisions

- **Frontmatter is the only order/description source** — no parallel duckboard hardcode
  table.

- **Strict `N` / `N.D` parse into `u32` tenths** — no `f64` on the type; multi-digit
  fractions rejected, not rounded.

- **No stage numbers / “you are here”** in the popup.

- **Consumers own sort** — filter + `/help`, not discovery alpha order.

- **Unknown `ds-*` last** among Workflow when `order_key` is `None`.

## Risks

- **Stale `.claude/commands` without frontmatter** → empty descriptions + unknown order
  until reinstall. Mitigation: ship content/; document re-run of command install; behavior
  degrades safely to “last + no blurb.”

- **Parallel `system-slash-commands`** → same cap area in flight. Mitigation: implement
  against current kinded API; write `spec.delta.md` once the base cap is in `caps/`, or
  land after archive of that change.

- **Author typos `3.10`** → rejected (None), not silently `3.1`. Mitigation: unit tests on
  the parser; content files use only valid forms.
