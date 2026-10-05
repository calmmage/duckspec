//! Trailing A–B–C option runs in assistant prose → next-action seeds.
//!
//! When no trailing `next` meta card is present, a conservative parse of the
//! last assistant message may yield 2–3 `NextAction`s for empty-composer ghost
//! / Tab / Enter (hosts wire this via [`next_actions_from_assistant`]).

use crate::meta_card::{MAX_NEXT_ACTIONS, NextAction, trailing_next_actions};

/// Max characters for a prose option `send` string (ellipsis if longer).
pub const PROSE_OPTION_SEND_BUDGET: usize = 96;

/// Non-empty session next actions: trailing `next` meta card wins; otherwise
/// trailing A–B–C prose options (0 or 2–3).
pub fn next_actions_from_assistant(source: &str) -> Vec<NextAction> {
    let meta = trailing_next_actions(source);
    if !meta.is_empty() {
        return meta;
    }
    extract_trailing_abc_options(source)
}

/// Extract a trailing strict A→B→C… option run from `source`.
///
/// Returns empty unless at least two consecutive letter options are found at
/// the end of the message (blank lines after the run are allowed). At most
/// [`MAX_NEXT_ACTIONS`] entries. Skips fenced code interiors.
pub fn extract_trailing_abc_options(source: &str) -> Vec<NextAction> {
    let lines: Vec<&str> = source.lines().collect();
    if lines.is_empty() {
        return Vec::new();
    }

    let in_fence = fence_interior_flags(&lines);

    let mut options: Vec<(usize, char, String)> = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if in_fence[i] {
            continue;
        }
        if let Some((letter, body)) = parse_option_line(line) {
            options.push((i, letter, body));
        }
    }
    if options.len() < 2 {
        return Vec::new();
    }

    // Trailing: last non-empty, non-fence line must be the last option line
    // (blanks after the run are fine).
    let last_content = lines.iter().enumerate().rev().find_map(|(i, l)| {
        if in_fence[i] {
            return None;
        }
        if l.trim().is_empty() {
            return None;
        }
        Some(i)
    });
    let Some(last_content) = last_content else {
        return Vec::new();
    };
    let last_opt_line = options.last().map(|&(i, _, _)| i).unwrap();
    if last_opt_line != last_content {
        return Vec::new();
    }

    // Grow a suffix of `options` that is strict A→B→C… with only blanks/fence
    // between consecutive option lines.
    let end = options.len();
    let mut start = end - 1;

    let last_letter = options[start].1.to_ascii_uppercase();
    if !('A'..='C').contains(&last_letter) {
        return Vec::new();
    }

    while start > 0 {
        let prev = start - 1;
        let (li, ll, _) = options[prev];
        let (ri, rl, _) = options[start];
        let ll = ll.to_ascii_uppercase();
        let rl = rl.to_ascii_uppercase();
        if !('A'..='C').contains(&ll) || !('A'..='C').contains(&rl) {
            break;
        }
        if (ll as u8) + 1 != rl as u8 {
            break;
        }
        if !only_blank_or_fence_between(&lines, &in_fence, li, ri) {
            break;
        }
        start = prev;
    }

    let slice = &options[start..end];
    if slice.is_empty() {
        return Vec::new();
    }
    // Must start at A and continue A,B,C…
    let mut valid_len = 0;
    for (i, &(_, l, _)) in slice.iter().enumerate() {
        let expected = (b'A' + i as u8) as char;
        if l.to_ascii_uppercase() != expected {
            break;
        }
        valid_len = i + 1;
    }
    actions_from_slice(&slice[..valid_len])
}

fn only_blank_or_fence_between(
    lines: &[&str],
    in_fence: &[bool],
    left: usize,
    right: usize,
) -> bool {
    if right <= left + 1 {
        return true;
    }
    for i in (left + 1)..right {
        if in_fence.get(i).copied().unwrap_or(false) {
            continue;
        }
        if !lines[i].trim().is_empty() {
            return false;
        }
    }
    true
}

fn actions_from_slice(slice: &[(usize, char, String)]) -> Vec<NextAction> {
    if slice.len() < 2 {
        return Vec::new();
    }
    slice
        .iter()
        .take(MAX_NEXT_ACTIONS)
        .map(|(_, letter, body)| {
            let letter = letter.to_ascii_uppercase();
            let send = clip_send(&format!("{letter} — {body}"));
            NextAction { send, reason: None }
        })
        .collect()
}

fn fence_interior_flags(lines: &[&str]) -> Vec<bool> {
    let mut flags = vec![false; lines.len()];
    let mut in_fence = false;
    for (i, line) in lines.iter().enumerate() {
        if is_fence_line(line) {
            flags[i] = true;
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            flags[i] = true;
        }
    }
    flags
}

fn is_fence_line(line: &str) -> bool {
    line.trim_start().starts_with("```")
}

/// Parse one option line → (letter, body) if it matches the ABC option shape.
fn parse_option_line(line: &str) -> Option<(char, String)> {
    let mut s = line.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(rest) = s.strip_prefix('|') {
        s = rest.trim_start();
    }
    s = strip_list_prefix(s);

    let (letter, rest) = parse_letter_prefix(s)?;
    if !('A'..='C').contains(&letter.to_ascii_uppercase()) {
        return None;
    }
    let rest = rest.trim_start();
    let body = strip_separator(rest)?;
    let body = body.trim().trim_end_matches('|').trim();
    if body.is_empty() {
        return None;
    }
    Some((letter, body.to_string()))
}

