# Chat viewer style

Global Answer presentation mode: stored style, effective resolution, Settings choices, and
Classic as the identity Answer body path under default settings.

## Requirement: Stored viewer style

The application SHALL persist a single global chat viewer style whose allowed values are
classic, focus, and document. When no value has been stored, the stored style SHALL be
classic.

> test: code

### Scenario: Default stored style is classic

- **GIVEN** no chat viewer style has been stored
- **WHEN** the stored viewer style is read
- **THEN** the stored style is classic

> test: code
> - crates/duckboard/src/config.rs:513

### Scenario: Chosen style round-trips through save and load

- **GIVEN** the user selects a viewer style value among classic, focus, and document
- **WHEN** the setting is saved and loaded again
- **THEN** the stored style is that selected value

> test: code
> - crates/duckboard/src/config.rs:526

## Requirement: Effective viewer style

Presentation SHALL resolve an effective viewer style from the stored style. The effective
style SHALL be classic when the stored style is unknown, is document, or is any style not
yet implemented for Answer rendering. When the stored style is an implemented Answer
style, the effective style SHALL be that stored style.

> test: code

### Scenario: Stored classic yields effective classic

- **GIVEN** the stored viewer style is classic
- **WHEN** the effective viewer style is resolved
- **THEN** the effective style is classic

> test: code
> - crates/duckboard/src/config.rs:549

### Scenario: Stored focus yields effective focus when Focus is implemented

- **GIVEN** Focus Answer presentation is implemented
- **AND** the stored viewer style is focus
- **WHEN** the effective viewer style is resolved
- **THEN** the effective style is focus

> test: code
> - crates/duckboard/src/config.rs:577

### Scenario: Stored document yields effective classic while unimplemented

- **GIVEN** Document Answer presentation is not implemented
- **AND** the stored viewer style is document
- **WHEN** the effective viewer style is resolved
- **THEN** the effective style is classic

> test: code
> - crates/duckboard/src/config.rs:562

## Requirement: Settings choices

Settings SHALL offer a chat viewer-style control whose choices are exactly the styles
implemented for Answer rendering. Classic SHALL always be offered. Focus SHALL be offered
when Focus Answer presentation is implemented. Document SHALL NOT be offered while
Document Answer presentation is not implemented.

> test: code

### Scenario: Settings lists classic and focus when Focus is implemented

- **GIVEN** Focus Answer presentation is implemented
- **WHEN** the Settings viewer-style choices are built
- **THEN** the choices are classic and focus

> test: code
> - crates/duckboard/src/area/settings.rs:700

### Scenario: Document is not offered while unimplemented

- **GIVEN** Document Answer presentation is not implemented
- **WHEN** the Settings viewer-style choices are built
- **THEN** document is not among the choices

> test: code
> - crates/duckboard/src/area/settings.rs:715

## Requirement: Classic Answer identity path

When the effective viewer style is classic, each Answer SHALL be presented with the
full-body classic Answer path — the pre-change single-body presentation, not Focus slices.
Viewer style SHALL NOT change the presentation mode of non-Answer segments.

> test: code

### Scenario: Effective classic presents Answer as one full body

- **GIVEN** the effective viewer style is classic
- **AND** a settled Answer with body text
- **WHEN** the Answer is presented
- **THEN** the Answer uses the full-body classic presentation

> test: code
> - crates/duckboard/src/widget/agent_chat.rs:2767

### Scenario: Non-Answer segments ignore viewer style for presentation mode

- **GIVEN** the effective viewer style is classic or focus
- **AND** a transcript that includes User, Thinking, or Activity segments
- **WHEN** those non-Answer segments are presented
- **THEN** their presentation mode is not selected by the viewer style

> test: code
> - crates/duckboard/src/widget/agent_chat.rs:2791
