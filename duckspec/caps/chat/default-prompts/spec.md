# Chat default prompts

Conversation-local empty-input defaults from a cheap-model oneshot: parse ordered `REPLY:`
suggestions (heuristic passed only as a soft hint), show and arm them only after the
oneshot settles, and drive empty Enter plus fill and cycle from the next-action list
alone.

Empty-composer next actions from an inherited donor list (empty change sessions after
new-session creation), lifecycle bootstrap, a trailing `next` meta card, or a trailing
A–B–C prose option run when no meta card is present, shown as ghost text with empty Enter
send, Tab/right-arrow fill, and Shift-Tab cycle; optional settings-gated oneshot reply
suggestions (up to three freeform `REPLY:` lines) that may fill fast-response chips only
when there is no next-action ghost.

## Requirement: Parsed suggestion list

Raw model output SHALL be reduced to at most three non-empty strings taken from lines that
start with the prefix `REPLY:`, in source order. Lines that do not match that prefix SHALL
be ignored. Text after the prefix is trimmed; empty results after trim are dropped.
Unknown slash forms (including command names not in any allow-list) SHALL be kept as
written. A soft character budget in the oneshot instruction SHALL NOT cause the parser to
truncate reply text — over-budget strings SHALL be kept in full after trim. When more than
three `REPLY:` lines are present, only the first three non-empty results SHALL be kept.
When fewer than three non-empty results are present, all of them SHALL be kept in order.

### Scenario: No matching lines yields an empty list

- **GIVEN** model output with no line starting with `REPLY:`
- **WHEN** the suggestion list is parsed
- **THEN** the list is empty

> test: code
> - crates/duckchat/src/reply_suggest.rs:94

### Scenario: Unknown slash text is preserved

- **GIVEN** model output with a `REPLY:` line whose text is an unknown slash form
- **WHEN** the suggestion list is parsed
- **THEN** the list contains that slash text unchanged

> test: code
> - crates/duckchat/src/reply_suggest.rs:101

### Scenario: Reply longer than 100 characters is preserved in full

- **GIVEN** model output with a `REPLY:` line whose text after trim is longer than 100
  characters

- **WHEN** the suggestion list is parsed

- **THEN** the list contains that full reply text unchanged (no character truncation)

> test: code
> - crates/duckchat/src/reply_suggest.rs:114

### Scenario: REPLY lines capped at three

- **GIVEN** model output with four lines each starting with `REPLY:` and non-empty text
- **WHEN** the suggestion list is parsed
- **THEN** the list has exactly three entries
- **AND** those entries are the first three reply texts in source order

> test: code
> - crates/duckchat/src/reply_suggest.rs:65

### Scenario: Fewer than three REPLY lines are kept as-is

- **GIVEN** model output with two lines each starting with `REPLY:` and non-empty text
- **WHEN** the suggestion list is parsed
- **THEN** the list has exactly two entries
- **AND** those entries are the two reply texts in source order

> test: code
> - crates/duckchat/src/reply_suggest.rs:81

## Requirement: Oneshot request framing

The reply-suggestion request SHALL carry the full last assistant message and the preceding
user message when present, without line-count truncation and without a truncation marker
for omitted earlier lines. The request SHALL NOT include a lifecycle heuristic. The
request SHALL NOT include discovered slash command names as priming hints. The instruction
framing SHALL ask for up to three lines of the form `REPLY: <text>` that suggest natural
freeform user responses continuing the dialogue — in order: most likely reply, alternative
reply, and negative or decline reply — and SHALL allow omitting a line when it does not
fit. The instruction SHALL NOT prefer duckspec stage slash commands as the default job of
the oneshot. An empty assistant message SHALL yield an empty suggestion list without
calling the model.

### Scenario: Full assistant and user messages are embedded without line truncation

- **GIVEN** a last assistant message longer than 40 lines
- **AND** a preceding user message longer than 12 lines
- **WHEN** the reply-suggestion request prompt body is built
- **THEN** the embedded assistant body includes the full assistant message
- **AND** the embedded user body includes the full user message
- **AND** no line-truncation marker is present for omitted earlier content

> test: code
> - crates/duckchat/src/reply_suggest.rs:124

### Scenario: Lifecycle heuristic is not included in the request

- **GIVEN** a session that has a first lifecycle option
- **WHEN** the reply-suggestion request prompt body is built
- **THEN** the prompt body does not include a lifecycle heuristic block

> test: code
> - crates/duckchat/src/reply_suggest.rs:169

