# Local install

Developer install from this repo refreshes cargo bins and the macOS Applications app from
the current tree so Dock and Finder launch the build that was just installed.

## What it does

From the project root:

```
just install
  ├─ cargo install  →  ~/.cargo/bin  (ds, duckboard, duckchat-claude-acp)
  ├─ just bundle    →  dist/Duckboard.app
  └─ copy           →  /Applications/Duckboard.app
```

Terminal `duckboard` on PATH and the Dock/Finder app both come from the tree you just
built. The recipe does not quit or relaunch a running process — quit first, install, then
reopen Applications (or the cargo binary if that is your launch path).

## When to use

After landing dogfood UI changes in this repo, before judging them in the daily-driver
app. The Update plaque’s copyable command is this same entrypoint.
