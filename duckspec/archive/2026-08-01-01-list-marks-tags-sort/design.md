# List marks, tags, and sort - Design

Idea frontmatter owns exclusive boosts; CHANGE and Ideas list rows inherit, sort with a
shared pure ladder, and render type/phase pillows with density rules.

## Approach

```
                    ┌─────────────────────────┐
  mark / tag edit ─►│ Idea frontmatter (YAML) │
                    │  mark, favored_at, tags │
                    └───────────┬─────────────┘
                                │ resolve by
                    exploration id / change name
                                ▼
┌──────────────┐    ┌───────────────────────┐    ┌────────────────┐
│ chats/*      │───►│ queue_row projection  │───►│ list render    │
│ last message │    │ + sort ladder         │    │ mark · title · │
│              │    │ + list prefs          │    │ pillows        │
└──────────────┘    └───────────────────────┘    └────────────────┘
                                ▲
                     change_scope_facts.phase
                     (change-linked only)
```

- **Store once** on the idea; never on the change slug or a second fav map.

- **Project** a `QueueRow` for each list entry (exploration, active change, idea leaf).

- **Sort** is pure over projections + prefs; UI only applies order.

- **Duckboard-only** — no duckpond / `ds` schema changes; ideas stay under project data
  dir.

## Idea mark model

Extend `Frontmatter` in `crates/duckboard/src/idea_store.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IdeaMark {
    #[default]
    None,
    Star,
    Hot,
    Cool,
}

// on Frontmatter:
// mark: IdeaMark                 // default None, skip serialize if None
// favored_at: Option<String>     // RFC3339; Some only when mark == Star
```

```rust
pub fn cycle_mark(m: IdeaMark) -> IdeaMark {
    // None → Star → Hot → Cool → None
}

pub fn apply_mark(fm: &mut Frontmatter, next: IdeaMark, now: OffsetDateTime) {
    fm.mark = next;
    fm.favored_at = match next {
        IdeaMark::Star => Some(format_rfc3339(now)), // refresh pin time on re-star
        _ => None,
    };
}
```

- Unknown YAML values deserialize as `None` (or reject → `None`) so hand-edits never brick
  load.

- Tags stay `Vec<String>`: `tags[0]` primary (path tree); `tags[1..]` type pillows. No new
  `type` field.

## Inheritance index

Build a small lookup when ideas are loaded / saved:

```rust
pub struct IdeaLinks {
    by_exploration: HashMap<String, PathBuf>, // exploration id → idea path
    by_change: HashMap<String, PathBuf>,      // change name → idea path
}

pub fn idea_for_exploration(ideas: &[Idea], id: &str) -> Option<&Idea>;
pub fn idea_for_change(ideas: &[Idea], name: &str) -> Option<&Idea>;
```

CHANGE list already uses `idea_path_for_change`; extend the same helpers rather than a
parallel store. Exploration rows use `frontmatter.exploration` / `Exploration.idea_path`
(keep both in sync on mint and rename).

## Auto-mint idea on first tag

Trigger: first successful **tag** mutation on a free exploration or unlinked active change
(no linked idea yet). Mark cycle alone does **not** mint (proposal: first tag).

```rust
pub enum MintTarget {
    Exploration { id: String, display_name: String },
    Change { name: String },
}

pub fn mint_linked_idea(
    ideas: &mut Vec<Idea>,
    target: MintTarget,
    first_tag: &str,
    project_root: Option<&Path>,
) -> Result<PathBuf, …>;
```

On mint:

```
| Target | Idea `state` | Frontmatter | Side effects |
| --- | --- | --- | --- |
| Exploration | `Exploration` | `title = display_name`, `exploration = id`, `tags = [first]` | set `Exploration.idea_path`; persist explorations |
| Change | `Change` | `title = prettified slug` (kebab → spaces, light title-case), `change = name`, `tags = [first]` | none on folder |
```

CHANGE list **stops filtering** `idea_path.is_none()` so linked explorations remain
visible.

Mark cycle on a row with no idea: **no-op** (or toast-less ignore) until an idea exists —
first tag creates the home. Tag UI entry points on CHANGE rows: enable “+ Tag” / chip path
that routes through mint when unlinked.

## Queue projection and sort

```rust
pub struct QueueRowMeta<'a> {
    pub key: QueueKey,              // Exploration(id) | Change(name) | Idea(path)
    pub title: &'a str,
    pub mark: IdeaMark,
    pub favored_at: Option<&'a str>,
    pub type_tags: &'a [String],    // tags[1..]
    pub phase: Option<&'static str>, // change_scope_facts.phase
    pub last_message_at: Option<i128>, // unix nanos; None → bottom of activity sort
}

pub enum SortKey {
    LastMessage, // default
    Phase,
    Created,     // idea.created or exploration/change fallback
}

pub fn sort_queue(rows: &mut [QueueRowMeta], key: SortKey) {
    // 1) pin: among mark==Star, take up to 3 with newest favored_at; those first (favored_at desc)
    // 2) rest (including surplus stars): by key
    // 3) tie-break: title asc
}
```

**Last message:** max of latest non-priming message timestamp across sessions under the
row’s chat scope (`chats/<scope>/`). Scope = exploration id or change name. Pure helper
over loaded sessions or a lightweight scan; cache on project refresh / chat save if redraw
cost shows up.

