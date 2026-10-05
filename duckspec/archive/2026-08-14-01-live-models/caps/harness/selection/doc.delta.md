# @ Harness selection

## ~ Default resolution

The preferred model for a turn is resolved from a three-step cascade, most specific first:

```
per-chat pin  →  project override  →  resolved global default
```

The first level that is set wins. A per-chat pin overrides a project override; a project
override overrides the resolved global default. Clearing the project override means “use
global,” not “let the CLI pick.”

The resolved global default is follow-backend when the setting is unset, or when the
stored value is still the former built-in (`grok` / `grok-4.5`). Follow-backend prefers
the Grok advertised default when that model is in the process catalog, otherwise the first
model in catalog order. Any other stored global value is a pin.

The preferred model is **available** only when it appears in the process model catalog.
When follow-backend yields no model, or the preferred model is absent from the catalog,
the turn has no available model: the chat model control shows **Missing**, and send is
blocked until the user picks a catalog model (or availability returns). The app does not
invent a substitute.

The global default setting is optional. A concrete choice is stored only when the user
picks one. Follow-backend is not written as a model id. Clearing the setting returns to
follow-backend.
