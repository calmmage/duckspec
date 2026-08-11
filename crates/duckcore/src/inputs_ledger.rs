//! Derived package ledger of verbatim user chat under a change.
//!
//! Full rebuild from `chat_store` sessions into `duckspec/changes/<name>/inputs.md`.

use std::path::{Path, PathBuf};

use crate::chat_store::{self, ChatMessage, ChatSession, ContentBlock, Role};

/// Absolute path: `{project_root}/duckspec/changes/{change}/inputs.md`.
pub fn inputs_path(project_root: &Path, change: &str) -> PathBuf {
    project_root
        .join("duckspec")
        .join("changes")
        .join(change)
        .join("inputs.md")
}

/// True when the change package has a non-empty inputs ledger on disk.
pub fn inputs_ledger_is_present(project_root: &Path, change: &str) -> bool {
    let path = inputs_path(project_root, change);
    std::fs::metadata(&path)
        .map(|m| m.len() > 0)
        .unwrap_or(false)
}

/// `{project_root}/duckspec/changes/{change}/`.
pub fn change_dir(project_root: &Path, change: &str) -> PathBuf {
    project_root.join("duckspec").join("changes").join(change)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportOutcome {
    Written,
    Unchanged,
    SkippedNoChangeDir,
    SkippedEmpty,
}

/// Load sessions for `change`, render, write atomically if needed.
///
/// Skips when the change directory is missing, when there are no qualifying
/// user messages (and removes a stale empty ledger if present), or when the
/// on-disk file already matches the rebuild.
pub fn export_inputs_for_change(
    change: &str,
    project_root: &Path,
) -> anyhow::Result<ExportOutcome> {
    let dir = change_dir(project_root, change);
    if !dir.is_dir() {
        return Ok(ExportOutcome::SkippedNoChangeDir);
    }

    let sessions = chat_store::load_sessions_for(change, Some(project_root));
    let rendered = render_inputs_markdown(change, &sessions);
    let path = inputs_path(project_root, change);

    if rendered.is_empty() {
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        return Ok(ExportOutcome::SkippedEmpty);
    }

    if path.is_file()
        && let Ok(existing) = std::fs::read_to_string(&path)
        && existing == rendered
    {
        return Ok(ExportOutcome::Unchanged);
    }

    write_atomic(&path, rendered.as_bytes())?;
    Ok(ExportOutcome::Written)
}

/// Atomic write for the ledger (temp beside target + rename).
fn write_atomic(path: &Path, data: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("md.tmp");
    if let Err(e) = std::fs::write(&tmp, data) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(())
}

/// Full rebuild markdown from sessions (any order; sorted by creation ascending).
///
/// Returns empty string when there are no qualifying user messages.
pub fn render_inputs_markdown(change: &str, sessions: &[ChatSession]) -> String {
    let mut ordered: Vec<&ChatSession> = sessions.iter().collect();
    ordered.sort_by_key(|s| s.created_at_nanos);

    let mut body = String::new();
    for session in ordered {
        let mut session_block = String::new();
        for msg in &session.messages {
            if let Some(text) = qualifying_user_text(msg) {
                if !session_block.is_empty() {
                    session_block.push('\n');
                }
                if !msg.timestamp.is_empty() {
                    session_block.push_str(&msg.timestamp);
                    session_block.push('\n');
                    session_block.push('\n');
                }
                session_block.push_str(&text);
                session_block.push('\n');
            }
        }
        if session_block.is_empty() {
            continue;
        }
        if !body.is_empty() {
            body.push('\n');
        }
        body.push_str("## ");
        body.push_str(&session_label(session));
        body.push('\n');
        body.push('\n');
        body.push_str(&session_block);
    }

    if body.is_empty() {
        return String::new();
    }

    format!(
        "# Raw user inputs\n\n\
         Verbatim user messages for change `{change}`. Not a summary.\n\n\
         {body}"
    )
}

fn session_label(session: &ChatSession) -> String {
    if let Some(title) = session.title.as_ref().filter(|t| !t.is_empty()) {
        return title.clone();
    }
    if !session.display_name.is_empty() {
        return session.display_name.clone();
    }
    session.id.clone()
}

/// Non-priming user Text blocks concatenated in order; `None` if not included.
fn qualifying_user_text(msg: &ChatMessage) -> Option<String> {
    if msg.role != Role::User || msg.is_priming {
        return None;
    }
    let mut parts: Vec<&str> = Vec::new();
    for block in &msg.content {
        if let ContentBlock::Text(t) = block {
            parts.push(t.as_str());
        }
    }
    if parts.is_empty() {
        return None;
    }
    Some(parts.concat())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat_store::{ChatMessage, ChatSession, ContentBlock, Role};

    fn user_msg(text: &str) -> ChatMessage {
        ChatMessage {
            role: Role::User,
            content: vec![ContentBlock::Text(text.into())],
            timestamp: String::new(),
            is_priming: false,
        }
    }

    fn priming_msg(text: &str) -> ChatMessage {
        ChatMessage {
            role: Role::User,
            content: vec![ContentBlock::Text(text.into())],
            timestamp: String::new(),
            is_priming: true,
        }
    }

    fn assistant_msg(text: &str) -> ChatMessage {
        ChatMessage {
            role: Role::Assistant,
            content: vec![ContentBlock::Text(text.into())],
            timestamp: String::new(),
            is_priming: false,
        }
    }

    fn system_msg(text: &str) -> ChatMessage {
        ChatMessage {
            role: Role::System,
            content: vec![ContentBlock::Text(text.into())],
            timestamp: String::new(),
            is_priming: false,
        }
    }

    fn session(scope: &str, created: i128, messages: Vec<ChatMessage>) -> ChatSession {
        let mut s = ChatSession::new(scope.into());
        s.created_at_nanos = created;
        s.id = created.to_string();
        s.display_name = format!("session-{created}");
        s.messages = messages;
        s
    }

    /// @spec chat/inputs-ledger Package path: Ledger path is under the change directory
    #[test]
    fn ledger_path_is_under_the_change_directory() {
        let root = Path::new("/proj");
        let path = inputs_path(root, "my-change");
        assert_eq!(
            path,
            PathBuf::from("/proj/duckspec/changes/my-change/inputs.md")
        );
    }

    /// @spec chat/inputs-ledger Verbatim user-only content: Non-priming user text is present verbatim
    #[test]
    fn non_priming_user_text_is_present_verbatim() {
        let exact = "preserve raw user inputs\n- exact phrasing!";
        let s = session("raw-user-inputs", 1, vec![user_msg(exact)]);
        let md = render_inputs_markdown("raw-user-inputs", &[s]);
        assert!(
            md.contains(exact),
            "ledger must contain user text exactly; got:\n{md}"
        );
    }

    /// @spec chat/inputs-ledger Verbatim user-only content: Priming user messages are omitted
    #[test]
    fn priming_user_messages_are_omitted() {
        let priming = "AGENTS.md conventions inject";
        let real = "real human intent";
        let s = session(
            "c",
            1,
            vec![priming_msg(priming), user_msg(real)],
        );
        let md = render_inputs_markdown("c", &[s]);
        assert!(!md.contains(priming), "priming leaked:\n{md}");
        assert!(md.contains(real));
    }

    /// @spec chat/inputs-ledger Verbatim user-only content: Non-user roles are omitted
    #[test]
    fn non_user_roles_are_omitted() {
        let user = "only me";
        let s = session(
            "c",
            1,
            vec![
                assistant_msg("assistant secret"),
                system_msg("system secret"),
                ChatMessage {
                    role: Role::Assistant,
                    content: vec![
                        ContentBlock::Reasoning("think secret".into()),
                        ContentBlock::ToolUse {
                            id: "1".into(),
                            name: "Bash".into(),
                            input: "rm -rf /".into(),
                        },
                        ContentBlock::ToolResult {
                            id: "1".into(),
                            name: "Bash".into(),
                            output: "tool secret".into(),
                        },
                    ],
                    timestamp: String::new(),
                    is_priming: false,
                },
                user_msg(user),
            ],
        );
        let md = render_inputs_markdown("c", &[s]);
        assert!(md.contains(user));
        assert!(!md.contains("assistant secret"));
        assert!(!md.contains("system secret"));
        assert!(!md.contains("think secret"));
        assert!(!md.contains("rm -rf /"));
        assert!(!md.contains("tool secret"));
    }

    /// @spec chat/inputs-ledger Verbatim user-only content: Sessions appear in creation order
    #[test]
    fn sessions_appear_in_creation_order() {
        let later = session("c", 200, vec![user_msg("SECOND")]);
        let earlier = session("c", 100, vec![user_msg("FIRST")]);
        // Pass later first so sort must reorder.
        let md = render_inputs_markdown("c", &[later, earlier]);
        let i_first = md.find("FIRST").expect("FIRST");
        let i_second = md.find("SECOND").expect("SECOND");
        assert!(
            i_first < i_second,
            "earlier session must appear first:\n{md}"
        );
    }

    use crate::test_support::{FsTmp, with_home};

    fn ensure_change_dir(root: &Path, name: &str) {
        std::fs::create_dir_all(change_dir(root, name)).unwrap();
    }

    /// @spec chat/inputs-ledger Empty and missing change: No qualifying messages leaves no ledger file
    #[test]
    fn no_qualifying_messages_leaves_no_ledger_file() {
        let tmp = FsTmp::new();
        with_home(tmp.path(), || {
            let root = tmp.path().join("proj");
            std::fs::create_dir_all(&root).unwrap();
            ensure_change_dir(&root, "empty-change");
            let mut s = session("empty-change", 1, vec![priming_msg("AGENTS only")]);
            chat_store::save_session(&s, Some(&root)).unwrap();
            // Save already exports; also call export explicitly.
            let outcome = export_inputs_for_change("empty-change", &root).unwrap();
            assert_eq!(outcome, ExportOutcome::SkippedEmpty);
            assert!(!inputs_path(&root, "empty-change").exists());
            // Stale file is removed.
            std::fs::write(inputs_path(&root, "empty-change"), "stale").unwrap();
            s.messages.clear();
            chat_store::save_session(&s, Some(&root)).unwrap();
            assert!(!inputs_path(&root, "empty-change").exists());
        });
    }

    /// @spec chat/inputs-ledger Empty and missing change: Missing change directory skips export
    #[test]
    fn missing_change_directory_skips_export() {
        let tmp = FsTmp::new();
        with_home(tmp.path(), || {
            let root = tmp.path().join("proj");
            std::fs::create_dir_all(&root).unwrap();
            let mut s = session("ghost", 1, vec![user_msg("hello")]);
            chat_store::save_session(&s, Some(&root)).unwrap();
            let outcome = export_inputs_for_change("ghost", &root).unwrap();
            assert_eq!(outcome, ExportOutcome::SkippedNoChangeDir);
            assert!(!root.join("duckspec").join("changes").join("ghost").exists());
            assert!(!inputs_path(&root, "ghost").exists());
            let _ = &mut s;
        });
    }

    /// @spec chat/inputs-ledger Fresh full rebuild: Save refreshes the ledger from all sessions
    #[test]
    fn save_refreshes_the_ledger_from_all_sessions() {
        let tmp = FsTmp::new();
        with_home(tmp.path(), || {
            let root = tmp.path().join("proj");
            std::fs::create_dir_all(&root).unwrap();
            ensure_change_dir(&root, "multi");
            let mut a = session("multi", 10, vec![user_msg("from-A")]);
            a.id = "10".into();
            chat_store::save_session(&a, Some(&root)).unwrap();
            let mut b = session("multi", 20, vec![user_msg("from-B")]);
            b.id = "20".into();
            chat_store::save_session(&b, Some(&root)).unwrap();
            let md = std::fs::read_to_string(inputs_path(&root, "multi")).unwrap();
            assert!(md.contains("from-A"), "missing A:\n{md}");
            assert!(md.contains("from-B"), "missing B:\n{md}");
        });
    }

    /// @spec chat/inputs-ledger Fresh full rebuild: Unchanged content does not rewrite the file
    #[test]
    fn unchanged_content_does_not_rewrite_the_file() {
        let tmp = FsTmp::new();
        with_home(tmp.path(), || {
            let root = tmp.path().join("proj");
            std::fs::create_dir_all(&root).unwrap();
            ensure_change_dir(&root, "stable");
            let mut s = session("stable", 1, vec![user_msg("same")]);
            s.id = "1".into();
            chat_store::save_session(&s, Some(&root)).unwrap();
            let path = inputs_path(&root, "stable");
            let meta1 = std::fs::metadata(&path).unwrap();
            let mtime1 = meta1.modified().unwrap();
            std::thread::sleep(std::time::Duration::from_millis(20));
            let outcome = export_inputs_for_change("stable", &root).unwrap();
            assert_eq!(outcome, ExportOutcome::Unchanged);
            let meta2 = std::fs::metadata(&path).unwrap();
            assert_eq!(meta2.modified().unwrap(), mtime1);
        });
    }

    /// @spec chat/inputs-ledger Non-change scopes: Non-change scope save does not create inputs.md under changes
    #[test]
    fn non_change_scope_save_does_not_create_inputs_under_changes() {
        let tmp = FsTmp::new();
        with_home(tmp.path(), || {
            let root = tmp.path().join("proj");
            std::fs::create_dir_all(&root).unwrap();
            // Ensure duckspec/changes exists empty so a wrong write would be visible.
            std::fs::create_dir_all(root.join("duckspec").join("changes")).unwrap();
            for scope in ["caps", "codex", "exploration-999"] {
                let mut s = session(scope, 1, vec![user_msg("noise")]);
                s.id = format!("{scope}-1");
                chat_store::save_session(&s, Some(&root)).unwrap();
            }
            let changes = root.join("duckspec").join("changes");
            let any_inputs = std::fs::read_dir(&changes)
                .unwrap()
                .flatten()
                .any(|e| e.path().join("inputs.md").is_file() || e.path().extension().is_some_and(|x| x == "md"));
            assert!(
                !any_inputs,
                "non-change saves must not create package inputs"
            );
            assert!(!inputs_path(&root, "caps").exists());
            assert!(!inputs_path(&root, "codex").exists());
            assert!(!inputs_path(&root, "exploration-999").exists());
        });
    }

    /// @spec chat/inputs-ledger Promotion export: Post-promotion export includes migrated user text
    #[test]
    fn post_promotion_export_includes_migrated_user_text() {
        let tmp = FsTmp::new();
        with_home(tmp.path(), || {
            let root = tmp.path().join("proj");
            std::fs::create_dir_all(&root).unwrap();
            let exp_id = "exploration-42";
            let change = "promoted-change";
            ensure_change_dir(&root, change);
            let mut s = session(exp_id, 5, vec![user_msg("from exploration")]);
            s.id = "5".into();
            chat_store::save_session(&s, Some(&root)).unwrap();
            // Exploration save must not have written under the change yet with that text
            // until merge; change dir exists so accidental export with wrong scope wouldn't
            // include exploration text. After merge_scope:
            chat_store::merge_scope(exp_id, change, Some(&root));
            let md = std::fs::read_to_string(inputs_path(&root, change)).unwrap();
            assert!(
                md.contains("from exploration"),
                "promoted user text missing:\n{md}"
            );
        });
    }
}
