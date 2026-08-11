# Chat expand and scroll keys

Wire transcript expand and scroll so chat-pane APIs are reachable in the product.

## Prerequisites

- [x] @step chat-pane

## Context

Review finding 3: `expand_segment` and `scroll_up` are unit-tested but unbound in the
event loop.

## Tasks

- [x] 1. Scroll keys when chat is focused (e.g. PgUp/PgDn; ↑↓ when they do not steal
         composer typing)

- [x] 2. Expand or collapse settled Thinking/Activity from the keyboard (focused segment
         or next collapsed)

- [x] 3. Keep streaming stick-to-bottom behavior; manual scroll still releases the pin
