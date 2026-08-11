# Viewer style config

Add global `viewer_style` storage and effective-style resolution (Focus not implemented
yet).

## Tasks

- [x] 1. Add `ViewerStyle` enum and `ChatConfig.viewer_style` (default classic; serde
         lowercase)

- [x] 2. Implement `effective_viewer_style` / implemented-styles helper (Focus not
         implemented → classic)

- [x] 3. Unit tests for store and effective resolution

- [x] 4. @spec chat/viewer-style Stored viewer style: Default stored style is classic

- [x] 5. @spec chat/viewer-style Stored viewer style: Chosen style round-trips through save and load

- [x] 6. @spec chat/viewer-style Effective viewer style: Stored classic yields effective classic

- [x] 7. @spec chat/viewer-style Effective viewer style: Stored document yields effective classic while unimplemented

## Outcomes

- `FOCUS_ANSWER_IMPLEMENTED` is `false` in this step so stored `focus` still resolves to
  Classic; step 04 flips it when Focus presentation ships.
