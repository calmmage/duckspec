# Build pilot

Duckboard-local `/build-auto` and `/build-fast` arm a session pilot that kicks the
workflow and auto-sends safe next steps so a change can advance without pressing Enter
after every agent turn.

## Motivation

The duckspec loop is already explicit: agents emit trailing `next` cards, and the composer
can ghost the rank-1 action. Moving a change still means a human click/Enter at every gate
and handoff. That friction is the main cost of “just run the scenario.”

Why now: meta-card send tokens and system slash handling are stable enough that the client
can own “send the next message,” instead of hoping the agent keeps driving itself.

## Intent

- User can start **auto** or **fast** pilot from exploration or any non-archived change
  via system commands `/build-auto` and `/build-fast`, with optional free text after the
  command

- Submitting those commands arms the pilot, shows a clear mode plaque above the input, and
  immediately sends a normal user message that is the rewritten kick (`/ds-explore …` on
  exploration, or the current lifecycle head + args on a change)—not the `/build-*` string

- While armed, after each agent turn settles, duckboard auto-sends the rank-1 trailing
  `next` token when it is safe (`confirm` always; allowlisted `/ds-*` stage advances per
  mode rules)

- **Auto** may follow review and rework loops the agent offers; **fast** still
  auto-confirms and advances stages, stitches design into spec when that handoff appears,
  and does not auto-send review

- Neither mode auto-sends archive, codex, or verify; unsafe or missing `next` (reject,
  revise, freeform, multi-option without a safe default, no card) fully disarms the pilot
  until `/build-*` is run again

- Esc-Esc cancels an in-flight turn if streaming and always disarms the pilot; pilot does
  not persist across app restart for now

- When an exploration promotes into a change mid-flight, the pilot stays armed and
  continues under the same rules on that change

- A setting exists for “reactivate pilot after crash/restart/error,” default **off**
  (behavior may stay stubbed or documented until a later pass; the control is part of this
  change’s product surface)

## Non-goals

- Keyword/NLP scanning of assistant prose for “questions” or “critical issues” (structure
  and send tokens only)

- Turn, time, or cost caps as a stop reason

- Auto-archiving a change

- A separate `/build-stop` command (Esc-Esc is enough for v1)

- Making `/build-*` agent/workflow skills or renaming the `/ds-*` stage commands

- Persisting pilot arm state across restarts beyond the default-off reactivate setting’s
  future behavior

- Changing meta-card syntax or inventing new card kinds

- Teaching the agent new multi-stage mega-skills (fast “join design and spec” is
  client-side auto-send of the next stage, not a new template)
