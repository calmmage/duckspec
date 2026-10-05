# @ Grok harness

## @ Requirement: Model discovery

Listing models SHALL return grok's available models, each tagged with the grok harness,
carrying a human-readable display name, and carrying its own context window. The harness
advertised default SHALL be the handshake `currentModelId` when that id is among the
available models. Title-summary and reply-suggestion oneshots SHALL select the preferred
oneshot model for the grok harness when that model is among the available models, and
SHALL fall back to another available model when the preferred model is absent.

> test: code

### + Scenario: Advertised default is the handshake current model when that id is listed

- **GIVEN** a grok handshake advertising available models
- **AND** a `currentModelId` that is among those models
- **WHEN** the harness advertised default is read
- **THEN** the advertised default is that `currentModelId`

> test: code