### Scenario: Empty assistant yields empty list without a model call

- **GIVEN** an empty assistant message
- **WHEN** reply suggestions are requested
- **THEN** the suggestion list is empty
- **AND** no model call is made

> test: code
> - crates/duckchat/src/reply_suggest.rs:224

### Scenario: Instruction asks for up to three ordered freeform REPLY lines

- **GIVEN** the shared reply-suggestion instruction text

- **WHEN** the instruction is inspected

- **THEN** it asks for natural freeform user replies continuing the dialogue

- **AND** it allows up to three `REPLY:` lines in order most likely, alternative, and
  negative or decline

- **AND** it does not prefer stage slash commands as the oneshot's primary job

> test: code
> - crates/duckchat/src/reply_suggest.rs:186

## Requirement: Oneshot readiness

After a non-priming turn completes and a reply-suggestion oneshot is started for a
generation, oneshot suggestions SHALL be pending until that oneshot settles (success or
failure for any reason, including oneshot timeout) for the same generation. While pending,
the system SHALL NOT present oneshot loading chrome and SHALL NOT present oneshot chip
options from that in-flight generation. Pending oneshot state SHALL NOT block empty Enter
from sending an armed next action. When the oneshot settles for the matching generation,
oneshot suggestions SHALL become ready: a non-empty parse is the settled list used for
chip eligibility; an empty parse or failure yields a ready empty list. Results for a
superseded generation SHALL NOT change the session's ready oneshot list. When no oneshot
is outstanding, oneshot suggestions are ready (the list may be empty). If the chat agent
handle ends while a reply-suggestion oneshot is outstanding for the current generation
without a matching settle, oneshot suggestions SHALL become ready (they SHALL NOT remain
pending). There is no under-input oneshot suggestion row and no empty Cmd-Enter path for
oneshot suggestions.

### Scenario: Empty Enter still sends next action while oneshot pending

- **GIVEN** a pending reply-suggestion oneshot
- **AND** an empty composer input
- **AND** a non-empty next-action list with an active entry
- **WHEN** the user submits with empty input
- **THEN** the sent text is that active next-action entry

> test: code
> - crates/duckboard/src/default_prompts.rs:528

### Scenario: Superseded generation does not arm oneshot

- **GIVEN** a oneshot result whose generation no longer matches the session
- **WHEN** that result is applied
- **THEN** the session's ready oneshot suggestion is unchanged

> test: code
> - crates/duckboard/src/default_prompts.rs:576

### Scenario: Failed or timed-out oneshot settles without presenting suggestions

- **GIVEN** a pending reply-suggestion oneshot
- **AND** that oneshot settles as a failure for the current generation
- **WHEN** the settled oneshot list is inspected
- **THEN** oneshot suggestions are ready
- **AND** the settled list is empty when the failure produced no parse

> test: code
> - crates/duckboard/src/default_prompts.rs:586

### Scenario: Agent handle end while pending leaves suggestions ready empty

- **GIVEN** a pending reply-suggestion oneshot
- **AND** the chat agent handle ends without a settle for that generation
- **WHEN** oneshot readiness is inspected
- **THEN** oneshot suggestions are ready
- **AND** no oneshot loading chrome is shown

> test: code
> - crates/duckboard/src/default_prompts.rs:606

### Scenario: Pending oneshot presents no loading chrome

- **GIVEN** a pending reply-suggestion oneshot
- **AND** an empty composer input
- **WHEN** empty-composer oneshot chrome is evaluated
- **THEN** no oneshot loading indicator is shown
- **AND** no oneshot suggestion row is shown under the input

> test: code
> - crates/duckboard/src/default_prompts.rs:621

## Requirement: Agent input hints gate

A global agent input hints setting SHALL control whether reply-suggestion oneshots run
after turns. The setting SHALL default to disabled. When agent input hints is disabled, a
reply-suggestion oneshot SHALL NOT be started after a non-priming turn completes. When
agent input hints is enabled, oneshot launch follows the non-priming turn rules of this
capability (assistant text present and other launch conditions) and SHALL NOT start when
the next-action list for that session is non-empty after the turn's next-action refresh.
There is no separate auto-messages setting that suppresses oneshots or next-action lists.
Empty-session next-action bootstrap and the next-action list SHALL NOT depend on the agent
input hints setting.

### Scenario: Default agent input hints setting is disabled

- **GIVEN** application config defaults
- **WHEN** the agent input hints setting is read
- **THEN** it is disabled

> test: code
> - crates/duckboard/src/config.rs:441

