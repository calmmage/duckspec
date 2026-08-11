# Hybrid Focus paint and open-region band

Open region uses Classic TextEdit; expanded sections stay plain; last-answer band only on
the open region; real Hybrid C and band tests replace the paint stub.

## Prerequisites

- [x] @step focus-presentation-and-folds

## Tasks

- [x] 1. Wire open-region slice through Classic `TextEdit` body recipe (reuse Classic
         setup; not multi-slice editors for sections)

- [x] 2. Keep expanded section bodies as plain content-font text; never apply last-answer
         band to them

- [x] 3. Apply `chat_last_answer_band` only to the open-region widget when the Answer is
         the band target

- [x] 4. Remove or stop using the `focus_body_uses_classic_paint` stub as the paint
         contract

- [x] 5. @spec chat/focus-answer Focus slice presentation (Hybrid C): Open region uses classic Answer body paint

- [x] 6. @spec chat/focus-answer Focus slice presentation (Hybrid C): Expanded foldable section uses plain content presentation

- [x] 7. @spec chat/focus-answer Last-answer band under Focus: Last Focus Answer bands only the open region

- [x] 8. @spec chat/focus-answer Last-answer band under Focus: Expanded section of last Focus Answer is not banded

## Outcomes

- Focus sectioned Answers store open-region lines only in `chat_editors[i]` so the open
  region can reuse the block's read-only TextEdit (tables, meta tint, find on that
  surface). Full Answer lines remain on `chat_blocks[i]` for section plain text.

- Classic / live / passthrough still materialize the full body into the editor
  (`answer_editor_desired_lines`).
