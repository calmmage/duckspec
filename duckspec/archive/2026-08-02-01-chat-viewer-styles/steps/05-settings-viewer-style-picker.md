# Settings viewer style picker

Expose Classic and Focus in Settings; keep Document hidden.

## Prerequisites

- [x] @step focus-presentation-and-folds

## Tasks

- [x] 1. Settings Chat pick list from implemented styles (classic + focus)

- [x] 2. @spec chat/viewer-style Settings choices: Settings lists classic and focus when Focus is implemented

- [x] 3. @spec chat/viewer-style Settings choices: Document is not offered while unimplemented

- [x] 4. Manual smoke: Settings change restyles settled Answers without restart

## Outcomes

- Manual smoke: view path uses `config.chat.effective_viewer_style()` each frame and
  stamps sessions on interaction update — changing Settings saves config and the next
  paint/restyle applies without process restart.
