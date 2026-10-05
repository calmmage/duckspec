# Local install

Developer install from this repo refreshes cargo bins and the macOS Applications app from
the current tree so Dock and Finder launch the build that was just installed.

## What it does

From the project root:

```
just install
  ├─ just build-release  →  target/release/{ds,duckboard,duckchat-*-acp}
  ├─ install bins        →  ~/.cargo/bin
  ├─ just bundle-app     →  dist/Duckboard.app  (same binaries, no rebuild)
  └─ copy                →  /Applications/Duckboard.app
```

One workspace release build feeds both PATH binaries and the app bundle. Harness agents
installed: `duckchat-claude-acp`, `duckchat-codex-acp`, `duckchat-agy-acp`.

Terminal `duckboard` on PATH and the Dock/Finder app both come from the tree you just
built. The recipe does not quit or relaunch a running process — quit first, install, then
reopen Applications (or the cargo binary if that is your launch path).

## When to use

After landing dogfood UI changes in this repo, before judging them in the daily-driver
app. The Update plaque’s copyable command is this same entrypoint.
