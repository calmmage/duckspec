# Raw user inputs in the change package

Preserve the user's verbatim chat inputs as a durable markdown ledger inside the change
package — for agent re-grounding, human audit, and archive archaeology — in the spirit of
precise-secretary (structure only; do not rewrite).

## Motivation

Chat already persists in local duckboard data (`chat_store`), but that is not part of the
committed change package. Proposal and design only hold *synthesized* intent. Later agents
invent or rewrite motivation; humans cannot open the package and see what was actually
typed; after archive only the polished docs remain.

Why now: the package is the shared source of truth for agents and VCS; raw human words
that drove a change should ride with it the same way proposal/design do.

## Intent

- Every change that has user chat in its scope gets a package file of **raw user inputs**,
  chronological, **verbatim** (exact phrasing, punctuation, structure)

- The file lives **under the change directory** so `ds archive` keeps it forever

- While the change is active, the ledger stays **fresh enough** that agents can re-ground
  on current human words (not a stale snapshot only at the end)

- Priming and similar non-human injects are **not** treated as user inputs

- Optional later: thin agent-question context; not required for success of this change

- Proposal/design remain the synthesized layer; this file is the source-quote layer, not a
  second design

## Non-goals

- Full forensic transcript (assistant answers, tools, reasoning) as the primary artifact
- Auto-generated takeaways, summaries, or rewritten motivation in this file
- Replacing proposal or design
- Changing chat_store durability / promotion semantics except as needed to feed the export
- Editorial “highlights only” curation as v1
