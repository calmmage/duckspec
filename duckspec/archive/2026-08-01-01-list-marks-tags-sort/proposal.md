# List marks, tags, and sort

Give the CHANGE and Ideas lists a shared queue language — exclusive boosts, type and phase
pillows, and configurable sort — with the idea as the only annotation store.

## Motivation

The CHANGE and Ideas lists grow into long queues with only creation order and crude
structure (folder slug, primary-tag tree). There is no lightweight way to pin a few hot
items, mark temperature, scan topic or lifecycle at a glance, or sort by recent activity —
so the eye does the full job every time.

Why now: tags and idea↔change links already exist, phase is already derived for real
changes, and list density is the next bottleneck once several explorations and changes sit
side by side.

## Intent

- Exclusive boost mark on each idea: none, star, hot, or cool — shown as emoji at the
  start of the row title; hover shows a star outline when unmarked; clicks cycle the mark

- Up to three most-recently starred items pin to the top of the relevant list; additional
  stars do not pin and follow normal sort with everything else

- Primary tag remains the filing tree only; secondary tags render as type pillows;
  duckspec phase (from existing phase facts) renders as a status pillow when the row is
  change-linked

- Type and phase pillows are each show/hide configurable; when title plus visible pillows
  exceed the row width, pillows hide and appear only on hover; primary is never a pillow

- Default list order after the fav pin is last chat-message time; a sort control next to
  the Change section add button opens options (including sort keys and pillow visibility)

- Idea frontmatter is the sole annotation surface; CHANGE and exploration rows inherit
  marks and type tags from the linked idea

- Free explorations stay on the CHANGE list after they gain an idea link; the first tag on
  a free exploration or unlinked change auto-mints and links an idea so tags and marks
  have a home

- Real change folder names stay identity-only (no emoji or tags in the slug)

## Non-goals

- Closed type taxonomy or a separate `type` field distinct from secondary tags
- Editable/stored work-status separate from derived duckspec phase
- Auto-minting an idea on first chat message (mint is on first tag)
- Renaming real change directories or putting boosts into the filesystem slug
- A second fav/mark store on explorations or changes outside the idea
- Full redesign of list-row chrome beyond marks, pillows, sort menu, and inheritance
