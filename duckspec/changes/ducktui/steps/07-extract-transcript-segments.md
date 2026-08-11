# Extract transcript segments

Move UI-neutral transcript segment construction into duckcore so duckboard and ducktui
share one Thinking / Activity / Answer model.

## Prerequisites

- [x] @step extract-duckcore

## Context

Review finding 2 and design: duckcore should expose the shared segment model from
`chat/transcript`. Today `TranscriptSeg` / `build_transcript_segments` live in
`crates/duckboard/src/widget/agent_chat.rs`; ducktui uses a parallel `PresentSeg`.

## Tasks

- [x] 1. Move segment construction (and collapse-default helpers as needed) from
         `agent_chat.rs` into duckcore

- [x] 2. Rewire duckboard to the shared builder; keep iced presentation local

- [x] 3. Feed ducktui `ChatPane` from real session segments instead of only injected
         presentation segments

- [x] 4. Keep existing `chat/transcript` and `tui/chat` tests green; update `@spec`
         backlink module paths if tests move with the builder