### Scenario: Oneshot launch requires agent input hints enabled

- **GIVEN** agent input hints disabled
- **AND** a non-priming turn that would otherwise qualify for reply suggestions
- **WHEN** oneshot launch is decided
- **THEN** a reply-suggestion oneshot is not started

> test: code
> - crates/duckboard/src/default_prompts.rs:555

### Scenario: Empty-session next actions remain when agent input hints disabled

- **GIVEN** agent input hints disabled
- **AND** an empty session transcript
- **AND** a first lifecycle option for that session
- **WHEN** the next-action list is built
- **THEN** the list is exactly that single lifecycle option in empty-send form

> test: code
> - crates/duckboard/src/area/interaction.rs:1513

### Scenario: Oneshot launch is skipped when the next-action list is non-empty

- **GIVEN** agent input hints enabled
- **AND** a non-priming turn that has assistant text
- **AND** a non-empty next-action list after that turn's next-action refresh
- **WHEN** oneshot launch is decided
- **THEN** a reply-suggestion oneshot is not started

> test: code
> - crates/duckboard/src/default_prompts.rs:567

## Requirement: Next-action list

The next-action list for the empty composer SHALL be built as follows. When the session
transcript is empty and a non-empty inherited next-action list is present, the list SHALL
be exactly that inherited list (in order). When the session transcript is empty, no
non-empty inherited list is present, and a first lifecycle option is present, the list
SHALL be exactly that option in empty-send form (a single entry). When the session
transcript is empty and neither a non-empty inherited list nor a first lifecycle option is
present, the list SHALL be empty. When the session transcript is non-empty, the list SHALL
be built from the last non-priming assistant message only: if trailing next actions exist
(via chat meta-card recognition), the list SHALL be exactly those tokens; otherwise, if a
trailing A–B–C prose option run is present (see Prose option next-actions), the list SHALL
be exactly those prose option send strings; otherwise the list SHALL be empty. A non-empty
transcript SHALL NOT use an inherited next-action list. Settled oneshot suggestion strings
SHALL NOT be appended, merged, or substituted into the next-action list. Disk lifecycle
options beyond the empty-session bootstrap SHALL NOT fill the list after the first turn.
Meta-card next tokens and prose options SHALL NOT be merged into one list.

For an empty exploration session, the first lifecycle option SHALL be the explore stage
command. For an empty change session, the first lifecycle option SHALL be the first option
of that change's lifecycle ladder from its artifact and step state. Sessions with no
lifecycle ladder (including caps and codex) SHALL have no first lifecycle option from this
bootstrap.

### Scenario: Empty session seeds first lifecycle

- **GIVEN** an empty session transcript
- **AND** a first lifecycle option in empty-send form
- **WHEN** the next-action list is built
- **THEN** the list is exactly that single lifecycle option

> test: code
> - crates/duckboard/src/default_prompts.rs:255

### Scenario: Empty session without lifecycle yields empty

- **GIVEN** an empty session transcript
- **AND** no first lifecycle option
- **WHEN** the next-action list is built
- **THEN** the list is empty

> test: code
> - crates/duckboard/src/default_prompts.rs:268

### Scenario: Non-empty session uses trailing next actions only

- **GIVEN** a non-empty session transcript

- **AND** a last non-priming assistant message whose trailing next actions are two
  distinct send tokens

- **AND** a first lifecycle option that differs from those tokens

- **WHEN** the next-action list is built

- **THEN** the list is exactly those two trailing next send tokens in order

> test: code
> - crates/duckboard/src/default_prompts.rs:278

### Scenario: Non-empty session without trailing next yields empty

- **GIVEN** a non-empty session transcript

- **AND** a last non-priming assistant message with no trailing `next` meta card and no
  trailing A–B–C prose option run

- **AND** a present first lifecycle option

- **WHEN** the next-action list is built

- **THEN** the list is empty

> test: code
> - crates/duckboard/src/default_prompts.rs:298

### Scenario: Non-empty session uses prose ABC when no meta next

- **GIVEN** a non-empty session transcript
- **AND** a last non-priming assistant message with no trailing `next` meta card
- **AND** that message ends with a trailing A–B–C prose option run of two options
- **AND** a first lifecycle option that differs from those option send strings
- **WHEN** the next-action list is built
- **THEN** the list is exactly those two prose option send strings in order
- **AND** the lifecycle option is not in the list

> test: code
> - crates/duckboard/src/default_prompts.rs:307

### Scenario: Oneshot results do not enter the next-action list

