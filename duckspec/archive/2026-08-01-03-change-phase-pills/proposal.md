# Change phase pills

Surface each change’s recognized lifecycle (and late-stage VCS dirty state) as short,
color-coded pills on the change list and above chat, with per-surface settings and
click-to-send for the next stage command.

## Motivation

Duckboard already derives where a change sits in the workflow for agent orientation and
empty-session bootstrap, but humans scanning the change list or working in chat never see
that state. Phase is only visible indirectly (overview files, step icons, archive
section). Lifecycle chips that once made next steps visible were deliberately removed from
chrome, so the human-facing “where am I?” gap is larger now.

Why now: the phase ladder is stable and shared; showing it as display chrome (not a second
status system) closes the gap without reintroducing multi-option lifecycle ladders under
the input.

## Intent

- Active changes and explorations show a short stage pill on the change list and, for the
  focused change/session, above the chat composer

- Each surface can be turned on or off independently in Settings; both default on

- Pill face is a short stage label; hover shows the existing long phase description

- Pills are soft color-coded by stage

- Clicking the lifecycle pill sends the current next-stage command into that change’s chat
  (empty-send `/ds-…` form) — it does not invent or set a freeform status

- On late stages only (`ready` / `archived`), a second pill reflects working-tree
  committed vs uncommitted (repo-wide dirty for this cut); hover states that honestly;
  click on uncommitted sends `Commit`

- Explorations use a muted `explore` pill; click sends `/ds-explore` only when that
  session is empty

- List-row click selects that change, then sends into its active/last session

- Phase remains derived from disk artifacts (and VCS dirty for the second pill) — no
  sidecar status field

## Non-goals

- Freeform or click-to-set status that overrides disk-derived phase

- Restoring multi-option lifecycle chip ladders, affirm/decline gates, or auto-messages
  under the input

- Change-scoped dirty detection (per-change “what’s uncommitted for this work only”)

- Stage badges or “you are here” chrome in the slash-command completion popup

- Redesigning agent orientation text or the phase ladder’s next-command rules

- Automatic commit or archive without the existing human/agent confirmation paths
