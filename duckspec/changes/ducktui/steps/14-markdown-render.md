# Markdown render

Implement design `md_render` and feed Answer / expanded bodies through it.

## Prerequisites

- [x] @step chat-pane
- [x] @step extract-transcript-segments

## Context

Review finding 4: design and `tui/chat` doc require pulldown-cmark styled lines and table
fit-to-width; presentation is plain strings today.

## Tasks

- [x] 1. Add ducktui `md_render` (pulldown-cmark → owned styled lines)

- [x] 2. Tables fit pane width; narrow panes degrade to plain cell wrap

- [x] 3. Wire Answer and expanded Thinking/Activity bodies through the renderer in the
         draw path

- [x] 4. Meta-card chrome as bordered blocks where the design expects it (beyond numbered
         next hints only)
