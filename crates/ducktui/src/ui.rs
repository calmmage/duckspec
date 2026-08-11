//! ratatui drawing for the shell screens.

use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};

use crate::md_render::{SpanStyle, StyledLine};
use crate::shell::{Overlay, PaneFocus, PickerFocus, Screen, Shell, TurnState};

pub fn draw(frame: &mut Frame, shell: &mut Shell) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(area);

    match shell.screen {
        Screen::ProjectPicker => draw_project_picker(frame, chunks[0], shell),
        Screen::Work => draw_work(frame, chunks[0], shell),
        Screen::Settings => draw_settings(frame, chunks[0], shell),
    }
    draw_status(frame, chunks[1], shell);

    if shell.overlay.is_some() {
        draw_overlay(frame, area, shell);
    }
}

fn draw_project_picker(frame: &mut Frame, area: Rect, shell: &Shell) {
    let path_focused = matches!(shell.picker.focus, PickerFocus::Path);
    let path_marker = if path_focused { "▸ " } else { "  " };
    let mut lines = vec![
        Line::from("Project picker"),
        Line::from(""),
        Line::from(format!(
            "{path_marker}path: {}",
            if shell.picker.path_input.is_empty() {
                "(type a project path)"
            } else {
                shell.picker.path_input.as_str()
            }
        )),
        Line::from(""),
        Line::from("Recent (shared with duckboard):"),
    ];
    if shell.picker.recents.is_empty() {
        lines.push(Line::from("  (none)"));
    } else {
        for (i, path) in shell.picker.recents.iter().enumerate() {
            let sel = matches!(shell.picker.focus, PickerFocus::Recent(j) if j == i);
            let marker = if sel { "▸ " } else { "  " };
            lines.push(Line::from(format!("{marker}{}", path.display())));
        }
    }
    lines.push(Line::from(""));
    lines.push(Line::from(
        "Enter bind · j/k or ↑↓ move · Tab path/recents · type path · q quit",
    ));
    let body = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" ducktui "));
    frame.render_widget(body, area);
}

fn draw_work(frame: &mut Frame, area: Rect, shell: &mut Shell) {
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    let nav_title = if shell.focus == PaneFocus::Navigator {
        " navigator * "
    } else {
        " navigator "
    };
    let chat_title = if shell.focus == PaneFocus::Chat {
        " chat * "
    } else {
        " chat "
    };

    let nav_lines: Vec<Line> = shell
        .navigator
        .visible_rows()
        .into_iter()
        .map(|row| {
            let selected = shell.navigator.selected.as_ref() == Some(&row.id);
            let mut text = String::new();
            if selected {
                text.push_str("▸ ");
            } else {
                text.push_str("  ");
            }
            if let Some(phase) = &row.phase {
                text.push_str(phase);
                text.push(' ');
            }
            text.push_str(&row.label);
            if let Some(n) = row.session_badge {
                text.push_str(&format!(" ({n})"));
            }
            Line::from(text)
        })
        .collect();
    let nav = Paragraph::new(nav_lines)
        .block(Block::default().borders(Borders::ALL).title(nav_title));
    let chat_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(panes[1]);

    // Sync viewport to pane geometry for markdown wrap and scroll window.
    shell.chat.viewport.width = (chat_chunks[0].width.saturating_sub(2) as usize).max(1);
    shell.chat.viewport.height = (chat_chunks[0].height.saturating_sub(2) as usize).max(1);

    let mut transcript_lines: Vec<Line> = shell
        .chat
        .visible_styled_lines()
        .into_iter()
        .map(styled_to_line)
        .collect();
    if transcript_lines.is_empty() {
        let hint = match &shell.scope_label {
            Some(scope) => format!("chat bound: {scope}"),
            None => "select a scope".into(),
        };
        transcript_lines.push(Line::from(hint));
    }
    let transcript = Paragraph::new(transcript_lines)
        .block(Block::default().borders(Borders::ALL).title(chat_title));
    frame.render_widget(transcript, chat_chunks[0]);

    let hints: Vec<String> = shell
        .chat
        .numbered_hints_active()
        .into_iter()
        .map(|h| format!("{}:{}", h.index, h.label))
        .collect();
    let hint_line = if hints.is_empty() {
        String::new()
    } else {
        hints.join("  ")
    };
    frame.render_widget(
        Paragraph::new(hint_line).block(Block::default().borders(Borders::ALL).title(" actions ")),
        chat_chunks[1],
    );

    let composer = Paragraph::new(shell.chat.composer.as_str())
        .block(Block::default().borders(Borders::ALL).title(" composer "));
    frame.render_widget(composer, chat_chunks[2]);

    frame.render_widget(nav, panes[0]);
}

fn draw_settings(frame: &mut Frame, area: Rect, shell: &Shell) {
    let mut lines = vec![
        Line::from("Settings (shared config)"),
        Line::from("Theme is terminal-local and not stored here."),
        Line::from(""),
    ];
    for (i, field) in shell.settings.fields.iter().enumerate() {
        let sel = i == shell.settings.cursor;
        let marker = if sel { "▸ " } else { "  " };
        let value = match field.as_str() {
            "default_model" => shell
                .settings
                .default_model
                .as_ref()
                .map(|m| format!("{}/{}", m.harness, m.model))
                .unwrap_or_else(|| "(none — cycle when catalog loads)".into()),
            "agent_input_hints" => {
                if shell.settings.agent_input_hints {
                    "on".into()
                } else {
                    "off".into()
                }
            }
            other if other.starts_with("oneshot:") => {
                let harness = &other["oneshot:".len()..];
                shell
                    .settings
                    .oneshot_models
                    .get(harness)
                    .cloned()
                    .unwrap_or_else(|| "(default)".into())
            }
            other => other.to_string(),
        };
        let label = match field.as_str() {
            "default_model" => "Default model".to_string(),
            "agent_input_hints" => "Agent input hints".to_string(),
            other if other.starts_with("oneshot:") => {
                format!("Oneshot ({})", &other["oneshot:".len()..])
            }
            other => other.to_string(),
        };
        lines.push(Line::from(format!("{marker}{label}: {value}")));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(
        "j/k move · ←/→ or Enter cycle · Space toggle hints · Esc leave",
    ));
    let body = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(" settings "));
    frame.render_widget(body, area);
}

