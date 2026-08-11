# List digit switch - Design

Shell `Ctrl+1/2/3` selects the nth digit-indexed row of the focused Change or Ideas queue,
using the same paint-order sequences the lists already use for those rows, with no new
list chrome.

## Flow

```
KeyPress Ctrl+1|2|3
        │
        ▼
 keybind_list_digit(state, n)     // n ∈ {1,2,3}
        │
        ├─ modal / rename capture ──► None
        ├─ area ∉ {Change, Ideas} ──► None
        ├─ no project / no nth row ─► None
        │
        ▼
 ListDigitAction { Change(id) | Idea(path) }
        │
        ├─ already selected ──► no-op
        └─ else ──► SelectChange / SelectIdea
                    (stay in area; no exploration rename)
```

## Visible-row model

Index `n` is 1-based over the **digit-indexed paint sequence** for the active area (not
every decorative UI line).

```
| Area | Sequence |
| --- | --- |
| Change | Full painted live body from `ordered_live_queue`: live explorations + active changes after shared queue sort/pins, **then** pending-commit archives (when shown after live WIP). The separate **Archived** section and the files explorer are not indexed. |
| Ideas | Selectable **idea** rows only, in paint order: top-to-bottom walk of **expanded** sections (`Inbox` → `Exploration` → `Change` → `Archive`); nested idea paths under expanded tags included. Tag-folder chevron rows and other non-idea chrome are **not** indexed. Collapsed sections contribute nothing. |
```

### Shared builders (required)

Digit resolution must not maintain a second ordering algorithm:

```
| Area | Builder |
| --- | --- |
| Change | Share `ordered_live_queue` (or an equivalent single helper) with the Change list view — already the path via `painted_live_queue_ids`. |
| Ideas | One shared pure helper that yields idea paths in paint order; **both** the Ideas list view and digit resolution call it (or derive from it). A parallel re-implementation of sort/tag expand for digits alone is not allowed. |
```

That keeps sort key, star pins, `is_on_live_list`, section expand, and tag collapse from
drifting between click targets and `Ctrl+n`.

## When the chord fires

```
| Condition | Result |
| --- | --- |
| Project open and `active_area` is Change or Ideas | eligible |
| File finder, project picker, quick idea, new file, find, text search, or exploration rename owns the keyboard | no-op; existing handlers win |
| Chat or content column focused inside Change/Ideas | still eligible (area-level, not column) |
| Dashboard, Caps, Codex, Settings | no-op |
| `n` past digit-indexed length | no-op |
```

## Dispatch path

- `keybinds::keybind_list_digit(state, n) -> Option<ListDigitAction>` in
  `crates/duckboard/src/keybinds.rs`.

- `main` KeyPress: physical Control + `1`/`2`/`3` (not Command/logo as the primary
  modifier); thin call into the resolver and dispatch.

- **Change:** emit `SelectChange` only when the target id differs from `selected_change`
  (no exploration rename on re-select).

- **Ideas:** emit `SelectIdea` only when the path differs from the current selection.

```rust
pub enum ListDigitAction {
    SelectChange(String),
    SelectIdea(PathBuf),
}

pub fn keybind_list_digit(state, n) -> Option<ListDigitAction>
// n is 1..=3; None when gated out or no nth digit-indexed row
```

## Chrome

Keys only. No `⌃n` badges on queue rows in this change.

## Out of scope (unchanged)

Multi-project sessions, project top-bar tabs, `Cmd+1..3` project slots, named area
hotkeys, Caps/Codex digit jump, a hot-switch order different from the list paint model.

## Settled choices

- Visible paint-order sequences only — no separate MRU/pin model for digits.
- Ideas digits = selectable idea rows, not tag chrome.
- Change digits include pending-commit archives when they appear on the live list.
- Ideas list and digits share one pure paint-order path for idea rows.
- Area-scoped chord, not column-scoped.
- Change + Ideas only for v1.
- No-op when already selected (avoids exploration rename).
- No list UI affordance in v1.

## Review amendments

From `reviews/01-review-list-digit-switch-fidelity.md`: idea-only Ideas index wording;
pending archives named on the Change live body; mandatory shared Ideas paint-order helper.
