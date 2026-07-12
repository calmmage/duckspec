# Post-implementation review: Stale build plaque

Reviewed the full chain for `stale-build-plaque` after apply. Behavior matches the
proposal and design; pure evaluation is well-tested and the shell wiring stays read-only
(no auto-install).

## Scope

- `proposal.md`, `design.md`, `caps/shell/stale-build/{spec,doc}.md`, both steps

- Code: `crates/duckboard/src/self_version.rs`, `widget/status_bar.rs`,
  `widget/stale_build.rs`, `main.rs` state/message/view wiring

- `ds check`, `ds audit stale-build-plaque`, `cargo test -p duckboard self_version` (7
  passed)

## Summary

```
| # | sev | lens | title | → next |
| --- | --- | --- | --- | --- |
| — | — | — | No findings | `/ds-archive` |
```

## Verdict

Accept. Ready to archive when you want it on mainline: self-detection and ahead- only
comparison fail closed, install is copy-only, and the status-bar Update chip plus recipe
panel realize the safer dogfood signal without process control.