fn draw_status(frame: &mut Frame, area: Rect, shell: &Shell) {
    let bar = shell.status_bar();
    let scope = bar.scope.as_deref().unwrap_or("—");
    let model = bar
        .model
        .as_ref()
        .map(|m| format!("{}/{}", m.harness, m.model))
        .unwrap_or_else(|| "—".into());
    let ctx = match bar.context.window {
        Some(w) if w > 0 => format!("{}/{}", bar.context.used, w),
        _ => format!("{}", bar.context.used),
    };
    let turn = match bar.turn {
        TurnState::Idle => "idle",
        TurnState::Streaming => "streaming",
        TurnState::AwaitingChoice => "awaiting",
    };
    let line = Line::from(vec![
        Span::raw(format!(" scope:{scope} ")),
        Span::raw(format!("model:{model} ")),
        Span::raw(format!("ctx:{ctx} ")),
        Span::styled(
            format!("turn:{turn}"),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn draw_overlay(frame: &mut Frame, area: Rect, shell: &Shell) {
    let Some(overlay) = shell.overlay else {
        return;
    };
    let title = match overlay {
        Overlay::ModelPicker => " model picker ",
        Overlay::SlashPalette => " slash ",
        Overlay::SessionSwitcher => " sessions ",
        Overlay::QuickIdea => " quick idea ",
        Overlay::Help => " help ",
    };
    let body_lines: Vec<Line> = match overlay {
        Overlay::Help => vec![
            Line::from("Global"),
            Line::from("  q / Ctrl+c  quit"),
            Line::from("  ? / F1      help"),
            Line::from("  Ctrl+,      settings"),
            Line::from("  Ctrl+m      model picker"),
            Line::from("  Ctrl+s      session switcher"),
            Line::from("  Ctrl+i      quick idea"),
            Line::from("  Esc         close overlay / leave settings"),
            Line::from(""),
            Line::from("Work screen"),
            Line::from("  Tab         toggle navigator / chat focus"),
            Line::from("  Nav j/k ↑↓  move · Enter activate"),
            Line::from("  Nav r       refresh tree · Ctrl+r anywhere on work screen"),
            Line::from("  Chat PgUp/PgDn scroll · ↑↓ (empty composer)"),
            Line::from("  Chat e      expand/collapse thinking/activity"),
            Line::from("  Chat /      slash palette · Alt+Enter newline"),
            Line::from("  1–9         next / fast-response hints"),
            Line::from(""),
            Line::from("No middle content browser — edit artifacts in your editor."),
        ],
        Overlay::SlashPalette | Overlay::ModelPicker | Overlay::SessionSwitcher => {
            let mut lines = Vec::new();
            if shell.overlay_list.rows.is_empty() {
                lines.push(Line::from("(empty)"));
            } else {
                for (i, row) in shell.overlay_list.rows.iter().enumerate() {
                    let marker = if i == shell.overlay_list.cursor {
                        "▸ "
                    } else {
                        "  "
                    };
                    lines.push(Line::from(format!("{marker}{row}")));
                }
            }
            lines.push(Line::from(""));
            let hint = match overlay {
                Overlay::SlashPalette => "j/k move · Enter select · type filter · Esc close",
                Overlay::ModelPicker => "j/k move · Enter set model · Esc close",
                Overlay::SessionSwitcher => "j/k move · Enter load · + New session · Esc close",
                _ => "Esc close",
            };
            lines.push(Line::from(hint));
            lines
        }
        Overlay::QuickIdea => {
            let mut lines = vec![
                Line::from("Quick idea (shared inbox under project data/ideas/)"),
                Line::from(""),
                Line::from(if shell.quick_idea_text.is_empty() {
                    "(type idea text…)".to_string()
                } else {
                    shell.quick_idea_text.clone()
                }),
                Line::from(""),
                Line::from("Enter save · Esc cancel"),
            ];
            if let Some(status) = &shell.overlay_status {
                lines.push(Line::from(status.clone()));
            }
            lines
        }
    };
    let popup = centered_rect(70, 50, area);
    frame.render_widget(Clear, popup);
    let widget = Paragraph::new(body_lines)
        .wrap(Wrap { trim: false })
        .block(Block::default().borders(Borders::ALL).title(title));
    frame.render_widget(widget, popup);
}

fn styled_to_line(line: StyledLine) -> Line<'static> {
    if line.spans.is_empty() {
        return Line::from("");
    }
    let spans: Vec<Span> = line
        .spans
        .into_iter()
        .map(|s| Span::styled(s.text, span_style(s.style)))
        .collect();
    Line::from(spans)
}

fn span_style(s: SpanStyle) -> Style {
    let mut style = Style::default();
    if s.bold || s.heading {
        style = style.add_modifier(Modifier::BOLD);
    }
    if s.italic {
        style = style.add_modifier(Modifier::ITALIC);
    }
    if s.code {
        style = style.fg(Color::Cyan);
    }
    if s.meta {
        style = style.fg(Color::Yellow);
    }
    style
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
