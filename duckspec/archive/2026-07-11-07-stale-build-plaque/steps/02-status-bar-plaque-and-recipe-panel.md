# Status bar plaque and recipe panel

Wire stale evaluation into shell state, show the Update chip in the status bar, and open a
manual install recipe panel with clipboard copy.

## Prerequisites

- [x] @step self-version-evaluation

## Tasks

- [x] 1. Store `stale: Option<StaleBuildInfo>` and `stale_panel_open` on `State`; refresh
         on project open, manual refresh, and root `Cargo.toml` file events; close panel
         when signal clears

- [x] 2. Extend status bar to accept an optional clickable Update chip; render it when
         stale is present

- [x] 3. Add recipe panel overlay (versions, quit + command instructions, Copy, dismiss)

- [x] 4. Wire `StaleBuildOpen` / `StaleBuildClose` / `StaleBuildCopy` messages; Copy uses
         clipboard write only — no install spawn

- [x] 5. @spec shell/stale-build Update plaque when stale: Plaque visible only when stale

- [x] 6. @spec shell/stale-build Update plaque when stale: Plaque opens the recipe panel without installing

- [x] 7. @spec shell/stale-build Recipe panel: Panel shows versions and copyable recipe

- [x] 8. @spec shell/stale-build Recipe panel: Panel dismisses without installing
