# AGY harness

Duckboard drives Antigravity CLI (`agy`) through the shared ACP client and an owned
workspace agent binary. The agent wraps headless `agy` print mode; the host never speaks
print-mode itself and never depends on the interactive AGY TUI, language server, or
sidecar HTTP APIs.

## Process tree

```
duckboard / duckchat worker
   │  ACP (shared client)
   ▼
duckchat-agy-acp             (owned agent)
   │  cold print per main prompt
   ▼
agy -p                       (headless CLI)
```

Selecting the AGY harness only changes the provider launch (the agent binary). Turn
lifecycle for the **agent** process and profile event mapping are the shared ACP client.
This capability owns AGY-specific behavior: agent binary discovery, when the inner `agy`
process starts, durable conversation ids after the first prompt, cold print per main turn
with tool auto-approve, batch final-answer emission, and main-only title/reply handling.

## Session ids

Opening a new AGY conversation does not start `agy`. The open step may use a short-lived
ACP handle; the print process starts when the first user prompt is submitted. Completing
that turn surfaces a durable AGY conversation id — that is the id the host persists for
resume. A later turn with that id resumes the same conversation. A missing or unusable
conversation surfaces through the shared client's session-not-found path so the host can
clear the stored id and open fresh.

## Cold print turns

Each main prompt runs one cold `agy -p` child to completion (or cancel/timeout). There is
no process-hot inner AGY duplex. Tool permission prompts are auto-approved so a headless
host never waits on AGY UI. Auth, tools, and model work stay inside the AGY CLI.

## Batch final answer

A successful turn delivers the assistant's final printed answer as profile content the
shared client maps to the transcript answer channel. Progressive tool rows, reasoning
chunks, and usage telemetry are not required on the wire even when AGY used tools
internally.

## Title and reply suggestions

Title summaries for AGY chats use a local heuristic and do not spawn `agy`. Reply
suggestions return empty without spawning `agy`. Main-chat turns remain the only path that
spends AGY runtime and quota through the adapter.

## Agent binary discovery

```
1. DUCKCHAT_AGY_ACP (explicit override)
2. sibling of the running executable
3. PATH
```

Local builds place `duckchat-agy-acp` next to `duckboard` under `target/`. If no binary
can be launched, an AGY turn fails with a typed error — the same operator class as a
missing Claude or Grok agent binary.

## Models

The harness offers a curated list of AGY model labels tagged with the AGY harness
identity. Those labels are what the backend accepts as model selection for print mode.

## Backend boundary

The agent translates protocols only. Tool execution, auth, skills, and AGY behavior stay
inside the official `agy` CLI. The harness does not reimplement Antigravity over a private
API and does not attach to a live interactive AGY session.
