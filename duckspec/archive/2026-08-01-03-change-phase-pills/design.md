# Change phase pills - Design

Add a pure phase-display model on top of `change_scope_facts` + repo dirty, render short
color-coded pills on change-list rows and above the chat composer (Settings-gated), and on
click route the next lifecycle/`Commit` send into the right session.

## Approach

```
project artifacts + steps          vcs::changed_files (repo-wide)
        │                                    │
        ▼                                    ▼
change_scope_facts(name)              dirty: bool
        │                                    │
        └────────────┬───────────────────────┘
                     ▼
            phase_display(scope)     // pure
                     │
          PhaseDisplay { short, long, send?, vcs? }
                     │
        ┌────────────┴────────────┐
        ▼                         ▼
 list trailing (if ui.list)   composer strip (if ui.chat)
        │                         │
        └──── click ──► select scope → send_prompt_text(send)
```

- **No new source of truth.** Short labels and VCS pill are projections of existing
  disk/VCS facts.

- **Do not revive obvious chrome.** Pills are independent chrome; `build_obvious_chrome`
  stays empty on the product path.

- **One pure builder, two thin views.** List and chat only pass settings + layout; labels,
  colors, clickability, and send text come from `PhaseDisplay`.

Boundaries:

```
| In | Out |
| --- | --- |
| `change_scope_facts` mapping + short enum | Changing ladder / orientation copy |
| Pill widget + list/chat placement | Multi-option lifecycle ladders under input |
| Settings toggles under expanded `[ui]` | Change-scoped dirty |
| Click → `format_lifecycle_command` / `Commit` send | Freeform status writes |
```

## Phase display model

Extend the pure surface next to `change_scope_facts` in
`crates/duckboard/src/area/change.rs` (or a sibling `phase_display` module re-exported
from change). Keep `ChangeScopeFacts` orientation contract unchanged; add a display DTO.

```rust
/// Compact stage for pill face (UI only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseShort {
    Explore,
    Empty,
    Proposal,
    Design,
    Specs,
    Steps,
    Review,
    Ready,
    Archived,
}

impl PhaseShort {
    pub fn label(self) -> &'static str { /* "explore", "empty", … "archived" */ }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VcsPill {
    /// Working tree clean vs HEAD.
    Committed,
    /// Any repo-wide dirty path.
    Uncommitted,
}

#[derive(Debug, Clone)]
pub struct PhaseDisplay {
    pub short: PhaseShort,
    /// Hover body: long phase string and/or VCS honesty.
    pub lifecycle_hover: String,
    /// Empty-send text for lifecycle click (`/ds-…`), if clickable.
    pub lifecycle_send: Option<String>,
    pub vcs: Option<VcsPill>,
    pub vcs_hover: Option<&'static str>,
    /// `Some("Commit")` only when `vcs == Uncommitted`.
    pub vcs_send: Option<&'static str>,
}

/// Build display for an active change name.
pub fn phase_display_for_change(
    name: &str,
    project: &ProjectData,
    vcs_dirty: bool,
) -> Option<PhaseDisplay> { todo!() }

/// Archived row: short=Archived, long fixed, no lifecycle send;
/// VCS pill always present (ready/archived rule).
pub fn phase_display_for_archived(vcs_dirty: bool) -> PhaseDisplay { todo!() }

/// Exploration row.
pub fn phase_display_for_exploration(
    session_empty: bool,
) -> PhaseDisplay { todo!() }
```

Mapping from existing arms (active changes):

```
| `change_scope_facts` situation | `short` | `lifecycle_send` |
| --- | --- | --- |
| no artifacts | `Empty` | `/ds-propose` |
| proposal, no design | `Proposal` | `/ds-design` |
| design, no caps | `Design` | `/ds-spec` |
| caps, no steps | `Specs` | `/ds-step` |
| open steps | `Steps` | `/ds-apply` |
| all done, no review | `Ready` | `/ds-archive` |
| no open steps + review | `Review` | `/ds-step` |
| archived (not in facts today) | `Archived` | none |
| exploration | `Explore` | `/ds-explore` iff session empty |
```

Long hover for active changes: reuse `ChangeScopeFacts.phase` verbatim. Archived:
`"archived"` (or a single fixed long line). Explore: fixed short explanation.

**VCS pill visibility:** only when `short` is `Ready` or `Archived`.

- dirty → `Uncommitted`, hover e.g. `"working tree has uncommitted changes (repo-wide)"`,
  send `"Commit"`

- clean → `Committed`, hover e.g. `"working tree clean vs HEAD"`, no send

`change_scope_facts` continues to return `None` for archived (orientation/next-stage
unchanged). Display uses the dedicated archived helper.

## Pill widget

Small shared view in `crates/duckboard/src/widget/phase_pill.rs` (style kin to status-bar
Update chip / idea tag chips).

```rust
pub enum PillKind {
    Lifecycle(PhaseShort),
    Vcs(VcsPill),
}

pub fn view_pill<'a, Msg: Clone + 'a>(
    kind: PillKind,
    hover: &'a str,
    on_press: Option<Msg>,
) -> Element<'a, Msg> { todo!() }

pub fn view_pair<'a, Msg: Clone + 'a>(
    display: &'a PhaseDisplay,
    on_lifecycle: Option<Msg>,
    on_vcs: Option<Msg>,
) -> Element<'a, Msg> { todo!() }
```