- **GIVEN** a non-empty session transcript
- **AND** a last non-priming assistant message with no trailing `next` meta card
- **AND** a settled oneshot whose parse produced a non-empty suggestion
- **WHEN** the next-action list is built
- **THEN** the list is empty

> test: code
> - crates/duckboard/src/default_prompts.rs:323

### Scenario: Empty exploration session seeds explore

- **GIVEN** an empty exploration session transcript
- **WHEN** the next-action list is built
- **THEN** the list is exactly the explore stage command in empty-send form

> test: code
> - crates/duckboard/src/area/interaction.rs:1047

### Scenario: Empty change session with unfinished steps seeds apply

- **GIVEN** an empty change session transcript
- **AND** that change has at least one unfinished step
- **WHEN** the next-action list is built
- **THEN** the list is exactly the apply stage command in empty-send form

> test: code
> - crates/duckboard/src/area/interaction.rs:1060

### Scenario: Empty session with inherited next actions uses inherited list

- **GIVEN** an empty session transcript
- **AND** a non-empty inherited next-action list of two distinct send tokens
- **AND** a first lifecycle option that differs from those tokens
- **WHEN** the next-action list is built
- **THEN** the list is exactly those two inherited send tokens in order

> test: code
> - crates/duckboard/src/default_prompts.rs:338

### Scenario: Empty session without inherited falls back to lifecycle

- **GIVEN** an empty session transcript
- **AND** no non-empty inherited next-action list
- **AND** a first lifecycle option in empty-send form
- **WHEN** the next-action list is built
- **THEN** the list is exactly that single lifecycle option

> test: code
> - crates/duckboard/src/default_prompts.rs:360

### Scenario: Non-empty session drops inheritance

- **GIVEN** a non-empty session transcript

- **AND** a non-empty inherited next-action list

- **AND** a last non-priming assistant message whose trailing next actions are a single
  send token that differs from the inherited list

- **WHEN** the next-action list is built

- **THEN** the list is exactly that trailing next send token

- **AND** the inherited list is not used

> test: code
> - crates/duckboard/src/default_prompts.rs:374

## Requirement: Prose option next-actions

A trailing A–B–C prose option run SHALL be extracted only from assistant markdown outside
fenced code blocks. An option line SHALL begin (after optional list marker or leading
table pipe) with letter A–C or a–c (optional bold wrap), then a separator (`—`, `-`, `:`,
or `|`), then a non-empty body. The run SHALL be the last contiguous sequence of such
lines at the end of the message (blank lines between option lines and after the run are
allowed; a non-option non-blank line after the last option disqualifies the run). Letters
SHALL form strict A→B→C… order starting at A. Fewer than two options SHALL yield no prose
actions. At most three options SHALL be kept. Each action's send text SHALL be
`{Letter} — {body}` with the letter uppercased, clipped to a fixed character budget with a
trailing ellipsis when longer. The reason field SHALL be empty. When both a trailing
`next` meta card and a prose option run are present, only the meta-card actions SHALL be
used (see Next-action list).

### Scenario: Bullet ABC run yields three sends

- **GIVEN** assistant markdown ending with three bullet lines `**A**` / `**B**` / `**C**`
  each with a body after an em dash

- **WHEN** trailing A–B–C prose options are extracted

- **THEN** exactly three next actions are produced

- **AND** each send text is `{Letter} — {body}` in A–B–C order

- **AND** each reason is empty

> test: code
> - crates/duckcore/src/prose_options.rs:267

### Scenario: Meta next wins over prose ABC

- **GIVEN** assistant markdown that contains a trailing A–B–C prose option run and also
  ends with a trailing `next` meta card with two send tokens

- **WHEN** next actions are taken from that assistant message

- **THEN** the list is exactly the two meta-card send tokens

- **AND** no prose option send string is included

> test: code
> - crates/duckcore/src/prose_options.rs:285

### Scenario: Prose fills when no meta next

- **GIVEN** assistant markdown ending with a two-option A–B prose run and no trailing
  `next` meta card

- **WHEN** next actions are taken from that assistant message

- **THEN** the list is exactly the two prose option send strings

> test: code
> - crates/duckcore/src/prose_options.rs:303
> - crates/ducktui/src/chat_pane.rs:700

### Scenario: Single option yields empty

- **GIVEN** assistant markdown whose only option line is a single A line
- **WHEN** trailing A–B–C prose options are extracted
- **THEN** the list is empty

> test: code
> - crates/duckcore/src/prose_options.rs:318

### Scenario: Fence interior options ignored

