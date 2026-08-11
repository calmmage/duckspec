# Chat focus answer

Focus Answer presentation: trailing meta open region, H2/H3 section folds, Hybrid C slice
paint, and open-region-only last-answer band.

## Requirement: When Focus applies

Focus layout SHALL apply to an Answer only when the effective viewer style is focus and
that Answer is not live. When Focus layout does not apply, the Answer SHALL use classic
presentation — including when the effective style is focus but the Answer has no trailing
`next` meta card (classic passthrough).

> test: code

### Scenario: Live Answer under focus style uses classic presentation

- **GIVEN** the effective viewer style is focus
- **AND** a live Answer segment
- **WHEN** the Answer is presented
- **THEN** the Answer uses classic presentation

### Scenario: Settled Answer without trailing next uses classic passthrough

- **GIVEN** the effective viewer style is focus
- **AND** a settled Answer with no trailing `next` meta card
- **WHEN** the Answer is presented
- **THEN** the Answer uses classic presentation

### Scenario: Settled Answer with trailing next uses Focus layout

- **GIVEN** the effective viewer style is focus
- **AND** a settled Answer that ends with a trailing `next` meta card
- **WHEN** the Answer is presented
- **THEN** the Answer uses Focus layout

## Requirement: Open region

When Focus layout applies, the open region SHALL cover the trailing `next` meta card. When
a `write` meta card ends before that trailing `next` and no other meta card lies strictly
between them, the open region SHALL start at that `write` card and include all lines
through the trailing `next` (preview prose between the cards included). Meta cards earlier
in the Answer SHALL NOT extend or replace the open region.

> test: code

### Scenario: Trailing next alone opens from next through answer end

- **GIVEN** a settled Answer under Focus whose only meta card is a trailing `next`
- **WHEN** the open region is computed
- **THEN** the open region starts at the `next` card start
- **AND** the open region ends at the last line of the Answer

### Scenario: Write then preview then trailing next opens from write through next

- **GIVEN** a settled Answer under Focus with a `write` card, non-card preview lines, then
  a trailing `next` card

- **AND** no other meta card between that `write` and `next`

- **WHEN** the open region is computed

- **THEN** the open region starts at the `write` card start

- **AND** the open region ends at the `next` card end

### Scenario: Meta card between write and trailing next leaves only next open

- **GIVEN** a settled Answer under Focus with a `write` card, another meta card, then a
  trailing `next` card

- **WHEN** the open region is computed

- **THEN** the open region starts at the trailing `next` card start

- **AND** the open region does not include the earlier `write` card

## Requirement: Section partition

Under Focus layout, lines before the open region SHALL be partitioned into foldable
sections. Outside fenced code blocks, an ATX heading line at level 2 or 3 SHALL start a
new section; level-1 headings SHALL NOT start a section. Heading-like lines inside open
fences SHALL NOT start sections. Lines before the first section-starting heading SHALL
form one preamble section. When there is an open region and no section-starting heading
before it, the body before the open region SHALL be one foldable section. Section line
ranges SHALL NOT include any open-region line.

> test: code

### Scenario: H2 and H3 outside fences start sections and H1 does not

- **GIVEN** Answer body lines before the open region that include an H1, an H2, and an H3
  outside fences

- **WHEN** sections are partitioned

- **THEN** the H2 and H3 each start a foldable section

- **AND** the H1 does not start a foldable section

### Scenario: Heading-like lines inside fences do not start sections

- **GIVEN** Answer body lines before the open region that place an `##` line only inside a
  fenced code block

- **WHEN** sections are partitioned

- **THEN** that line does not start a foldable section

### Scenario: Preamble before first H2 or H3 is one foldable section

- **GIVEN** Answer body lines before the open region with prose before the first H2 or H3
- **WHEN** sections are partitioned
- **THEN** those leading lines form exactly one preamble section

### Scenario: No headings with open region yields one body section plus open region

- **GIVEN** a Focus layout Answer with an open region and no H2 or H3 outside fences
  before it

- **WHEN** sections are partitioned

- **THEN** there is exactly one foldable section covering the body before the open region

- **AND** the open region remains separate from that section

### Scenario: Section ranges exclude open-region lines

- **GIVEN** a Focus layout Answer with foldable body lines and an open region
- **WHEN** sections are partitioned
- **THEN** no section range includes a line that belongs to the open region

## Requirement: Fold defaults and toggles

Under Focus layout, each foldable section SHALL start collapsed and the open region SHALL
always be shown (not collapsible). A user toggle on a section SHALL flip that section's
collapsed state and mark it user-set so later auto defaults do not override it while the
section key remains. Section fold state SHALL be separate from transcript segment collapse
and SHALL NOT persist across sessions. When the effective viewer style leaves focus, all
Focus section fold state SHALL be cleared. On rematerialize, fold state for section keys
that still exist SHALL be retained when user-set; new keys SHALL receive the default
collapsed state.

> test: code

### Scenario: First sight collapses foldable sections and shows open region

- **GIVEN** a settled Answer under Focus layout with at least one foldable section and an
  open region

- **AND** no user fold overrides for that Answer

- **WHEN** the Answer is first presented under Focus

- **THEN** every foldable section is collapsed

- **AND** the open region is shown

### Scenario: User expand survives rematerialize for the same section key

- **GIVEN** a Focus layout Answer with a foldable section the user has expanded
- **WHEN** the Answer is rematerialized with that section key still present
- **THEN** that section remains expanded

### Scenario: Leaving Focus clears section fold state

- **GIVEN** Focus section fold state for one or more Answers
- **WHEN** the effective viewer style is no longer focus
- **THEN** all Focus section fold state is cleared

## Requirement: Focus slice presentation (Hybrid C)

When Focus layout applies, the open region SHALL use classic Answer body paint. An
expanded foldable section body SHALL use plain content-font source presentation and SHALL
NOT use classic Answer body paint. A collapsed foldable section SHALL show a header label
only: the heading text for heading sections, or a preamble label that includes the section
line count for the preamble section.

> test: code

### Scenario: Open region uses classic Answer body paint

- **GIVEN** a Focus layout Answer with an open region
- **WHEN** the open region is presented
- **THEN** the open region uses classic Answer body paint

### Scenario: Expanded foldable section uses plain content presentation

- **GIVEN** a Focus layout Answer with an expanded foldable section
- **WHEN** that section body is presented
- **THEN** the section body uses plain content-font source presentation
- **AND** the section body does not use classic Answer body paint

### Scenario: Collapsed preamble label uses line count form

- **GIVEN** a Focus layout Answer with a collapsed preamble section of known line count
- **WHEN** the collapsed preamble header is presented
- **THEN** the label includes that line count

## Requirement: Last-answer band under Focus

When a Focus layout Answer is the transcript's last-answer band target, last-answer band
styling SHALL apply only to the open-region widget. Expanded foldable section bodies of
that Answer SHALL NOT receive last-answer band styling.

> test: code

### Scenario: Last Focus Answer bands only the open region

- **GIVEN** a Focus layout Answer that is the last-answer band target
- **AND** that Answer has an open region
- **WHEN** the Answer is presented
- **THEN** the open region has last-answer band styling

### Scenario: Expanded section of last Focus Answer is not banded

- **GIVEN** a Focus layout Answer that is the last-answer band target
- **AND** an expanded foldable section on that Answer
- **WHEN** that section body is presented
- **THEN** the section body does not have last-answer band styling
