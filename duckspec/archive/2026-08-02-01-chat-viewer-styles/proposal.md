# Chat viewer styles

Duckboard gets a user-selectable Answer viewer style: keep the current path as the default
Classic style, add a Focus style that collapses long replies around the trailing meta
gate, and leave room for a later Document (rich markdown) style on the same switch.

The chat Answer surface is already a solid source-faithful viewer (read-only editor,
markdown paint, hybrid tables, meta-card tint). What is missing is not a forced redesign
of that path, but a way to choose how Answers are *presented* when reading long agent
replies and when hunting for the actionable gate.

```
stored Answer markdown (unchanged)
              │
              ▼
     chat viewer style
    ┌─────┬─────┬──────┐
    │     │     │      │
 classic focus document
 (now)  (next)  (later)
```

```
| Style | Role |
| --- | --- |
| **Classic** | Default. Today's Answer presentation; incremental polish stays here. |
| **Focus** | Classic paint plus section collapse: fold other H2/H3 sections; keep the trailing meta region open. |
| **Document** | Later. Richer, note-like reading surface (Obsidian-ish); same switch, not a separate product. |
```

**Focus open region (settled):** the trailing action region only — the trailing `next`
meta card, and when a write gate is present, the preceding `write` card plus the ordinary
preview markdown between `write` and `next`. Everything else in the Answer collapses by
default. Manual expand remains available for folded sections.

**Out of scope for this change's intent:** harness CLIs/TUIs, rewriting agent markdown,
Thinking/Activity redesign, and treating "TUI-like" as a fourth style (the current viewer
is already that family; further affinity is Classic polish).

**Constraints the exploration fixed:** styles are presentation-only over the same session
content; the switch is a chat setting (global config, Settings pick list; optional
chat-chrome control later can share the same value); ship spine first (Classic wired),
then Focus, then Document when Classic is not enough to read.
