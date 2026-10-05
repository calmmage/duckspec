# @ Harness selection

Model choices carry an explicit harness identity that persists across restarts, turns
dispatch to the provider their model names, and the default model cascade prefers a
per-chat pin, then a project override, then a resolved global default — follow-backend
when unset or still the former built-in, otherwise the stored pin — available only when
that preferred model is in the process catalog.

## @ Requirement: Default model resolution

Resolving the preferred model for a turn SHALL prefer a per-chat pin, then a project
override, then the resolved global default. An unset global default, or a stored global
default that is the former built-in (`grok` / `grok-4.5`), SHALL resolve as
follow-backend: the Grok advertised default when that model is in the catalog, otherwise
the first catalog model. The preferred model is **available** only when it is present in
the process model catalog. When follow-backend yields no model, or the preferred model is
absent from the catalog, the model for the turn is not available.

> test: code

### - Scenario: With no preferred model at any cascade level, the model is not available

### + Scenario: Unset global follows the Grok advertised default when that model is in the catalog

- **GIVEN** no per-chat pin
- **AND** no project override
- **AND** no stored global default
- **AND** a Grok advertised default that is in the process model catalog
- **WHEN** the preferred model for a turn is resolved
- **THEN** the preferred model is that Grok advertised default

> test: code

### + Scenario: Unset global follows the first catalog model when the Grok advertised default is absent

- **GIVEN** no per-chat pin

- **AND** no project override

- **AND** no stored global default

- **AND** a non-empty process model catalog that does not include the Grok advertised
  default

- **WHEN** the preferred model for a turn is resolved

- **THEN** the preferred model is the first model in catalog order

> test: code

### + Scenario: A former built-in stored global follows the advertised default

- **GIVEN** no per-chat pin
- **AND** no project override
- **AND** a stored global default of `grok` / `grok-4.5`
- **AND** a Grok advertised default that is in the process model catalog
- **WHEN** the preferred model for a turn is resolved
- **THEN** the preferred model is that Grok advertised default

> test: code

### + Scenario: Follow-backend with an empty catalog is not available

- **GIVEN** no per-chat pin
- **AND** no project override
- **AND** no stored global default
- **AND** an empty process model catalog
- **WHEN** the effective model for a turn is resolved
- **THEN** the model for the turn is not available

> test: code

## @ Requirement: Global default model setting

The application SHALL store a global main-chat default model as an application setting
(not scoped to a project). Follow-backend SHALL NOT be persisted as a concrete model
choice. Clearing the setting SHALL return to follow-backend.

> test: code

### - Scenario: An unset global default is seeded from the former built-in when that model is in the catalog

### - Scenario: An unset global default is seeded from the first catalog model when the former built-in is absent

### + Scenario: An unset global default is not persisted as a concrete model

- **GIVEN** no stored global default
- **AND** a non-empty process model catalog
- **WHEN** the global default is resolved
- **THEN** no concrete model choice is written to the global setting

> test: code

### + Scenario: Clearing the global default returns to follow-backend

- **GIVEN** a stored global default that is not the former built-in
- **WHEN** the global default setting is cleared
- **THEN** the stored global default is unset
- **AND** the resolved global default is follow-backend

> test: code
