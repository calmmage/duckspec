//! Markdown → owned styled lines for the terminal chat pane.
//!
//! Uses pulldown-cmark. Tables fit to pane width; narrow panes wrap cells as
//! plain text. Meta cards from duckcore are drawn as bordered blocks.

use duckcore::meta_card::{MetaCardKind, parse_meta_cards};
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

/// Terminal style flags for one span.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SpanStyle {
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
    pub heading: bool,
    pub meta: bool,
}

/// One run of text with uniform style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyledSpan {
    pub text: String,
    pub style: SpanStyle,
}

/// One display row.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StyledLine {
    pub spans: Vec<StyledSpan>,
}

impl StyledLine {
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            spans: vec![StyledSpan {
                text: text.into(),
                style: SpanStyle::default(),
            }],
        }
    }

    pub fn styled(text: impl Into<String>, style: SpanStyle) -> Self {
        Self {
            spans: vec![StyledSpan {
                text: text.into(),
                style,
            }],
        }
    }

    pub fn as_plain(&self) -> String {
        self.spans.iter().map(|s| s.text.as_str()).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.spans.is_empty() || self.spans.iter().all(|s| s.text.is_empty())
    }
}

/// Render markdown for a pane of `width` columns (at least 1).
pub fn render(md: &str, width: usize) -> Vec<StyledLine> {
    let width = width.max(1);
    if md.is_empty() {
        return Vec::new();
    }

    let cards = parse_meta_cards(md);
    let source_lines: Vec<&str> = md.lines().collect();
    if source_lines.is_empty() {
        // Markdown with no newlines — still parse as one chunk.
        return render_md_chunk(md, width);
    }

    let mut in_card = vec![false; source_lines.len()];
    for card in &cards {
        for i in card.line_start..=card.line_end {
            if let Some(slot) = in_card.get_mut(i) {
                *slot = true;
            }
        }
    }

    let mut out = Vec::new();
    let mut i = 0;
    while i < source_lines.len() {
        if in_card[i] {
            let card = cards
                .iter()
                .find(|c| i >= c.line_start && i <= c.line_end)
                .expect("card flag implies card");
            let slice = &source_lines[card.line_start..=card.line_end];
            out.extend(render_meta_card(card.kind, slice, width));
            i = card.line_end + 1;
        } else {
            let start = i;
            while i < source_lines.len() && !in_card[i] {
                i += 1;
            }
            let chunk = source_lines[start..i].join("\n");
            if !chunk.trim().is_empty() {
                out.extend(render_md_chunk(&chunk, width));
            }
        }
    }
    out
}

/// Plain-string lines for viewport / scroll math.
pub fn render_plain_lines(md: &str, width: usize) -> Vec<String> {
    render(md, width)
        .into_iter()
        .map(|l| l.as_plain())
        .collect()
}

fn render_meta_card(kind: MetaCardKind, lines: &[&str], width: usize) -> Vec<StyledLine> {
    let title = match kind {
        MetaCardKind::Write => " write ",
        MetaCardKind::Next => " next ",
    };
    let inner_w = width.saturating_sub(4).max(8);
    let mut out = Vec::new();
    let top = format!("┌─{title}{}", "─".repeat(inner_w.saturating_sub(title.len())));
    out.push(StyledLine::styled(
        truncate_fit(&top, width),
        SpanStyle {
            meta: true,
            bold: true,
            ..Default::default()
        },
    ));
    for line in lines {
        let content = line
            .strip_prefix('>')
            .map(|s| s.strip_prefix(' ').unwrap_or(s))
            .unwrap_or(line);
        let body = format!("│ {content}");
        out.push(StyledLine::styled(
            truncate_fit(&body, width),
            SpanStyle {
                meta: true,
                ..Default::default()
            },
        ));
    }
    let bot = format!("└{}", "─".repeat(width.saturating_sub(1).max(1)));
    out.push(StyledLine::styled(
        truncate_fit(&bot, width),
        SpanStyle {
            meta: true,
            ..Default::default()
        },
    ));
    out
}