- **GIVEN** assistant markdown whose only A–B option lines sit inside a fenced code block
- **WHEN** trailing A–B–C prose options are extracted
- **THEN** the list is empty

> test: code
> - crates/duckcore/src/prose_options.rs:325

### Scenario: Non-trailing interrupted run ignored

- **GIVEN** assistant markdown with an A–B option run followed by non-option prose that is
  the last non-empty content

- **WHEN** trailing A–B–C prose options are extracted

- **THEN** the list is empty

> test: code
> - crates/duckcore/src/prose_options.rs:339

### Scenario: Strict letter order required

- **GIVEN** assistant markdown ending with option lines A then C (no B)
- **WHEN** trailing A–B–C prose options are extracted
- **THEN** the list is empty

> test: code
> - crates/duckcore/src/prose_options.rs:351

### Scenario: Long body is clipped

- **GIVEN** assistant markdown ending with A and B option lines where A's body is longer
  than the send character budget

- **WHEN** trailing A–B–C prose options are extracted

- **THEN** two actions are produced

- **AND** the first send text length is at most the budget

- **AND** the first send text ends with an ellipsis

> test: code
> - crates/duckcore/src/prose_options.rs:361

### Scenario: Cap at three options

- **GIVEN** assistant markdown ending with A, B, and C option lines
- **WHEN** trailing A–B–C prose options are extracted
- **THEN** exactly three actions are produced

> test: code
> - crates/duckcore/src/prose_options.rs:373

### Scenario: Bare letter dash lines work

- **GIVEN** assistant markdown ending with bare lines `A — …` and `B — …` (no list marker)
- **WHEN** trailing A–B–C prose options are extracted
- **THEN** exactly two actions are produced with the corresponding send strings

> test: code
> - crates/duckcore/src/prose_options.rs:386

## Requirement: Next-action empty-input send, fill, and cycle

When the composer input is empty and the next-action list is non-empty, submit SHALL send
the active entry's send text. When the next-action list is empty, empty submit SHALL NOT
send a next action. When the composer input is empty, the main turn is not streaming, the
next-action list is non-empty, and slash-command completion is not consuming Tab, bare Tab
and bare right-arrow SHALL insert the active entry's send text into the composer input and
SHALL NOT start a turn. When the input is empty, the list has at least two entries, and
the main turn is not streaming, Shift-Tab SHALL advance the active next-action index with
wrap. Cycling SHALL NOT insert text into the composer input. Reverse cycle SHALL NOT be
required. When the next-action list has more than one entry and the input is empty, a
cycle marker SHALL be shown before the ghost (next-action) affordance as a display-only
Shift-Tab cue. When the list has zero or one entry, that cycle marker SHALL NOT be shown.
The empty composer's ghost (placeholder) text SHALL be the active next-action send text
when the list is non-empty and the main turn is not streaming. Display-only marker
prefixes SHALL NOT be part of fill or send text.

> test: code

### Scenario: Empty submit sends the active next action

- **GIVEN** an empty composer input
- **AND** a non-empty next-action list
- **AND** an active index into that list
- **WHEN** the user submits
- **THEN** the sent text is the send text of the entry at the active index

> test: code
> - crates/duckboard/src/default_prompts.rs:401

### Scenario: Empty submit is a no-op when the next-action list is empty

- **GIVEN** an empty composer input
- **AND** an empty next-action list
- **WHEN** the user submits
- **THEN** no next-action message is sent

> test: code
> - crates/duckboard/src/default_prompts.rs:424

### Scenario: Shift-Tab cycles next actions with wrap

- **GIVEN** an empty composer input
- **AND** a next-action list of at least two entries
- **AND** the main turn is not streaming
- **WHEN** the user presses Shift-Tab at the last index
- **THEN** the active index wraps to the first entry
- **AND** the composer input remains empty

> test: code
> - crates/duckboard/src/default_prompts.rs:450

### Scenario: Tab or right-arrow fills the active next action

- **GIVEN** an empty composer input
- **AND** a non-empty next-action list
- **AND** an active index into that list
- **AND** the main turn is not streaming
- **AND** slash-command completion is not consuming Tab
- **WHEN** the user presses bare Tab or bare right-arrow
- **THEN** the composer input text is the send text of the entry at the active index
- **AND** no agent turn is started

> test: code
> - crates/duckboard/src/default_prompts.rs:475

### Scenario: Multi next shows a cycle marker

