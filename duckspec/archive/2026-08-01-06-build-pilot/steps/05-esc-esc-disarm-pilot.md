# Esc-Esc disarm pilot

Esc-Esc disarms an armed pilot; while streaming it still cancels the turn, and while idle
it only disarms.

## Prerequisites

- [x] @step pilot-state-module-and-reactivate-setting

## Tasks

- [x] 1. Extend the Esc-Esc handler to clear `ax.pilot` when armed

- [x] 2. @spec chat/build-pilot Disarm controls: Esc-Esc while streaming cancels turn and disarms

- [x] 3. @spec chat/build-pilot Disarm controls: Esc-Esc while idle and armed only disarms