fn render_md_chunk(md: &str, width: usize) -> Vec<StyledLine> {
    // Pre-handle GFM pipe tables as whole source lines for reliable fit.
    let lines: Vec<&str> = md.lines().collect();
    if let Some(regions) = split_table_regions(&lines) {
        let mut out = Vec::new();
        for region in regions {
            match region {
                Region::Text(chunk) => out.extend(render_pulldown(&chunk, width)),
                Region::Table(table_lines) => out.extend(render_table(&table_lines, width)),
            }
        }
        return out;
    }
    render_pulldown(md, width)
}

enum Region {
    Text(String),
    Table(Vec<String>),
}

fn split_table_regions(lines: &[&str]) -> Option<Vec<Region>> {
    let mut regions = Vec::new();
    let mut i = 0;
    let mut saw_table = false;
    while i < lines.len() {
        if is_table_header_row(lines, i) {
            saw_table = true;
            let start = i;
            i += 2; // header + separator
            while i < lines.len() && is_table_data_row(lines[i]) {
                i += 1;
            }
            let table: Vec<String> = lines[start..i].iter().map(|s| (*s).to_string()).collect();
            regions.push(Region::Table(table));
        } else {
            let start = i;
            while i < lines.len() && !is_table_header_row(lines, i) {
                i += 1;
            }
            regions.push(Region::Text(lines[start..i].join("\n")));
        }
    }
    saw_table.then_some(regions)
}

fn is_table_header_row(lines: &[&str], i: usize) -> bool {
    lines
        .get(i)
        .is_some_and(|l| l.contains('|') && lines.get(i + 1).is_some_and(|s| is_separator_row(s)))
}

fn is_separator_row(line: &str) -> bool {
    let t = line.trim();
    if !t.contains('|') || !t.contains('-') {
        return false;
    }
    t.chars()
        .all(|c| c == '|' || c == '-' || c == ':' || c == ' ')
}

fn is_table_data_row(line: &str) -> bool {
    line.contains('|') && !is_separator_row(line)
}

fn parse_row(line: &str) -> Vec<String> {
    let t = line.trim().trim_matches('|');
    t.split('|').map(|c| c.trim().to_string()).collect()
}

fn render_table(table_lines: &[String], width: usize) -> Vec<StyledLine> {
    if table_lines.len() < 2 {
        return table_lines.iter().map(StyledLine::plain).collect();
    }
    let rows: Vec<Vec<String>> = table_lines
        .iter()
        .filter(|l| !is_separator_row(l))
        .map(|l| parse_row(l))
        .collect();
    if rows.is_empty() {
        return Vec::new();
    }
    let cols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if cols == 0 {
        return Vec::new();
    }

    // Fit columns: min 3 each; if total needed > width, plain wrap cells.
    let mut col_w: Vec<usize> = (0..cols)
        .map(|c| {
            rows.iter()
                .filter_map(|r| r.get(c))
                .map(|s| s.chars().count().max(1))
                .max()
                .unwrap_or(1)
                .min(width.saturating_sub(cols).max(3))
        })
        .collect();
    let gaps = cols.saturating_sub(1) * 3; // " │ "
    let mut total: usize = col_w.iter().sum::<usize>() + gaps;
    if total > width || width < cols * 3 + gaps {
        // Narrow: one cell per line, plain wrap.
        let mut out = Vec::new();
        for (ri, row) in rows.iter().enumerate() {
            for (ci, cell) in row.iter().enumerate() {
                let label = if ri == 0 {
                    format!("[{ci}] {cell}")
                } else {
                    cell.clone()
                };
                for wrapped in wrap_text(&label, width) {
                    out.push(StyledLine::plain(wrapped));
                }
            }
            if ri + 1 < rows.len() {
                out.push(StyledLine::plain("─".repeat(width.min(20))));
            }
        }
        return out;
    }

    // Shrink widest columns until fit.
    while total > width {
        if let Some((i, _)) = col_w
            .iter()
            .enumerate()
            .filter(|(_, w)| **w > 3)
            .max_by_key(|(_, w)| *w)
        {
            col_w[i] -= 1;
            total -= 1;
        } else {
            break;
        }
    }

    let mut out = Vec::new();
    for (ri, row) in rows.iter().enumerate() {
        let mut parts = Vec::new();
        for (c, w) in col_w.iter().enumerate() {
            let cell = row.get(c).map(String::as_str).unwrap_or("");
            parts.push(pad_truncate(cell, *w));
        }
        let line = parts.join(" │ ");
        let style = if ri == 0 {
            SpanStyle {
                bold: true,
                ..Default::default()
            }
        } else {
            SpanStyle::default()
        };
        out.push(StyledLine::styled(truncate_fit(&line, width), style));
        if ri == 0 {
            let rule = col_w
                .iter()
                .map(|w| "─".repeat(*w))
                .collect::<Vec<_>>()
                .join("─┼─");
            out.push(StyledLine::plain(truncate_fit(&rule, width)));
        }
    }
    out
}

