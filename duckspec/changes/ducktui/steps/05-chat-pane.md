# Chat pane

Terminal transcript (collapse, autoscroll), numbered next and fast-response hints,
composer Enter/Alt+Enter, slash palette open and `//` pass-through; drive turns via
duckcore agent.

## Prerequisites

- [x] @step navigator
- [x] @step session-sharing

## Tasks

- [x] 1. Render Thinking and Activity collapsed (expandable) and Answer always expanded
         from the shared segment model

- [x] 2. Stick-to-bottom pin while streaming; release on manual scroll up

- [x] 3. Numbered 1–9 hints for trailing `next` tokens and fast-response options;
         number-key activation

- [x] 4. Composer: Enter sends, Alt+Enter inserts newline; `/` opens slash palette; `//`
         does not open as a single-slash catalog

- [x] 5. Wire submit path to the duckcore agent turn driver and session drive/write policy

- [x] 6. @spec tui/chat Settled segment collapse: Settled Thinking shows one summary line until expanded

- [x] 7. @spec tui/chat Settled segment collapse: Settled Activity shows one summary line until expanded

- [x] 8. @spec tui/chat Settled segment collapse: Answer body is fully visible without expand

- [x] 9. @spec tui/chat Autoscroll pin during stream: Streaming with stick engaged keeps latest lines in view

- [x] 10. @spec tui/chat Autoscroll pin during stream: Manual scroll up releases the pin

- [x] 11. @spec tui/chat Numbered action hints: Trailing next tokens appear as numbered hints

- [x] 12. @spec tui/chat Numbered action hints: Number key activates the matching next token

- [x] 13. @spec tui/chat Numbered action hints: Fast-response options appear as numbered hints while awaiting

- [x] 14. @spec tui/chat Numbered action hints: Number key activates the matching fast-response option

- [x] 15. @spec tui/chat Composer send and newline: Enter with non-empty composer submits

- [x] 16. @spec tui/chat Composer send and newline: Alt+Enter inserts a newline without submitting

- [x] 17. @spec tui/chat Slash palette open: Slash at line start opens the palette

- [x] 18. @spec tui/chat Slash palette open: Double-slash does not open as a single-slash catalog
