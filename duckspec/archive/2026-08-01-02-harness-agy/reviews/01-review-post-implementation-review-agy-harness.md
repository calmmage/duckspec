# Post-implementation review: AGY harness

Reviewed `harness-agy` end-to-end (proposal → design → cap → steps → code). Architecture
matches the ACP adapter plan and audit is clean; two production-path soundness issues
should be fixed before accepting as done.

## Scope

`duckspec/changes/harness-agy/{proposal,design,caps,steps}`, `crates/duckchat/src/agy*`,
`crates/duckchat-agy-acp/**`, duckboard `agent.rs` dispatch, and `justfile` packaging.
Post-implementation; `ds audit harness-agy` is clean.

## Summary

```
| # | sev | lens | title | → next |
| --- | --- | --- | --- | --- |
| 1 | major | soundness | Inner `agy` spawn skips login-shell wrap | /ds-step |
| 2 | major | soundness | Missing durable id falls back to provisional | /ds-step |
| 3 | minor | quality | CI warm-build omits `duckchat-agy-acp` | /ds-step |
| 4 | minor | quality | Print log files never cleaned up | ignore |
```

## Findings

### 1. Inner `agy` spawn skips login-shell wrap - soundness/major

**Where:** `crates/duckchat-agy-acp/src/print.rs:28-49` (`default_spawn_factory`)

**Why:** Claude and Grok launch through `SHELL -ilc 'exec "$@"'` so per-directory tool
managers and a full login `PATH` apply — critical for Finder-launched Duckboard, which
inherits a skeletal GUI environment. The AGY agent binary itself is sibling-discovered,
but the **inner** backend is `Command::new("agy")` with no shell wrap. Users with `agy`
only on an interactive shell PATH will get hard spawn failures for AGY turns even when
Claude/Grok work from the same app build.

**Action:** Wrap production `agy` argv like Claude’s `claude_argv_prefix` (login shell +
`exec "$@"`), with a test override path so scripted peers keep working without a shell.

### 2. Missing durable id falls back to provisional - soundness/major

**Where:** `crates/duckchat-agy-acp/src/agent.rs:258-261`

**Why:** After a successful print turn, durable conversation id is taken from the log
parse or `last_conversations.json`, else **the provisional open id** (`pending-*`). The
host persists that as `agent_session_id`. The next turn then passes
`--conversation pending-…`, which is not an AGY conversation uuid. Spec/design require a
durable AGY conversation id for resume; this edge case poisons resume after an otherwise
successful first turn (e.g. log path unreadable, cache miss, or log format drift).

**Action:** On success without a recovered uuid, fail the turn or omit rebind rather than
treating the provisional handle as durable; add a regression test for “success but no
conversation id in log/cache.”

### 3. CI warm-build omits `duckchat-agy-acp` - quality/minor

**Where:** `.github/workflows/release.yml:64`

**Why:** The release warm-build step only compiles `duckchat-claude-acp`. The DMG path
still builds AGY via `just bundle` (which was updated), so shipping is likely fine; the
cache step is incomplete and comments still describe a single agent.

**Action:** Add `-p duckchat-agy-acp` to the release warm-build command and update the
comment.

### 4. Print log files never cleaned up - quality/minor

**Where:** `crates/duckchat-agy-acp/src/agent.rs` log files under `log_dir` / temp

**Why:** Each prompt writes a `duckchat-agy-*.log` and never removes it. Slow litter under
long use; not a functional break for v1.

**Action:** Optional best-effort delete after parse; acceptable to ignore for v1.

## Verdict

Not archive-ready. The ACP-adapter shape, duckboard registration, main-only oneshots, and
scenario coverage are in good shape. Fix login-shell spawn for `agy` and durable-id
handling before treating the change as shippable; items 3–4 are small follow-through.