fn render_pulldown(md: &str, width: usize) -> Vec<StyledLine> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(md, options);

    let mut out: Vec<StyledLine> = Vec::new();
    let mut cur_spans: Vec<StyledSpan> = Vec::new();
    let mut style = SpanStyle::default();
    let mut list_depth: usize = 0;
    let mut in_code_block = false;
    let mut pending_text = String::new();

    let flush_line = |spans: &mut Vec<StyledSpan>, out: &mut Vec<StyledLine>| {
        if spans.is_empty() {
            out.push(StyledLine::default());
        } else {
            out.push(StyledLine {
                spans: std::mem::take(spans),
            });
        }
    };

    let push_text = |text: &str, style: SpanStyle, spans: &mut Vec<StyledSpan>| {
        if text.is_empty() {
            return;
        }
        if let Some(last) = spans.last_mut()
            && last.style == style
        {
            last.text.push_str(text);
        } else {
            spans.push(StyledSpan {
                text: text.to_string(),
                style,
            });
        }
    };

    for event in parser {
        match event {
            Event::Start(Tag::Heading { .. }) => {
                style.heading = true;
                style.bold = true;
            }
            Event::End(TagEnd::Heading(_)) => {
                flush_line(&mut cur_spans, &mut out);
                style.heading = false;
                style.bold = false;
            }
            Event::Start(Tag::Paragraph) => {}
            Event::End(TagEnd::Paragraph) => {
                // Wrap accumulated paragraph.
                let plain: String = cur_spans.iter().map(|s| s.text.as_str()).collect();
                if !plain.is_empty() {
                    let styles = cur_spans.clone();
                    cur_spans.clear();
                    for wrapped in wrap_spans(&styles, width) {
                        out.push(wrapped);
                    }
                } else {
                    flush_line(&mut cur_spans, &mut out);
                }
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                in_code_block = true;
                let label = if lang.is_empty() {
                    "```".into()
                } else {
                    format!("```{lang}")
                };
                out.push(StyledLine::styled(
                    label,
                    SpanStyle {
                        code: true,
                        ..Default::default()
                    },
                ));
            }
            Event::Start(Tag::CodeBlock(_)) => {
                in_code_block = true;
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                if !pending_text.is_empty() {
                    for line in pending_text.lines() {
                        for w in wrap_text(line, width.saturating_sub(2).max(1)) {
                            out.push(StyledLine::styled(
                                format!("  {w}"),
                                SpanStyle {
                                    code: true,
                                    ..Default::default()
                                },
                            ));
                        }
                    }
                    pending_text.clear();
                }
                out.push(StyledLine::styled(
                    "```",
                    SpanStyle {
                        code: true,
                        ..Default::default()
                    },
                ));
            }
            Event::Start(Tag::Item) => {
                let indent = "  ".repeat(list_depth.saturating_sub(1));
                push_text(&format!("{indent}• "), style, &mut cur_spans);
            }
            Event::End(TagEnd::Item) if !cur_spans.is_empty() => {
                let styles = std::mem::take(&mut cur_spans);
                for wrapped in wrap_spans(&styles, width) {
                    out.push(wrapped);
                }
            }
            Event::End(TagEnd::Item) => {}
            Event::Start(Tag::List(_)) => {
                list_depth += 1;
            }
            Event::End(TagEnd::List(_)) => {
                list_depth = list_depth.saturating_sub(1);
            }
            Event::Start(Tag::Emphasis) => style.italic = true,
            Event::End(TagEnd::Emphasis) => style.italic = false,
            Event::Start(Tag::Strong) => style.bold = true,
            Event::End(TagEnd::Strong) => style.bold = false,
            Event::Code(code) => {
                push_text(
                    &code,
                    SpanStyle {
                        code: true,
                        ..style
                    },
                    &mut cur_spans,
                );
            }
            Event::Text(text) => {
                if in_code_block {
                    pending_text.push_str(&text);
                } else {
                    push_text(&text, style, &mut cur_spans);
                }
            }
            // Terminal chat: treat soft breaks as hard lines so pre-split
            // transcript body lines and agent output keep their line structure.
            Event::SoftBreak | Event::HardBreak => {
                if !cur_spans.is_empty() {
                    let styles = std::mem::take(&mut cur_spans);
                    for wrapped in wrap_spans(&styles, width) {
                        out.push(wrapped);
                    }
                } else {
                    flush_line(&mut cur_spans, &mut out);
                }
            }
            Event::Rule => {
                out.push(StyledLine::plain("─".repeat(width.min(40))));
            }
            // Tables handled in render_table path; ignore table events here.
            Event::Start(Tag::Table(_))
            | Event::End(TagEnd::Table)
            | Event::Start(Tag::TableHead)
            | Event::End(TagEnd::TableHead)
            | Event::Start(Tag::TableRow)
            | Event::End(TagEnd::TableRow)
            | Event::Start(Tag::TableCell)
            | Event::End(TagEnd::TableCell) => {}
            _ => {}
        }
    }
    if !cur_spans.is_empty() {
        for wrapped in wrap_spans(&cur_spans, width) {
            out.push(wrapped);
        }
    }
    out
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    if text.chars().count() <= width {
        return vec![text.to_string()];
    }
    let mut out = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        if cur.is_empty() {
            if word.chars().count() > width {
                for chunk in chars_chunks(word, width) {
                    out.push(chunk);
                }
            } else {
                cur = word.to_string();
            }
            continue;
        }
        if cur.chars().count() + 1 + word.chars().count() <= width {
            cur.push(' ');
            cur.push_str(word);
        } else {
            out.push(std::mem::take(&mut cur));
            if word.chars().count() > width {
                for chunk in chars_chunks(word, width) {
                    out.push(chunk);
                }
            } else {
                cur = word.to_string();
            }
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

fn chars_chunks(s: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for ch in s.chars() {
        if cur.chars().count() >= width {
            out.push(std::mem::take(&mut cur));
        }
        cur.push(ch);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn wrap_spans(spans: &[StyledSpan], width: usize) -> Vec<StyledLine> {
    let plain: String = spans.iter().map(|s| s.text.as_str()).collect();
    if plain.chars().count() <= width {
        return vec![StyledLine {
            spans: spans.to_vec(),
        }];
    }
    // Word-wrap plain text; re-apply first span style for simplicity.
    let style = spans
        .first()
        .map(|s| s.style)
        .unwrap_or_default();
    wrap_text(&plain, width)
        .into_iter()
        .map(|t| StyledLine::styled(t, style))
        .collect()
}

fn pad_truncate(s: &str, width: usize) -> String {
    let n = s.chars().count();
    if n > width {
        s.chars().take(width.saturating_sub(1).max(1)).collect::<String>() + "…"
    } else {
        format!("{s}{}", " ".repeat(width - n))
    }
}

fn truncate_fit(s: &str, width: usize) -> String {
    let n = s.chars().count();
    if n <= width {
        s.to_string()
    } else {
        s.chars().take(width.saturating_sub(1).max(1)).collect::<String>() + "…"
    }
}
