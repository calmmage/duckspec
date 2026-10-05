# Follow-backend global default

Resolve an unset or former-built-in global default from the Grok advertised default (else
first catalog model), without persisting a seed. Wire the cascade, Settings clear, and
ducktui through the same helper.

## Prerequisites

- [x] @step discover-agy-models-live

## Tasks

- [x] 1. Add `follow_backend_default` / `is_follow_backend` and store Grok's advertised id
         on the catalog

- [x] 2. Stop persisting a concrete seed on catalog-ready, refresh, and Settings reset

- [x] 3. Resolve the cascade global slot through follow-backend; add a Follow sentinel on
         the Settings global picker

- [x] 4. Use the same resolve helper from ducktui instead of treating unset as empty

- [x] 5. @spec harness/selection Default model resolution: Unset global follows the Grok advertised default when that model is in the catalog

- [x] 6. @spec harness/selection Default model resolution: Unset global follows the first catalog model when the Grok advertised default is absent

- [x] 7. @spec harness/selection Default model resolution: A former built-in stored global follows the advertised default

- [x] 8. @spec harness/selection Default model resolution: Follow-backend with an empty catalog is not available

- [x] 9. @spec harness/selection Global default model setting: An unset global default is not persisted as a concrete model

- [x] 10. @spec harness/selection Global default model setting: Clearing the global default returns to follow-backend
