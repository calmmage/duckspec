use std::fs;
use std::path::Path;

use super::common::find_duckspec_root;
use crate::content;

pub fn run(name: String) -> anyhow::Result<()> {
    let template = content::template(&name)
        .ok_or_else(|| anyhow::anyhow!("unknown template: {name}"))?;

    let duckspec_root = find_duckspec_root().ok();
    let before = duckspec_root
        .as_ref()
        .and_then(|root| read_hook_content(root, &name, "before"));
    let after = duckspec_root
        .as_ref()
        .and_then(|root| read_hook_content(root, &name, "after"));

    let output = apply_hooks(template, before.as_deref(), after.as_deref());
    print!("{output}");

    Ok(())
}

/// Read a hook file and return its contents (trimmed). Returns `None` if the
/// file is missing, unreadable, or contains only whitespace.
fn read_hook_content(duckspec_root: &Path, stage: &str, position: &str) -> Option<String> {
    let path = duckspec_root.join(format!("hooks/{stage}-{position}.md"));
    let content = fs::read_to_string(path).ok()?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Replace `## Before write` and `## After write` placeholders. When a hook
/// is present, emit the header followed by the hook body. When absent, drop
/// the placeholder line entirely.
fn apply_hooks(template: &str, before: Option<&str>, after: Option<&str>) -> String {
    let mut output = String::new();
    let mut lines = template.lines().peekable();

    while let Some(line) = lines.next() {
        if line.trim() == "## Before write" {
            skip_section(&mut lines);
            if let Some(content) = before {
                output.push_str("## Before write\n\n");
                output.push_str(content);
                output.push_str("\n\n");
            }
        } else if line.trim() == "## After write" {
            skip_section(&mut lines);
            if let Some(content) = after {
                output.push_str("## After write\n\n");
                output.push_str(content);
                output.push('\n');
            }
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }

    output
}

/// Advance the iterator past the current section (until the next heading
/// of equal or higher level, or EOF).
fn skip_section(lines: &mut std::iter::Peekable<std::str::Lines<'_>>) {
    while let Some(next) = lines.peek() {
        if next.starts_with("## ") || next.starts_with("# ") {
            break;
        }
        lines.next();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hooks_removed_when_absent() {
        let template = "\
# Template

## Before write

## Instructions

Do stuff.

## After write
";
        let result = apply_hooks(template, None, None);
        assert_eq!(
            result,
            "\
# Template

## Instructions

Do stuff.

"
        );
    }

    #[test]
    fn hooks_inserted_with_headers_when_present() {
        let template = "\
# Template

## Before write

## Instructions

Do stuff.

## After write
";
        let result = apply_hooks(
            template,
            Some("Pre content here."),
            Some("Post content here."),
        );
        assert_eq!(
            result,
            "\
# Template

## Before write

Pre content here.

## Instructions

Do stuff.

## After write

Post content here.
"
        );
    }

    #[test]
    fn hook_without_h1_is_rendered_verbatim() {
        let template = "\
# Template

## Before write

## Body
";
        let result = apply_hooks(template, Some("Just text, no heading."), None);
        assert_eq!(
            result,
            "\
# Template

## Before write

Just text, no heading.

## Body
"
        );
    }

    #[test]
    fn empty_hook_file_treated_as_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let hooks_dir = tmp.path().join("hooks");
        fs::create_dir(&hooks_dir).unwrap();
        fs::write(hooks_dir.join("step-before.md"), "   \n\n  \t\n").unwrap();

        let result = read_hook_content(tmp.path(), "step", "before");
        assert!(result.is_none());
    }

    #[test]
    fn read_hook_content_returns_trimmed_body() {
        let tmp = tempfile::tempdir().unwrap();
        let hooks_dir = tmp.path().join("hooks");
        fs::create_dir(&hooks_dir).unwrap();
        fs::write(
            hooks_dir.join("step-before.md"),
            "\n\n  hello world  \n\n\n",
        )
        .unwrap();

        let result = read_hook_content(tmp.path(), "step", "before");
        assert_eq!(result.as_deref(), Some("hello world"));
    }

    #[test]
    fn every_stock_template_has_hook_placeholders() {
        let mut count = 0;
        for (name, body) in content::templates() {
            count += 1;
            assert!(
                body.contains("## Before write"),
                "{name} is missing `## Before write` placeholder"
            );
            assert!(
                body.contains("## After write"),
                "{name} is missing `## After write` placeholder"
            );
        }
        assert!(count > 0, "expected at least one template");
    }

    #[test]
    fn archive_handoff_requires_path_scoped_commit() {
        let path = Path::new(TEMPLATE_DIR).join("archive.md");
        let content = fs::read_to_string(&path).expect("archive template");
        let handoff = content
            .split("## Handoff")
            .nth(1)
            .expect("Handoff section")
            .split("## After write")
            .next()
            .expect("After write after Handoff");
        assert!(
            handoff.contains("path-scoped") || handoff.contains("path-scoped include"),
            "archive handoff must require a path-scoped include set"
        );
        assert!(
            handoff.contains("include set") || handoff.contains("path set"),
            "archive handoff must surface the include/path set"
        );
        assert!(
            handoff.contains("do not invent") || handoff.contains("do not invent a"),
            "archive handoff must forbid inventing a commit when empty"
        );
        assert!(
            handoff.contains("whole-tree")
                || handoff.contains("entire dirty")
                || handoff.contains("whole dirty"),
            "archive handoff must ban whole-tree commit defaults"
        );
        assert!(
            handoff.contains("`commit`"),
            "archive handoff must still use the commit confirm token"
        );
    }
}
