# AGY agent harness - Design

Owned ACP adapter wraps headless `agy -p`; thin `AgyProvider` uses the shared ACP client
already on tree so Gemini turns share the Claude/Grok host stack with batch-only fidelity.

## Approach

```
duckboard  "agy" → AgyProvider
        │
        ▼
   duckchat worker
        │
        ▼
 AcpMainRuntime / AcpOneshotRuntime    ← crates/duckchat/src/acp/
        │  AgentLaunch → duckchat-agy-acp
        ▼
 duckchat-agy-acp   (new; mirror duckchat-claude-acp)
        │  per main prompt: agy -p --dangerously-skip-permissions
        │    [--conversation] [--model] --print-timeout 15m --log-file
        ▼
      agy CLI
```

**Rule (same as Claude):** the host never speaks AGY print-mode. Only `AgentLaunch` and
harness registration differ from Claude/Grok. AGY is not native ACP, so it gets an owned
adapter binary, not a third host wire client.

**Turn fidelity:** one final `agent_message_chunk` (full answer text) after `agy -p` exits
successfully. No tool_call, thought chunks, or usage telemetry on the wire. Timeouts and
failed resumes map to ACP errors / `SessionNotFound` as below.

**v1 scope:** main-chat turns only through the adapter. Titles use a local heuristic (no
`agy` spawn); reply suggestions stay empty. Oneshot ACP against AGY is deferred — each
`-p` is cold, multi-minute, and quota-costly, so it is a bad fit for title/reply chrome.

**Base:** uniform ACP is already on tree (`duckchat::acp`, `duckchat-claude-acp`,
`harness/acp-client`). This change extends that pattern; it does not reintroduce a pre-ACP
dual stack.

## `duckchat-agy-acp` agent

New workspace member and binary, structured like `crates/duckchat-claude-acp`.

### Session lifecycle

Copy the Claude agent session table shape: provisional ids, cold open/load, bind durable
id on first prompt.

```
| ACP method | Behavior |
| --- | --- |
| `initialize` | `protocolVersion: 1`, `loadSession: true`, static models in `_meta.modelState` |
| `session/new` | allocate provisional id; store `PendingOpen { cwd, model, resume: None }`; **do not** spawn `agy` |
| `session/load` | record `PendingOpen { resume: Some(uuid) }` for the given id; no spawn; unknown/empty → `SessionNotFound` RPC (`Path not found` / `FS_NOT_FOUND`, same as Claude agent) |
| `session/prompt` | spawn one cold `agy -p`; parse durable conversation uuid; emit one `agent_message_chunk`; return `stopReason: end_turn` and rebound `sessionId` when durable ≠ open id |
| cancel | kill in-flight `agy` child |
```

Unlike Claude, there is **no process-hot inner duplex**. Each prompt runs one `agy -p`
child to completion (or cancel/timeout).

### Spawn argv

```text
agy -p
  --dangerously-skip-permissions
  --print-timeout 15m
  --log-file <agent-owned tmp>
  [--model <label>]
  [--conversation <uuid>]
  <prompt text from ACP content blocks, text-only for v1>
```

Main turns always use **15m** print timeout (AGY’s default 5m is too short for tool-heavy
work). Oneshot AGY prompts are out of v1, so no separate oneshot timeout.

### Session rebind

1. Open may return a provisional id (or the load uuid).

2. During/after prompt, parse the log for
   `Print mode: conversation=<uuid>, sending message`.

3. Fallback after success: read `~/.gemini/antigravity-cli/cache/last_conversations.json`
   keyed by normalized cwd.

4. If durable id ≠ open id, put rebound `sessionId` on the prompt result (shared ACP
   client already surfaces rebound ids to duckboard).

5. Later turns pass the durable uuid as `--conversation`.

### Exit mapping

```
| Outcome | Agent response |
| --- | --- |
| exit 0 + stdout body | one `agent_message_chunk`, `end_turn` |
| timeout / empty stdout on resume | prefer `SessionNotFound` so host clears the id |
| other non-zero / auth/quota text | process error with stderr detail |
| `agy` missing on PATH | spawn error (same operator class as missing Claude agent) |
```

**Stdout discipline:** ACP JSON-RPC only on agent stdout; tracing and `agy` noise on
stderr or files.

### Sketch

```rust
// crates/duckchat-agy-acp/src/agent.rs
struct PendingOpen {
    cwd: PathBuf,
    model: Option<String>,
    resume: Option<String>, // None = fresh; Some(uuid) = --conversation
}

struct Agent {
    sessions: HashSet<String>,
    pending: HashMap<String, PendingOpen>,
    provisional_to_native: HashMap<String, String>,
    in_flight: Option<Child>, // cold -p only; no duplex heat
}

impl Agent {
    fn initialize(&self) -> Value { /* loadSession + models */ }
    async fn session_new(&mut self, params: &Value) -> Result<Value, AgentError>;
    async fn session_load(&mut self, params: &Value) -> Result<Value, AgentError>;
    async fn run_prompt(
        &mut self,
        params: &Value,
        on_update: &mut (dyn FnMut(Value) + Send),
    ) -> Result<Value, AgentError>;
    async fn cancel(&mut self, session_id: Option<&str>);
}
```

## Thin `AgyProvider`