- Face: short label, soft tint fill + border (alpha ~0.12 / 0.35 like Update chip).

- Hover: `iced::widget::tooltip` (0.14) with `lifecycle_hover` / `vcs_hover`; no custom
  overlay state if tooltip works in list + composer.

- Clickable only when `on_press` is `Some`; inert pills keep the same look (no button
  affordance beyond cursor if easy).

- Soft palette (theme helpers): e.g. explore/empty muted; proposal–specs accent_dim; steps
  warning-ish; review accent; ready success; archived muted; uncommitted warning;
  committed success/muted.

## Change list surface

In `area/change.rs` `view_list`:

- If `config.ui.phase_pill_list` (see Settings), for each exploration / active / archived
  row, compute `PhaseDisplay` and attach `view_pair` via `ListRow::trailing` (after label;
  leaves `after_icon` for idea links / rename actions).

- Row `on_press` stays select-only. Pill clicks must **not** bubble as row select-only:
  pill button handles press; after click, main still ensures selection for the target
  scope before send.

- Click message (change-area): e.g. `PhasePillSend { target: ChangeTarget, text: String }`
  where target is exploration id / change name / archived name. Main handles:
  `SelectChange` → resolve active `AgentSession` for that scope →
  `interaction::send_prompt_text`.

Exploration session emptiness: use that exploration’s active (or sole) session
`messages.is_empty()` from interactions map; if no session yet, treat as empty (click
allowed → `/ds-explore`).

## Chat surface

In `widget/agent_chat.rs` composer column (sticky above input, outside transcript scroll):

```
composer_col:
  [phase pill row]   ← new, if settings + scope has display
  [oneshot / next ghosts…]
  input
  meta_row
```

- Main/three-column layout passes `Option<PhaseDisplay>` + click msgs only for
  Change/Exploration scopes when `config.ui.phase_pill_chat`.

- Caps/codex: no strip.

- Same `view_pair`; click sends into **current** session (already focused) — no select
  hop.

While `is_streaming`: do not start a parallel turn. Prefer **queue** if the submit path
already stages into `queue_editor`; else **no-op** (design choice: match empty-Enter
lifecycle send behavior). Implementation mirrors that path rather than inventing a third
send mechanism.

## Settings / config

Today `Config.ui` is a bare `FontConfig`. Expand to a real UI table without breaking TOML:

```rust
// crates/duckboard/src/config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub font_family: String,
    pub font_size: f32,
    /// Change-list phase pills. Default true.
    pub phase_pill_list: bool,
    /// Above-composer phase pills. Default true.
    pub phase_pill_chat: bool,
}

// Config.ui: FontConfig → UiConfig
// content: remains FontConfig
```

Existing `[ui] font_*` keeps loading; new keys default **true** via `serde(default)` /
`Default`.

Settings UI (`area/settings.rs`): section “Phase pills” (or under Chat/UI) with two
togglers — list / chat — same save path as agent input hints.

`config::ui_font` reads `config.ui.font_family` / `font_size` after the type change.

## Click routing (main)

```rust
// sketch — main update
PhasePillSend { scope_key, text } => {
    // 1. Select change/exploration if list-origin and not already selected
    // 2. Ensure interaction + active session for Scope::Change|Exploration
    // 3. send_prompt_text(active, text, highlighter)  // or queue if streaming
}
```

Send strings:

- Lifecycle: `obvious_bubble::format_lifecycle_command(&facts.next_command)` (already
  `/`-prefixed helper).

- VCS: literal `"Commit"` (historical affirm label; agent + standing VCS instructions
  already cover commit confirmation).

## Impact

- `config.ui` type change (`FontConfig` → `UiConfig`); content fonts untouched; TOML
  backward compatible with defaults.

- New widget module; list + agent_chat + settings + main message plumbing.

- No duckpond / ds CLI / orientation string changes.

- Caps likely: duckboard UI behavior (phase pills / settings) — not core parse/status.

- Tests: pure mapping unit tests for every ladder arm + archived + explore
  empty/non-empty; config default true; optional view smoke if cheap.

## Decisions

- **Display DTO separate from `ChangeScopeFacts`** — orientation stays stable; UI can grow
  short/VCS without bloating agent blurb fields. Alternative: overload facts (rejected:
  couples agent orientation to chrome).

- **Trailing on list rows** — end-of-name pills; `after_icon` remains idea/decoration.
  Alternative: replace icon (rejected: loses branch/explore icon).

- **Sticky composer strip, not status bar** — always next to the place you send from;
  status bar stays path crumbs. Alternative: status-bar chip (rejected: farther from
  action, fights Update chip).

- **Repo-wide dirty for VCS pill** — honest hover; no fake per-change dirty. Alternative:
  delay VCS pill until scoped dirty (rejected for v1 by proposal).

- **Do not re-enable obvious lifecycle chrome** — pills are the human phase UI;
  next-command send is single-action only.

## Risks

- **Narrow list + long names** → pills compete for width → trailing + horizontal pan
  already used; keep short labels (≤10 chars); allow clip/ellipsis on name not pill.

- **Tooltip flaky in scroll/list** → if iced tooltip misbehaves, fall back to status-bar
  hint or hover-only muted expansion; pure model still holds.

- **Click during streaming surprises** → mirror existing queue/no-op submit policy; never
  double-stream.

- **“Commit” send without local confirm UI** → agent standing instructions already forbid
  auto-commit; pill only injects the word, same as old affirm chrome.
