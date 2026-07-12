//! Discovery of project-level and plugin-level slash commands / skills that
//! `claude` will accept.

use std::path::{Path, PathBuf};

use crate::provider::{SlashCommand, SlashCommandKind};

pub fn discover_commands(project_root: &Path) -> Vec<SlashCommand> {
    let mut commands = Vec::new();

    let project_cmds = project_root.join(".claude/commands");
    if project_cmds.is_dir() {
        scan_command_dir(&project_cmds, &mut commands);
    }

    if let Ok(home) = std::env::var("HOME") {
        let claude_dir = PathBuf::from(home).join(".claude");
        let settings_path = claude_dir.join("settings.json");
        if let Ok(settings_str) = std::fs::read_to_string(&settings_path)
            && let Ok(settings) = serde_json::from_str::<serde_json::Value>(&settings_str)
            && let Some(plugins) = settings["enabledPlugins"].as_object()
        {
            for (key, enabled) in plugins {
                if enabled.as_bool() != Some(true) {
                    continue;
                }
                if let Some((plugin_name, marketplace)) = key.rsplit_once('@') {
                    let plugin_dir = claude_dir
                        .join("plugins/marketplaces")
                        .join(marketplace)
                        .join("plugins")
                        .join(plugin_name);
                    let cmd_dir = plugin_dir.join("commands");
                    if cmd_dir.is_dir() {
                        scan_command_dir(&cmd_dir, &mut commands);
                    }
                    let skills_dir = plugin_dir.join("skills");
                    if skills_dir.is_dir() {
                        scan_skills_dir(&skills_dir, &mut commands);
                    }
                }
            }
        }
    }

    // Filesystem skills/commands only — Claude interactive TUI builtins
    // (clear/compact/cost/help/model) are not real duckboard handlers and must
    // not pollute Grok (or headless Claude) completion lists.

    // Discovery order is non-authoritative; presentation sort lives in duckboard.
    commands.sort_by(|a, b| a.name.cmp(&b.name));
    commands.dedup_by(|a, b| a.name == b.name);
    commands
}

fn scan_command_dir(dir: &Path, commands: &mut Vec<SlashCommand>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "md") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if name.is_empty() {
            continue;
        }
        let meta = parse_frontmatter_meta(&path);
        // Kind is re-tagged when duckboard merges the completion catalog.
        commands.push(SlashCommand {
            name,
            description: meta.description,
            kind: SlashCommandKind::Agent,
            order_key: meta.order_key,
        });
    }
}

fn scan_skills_dir(dir: &Path, commands: &mut Vec<SlashCommand>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let skill_file = path.join("SKILL.md");
        if !skill_file.exists() {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        if name.is_empty() {
            continue;
        }
        let meta = parse_frontmatter_meta(&skill_file);
        commands.push(SlashCommand {
            name,
            description: meta.description,
            kind: SlashCommandKind::Agent,
            order_key: meta.order_key,
        });
    }
}

struct FrontmatterMeta {
    description: String,
    order_key: Option<u32>,
}

fn parse_frontmatter_meta(path: &Path) -> FrontmatterMeta {
    let empty = FrontmatterMeta {
        description: String::new(),
        order_key: None,
    };
    let Ok(content) = std::fs::read_to_string(path) else {
        return empty;
    };
    let Some(body) = content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))
    else {
        return empty;
    };
    let Some(end) = body.find("\n---") else {
        return empty;
    };
    let frontmatter = &body[..end];

    let mut description = String::new();
    let mut order_key = None;
    for line in frontmatter.lines() {
        if let Some(desc) = line.strip_prefix("description:") {
            let desc = desc.trim().trim_matches('"');
            if !desc.is_empty() {
                description = desc.to_string();
            }
        } else if let Some(raw) = line.strip_prefix("order:") {
            order_key = parse_order_key(raw.trim().trim_matches('"'));
        }
    }
    FrontmatterMeta {
        description,
        order_key,
    }
}

