# Chat focus answer

How Focus mode folds long Answers around the trailing meta gate so the actionable region
stays visible.

## When Focus applies

```
effective style = focus
        AND Answer not live
        AND trailing next meta card present
                → Focus layout
        else
                → classic for that Answer
```

Live drafts always stay classic (headings and gates are still moving). Settled Answers
without a trailing `next` also stay classic — Focus does not invent folds with nothing to
pin open. Other Answers in the transcript can already be Focus while a later turn streams.

## Layout

```
Answer lines
────────────────────────────────
 preamble / ## / ### sections     foldable
 …
── open region (always shown) ──
 optional write card
 preview prose
 trailing next card
────────────────────────────────
```

Open region uses existing meta-card ranges: trailing `next`, optionally extended up to a
preceding `write` when no other meta card sits between them. Section splits are
line-oriented and fence-aware on H2/H3 only (H1 is not a fold boundary).

## Interaction

Each foldable section has a chevron header. Defaults are collapsed; the open region has no
collapse control. User expands stick for a section key until the key disappears or
effective style leaves focus. Fold state is session UI only — not written into chat
storage — and is independent of Thinking / Activity / priming segment collapse.

Collapsed rows show the heading text, or a preamble label with line count.

## Paint (Hybrid C)

```
| Slice | Presentation |
| --- | --- |
| Collapsed section | Chevron + label only |
| Expanded foldable section | Plain content-font source text |
| Open region | Classic Answer body paint (TextEdit recipe) |
```

Expanded sections do not use classic body paint in v1. Find and selection hit visible
widgets only; open-region TextEdit participates like Classic; plain section text is
weaker.

## Last-answer band

When this Answer is the transcript's last-answer band target, band styling applies only to
the open-region widget. Expanded section bodies never take the band. Which Answer is the
target is still decided by answer landmarks (one target Answer).