- **GIVEN** an empty composer input
- **AND** a next-action list of at least two entries
- **AND** the main turn is not streaming
- **WHEN** the empty-composer next-action chrome is rendered
- **THEN** a cycle marker is shown before the ghost text

> test: code
> - crates/duckboard/src/default_prompts.rs:501

## Requirement: Oneshot chip eligibility

Settled oneshot replies SHALL be eligible to fill fast-response chips only when all of the
following hold: agent input hints is enabled; no main agent turn is streaming; the session
is not awaiting a user choice; the next-action list is empty; and the settled oneshot list
is non-empty. When any of those conditions fails, oneshot replies SHALL NOT be eligible to
fill chips.

> test: code

### Scenario: Eligible when idle with no next actions and a settled list

- **GIVEN** agent input hints enabled
- **AND** no main agent turn in progress
- **AND** the session is not awaiting a user choice
- **AND** an empty next-action list
- **AND** a non-empty settled oneshot list
- **WHEN** oneshot chip eligibility is evaluated
- **THEN** oneshot replies are eligible to fill chips

> test: code
> - crates/duckboard/src/default_prompts.rs:631

### Scenario: Ineligible when next-action list is non-empty

- **GIVEN** agent input hints enabled
- **AND** no main agent turn in progress
- **AND** the session is not awaiting a user choice
- **AND** a non-empty next-action list
- **AND** a non-empty settled oneshot list
- **WHEN** oneshot chip eligibility is evaluated
- **THEN** oneshot replies are not eligible to fill chips

> test: code
> - crates/duckboard/src/default_prompts.rs:637

### Scenario: Ineligible while awaiting a user choice

- **GIVEN** agent input hints enabled
- **AND** the session is awaiting a user choice
- **AND** an empty next-action list
- **AND** a non-empty settled oneshot list
- **WHEN** oneshot chip eligibility is evaluated
- **THEN** oneshot replies are not eligible to fill chips

> test: code
> - crates/duckboard/src/default_prompts.rs:643

### Scenario: Ineligible while streaming

- **GIVEN** agent input hints enabled
- **AND** a main agent turn in progress
- **AND** the session is not awaiting a user choice
- **AND** an empty next-action list
- **AND** a non-empty settled oneshot list
- **WHEN** oneshot chip eligibility is evaluated
- **THEN** oneshot replies are not eligible to fill chips

> test: code
> - crates/duckboard/src/default_prompts.rs:649

### Scenario: Ineligible when the settled list is empty

- **GIVEN** agent input hints enabled
- **AND** no main agent turn in progress
- **AND** the session is not awaiting a user choice
- **AND** an empty next-action list
- **AND** an empty settled oneshot list
- **WHEN** oneshot chip eligibility is evaluated
- **THEN** oneshot replies are not eligible to fill chips

> test: code
> - crates/duckboard/src/default_prompts.rs:655

## Requirement: New-session next-action inheritance

When a new change chat session is created for multi-session change chat, if the session
that was active at creation has a non-empty next-action list, the new empty session SHALL
receive that list as its inherited next-action list and its next-action list SHALL match
those send tokens in order. The active next action on the new session SHALL be the first
entry. When the active session's next-action list is empty at creation, the new empty
session SHALL NOT receive an inherited list and SHALL follow empty-session next-action
list rules without inheritance. Inheritance applies only while the new session transcript
remains empty. Oneshot reply suggestions SHALL NOT be copied onto the new session by this
path.

### Scenario: New change session inherits active session next actions

- **GIVEN** a change scope with multi-session chat
- **AND** the active session has a non-empty next-action list of two distinct send tokens
- **WHEN** a new chat session is created for that change
- **THEN** the new session's next-action list is exactly those two send tokens in order
- **AND** the new session transcript is empty

> test: code
> - crates/duckboard/src/area/interaction.rs:1081

### Scenario: New change session with empty donor keeps bootstrap behavior

- **GIVEN** a change scope with multi-session chat
- **AND** the active session has an empty next-action list
- **AND** a first lifecycle option for that change in empty-send form
- **WHEN** a new chat session is created for that change
- **THEN** the new session's next-action list is exactly that single lifecycle option

> test: code
> - crates/duckboard/src/area/interaction.rs:1110

### Scenario: Inherited list starts at first action

- **GIVEN** a change scope with multi-session chat
- **AND** the active session has a next-action list of two or more send tokens
- **AND** the active session's active next-action index is not the first entry
- **WHEN** a new chat session is created for that change
- **THEN** empty submit on the new session sends the first inherited send token

> test: code
> - crates/duckboard/src/area/interaction.rs:1136
