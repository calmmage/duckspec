# Config and settings toggles

Expand `Config.ui` to `UiConfig` with independent list/chat phase-pill flags (default on)
and wire Settings togglers that persist them.

## Tasks

- [x] 1. Replace `Config.ui: FontConfig` with `UiConfig` (`font_family`, `font_size`,
         `phase_pill_list`, `phase_pill_chat`) in `crates/duckboard/src/config.rs`; keep
         content fonts as `FontConfig`; both pill flags default `true` via `Default` /
         `serde(default)`

- [x] 2. Update `config::ui_font` and all `config.ui.font_*` call sites for the new shape;
         ensure legacy `[ui] font_*` TOML still loads

- [x] 3. Add Settings section with two togglers (list / chat) in
         `crates/duckboard/src/area/settings.rs`, saving via existing config save path

- [x] 4. @spec shell/phase-pills Surface settings: List and chat phase-pill settings default enabled