fn strip_list_prefix(s: &str) -> &str {
    let s = s.trim_start();
    if let Some(rest) = s.strip_prefix("- ") {
        return rest.trim_start();
    }
    if let Some(rest) = s.strip_prefix("* ") {
        return rest.trim_start();
    }
    if let Some(rest) = s.strip_prefix("• ") {
        return rest.trim_start();
    }
    let bytes = s.as_bytes();
    if !bytes.is_empty() && bytes[0].is_ascii_digit() {
        let mut i = 0;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i < bytes.len() && (bytes[i] == b'.' || bytes[i] == b')') {
            let rest = s[i + 1..].trim_start();
            if !rest.is_empty() {
                return rest;
            }
        }
    }
    s
}

fn parse_letter_prefix(s: &str) -> Option<(char, &str)> {
    let s = s.trim_start();
    if let Some(rest) = s.strip_prefix("**") {
        let mut chars = rest.chars();
        let c = chars.next()?;
        if !c.is_ascii_alphabetic() {
            return None;
        }
        let rest = chars.as_str();
        // Require closing ** after single letter when present
        let rest = if let Some(r) = rest.strip_prefix("**") {
            r
        } else {
            // **A — body (missing close) — still accept if separator follows via rest
            rest
        };
        return Some((c, rest));
    }
    let mut chars = s.chars();
    let c = chars.next()?;
    if !c.is_ascii_alphabetic() {
        return None;
    }
    Some((c, chars.as_str()))
}

fn strip_separator(s: &str) -> Option<&str> {
    let s = s.trim_start();
    for sep in ["—", "–", "-", ":", "|"] {
        if let Some(rest) = s.strip_prefix(sep) {
            return Some(rest);
        }
    }
    None
}

fn clip_send(s: &str) -> String {
    let count = s.chars().count();
    if count <= PROSE_OPTION_SEND_BUDGET {
        return s.to_string();
    }
    let keep = PROSE_OPTION_SEND_BUDGET.saturating_sub(1);
    let truncated: String = s.chars().take(keep).collect();
    format!("{truncated}…")
}

#[cfg(test)]
mod tests {
    use super::*;

    // @spec chat/default-prompts Prose option next-actions: Bullet ABC run yields three sends
    #[test]
    fn bullet_abc_run_yields_three_sends() {
        let src = "\
How do you want to resolve finding 2?

- **A** — harness-scoped resume (recommended)
- **B** — clear id on harness change only
- **C** — accept / document
";
        let got = extract_trailing_abc_options(src);
        assert_eq!(got.len(), 3);
        assert_eq!(got[0].send, "A — harness-scoped resume (recommended)");
        assert_eq!(got[1].send, "B — clear id on harness change only");
        assert_eq!(got[2].send, "C — accept / document");
        assert!(got.iter().all(|a| a.reason.is_none()));
    }

    // @spec chat/default-prompts Prose option next-actions: Meta next wins over prose ABC
    #[test]
    fn meta_next_wins_over_prose_abc() {
        let src = "\
- **A** — one
- **B** — two

> **next**
>
> `/ds-spec`
> `confirm`
";
        let got = next_actions_from_assistant(src);
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].send, "/ds-spec");
        assert_eq!(got[1].send, "confirm");
    }

    // @spec chat/default-prompts Prose option next-actions: Prose fills when no meta next
    #[test]
    fn prose_fills_when_no_meta_next() {
        let src = "\
Pick:

- **A** — first path
- **B** — second path
";
        let got = next_actions_from_assistant(src);
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].send, "A — first path");
        assert_eq!(got[1].send, "B — second path");
    }

    // @spec chat/default-prompts Prose option next-actions: Single option yields empty
    #[test]
    fn single_option_yields_empty() {
        let src = "- **A** — only one\n";
        assert!(extract_trailing_abc_options(src).is_empty());
    }

    // @spec chat/default-prompts Prose option next-actions: Fence interior options ignored
    #[test]
    fn fence_interior_options_ignored() {
        let src = "\
```
- **A** — inside
- **B** — inside too
```

Outside text.
";
        assert!(extract_trailing_abc_options(src).is_empty());
    }

    // @spec chat/default-prompts Prose option next-actions: Non-trailing interrupted run ignored
    #[test]
    fn non_trailing_interrupted_run_ignored() {
        let src = "\
- **A** — early
- **B** — early

Then more discussion without options at the end.
";
        assert!(extract_trailing_abc_options(src).is_empty());
    }

    // @spec chat/default-prompts Prose option next-actions: Strict letter order required
    #[test]
    fn strict_letter_order_required() {
        let src = "\
- **A** — one
- **C** — skips B
";
        assert!(extract_trailing_abc_options(src).is_empty());
    }

    // @spec chat/default-prompts Prose option next-actions: Long body is clipped
    #[test]
    fn long_body_is_clipped() {
        let long = "x".repeat(120);
        let src = format!("- **A** — {long}\n- **B** — short\n");
        let got = extract_trailing_abc_options(&src);
        assert_eq!(got.len(), 2);
        assert!(got[0].send.chars().count() <= PROSE_OPTION_SEND_BUDGET);
        assert!(got[0].send.ends_with('…'));
        assert!(got[0].send.starts_with("A — "));
    }

    // @spec chat/default-prompts Prose option next-actions: Cap at three options
    #[test]
    fn cap_at_three_options() {
        let src = "\
- **A** — a
- **B** — b
- **C** — c
";
        let got = extract_trailing_abc_options(src);
        assert!(got.len() <= MAX_NEXT_ACTIONS);
        assert_eq!(got.len(), 3);
    }

    // @spec chat/default-prompts Prose option next-actions: Bare letter dash lines work
    #[test]
    fn bare_letter_dash_lines_work() {
        let src = "\
A — alpha path
B — beta path
";
        let got = extract_trailing_abc_options(src);
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].send, "A — alpha path");
        assert_eq!(got[1].send, "B — beta path");
    }
}
