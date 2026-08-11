# Build pilot - Design

Client-side session pilot: System `/build-auto` / `/build-fast` arm ephemeral mode on the
chat session, kick a rewritten `/ds-*` user turn, and on each `TurnComplete` auto-send
safe rank-1 trailing `next` tokens (or fully disarm).

## Approach

```
Composer submit
      │
      ▼
parse_submit_slash  ──► /help → LocalHelp (unchanged)
      │
      ├─ /build-auto|fast [args] → LocalBuildPilot { mode, args }
      │         │
      │         ├─ arm ax.pilot = Auto|Fast  (ephemeral)
      │         ├─ plaque = mode
      │         └─ kick = format!("/{lifecycle_head}{args}")
      │                   send_agent_turn(display=kick, prompt=kick)
      │
      └─ else → Agent { … }

TurnComplete
      │
      ├─ refresh_next_actions(true)
      ├─ if pilot == Off → done
      └─ match pilot_decision(mode, next_actions[0]):
            AutoSend(text) → send_agent_turn(text, text)
            Disarm         → pilot = Off
```

Boundaries:

- **duckboard-only.** No template/agent skill for `/build-*`. No meta-card syntax change.

- **Authority for “what next”** remains trailing `next` (and empty-session lifecycle only
  for the *kick*, not for post-first-turn auto-send).

- **Pilot state is process-local** on `AgentSession` (same object survives
  exploration→change promotion). Not written into the session file.

- **Setting** `chat.pilot_reactivate_on_error` is real config + Settings UI, default off;
  **reactivation behavior is stubbed** (setting is ignored at runtime until a follow-up
  wires crash/restart recovery).

## System slash parse and registry

Extend `crates/duckboard/src/slash_commands.rs`.

```rust
pub enum BuildPilotMode { Auto, Fast }

pub enum SubmitSlash {
    LocalHelp,
    /// Arm pilot and immediately kick an agent turn (rewritten /ds-*).
    LocalBuildPilot { mode: BuildPilotMode, args: String },
    Agent { display: String, prompt: String },
}

// system_registry: help, build-auto, build-fast (kind System)
// parse: leading /build-auto|/build-fast as first token;
//        remainder (trimmed) is args (may be empty).
// Not limited to is_bare_slash_command (args contain whitespace).
```

`dispatch_user_submit`:

```rust
LocalBuildPilot { mode, args } => {
    match kick_command(ax) {
        Some(head) => {
            ax.pilot = PilotState::Armed(mode);
            let text = join_slash_args(head, &args); // "/ds-explore foo"
            send_agent_turn(ax, text.clone(), text, highlighter);
        }
        None => {
            // caps/codex/archived: optional system nack; pilot stays Off
            …
        }
    }
}
```

`agent_prompt_for_recovery`: `LocalBuildPilot` never appears as stored user text (kick
stores `/ds-…`); recovery stays Agent-only / LocalHelp.

## Pilot state and policy

New small module `crates/duckboard/src/build_pilot.rs` (pure policy + helpers).

```rust
pub enum PilotState {
    Off,
    Armed(BuildPilotMode),
}

/// After TurnComplete, given mode + ordered next actions (already from meta card).
pub enum PilotDecision {
    AutoSend(String),
    Disarm,
}

pub fn decide(mode: BuildPilotMode, next: &[NextAction]) -> PilotDecision { … }

pub fn is_auto_safe(mode: BuildPilotMode, send: &str) -> bool {
    // confirm → true (both modes)
    // /ds-archive, /ds-codex, /ds-verify → false
    // /ds-review → true only for Auto
    // other allowlisted /ds-* stages → true
    // reject, revise, freeform, empty → false
}
```

Allowlist (exact bare send text after trim; leading `/` normalized):

```
| Token | Auto | Fast |
| --- | --- | --- |
| `confirm` | yes | yes |
| `/ds-explore` … `/ds-propose` `/ds-design` `/ds-spec` `/ds-step` `/ds-apply` `/ds-followup` | yes | yes |
| `/ds-review` | yes | **no** |
| `/ds-archive` `/ds-codex` `/ds-verify` | no | no |
| `reject` `revise` other | no | no |
```

**Fast “join design and spec”:** no special prompt. When rank-1 is `/ds-design`, auto-send
it; when later rank-1 is `/ds-spec` (or `confirm` then `/ds-spec`), auto-send those.
Client-side stitch only.

**Disarm = full off** until a new `/build-*` submit. Human turns while off do not re-arm.

**Esc-Esc** (`interaction` key path ~3341): if `pilot != Off`, set `Off` in addition to
cancel-when-streaming. Idle + armed → disarm only.

## Kick target

