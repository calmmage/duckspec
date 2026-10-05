# Live models - Design

AGY lists models from `agy models` (curated table only as fallback). The global default
follows a harness-advertised id instead of a hardcoded `grok-4.5`, and an unset or
former-built-in saved value is follow-backend rather than a persisted pin.

## Approach

```
app start
   │
   ▼
refresh each provider.list_models
   │  Grok/Claude/Codex: ACP initialize (unchanged)
   │  AGY: spawn `agy models`, parse id<TAB>name
   │       empty/fail → curated fallback table
   ▼
catalog + per-harness advertised default
   │  Grok: modelState.currentModelId (in catalog)
   │  AGY: first discovered row
   │  others: unused for app-wide follow
   ▼
resolve global default
   stored None or former built-in (grok / grok-4.5)
        → follow grok advertised default, else first catalog model
   any other stored ModelRef
        → pin (unchanged cascade)
```

Picker, oneshot resolution, and pin → project → global order stay as they are. Follow
replaces only the compile-time seed id and the persist-seed step.

## AGY live discovery

`AgyProvider::list_models` is the picker source (process catalog refresh already calls
it). It no longer returns a static `AGY_MODELS` slice as the success path.

```
agy models stdout
  "Fetching available models..."     skip
  "<id>\t<display name>"             one model
  other / malformed line             skip
```

Each listed model uses the **display name** as both `ModelInfo.id` and `display`. That is
the current `--model` contract (`Gemini 3.5 Flash (High)`), so existing AGY pins keep
matching. The machine id column is not stored.

Discovery is memoized on the shared `AgyProvider` (same OnceLock pattern as Grok). Empty
output, spawn failure, or a missing binary yields the existing curated table — the same
names already shipped — so a machine without `agy` still shows a non-empty AGY slice
rather than wiping the harness.

`agy models` is spawned with the same login-shell wrap / `DUCKCHAT_AGY_BIN` override as
print-mode (`duckchat-agy-acp` argv prefix). Catalog refresh already runs off the UI
thread.

`duckchat-agy-acp` `initialize` advertise is not the host catalog. Leave it on the curated
table; session/new still receives the host-selected display-name id and passes it to
`--model`.

Rejected: switching advertised ids to `gemini-3.7-flash-high` (breaks saved AGY pins).
Rejected: host ACP handshake for AGY list_models (heavier than `agy models`, and the agent
does not discover live today). Rejected: another hand-edit that names Gemini 3.7.

## Advertised default

ACP `initialize` already nests `modelState.currentModelId`. Parse it onto `InitResult` and
keep it next to `models`. Grok caches that id with its handshake memo.

```rust
pub struct InitResult {
    pub load_session: bool,
    pub models: Vec<AcpModel>,
    pub current_model_id: Option<String>,
}

impl Provider {
    fn advertised_default(&self) -> Option<String> { None }
}
```

```
| Harness | `advertised_default` |
| --- | --- |
| Grok | handshake `currentModelId` when that id is in the advertised list |
| AGY | first `list_models` row |
| Claude / Codex | unused for app-wide follow |
```

`ModelCatalog` stores the per-harness advertised id beside each slice (written at
`apply_harness` / refresh from `Provider::advertised_default`). Follow resolution reads
Grok’s stored id, then falls back to the first catalog model — same order as today’s seed,
without naming `grok-4.5`.

```rust
pub fn follow_backend_default(
    catalog: &[ModelInfo],
    grok_default: Option<&str>,
) -> Option<ModelRef>
```

## Follow-backend vs pin

`config.default_model: Option<ModelRef>` stays. Meaning of `None` changes: it is
follow-backend, not “unconfigured / Missing.”

```
is_follow_backend(stored)
  None                              → yes
  Some(grok / grok-4.5)             → yes   (former auto-seed)
  any other Some(_)                 → no    (real pin)
```

Resolve:

```
preferred = pin || project || resolved_global
resolved_global =
    follow?  follow_backend_default(catalog, grok advertised)
    else     stored pin
```

Do **not** persist a concrete seed on catalog-ready, refresh, or Reset. A follow install
stays `None` on disk. A saved `grok` / `grok-4.5` is treated as follow at resolve time and
is not rewritten.

Settings global picker gains a leading “Follow backend default” sentinel (`id`/`harness`
`None`) that writes `None`. Closed label names the resolved follow model when one exists.
Choosing a catalog row still writes a concrete pin. Clearing the setting returns to
follow.

Send / Missing rules stay: if the resolved preferred model is absent from the catalog, the
turn is unavailable. Follow with an empty catalog is still Missing.

`FORMER_BUILTIN_*` remain only as the stale-seed detector, not as the preferred seed id.
ducktui uses the same duckcore resolve helper; it must not persist a seed either.

## Compatibility

- Existing non-`grok-4.5` global defaults, project overrides, and per-chat pins are
  unchanged

- Auto-seeded `grok-4.5` starts following Grok’s advertised default (4.6 on current CLIs)
  without a config rewrite

- AGY pins that use display names keep matching; new Gemini rows appear from `agy
  models`
  on the next catalog refresh

- No new source ids for `grok-4.6` or Gemini 3.7