/// Accept only `digits` or `digits . digit` (exactly one fractional digit).
/// Returns tenths: `"1"` → 10, `"3.1"` → 31.
fn parse_order_key(raw: &str) -> Option<u32> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    if let Some((whole, frac)) = raw.split_once('.') {
        if whole.is_empty() || frac.len() != 1 {
            return None;
        }
        if !whole.chars().all(|c| c.is_ascii_digit()) || !frac.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let major: u32 = whole.parse().ok()?;
        let minor: u32 = frac.parse().ok()?;
        major.checked_mul(10)?.checked_add(minor)
    } else {
        if !raw.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let major: u32 = raw.parse().ok()?;
        major.checked_mul(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TMP_SEQ: AtomicU64 = AtomicU64::new(0);

    fn tmp_cmd_dir() -> PathBuf {
        let n = TMP_SEQ.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("duckchat-discover-{n}"));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_cmd(dir: &Path, name: &str, body: &str) {
        fs::write(dir.join(format!("{name}.md")), body).unwrap();
    }

    fn discover_one(dir: &Path, name: &str) -> SlashCommand {
        let mut cmds = Vec::new();
        scan_command_dir(dir, &mut cmds);
        cmds.into_iter()
            .find(|c| c.name == name)
            .unwrap_or_else(|| panic!("missing command {name}"))
    }

    #[test]
    fn parse_order_key_accepts_integer_and_one_decimal() {
        assert_eq!(parse_order_key("1"), Some(10));
        assert_eq!(parse_order_key("7"), Some(70));
        assert_eq!(parse_order_key("0.5"), Some(5));
        assert_eq!(parse_order_key("3.1"), Some(31));
        assert_eq!(parse_order_key("6.2"), Some(62));
    }

    #[test]
    fn parse_order_key_rejects_invalid_forms() {
        assert_eq!(parse_order_key("3.10"), None);
        assert_eq!(parse_order_key("3.15"), None);
        assert_eq!(parse_order_key("1."), None);
        assert_eq!(parse_order_key(".5"), None);
        assert_eq!(parse_order_key("1e2"), None);
        assert_eq!(parse_order_key("-1"), None);
        assert_eq!(parse_order_key("3.1.2"), None);
        assert_eq!(parse_order_key(""), None);
        assert_eq!(parse_order_key("abc"), None);
    }

    // @spec chat/slash-commands Frontmatter order and description: Description from frontmatter is on the discovered command
    #[test]
    fn description_from_frontmatter_is_on_the_discovered_command() {
        // GIVEN a command markdown file whose frontmatter sets a non-empty description
        let dir = tmp_cmd_dir();
        write_cmd(
            &dir,
            "ds-explore",
            "---\ndescription: Orient and brainstorm\norder: 1\n---\nbody\n",
        );
        // WHEN slash commands are discovered from that file's directory
        let cmd = discover_one(&dir, "ds-explore");
        // THEN the catalog entry for that command name has that description text
        assert_eq!(cmd.description, "Orient and brainstorm");
        let _ = fs::remove_dir_all(&dir);
    }

    // @spec chat/slash-commands Frontmatter order and description: Integer order maps to tenths key
    #[test]
    fn integer_order_maps_to_tenths_key() {
        // GIVEN a command markdown file whose frontmatter sets order to an integer form such as 1
        let dir = tmp_cmd_dir();
        write_cmd(
            &dir,
            "ds-explore",
            "---\ndescription: Orient\norder: 1\n---\nbody\n",
        );
        // WHEN slash commands are discovered from that file's directory
        let cmd = discover_one(&dir, "ds-explore");
        // THEN the catalog entry for that command has order key 10 for order: 1 (value in tenths)
        assert_eq!(cmd.order_key, Some(10));
        let _ = fs::remove_dir_all(&dir);
    }

    // @spec chat/slash-commands Frontmatter order and description: One-decimal order maps to tenths key
    #[test]
    fn one_decimal_order_maps_to_tenths_key() {
        // GIVEN a command markdown file whose frontmatter sets order to a one-decimal form such as 3.1
        let dir = tmp_cmd_dir();
        write_cmd(
            &dir,
            "ds-verify",
            "---\ndescription: Phase-aware check\norder: 3.1\n---\nbody\n",
        );
        // WHEN slash commands are discovered from that file's directory
        let cmd = discover_one(&dir, "ds-verify");
        // THEN the catalog entry for that command has order key 31 for order: 3.1
        assert_eq!(cmd.order_key, Some(31));
        let _ = fs::remove_dir_all(&dir);
    }

    // @spec chat/slash-commands Frontmatter order and description: Invalid order forms yield no order key
    #[test]
    fn invalid_order_forms_yield_no_order_key() {
        // GIVEN a command markdown file whose frontmatter sets order to a rejected form such as 3.10, 1., or .5
        let dir = tmp_cmd_dir();
        for (name, order) in [("a", "3.10"), ("b", "1."), ("c", ".5")] {
            write_cmd(
                &dir,
                name,
                &format!("---\ndescription: x\norder: {order}\n---\nbody\n"),
            );
            // WHEN slash commands are discovered from that file's directory
            let cmd = discover_one(&dir, name);
            // THEN the catalog entry for that command has no order key
            assert_eq!(cmd.order_key, None, "order {order} should be rejected");
        }
        let _ = fs::remove_dir_all(&dir);
    }

    // @spec chat/slash-commands Frontmatter order and description: Missing order yields no order key
    #[test]
    fn missing_order_yields_no_order_key() {
        // GIVEN a command markdown file whose frontmatter has no order field
        let dir = tmp_cmd_dir();
        write_cmd(
            &dir,
            "ds-custom",
            "---\ndescription: Custom workflow\n---\nbody\n",
        );
        // WHEN slash commands are discovered from that file's directory
        let cmd = discover_one(&dir, "ds-custom");
        // THEN the catalog entry for that command has no order key
        assert_eq!(cmd.order_key, None);
        assert_eq!(cmd.description, "Custom workflow");
        let _ = fs::remove_dir_all(&dir);
    }
}