```rust
fn kick_command(ax: &AgentSession) -> Option<&'static str /* or String */> {
    match ax.scope_kind {
        ScopeKind::Exploration => Some("ds-explore"),
        ScopeKind::Change => ax.scope_facts.as_ref()?.next_command.as_deref(),
        ScopeKind::Caps | ScopeKind::Codex => None,
    }
}
// join: "/{head}" + optional " " + args
```

Reuse `lifecycle_bootstrap` / `ChangeScopeFacts::next_command` — same ladder as
empty-composer bootstrap.

## TurnComplete hook

In `main.rs` `AgentEvent::TurnComplete` after `refresh_next_actions(true)` (and after
priming short-circuit so priming does not auto-fire pilot):

```rust
if let Some(text) = build_pilot::maybe_auto_send(&mut ax) {
    // schedule same path as empty-Enter / SendObviousAction
    interaction::send_agent_turn(…);
}
```

Guardrails:

- Do not auto-send while still streaming (TurnComplete already cleared it).

- Do not nest if kick path is mid-priming (`was_priming` → skip pilot decide this turn, or
  only decide after user kick turn completes — prefer: pilot only acts on non-priming
  TurnComplete).

- Auto-send uses `send_agent_turn` with display == prompt == token (or full line from
  card).

If decision is `Disarm`, clear plaque; no system message required (plaque absence is the
signal). Optional quiet system line is a product polish — default **no** extra system
spam.

## Mode plaque (UI)

Above composer (same column as input / next-action ghost), when `pilot != Off`:

- Label: `Build auto` / `Build fast` (short, quiet chrome — peer to footer/resend style,
  not a transcript bubble).

- Not persisted; derived only from `ax.pilot`.

Touch points: `widget/agent_chat.rs` (or interaction view that builds the input stack) +
`AgentSession` field.

## Exploration → change

`promote_exploration` moves the same `AgentSession` values into the change scope
(`area/change.rs` ~352–365). Putting `pilot: PilotState` on `AgentSession` means promotion
**keeps** arm with zero extra migration — verify with a unit/integration assertion that
field is not reset on promote/merge.

`merge_sessions` path: ensure pilot fields on folded sessions are preserved (no `Default`
wipe).

## Config / Settings

```rust
// config.rs ChatConfig
pub struct ChatConfig {
    pub agent_input_hints: bool,
    /// When true, future: re-arm pilot after crash/restart/error. Default false.
    /// v1: loaded/saved and shown in Settings; runtime ignore.
    pub pilot_reactivate_on_error: bool,
}
```

Settings checkbox under chat affordances; `config.toml` key `pilot_reactivate_on_error`.
Docstring / Settings helper text notes “not active yet.”

## Capabilities (spec later)

```
| Cap | Role |
| --- | --- |
| **New** `chat/build-pilot` | Arm/disarm, decide/allowlist, kick rewrite, TurnComplete auto-send, plaque, Esc-Esc, promotion continuity, stub setting |
| **Delta** `chat/slash-commands` | Registry entries `build-auto` / `build-fast`; parse with optional args; submit routes to LocalBuildPilot (not bare-only) |
```

No change to `chat/meta-cards` syntax. `default-prompts` list building unchanged; pilot
*consumes* `next_actions` after refresh.

## Impact

- `slash_commands.rs`: `SubmitSlash` variant, registry, parse, tests
- `dispatch_user_submit` / Esc-Esc / TurnComplete wiring
- New `build_pilot.rs` + `AgentSession.pilot`
- Composer plaque UI
- `ChatConfig` + Settings + config tests
- Specs: new cap + slash-commands delta
- Help catalog automatically lists new System names

## Decisions

- **System `/build-*`, not `/ds-*`** — avoids Workflow kind collision; local arm is
  duckboard product control.

- **Ephemeral pilot only** — simpler; setting reserves future reactivate without v1
  session-file schema.

- **Disarm on unsafe next, not pause-and-resume** — matches “full off until re-launch.”

- **Token allowlist, not keywords** — same authority as empty Enter.

- **Fast = policy filter + normal multi-turn**, not a mega-prompt for design+spec.

- **No auto system chat noise on disarm** — plaque off is enough (can revisit in polish).

## Risks

- **Agent omits trailing `next`** → pilot disarms mid-run → user must re-`/build-*`.
  Mitigation: templates already require cards; missing card is a template/agent bug.

- **Rank-1 is wrong but allowlisted** → auto-sends a bad hop. Mitigation: only exact known
  tokens; never freeform rank-1.

- **Double-send races** if TurnComplete fires twice. Mitigation: arm only when
  `!is_streaming` and single-threaded iced update path; ignore if already streaming after
  schedule.

- **Promotion merge** into existing change scope could drop pilot if merge is naïve.
  Mitigation: copy `pilot` on session merge explicitly in tests.
