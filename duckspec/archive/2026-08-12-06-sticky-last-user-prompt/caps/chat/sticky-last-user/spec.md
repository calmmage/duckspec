# Chat sticky last-user

Smart sticky strip at the top of duckboard chat that mirrors the last real user ask only
while that ask’s in-transcript card has scrolled above the viewport top.

## Requirement: Source identity

The sticky’s source SHALL be the last non-priming User segment in transcript order.
Priming User segments, System segments, Thinking, Activity, Answer, and user-choice
segments SHALL NOT be the sticky source.

> test: code

### Scenario: Last non-priming User wins over priming and later non-user segments

- **GIVEN** a transcript with a priming User segment, a later non-priming User segment,
  and a still later Answer segment

- **WHEN** the sticky source is resolved

- **THEN** the source is that non-priming User segment

> test: code

### Scenario: Priming-only transcript has no source

- **GIVEN** a transcript whose only User segment is priming
- **AND** no non-priming User segment exists
- **WHEN** the sticky source is resolved
- **THEN** there is no sticky source

> test: code

### Scenario: User-choice answer is not the source

- **GIVEN** a transcript whose latest interactive segment is a user-choice answer
- **AND** an earlier non-priming User segment exists
- **WHEN** the sticky source is resolved
- **THEN** the source is the earlier non-priming User segment
- **AND** the user-choice answer is not the source

> test: code

## Requirement: Visibility

The sticky SHALL be shown only when a sticky source exists and that segment’s content top
is above the transcript viewport top, using the same top-edge slack as answer reply-jump
viewport comparisons. When there is no source, or the content top is at or below the
viewport top, the sticky SHALL be hidden. A missing scroll offset SHALL be treated as zero
for the visibility check.

> test: code

### Scenario: Content top above offset shows sticky

- **GIVEN** a sticky source with a known content top
- **AND** a scroll offset greater than that content top by more than the top-edge slack
- **WHEN** sticky visibility is evaluated
- **THEN** the sticky is shown

> test: code

### Scenario: Content top at or below offset hides sticky

- **GIVEN** a sticky source with a known content top
- **AND** a scroll offset at or below that content top within the top-edge slack
- **WHEN** sticky visibility is evaluated
- **THEN** the sticky is hidden

> test: code

### Scenario: No source hides sticky

- **GIVEN** no sticky source
- **WHEN** sticky visibility is evaluated
- **THEN** the sticky is hidden

> test: code

### Scenario: Missing offset treated as zero

- **GIVEN** a sticky source whose content top is greater than zero
- **AND** no scroll offset is recorded
- **WHEN** sticky visibility is evaluated
- **THEN** the sticky is hidden

> test: code

## Requirement: Mirror body

When the sticky body text is resolved from a source, it SHALL be the full text of that
source User segment. The sticky body SHALL be a read-only mirror of that text.

> test: code

### Scenario: Body equals the source User segment text

- **GIVEN** a sticky source User segment with known body text
- **WHEN** the sticky body text is resolved
- **THEN** the body text equals that segment’s full text

> test: code

## Requirement: Height cap

The sticky panel’s maximum height SHALL be six content line heights. Content that exceeds
that maximum SHALL be reachable by scrolling within the panel.

> test: code

### Scenario: Maximum height is six content line heights

- **GIVEN** the sticky panel height metrics
- **WHEN** the maximum sticky height is resolved
- **THEN** it equals six content line heights

> test: code