```rust
// crates/duckchat/src/agy.rs  (+ agent_bin.rs for resolve/launch)
const HARNESS: &str = "agy";

pub struct AgyProvider {
    launch: AgentLaunch, // → duckchat-agy-acp (main path only in v1)
}

impl Provider for AgyProvider {
    fn id(&self) -> &str { HARNESS }

    fn capabilities(&self) -> Capabilities {
        Capabilities {
            streaming: false,   // one blob after -p, not progressive deltas
            tool_use: false,    // tools run inside agy but not on the wire
            resume: true,       // best-effort conversation uuid
            reasoning: false,
            slash_commands: false,
        }
    }

    fn list_models(&self) -> Vec<ModelInfo> { /* static display labels as id+display */ }
    fn list_commands(&self, _: &Path) -> Vec<SlashCommand> { vec![] }

    fn open_main_runtime(&self, wd: &Path) -> Box<dyn MainRuntime> {
        Box::new(AcpMainRuntime::new(self.launch.clone(), wd))
    }

    // Trait still requires a oneshot factory; v1 never spends AGY quota here.
    fn open_oneshot_runtime(&self, wd: &Path) -> Box<dyn OneshotRuntime> {
        // Prefer a no-op / local oneshot runtime if one exists for this pattern;
        // otherwise open AcpOneshotRuntime but do not call it from title/reply.
        …
    }

    async fn title_summary(&self, req: TitleRequest, _: &Path) -> Result<String, Error> {
        // Local only: shared clean_title / first-line heuristic — no agy spawn.
        Ok(clean_title(/* opening user text from req */))
    }

    async fn reply_suggestions(
        &self,
        _: ReplySuggestionRequest,
        _: &Path,
    ) -> Result<Vec<String>, Error> {
        Ok(vec![]) // no AGY oneshot in v1
    }
}
```

Binary discovery (clone `claude_code/agent_bin.rs`):

```text
DUCKCHAT_AGY_ACP → sibling of current_exe named duckchat-agy-acp → PATH
```

Models: static list of current `agy models` display strings as both `id` and `display`
(what `--model` accepts). No context windows in v1 (`None`).

## duckboard registration

```rust
// crates/duckboard/src/agent.rs
enum Harness {
    ClaudeCode,
    Grok,
    Agy,
}

fn dispatch(harness: &str) -> Harness {
    match harness {
        "grok" => Harness::Grok,
        "agy" => Harness::Agy,
        _ => Harness::ClaudeCode,
    }
}

// available_models: + AgyProvider::new().list_models()
// agent_stream: Harness::Agy => drive_provider(AgyProvider::new(), …)
```

Persistence already has `agent_session_id` + `session_harness`. AGY uuids store under
harness `"agy"`. No chat-store schema change. Picker groups by `ModelInfo.harness`.

## Packaging / build

```
| Item | Change |
| --- | --- |
| `Cargo.toml` workspace | member `crates/duckchat-agy-acp` |
| `justfile` / release | build and copy `duckchat-agy-acp` next to `duckboard` (extend Claude agent recipe) |
| Operator docs | install `agy` on PATH; ship/agent sibling binary for the adapter |
```

## Impact

- New crate/binary `duckchat-agy-acp`

- New module `duckchat::agy` (+ `lib.rs` export)

- duckboard harness enum, dispatch, and model aggregation

- Specs: new harness capability; light selection / warm-runtime wording only if wording
  assumes exactly two harnesses

- Shared ACP client dialect unchanged unless a SessionNotFound edge case forces a tweak

- No Cursor or Codex work in this change

## Decisions

- **ACP adapter, not host `-p` client** — keeps the uniform ACP host. Alternatives:
  host-side cold AGY provider (rejected: dual stack); `agentapi` / sidecar HTTP (rejected:
  requires live interactive language server).

- **Cold `agy -p` per prompt** — only supported headless API. Alternative: long-lived
  interactive process (rejected: TTY / bubbletea, not scriptable).

- **Honest capabilities** (`streaming` / `tool_use` false) — match the CLI. Alternative:
  progressive tools via conversation DB scrape (rejected for v1; proposal non-goal).

- **Session id from log + `last_conversations` fallback** — only durable sources observed.
  Alternative: require id on stdout (unavailable).

- **Static model list** — avoids spawn-on-picker. Alternative: shell `agy models` each
  list (rejected: slow/brittle).

- **No slash-command discovery for AGY** — AGY does not load `.claude/commands` like Grok.

- **Mirror Claude provisional open/load** — reuses client rebind and SessionNotFound paths
  already proven with `duckchat-claude-acp`.

- **Main `--print-timeout` = 15m** — probes showed AGY’s default 5m often times out while
  tools still run. Alternatives: keep 5m (rejected: empty stdout / false failures); make
  timeout user-configurable in v1 (deferred).

- **Main turns only in v1 (no AGY oneshot title/reply)** — each `-p` is a cold, slow,
  quota-bearing agent run; using it for title/reply chrome is worse UX than a local title
  heuristic and empty reply suggestions. Alternatives: full oneshot ACP like Claude
  (rejected for v1 cost/latency); short 2m oneshot (rejected: still burns quota and often
  still incomplete). Later work may add cheap oneshot if AGY gains a faster headless path.

## Risks

- **`-p` timeouts / hung idle poll** → fixed 15m main timeout; map timeout on resume to
  clearable `SessionNotFound`

- **Resume creates a new conversation or times out** → clear stored id; next send is fresh

- **Auth / quota failures** → surface as turn error; no special auth UI

- **Missing adapter binary** → same operator class as missing `duckchat-claude-acp`

- **Missing or broken `agy` CLI** → spawn/process error with actionable message

- **Empty reply-suggest chrome on AGY chats** → accepted v1 trade-off; obvious-bubble /
  next cards still work from transcript templates
