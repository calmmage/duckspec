//! Focus Answer geometry: trailing-meta open region + H2/H3 section partition.
//!
//! Pure line-oriented helpers (no iced). Meta-card recognition comes from
//! `duckcore::meta_card`.

use duckcore::meta_card::{MetaCard, MetaCardKind, parse_meta_cards};

/// Inclusive 0-based line range into the Answer source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineRange {
    pub start: usize,
    pub end: usize,
}

impl LineRange {
    pub fn contains(self, line: usize) -> bool {
        line >= self.start && line <= self.end
    }
}

/// Foldable section kind before the open region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionKind {
    /// Lines before the first H2/H3, or the sole body section when there is no heading.
    Preamble,
    Heading {
        level: u8,
        text: String,
    },
}

/// One foldable slice of Answer body lines (never includes the open region).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoldableSection {
    pub kind: SectionKind,
    pub lines: LineRange,
}

/// Focus layout for a settled Answer body, or Classic passthrough.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusLayout {
    /// No trailing `next` — caller paints Classic full body.
    PassthroughClassic,
    Sectioned {
        sections: Vec<FoldableSection>,
        open: LineRange,
    },
}

/// Stable key for fold state (heading level+text, or preamble sentinel).
pub fn section_key(kind: &SectionKind) -> String {
    match kind {
        SectionKind::Preamble => "\0preamble".to_string(),
        SectionKind::Heading { level, text } => format!("{level}:{text}"),
    }
}

/// Collapsed header label for a foldable section.
pub fn section_collapsed_label(kind: &SectionKind, line_count: usize) -> String {
    match kind {
        SectionKind::Preamble => format!("Preamble · {line_count} lines"),
        SectionKind::Heading { text, .. } => text.clone(),
    }
}

/// Inclusive line count for a range.
pub fn range_line_count(range: LineRange) -> usize {
    range.end.saturating_sub(range.start).saturating_add(1)
}

/// Per-section collapse flag for Focus Answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionFoldState {
    pub collapsed: bool,
    pub user_set: bool,
}

/// Sync fold map with current section keys. New keys default collapsed.
/// Drops orphan keys. Does not override `user_set` entries' collapsed flag.
pub fn sync_section_folds(
    folds: &mut std::collections::HashMap<String, SectionFoldState>,
    sections: &[FoldableSection],
) {
    let keys: std::collections::HashSet<String> =
        sections.iter().map(|s| section_key(&s.kind)).collect();
    folds.retain(|k, _| keys.contains(k));
    for s in sections {
        let k = section_key(&s.kind);
        folds.entry(k).or_insert(SectionFoldState {
            collapsed: true,
            user_set: false,
        });
    }
}

/// Toggle one section; marks `user_set`.
pub fn toggle_section_fold(
    folds: &mut std::collections::HashMap<String, SectionFoldState>,
    key: &str,
) {
    if let Some(state) = folds.get_mut(key) {
        state.collapsed = !state.collapsed;
        state.user_set = true;
    }
}

/// Compute Focus geometry for Answer markdown.
pub fn focus_layout(source: &str) -> FocusLayout {
    let lines: Vec<&str> = source.lines().collect();
    let Some(open) = open_region(source) else {
        return FocusLayout::PassthroughClassic;
    };
    let sections = partition_sections(&lines, open.start);
    FocusLayout::Sectioned { sections, open }
}

/// Trailing-meta open region, if any.
///
/// - Trailing `next` only → `next.line_start ..= last line of answer`
/// - `write` then non-card preview then trailing `next` with no other meta card
///   between → `write.line_start ..= next.line_end`
/// - Other meta card between write and next → open is next only
pub fn open_region(source: &str) -> Option<LineRange> {
    let lines: Vec<&str> = source.lines().collect();
    if lines.is_empty() {
        return None;
    }
    let cards = parse_meta_cards(source);
    let next = cards
        .iter()
        .rev()
        .find(|c| c.kind == MetaCardKind::Next && is_trailing_card(c, &lines))?;

    let last_line = lines.len() - 1;

    // Last write that ends before trailing next.
    let write = cards.iter().rev().find(|c| {
        c.kind == MetaCardKind::Write && c.line_end < next.line_start
    });

    if let Some(w) = write {
        let has_card_between = cards.iter().any(|c| {
            // Any other card that begins strictly after write ends and strictly
            // before next starts.
            c.line_start > w.line_end && c.line_start < next.line_start
        });
        if !has_card_between {
            return Some(LineRange {
                start: w.line_start,
                end: next.line_end,
            });
        }
    }

    Some(LineRange {
        start: next.line_start,
        end: last_line,
    })
}