**Phase order** (when `SortKey::Phase`): stable ladder matching lifecycle arms already
used in `change_scope_facts` (proposal-only → design → … → implementing → …). Rows without
phase sort after phased ones (or before — pick “after” so active work floats when sorting
by phase).

Fav pin is **per list invocation** (CHANGE active list and each Ideas section sort
independently; each applies top-3 within its own row set).

## List preferences

Global (with other UI flags) in `~/.config/duckboard/config.toml`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ListConfig {
    pub sort_key: SortKey,          // default LastMessage
    pub show_type_pillows: bool,    // default true
    pub show_phase_pillows: bool,   // default true
}
// Config { …, pub list: ListConfig }
```

Sort control sits in the Change section header next to **+** (`collapsible` gains optional
trailing header actions, or Change builds a custom header row): opens a small menu — sort
key radios + pillow visibility toggles. **Ideas section headers get the same sort menu**
(shared `ListConfig` prefs; either surface writes the same config).

## List row chrome

Use existing `ListRow` `label_content` / `trailing` / `on_hover` in
`crates/duckboard/src/widget/list_view.rs`.

```
[leading/icon] [mark] [title…] [type pills…] [phase pill?] [row actions]
```

- **Mark:** emoji for Star/Hot/Cool; when `None` and row hovered, outline-star glyph;
  click → `CycleMark(key)` (cycles + save idea). Clicks on mark must not steal row select
  only — accept select+cycle or use a dedicated mark hit target with `stop` semantics as
  iced allows.

- **Type pillows:** secondary tags, compact chip style (reuse ideas chip visual language
  at smaller density). Max **display length** per tag (e.g. 12–16 chars + ellipsis); full
  string on hover tooltip if clipped.

- **Phase pillow:** `phase` string as-is from `change_scope_facts` when present and pref
  on.

- **Overflow:** measure title + visible pillows against row content width; if overflow,
  drop all pillows from the steady row and show them only while the row is hovered (same
  hover channel as mark outline / close / refresh). Mark + title always stay.

- Primary tag never appears as a pillow (tree already shows it on Ideas).

Ideas list leaf rows get the same mark + type pillows + sort within each section’s flat
idea set (tree groups still by primary path; **within** a tag node, apply queue sort).
Phase pillow on Ideas when `frontmatter.change` resolves.

## CHANGE list integration

`view` in `area/change.rs`:

1. Collect free + idea-linked explorations + active changes into `QueueRowMeta`.

2. `sort_queue` with prefs.

3. Render with mark / pillows / cycle handlers.

4. Header: sort menu + existing add exploration.

5. Archived section: out of scope for fav pin / activity sort in v1 (keep current archived
   ordering) unless cheap — default **leave archived as today**.

## Ideas list integration

- Persist `mark` / `favored_at` on save; cycle from row.

- Replace pure `created` desc within each section’s idea collection with `sort_queue`.

- Each Ideas section header includes the same sort menu as Change (shared prefs).

- Tag add already exists; when tagging from a CHANGE-driven mint path, land in ideas store
  then re-resolve links.

- Toolbar chips unchanged for primary/secondary editing; list pillows are read-mostly
  (click may still open edit later — non-goal for v1).

## Impact

- `idea_store::Frontmatter` schema extension (backward compatible defaults)

- `config::ListConfig` + Settings optional mirror later

- `area/change.rs` list filter + sort + header control

- `area/ideas.rs` sort + row mark

- Shared helpers module (e.g. `crates/duckboard/src/queue_list.rs`) for
  projection/sort/overflow policy

- `collapsible` header API may accept multiple trailing actions

- Chat activity scan helper (read timestamps; no chat format change)

- No duckpond, no change folder renames, no CLI

## Decisions

- **Idea-only store** — no `Exploration.mark`. Alternative: dual write (rejected: proposal
  sole surface).

- **Secondary tags = type pillows** — no `type:` field. Alternative: closed enum (rejected
  until vocabulary is real).

- **Mint on first tag only** — mark without idea is no-op. Alternative: mint on first mark
  (better star UX; defer unless star-on-bare feels broken in use).

- **Phase strings reused verbatim** from `change_scope_facts`. Alternative: short chip map
  (later if density hurts).

- **Prefs global in config.toml** — not per-project. Alternative: per-project data file
  (only if multi-project workflows demand it).

- **Fav top-3 scoped per list body** — not a global pin across Ideas+CHANGE
  simultaneously.

- **Archived list unchanged** in v1.

- **Sort menu on Ideas too** — same control on Ideas section headers; shared `ListConfig`.
  Alternative: CHANGE-only chrome (rejected: one queue language, two lists).

- **Re-star refreshes `favored_at`** — cycling back to Star always writes a new pin time
  so the top-3 pin reflects the latest star gesture. Alternative: preserve first star time
  (rejected: re-star would not re-boost).

- **Mint title for unlinked change = prettified slug** — kebab-case folder name → spaces +
  light title-case for the idea title; identity remains the raw change name in
  `frontmatter.change`. Alternative: raw folder name as title (rejected: uglier list
  labels).

## Risks

- **Activity scan cost** on every redraw → cache `last_message_at` per scope; invalidate
  on chat persist / project refresh.

- **Long phase strings** blow density → overflow-to-hover already mitigates; shorten later
  if needed.

- **Mark click vs row select** fight → dedicated mark control with clear hit target; row
  press remains on non-mark area.

- **Mint races / duplicate ideas** for same exploration → check `idea_path` and
  `by_exploration` before mint; single-flight save.
