# Order field and frontmatter discovery

Add `order_key` on `SlashCommand`, strict frontmatter parse in discovery, and
descriptions/`order` on installable `ds-*` command files.

## Tasks

- [x] 1. Add `order_key: Option<u32>` to `SlashCommand` in
         `crates/duckchat/src/provider.rs` and update every construction site (discovery,
         system registry, tests) with `None` where not yet parsed

- [x] 2. Replace description-only frontmatter parsing in
         `crates/duckchat/src/claude_code/discover.rs` with meta parse that sets
         `description` and `order_key` (strict `N` / `N.D` tenths; invalid → `None`)

- [x] 3. Add unit tests for `parse_order_key` / discovery covering accepted and rejected
         forms

- [x] 4. @spec chat/slash-commands Frontmatter order and description: Description from frontmatter is on the discovered command

- [x] 5. @spec chat/slash-commands Frontmatter order and description: Integer order maps to tenths key

- [x] 6. @spec chat/slash-commands Frontmatter order and description: One-decimal order maps to tenths key

- [x] 7. @spec chat/slash-commands Frontmatter order and description: Invalid order forms yield no order key

- [x] 8. @spec chat/slash-commands Frontmatter order and description: Missing order yields no order key

- [x] 9. Add matching `description` + `order` frontmatter to all `ds-*.md` under
         `crates/duckspec/content/commands/claude/` and `…/opencode/` per the design table

- [x] 10. Refresh project-installed copies under `.claude/commands/` (and opencode if
          present) so local discovery sees the new frontmatter
