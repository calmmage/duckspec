# CI warm-build includes duckchat-agy-acp

Include the AGY ACP agent in the release workflow warm-build so CI cache and comments
match the multi-agent bundle.

## Prerequisites

- [x] @step provider-duckboard-registration-and-packaging

## Context

Review finding 3: `.github/workflows/release.yml` only warms `duckchat-claude-acp`. The
DMG path already builds AGY via `just bundle`; this step only aligns the warm-build.

## Tasks

- [x] 1. Add `-p duckchat-agy-acp` to the release workflow “Build release binaries” step

- [x] 2. Update the adjacent comment so it names both ACP agents (Claude + AGY)
