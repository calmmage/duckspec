# Live models

New model releases appear in the picker and become the default from the backends, without
another duckboard change that names those models.

Grok, Claude, and Codex already advertise live catalogs. AGY does not: its picker is a
frozen table copied in two places, so Gemini 3.7 (and 3.6) stay invisible until someone
edits Rust. The global default has the same problem in a different form. Unset installs
are seeded to a hardcoded `grok` / `grok-4.5`, then that value is saved as if the user
chose it. Grok 4.6 is already the CLI default and already in the handshake
(`currentModelId`); the app ignores that signal and keeps 4.5 forever.

```
today                         wanted
─────                         ──────
AGY: static table        ──►  AGY: live `agy models` (table only as fallback)
default: hardcoded id    ──►  default: harness-advertised (Grok currentModelId)
seed persisted as pin    ──►  follow-backend until the user actually picks
```

A saved global default that still equals the former built-in is treated as follow-backend,
because that value was auto-seeded. Any other saved choice stays a pin. Clearing the
global setting returns to follow-backend.

This change does not add `grok-4.6` or Gemini 3.7 as source ids. It does not invent a new
discovery stack for harnesses that already advertise live. It does not redesign picker
chrome, oneshot resolution, or the pin → project → global cascade.