/// Partition body lines `[0, open_start)` into foldable sections.
///
/// Fence-aware ATX H2/H3 only; H1 is not a boundary. Empty body → no sections.
pub fn partition_sections(lines: &[&str], open_start: usize) -> Vec<FoldableSection> {
    let body_end = open_start.min(lines.len());
    if body_end == 0 {
        return Vec::new();
    }

    let mut in_fence = false;
    let mut starts: Vec<(usize, SectionKind)> = Vec::new();

    for i in 0..body_end {
        let line = lines[i];
        if is_fence_line(line) {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if let Some((level, text)) = parse_atx_h2_h3(line) {
            starts.push((
                i,
                SectionKind::Heading {
                    level,
                    text: text.to_string(),
                },
            ));
        }
    }

    if starts.is_empty() {
        return vec![FoldableSection {
            kind: SectionKind::Preamble,
            lines: LineRange {
                start: 0,
                end: body_end - 1,
            },
        }];
    }

    let mut sections = Vec::new();
    let first_heading_at = starts[0].0;
    if first_heading_at > 0 {
        sections.push(FoldableSection {
            kind: SectionKind::Preamble,
            lines: LineRange {
                start: 0,
                end: first_heading_at - 1,
            },
        });
    }

    for (idx, (start, kind)) in starts.iter().enumerate() {
        let end = if idx + 1 < starts.len() {
            starts[idx + 1].0 - 1
        } else {
            body_end - 1
        };
        sections.push(FoldableSection {
            kind: kind.clone(),
            lines: LineRange {
                start: *start,
                end,
            },
        });
    }
    sections
}

fn is_trailing_card(card: &MetaCard, lines: &[&str]) -> bool {
    let last_non_empty = lines.iter().rposition(|l| !l.trim().is_empty());
    match last_non_empty {
        None => false,
        Some(idx) => card.line_end >= idx,
    }
}

fn is_fence_line(line: &str) -> bool {
    line.trim_start().starts_with("```")
}

/// ATX H2/H3: `## ` / `### ` outside fences (caller tracks fences).
fn parse_atx_h2_h3(line: &str) -> Option<(u8, &str)> {
    let trimmed = line.trim_start();
    // H4+ must not become H2/H3 via prefix strip (`#### x`.strip_prefix("## ")).
    if trimmed.starts_with("####") {
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix("### ") {
        return Some((3, rest.trim_end()));
    }
    if let Some(rest) = trimmed.strip_prefix("## ") {
        return Some((2, rest.trim_end()));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines_of(src: &str) -> Vec<&str> {
        src.lines().collect()
    }

    /// @spec chat/focus-answer Open region: Trailing next alone opens from next through answer end
    #[test]
    fn trailing_next_alone_opens_from_next_through_answer_end() {
        // GIVEN a settled Answer under Focus whose only meta card is a trailing `next`
        let src = "\
## Body

Some prose.

> **next**
>
> `confirm`  do it
";
        // WHEN the open region is computed
        let open = open_region(src).expect("open region");
        let cards = parse_meta_cards(src);
        let next = cards.iter().find(|c| c.kind == MetaCardKind::Next).unwrap();
        let last = lines_of(src).len() - 1;

        // THEN the open region starts at the `next` card start
        // AND the open region ends at the last line of the Answer
        assert_eq!(open.start, next.line_start);
        assert_eq!(open.end, last);
    }

    /// @spec chat/focus-answer Open region: Write then preview then trailing next opens from write through next
    #[test]
    fn write_then_preview_then_trailing_next_opens_from_write_through_next() {
        // GIVEN a settled Answer under Focus with a `write` card, non-card preview lines, then
        // a trailing `next` card
        // AND no other meta card between that `write` and `next`
        let src = "\
## Context

Intro.

> **write**
>
> Proposal at path

# Preview Title

Preview body.

> **next**
>
> `confirm proposal`
";
        // WHEN the open region is computed
        let open = open_region(src).expect("open region");
        let cards = parse_meta_cards(src);
        let write = cards.iter().find(|c| c.kind == MetaCardKind::Write).unwrap();
        let next = cards.iter().find(|c| c.kind == MetaCardKind::Next).unwrap();

        // THEN the open region starts at the `write` card start
        // AND the open region ends at the `next` card end
        assert_eq!(open.start, write.line_start);
        assert_eq!(open.end, next.line_end);
    }

    /// @spec chat/focus-answer Open region: Meta card between write and trailing next leaves only next open
    #[test]
    fn meta_card_between_write_and_trailing_next_leaves_only_next_open() {
        // GIVEN a settled Answer under Focus with a `write` card, another meta card, then a
        // trailing `next` card
        let src = "\
## Context

> **write**
>
> earlier write

> **next**
>
> `mid`  not trailing because more follows

More prose after mid next.

> **next**
>
> `confirm`  real trailing
";
        // WHEN the open region is computed
        let open = open_region(src).expect("open region");
        let cards = parse_meta_cards(src);
        let trailing = cards
            .iter()
            .rev()
            .find(|c| c.kind == MetaCardKind::Next)
            .unwrap();
        let write = cards.iter().find(|c| c.kind == MetaCardKind::Write).unwrap();

        // THEN the open region starts at the trailing `next` card start
        // AND the open region does not include the earlier `write` card
        assert_eq!(open.start, trailing.line_start);
        assert!(!open.contains(write.line_start));
    }

    /// @spec chat/focus-answer Section partition: H2 and H3 outside fences start sections and H1 does not
    #[test]
    fn h2_and_h3_outside_fences_start_sections_and_h1_does_not() {
        // GIVEN Answer body lines before the open region that include an H1, an H2, and an H3
        // outside fences
        let body = "\
# Title H1

## Section Two

body

### Section Three

more
";
        let lines = lines_of(body);
        // WHEN sections are partitioned
        let sections = partition_sections(&lines, lines.len());

        // THEN the H2 and H3 each start a foldable section
        // AND the H1 does not start a foldable section
        let headings: Vec<_> = sections
            .iter()
            .filter_map(|s| match &s.kind {
                SectionKind::Heading { level, text } => Some((*level, text.as_str())),
                SectionKind::Preamble => None,
            })
            .collect();
        assert_eq!(headings, vec![(2, "Section Two"), (3, "Section Three")]);
        assert!(
            !sections
                .iter()
                .any(|s| matches!(&s.kind, SectionKind::Heading { level: 1, .. })),
            "H1 must not be a fold boundary"
        );
        // H1 lines belong to preamble before first H2
        assert!(matches!(sections[0].kind, SectionKind::Preamble));
    }

    /// @spec chat/focus-answer Section partition: Heading-like lines inside fences do not start sections
    #[test]
    fn heading_like_lines_inside_fences_do_not_start_sections() {
        // GIVEN Answer body lines before the open region that place an `##` line only inside a
        // fenced code block
        let body = "\
Preamble prose.

```
## Not A Section
```

Still preamble.
";
        let lines = lines_of(body);
        // WHEN sections are partitioned
        let sections = partition_sections(&lines, lines.len());

        // THEN that line does not start a foldable section
        assert_eq!(sections.len(), 1);
        assert!(matches!(sections[0].kind, SectionKind::Preamble));
    }

    /// @spec chat/focus-answer Section partition: Preamble before first H2 or H3 is one foldable section
    #[test]
    fn preamble_before_first_h2_or_h3_is_one_foldable_section() {
        // GIVEN Answer body lines before the open region with prose before the first H2 or H3
        let body = "\
Lead paragraph.

Another line.

## First
";
        let lines = lines_of(body);
        // WHEN sections are partitioned
        let sections = partition_sections(&lines, lines.len());

        // THEN those leading lines form exactly one preamble section
        let preambles: Vec<_> = sections
            .iter()
            .filter(|s| matches!(s.kind, SectionKind::Preamble))
            .collect();
        assert_eq!(preambles.len(), 1);
        assert_eq!(preambles[0].lines.start, 0);
        assert!(preambles[0].lines.end < lines.len() - 1);
    }

    /// @spec chat/focus-answer Section partition: No headings with open region yields one body section plus open region
    #[test]
    fn no_headings_with_open_region_yields_one_body_section_plus_open_region() {
        // GIVEN a Focus layout Answer with an open region and no H2 or H3 outside fences before
        // it
        let src = "\
Just prose without headings.

More lines.

> **next**
>
> `ok`
";
        // WHEN sections are partitioned
        let layout = focus_layout(src);
        let FocusLayout::Sectioned { sections, open } = layout else {
            panic!("expected Sectioned layout");
        };

        // THEN there is exactly one foldable section covering the body before the open region
        // AND the open region remains separate from that section
        assert_eq!(sections.len(), 1);
        assert!(matches!(sections[0].kind, SectionKind::Preamble));
        assert_eq!(sections[0].lines.end + 1, open.start);
        assert!(!sections[0].lines.contains(open.start));
    }

    /// @spec chat/focus-answer Section partition: Section ranges exclude open-region lines
    #[test]
    fn section_ranges_exclude_open_region_lines() {
        // GIVEN a Focus layout Answer with foldable body lines and an open region
        let src = "\
## Motivation

Text.

## Intent

More.

> **write**
>
> path

# Preview

> **next**
>
> `confirm`
";
        // WHEN sections are partitioned
        let layout = focus_layout(src);
        let FocusLayout::Sectioned { sections, open } = layout else {
            panic!("expected Sectioned");
        };

        // THEN no section range includes a line that belongs to the open region
        for s in &sections {
            for line in s.lines.start..=s.lines.end {
                assert!(
                    !open.contains(line),
                    "section {:?} line {line} overlaps open {:?}",
                    s.kind,
                    open
                );
            }
        }
    }
}
