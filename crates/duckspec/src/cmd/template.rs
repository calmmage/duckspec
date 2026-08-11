use std::fs;
use std::path::Path;

use super::common::find_duckspec_root;
use crate::content;

pub fn run(name: String) -> anyhow::Result<()> {
    let template =
        content::template(&name).ok_or_else(|| anyhow::anyhow!("unknown template: {name}"))?;

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

    fn archive_handoff_section() -> String {
        let content = content::template("archive").expect("embedded archive template");
        content
            .split("## Handoff")
            .nth(1)
            .expect("Handoff section")
            .split("## After write")
            .next()
            .expect("After write after Handoff")
            .to_string()
    }

    // @spec archive/path-scoped-commit Change-owned include set: Include set is dirty paths that belong to this change
    #[test]
    fn archive_handoff_include_set_is_change_owned_dirty_paths() {
        let handoff = archive_handoff_section();
        assert!(
            handoff.contains("path-scoped include set") || handoff.contains("path-scoped"),
            "handoff must name a path-scoped include set: {handoff}"
        );
        assert!(
            handoff.contains("belong") && handoff.contains("this") && handoff.contains("change"),
            "handoff must limit membership to this change: {handoff}"
        );
        assert!(
            handoff.contains("exclude")
                && (handoff.contains("other changes") || handoff.contains("unknown WIP")),
            "handoff must exclude other work: {handoff}"
        );
        assert!(
            handoff.contains("archive dir") || handoff.contains("caps/"),
            "handoff must list duckspec membership kinds: {handoff}"
        );
    }

    // @spec archive/path-scoped-commit Change-owned include set: Ambiguous membership never defaults to the whole dirty tree
    #[test]
    fn archive_handoff_ambiguous_membership_never_defaults_to_whole_tree() {
        let handoff = archive_handoff_section();
        assert!(
            handoff.contains("ambiguous") && handoff.contains("ask"),
            "handoff must require asking on ambiguous membership: {handoff}"
        );
        assert!(
            handoff.contains("never default") && handoff.contains("whole dirty tree")
                || handoff.contains("never default to\n     the whole dirty tree")
                || (handoff.contains("never default") && handoff.contains("whole dirty")),
            "handoff must ban whole-dirty-tree default on ambiguity: {handoff}"
        );
    }

    // @spec archive/path-scoped-commit Pre-commit visibility: Message and include set are shown before any VCS write
    #[test]
    fn archive_handoff_shows_message_and_include_set_before_vcs_write() {
        let handoff = archive_handoff_section();
        assert!(
            handoff.contains("commit message"),
            "handoff must propose a commit message: {handoff}"
        );
        assert!(
            handoff.contains("include set") && handoff.contains("message"),
            "handoff must show include set with the message: {handoff}"
        );
        assert!(
            handoff.contains("before any VCS write"),
            "handoff must show paths before any VCS write: {handoff}"
        );
    }

    // @spec archive/path-scoped-commit Commit offer and empty set: Nonempty include set offers commit
    #[test]
    fn archive_handoff_nonempty_include_set_offers_commit() {
        let handoff = archive_handoff_section();
        assert!(
            handoff.contains("`commit`"),
            "handoff must use the commit confirm token: {handoff}"
        );
        assert!(
            handoff.contains("only when the include set is\n   nonempty")
                || handoff.contains("only when the include set is nonempty")
                || (handoff.contains("nonempty") && handoff.contains("`commit`")),
            "handoff must offer commit only when include set is nonempty: {handoff}"
        );
    }

    // @spec archive/path-scoped-commit Commit offer and empty set: Empty include set reports no owned dirt and does not invent a commit
    #[test]
    fn archive_handoff_empty_include_set_does_not_invent_commit() {
        let handoff = archive_handoff_section();
        assert!(
            handoff.contains("nothing owned is dirty")
                || (handoff.contains("empty") && handoff.contains("dirty")),
            "handoff must report empty owned dirt: {handoff}"
        );
        assert!(
            handoff.contains("do not invent a\n   commit")
                || handoff.contains("do not invent a commit"),
            "handoff must forbid inventing a commit: {handoff}"
        );
        assert!(
            handoff.contains("omit") && handoff.contains("`commit`"),
            "handoff must omit commit token when empty: {handoff}"
        );
    }

    // @spec archive/path-scoped-commit Path-scoped execution: On commit, only the include set is committed
    #[test]
    fn archive_handoff_on_commit_only_include_set_is_committed() {
        let handoff = archive_handoff_section();
        assert!(
            handoff.contains("path-scoped")
                && (handoff.contains("include set") || handoff.contains("that include set")),
            "handoff must path-scope commit to the include set: {handoff}"
        );
        assert!(
            handoff.contains("whole-tree")
                || handoff.contains("entire dirty")
                || handoff.contains("whole dirty"),
            "handoff must ban whole-tree commit defaults: {handoff}"
        );
        assert!(
            handoff.contains("unowned") || handoff.contains("unrelated"),
            "handoff must leave unowned dirty out: {handoff}"
        );
    }

    // @spec archive/path-scoped-commit Path-scoped execution: Handoff never auto-commits without user commit
    #[test]
    fn archive_handoff_never_auto_commits_without_user_commit() {
        let handoff = archive_handoff_section();
        assert!(
            handoff.contains("Never auto-commit") || handoff.contains("never auto-commit"),
            "handoff must forbid auto-commit: {handoff}"
        );
        assert!(
            handoff.contains("wait for the user") || handoff.contains("wait for"),
            "handoff must wait for user choice: {handoff}"
        );
    }
}
