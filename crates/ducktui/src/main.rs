//! ducktui — terminal UI for duckspec (navigate scopes + drive agents).

use std::io::{self, stdout};
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyEvent, KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use duckcore::agent::AgentEvent;
use duckcore::watcher::FileEvent;
use futures::StreamExt;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tokio::sync::mpsc;

use ducktui::chat_pane::{ChatAction, ComposerKey};
use ducktui::runtime::{self, AgentRuntime};
use ducktui::shell::{KeyEffect, Overlay, PaneFocus, Screen, Shell};

#[derive(Debug)]
enum AppEvent {
    Input(Event),
    Tick,
    Agent(AgentEvent),
    Files(Vec<FileEvent>),
}

enum DispatchResult {
    Continue,
    Quit,
    /// Host should ensure a project file watcher is running.
    ProjectBound,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(io::stderr)
        .with_max_level(tracing::Level::INFO)
        .init();

    // Catalog refresh is blocking (provider handshakes); do it once at start.
    duckcore::agent::refresh_model_catalog();

    let project = std::env::args().nth(1).map(PathBuf::from);
    let mut shell = Shell::new(None);
    if let Some(path) = project {
        shell.bind_project_and_promote(path);
    } else {
        shell.refresh_picker_recents();
    }
    shell.model = shell
        .shared_config
        .default_model
        .clone()
        .or(shell.model.clone());

    enable_raw_mode().context("enable raw mode")?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut shell).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    shell: &mut Shell,
) -> anyhow::Result<()> {
    let (tx, mut rx) = mpsc::channel::<AppEvent>(256);
    let mut agent_rt = AgentRuntime::default();
    let mut watcher_root: Option<PathBuf> = None;

    // Crossterm input
    let input_tx = tx.clone();
    tokio::spawn(async move {
        let mut events = EventStream::new();
        while let Some(Ok(ev)) = events.next().await {
            if input_tx.send(AppEvent::Input(ev)).await.is_err() {
                break;
            }
        }
    });

    // Streaming cadence tick (~30ms design)
    let tick_tx = tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(30));
        loop {
            interval.tick().await;
            if tick_tx.send(AppEvent::Tick).await.is_err() {
                break;
            }
        }
    });

    let (agent_tx, mut agent_rx) = mpsc::channel::<AgentEvent>(256);
    {
        let app_tx = tx.clone();
        tokio::spawn(async move {
            while let Some(ev) = agent_rx.recv().await {
                if app_tx.send(AppEvent::Agent(ev)).await.is_err() {
                    break;
                }
            }
        });
    }

    ensure_watcher(shell, &tx, &mut watcher_root);

    loop {
        terminal.draw(|f| ducktui::ui::draw(f, shell))?;

        let Some(event) = rx.recv().await else {
            break;
        };

        match event {
            AppEvent::Tick => {
                runtime::materialize_if_dirty(shell, &mut agent_rt);
            }
            AppEvent::Input(Event::Key(key)) => {
                match dispatch_key(shell, &mut agent_rt, agent_tx.clone(), key) {
                    DispatchResult::Quit => break,
                    DispatchResult::ProjectBound => {
                        ensure_watcher(shell, &tx, &mut watcher_root);
                    }
                    DispatchResult::Continue => {}
                }
            }
            AppEvent::Input(_) => {}
            AppEvent::Agent(ev) => {
                runtime::apply_agent_event(shell, &mut agent_rt, ev);
            }
            AppEvent::Files(batch) => {
                runtime::apply_file_events(shell, &batch);
            }
        }

        if shell.quit_requested {
            break;
        }
    }

    Ok(())
}

/// Spawn the project watcher once per bound root.
fn ensure_watcher(
    shell: &Shell,
    tx: &mpsc::Sender<AppEvent>,
    watcher_root: &mut Option<PathBuf>,
) {
    let Some(root) = shell.project_root.clone() else {
        return;
    };
    if watcher_root.as_ref() == Some(&root) {
        return;
    }
    *watcher_root = Some(root.clone());
    let (file_tx, mut file_rx) = mpsc::channel::<Vec<FileEvent>>(32);
    let _watch = runtime::start_watcher(root, file_tx);
    let app_tx = tx.clone();
    tokio::spawn(async move {
        while let Some(batch) = file_rx.recv().await {
            if app_tx.send(AppEvent::Files(batch)).await.is_err() {
                break;
            }
        }
    });
}

/// Map crossterm keys into shell codes / chat composer.
fn dispatch_key(
    shell: &mut Shell,
    agent_rt: &mut AgentRuntime,
    agent_tx: mpsc::Sender<AgentEvent>,
    key: KeyEvent,
) -> DispatchResult {
    // Open overlays: list navigation and slash filter typing.
    if shell.overlay.is_some() {
        match key.code {
            KeyCode::Esc => {
                let _ = shell.handle_key("escape");
                return DispatchResult::Continue;
            }
            KeyCode::Up => {
                let _ = shell.handle_key("up");
                return DispatchResult::Continue;
            }
            KeyCode::Down => {
                let _ = shell.handle_key("down");
                return DispatchResult::Continue;
            }
            KeyCode::Enter => {
                let _ = shell.handle_key("enter");
                return DispatchResult::Continue;
            }
            KeyCode::Backspace => {
                let _ = shell.handle_key("backspace");
                return DispatchResult::Continue;
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                // j/k navigate list overlays; type into quick-idea buffer.
                if matches!(c, 'j' | 'k')
                    && !matches!(shell.overlay, Some(Overlay::QuickIdea))
                {
                    let _ = shell.handle_key(if c == 'j' { "j" } else { "k" });
                } else {
                    let _ = shell.handle_key(&format!("char:{c}"));
                }
                return DispatchResult::Continue;
            }
            _ => {}
        }
    }

    // Chat viewport + composer keys when work screen, chat focused, no overlay.
    if shell.screen == Screen::Work && shell.focus == PaneFocus::Chat && shell.overlay.is_none() {
        // Scroll / expand never go into the composer buffer.
        match key.code {
            KeyCode::PageUp => {
                let _ = shell.handle_key("page-up");
                return DispatchResult::Continue;
            }
            KeyCode::PageDown => {
                let _ = shell.handle_key("page-down");
                return DispatchResult::Continue;
            }
            KeyCode::Up if shell.chat.composer.is_empty() => {
                let _ = shell.handle_key("scroll-up");
                return DispatchResult::Continue;
            }
            KeyCode::Down if shell.chat.composer.is_empty() => {
                let _ = shell.handle_key("scroll-down");
                return DispatchResult::Continue;
            }
            KeyCode::Char('e')
                if shell.chat.composer.is_empty()
                    && !key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                let _ = shell.handle_key("expand");
                return DispatchResult::Continue;
            }
            _ => {}
        }

        match key.code {
            KeyCode::Enter
                if key.modifiers.contains(KeyModifiers::ALT)
                    || key.modifiers.contains(KeyModifiers::SHIFT) =>
            {
                shell.chat.handle_composer_key(ComposerKey::AltEnter);
                return DispatchResult::Continue;
            }
            KeyCode::Enter => {
                let action = shell.chat.handle_composer_key(ComposerKey::Enter);
                if let ChatAction::Submitted(text) = action {
                    runtime::activate_hint(
                        shell,
                        agent_rt,
                        agent_tx,
                        ChatAction::Submitted(text),
                    );
                }
                return DispatchResult::Continue;
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                if c.is_ascii_digit() && c != '0' && shell.chat.composer.is_empty() {
                    let n = c.to_digit(10).unwrap_or(0) as u8;
                    if !shell.chat.numbered_hints_active().is_empty() {
                        let action = shell.chat.activate_number(n);
                        runtime::activate_hint(shell, agent_rt, agent_tx, action);
                        return DispatchResult::Continue;
                    }
                }
                let action = shell.chat.handle_composer_key(ComposerKey::Char(c));
                if matches!(action, ChatAction::OpenSlashPalette) {
                    shell.open_overlay(Overlay::SlashPalette);
                }
                return DispatchResult::Continue;
            }
            KeyCode::Backspace => {
                shell.chat.handle_composer_key(ComposerKey::Backspace);
                if !shell.chat.slash_palette_open && shell.overlay == Some(Overlay::SlashPalette) {
                    shell.close_overlay();
                }
                return DispatchResult::Continue;
            }
            _ => {}
        }
    }

    // Project picker: feed path entry characters before global layer.
    if shell.screen == Screen::ProjectPicker {
        match key.code {
            KeyCode::Enter => {
                return match shell.handle_key("enter") {
                    KeyEffect::ProjectBound => DispatchResult::ProjectBound,
                    KeyEffect::Quit => DispatchResult::Quit,
                    _ => DispatchResult::Continue,
                };
            }
            KeyCode::Backspace => {
                let _ = shell.handle_key("backspace");
                return DispatchResult::Continue;
            }
            KeyCode::Up => {
                let _ = shell.handle_key("up");
                return DispatchResult::Continue;
            }
            KeyCode::Down => {
                let _ = shell.handle_key("down");
                return DispatchResult::Continue;
            }
            KeyCode::Tab => {
                let _ = shell.handle_key("tab");
                return DispatchResult::Continue;
            }
            KeyCode::Char(c)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && c != 'q'
                    && c != '?' =>
            {
                // j/k navigation when not typing a path fragment that needs them
                // always available via arrows; j/k navigate when path empty or on
                // recent focus — still allow typing j/k into the path field when
                // path is focused (char path).
                if matches!(c, 'j' | 'k')
                    && (shell.picker.path_input.is_empty()
                        || !matches!(shell.picker.focus, ducktui::shell::PickerFocus::Path))
                {
                    let _ = shell.handle_key(if c == 'j' { "j" } else { "k" });
                    return DispatchResult::Continue;
                }
                let _ = shell.handle_key(&format!("char:{c}"));
                return DispatchResult::Continue;
            }
            _ => {}
        }
    }

    // Settings: field navigation before generic mapping.
    if shell.screen == Screen::Settings {
        match key.code {
            KeyCode::Up => {
                let _ = shell.handle_key("up");
                return DispatchResult::Continue;
            }
            KeyCode::Down => {
                let _ = shell.handle_key("down");
                return DispatchResult::Continue;
            }
            KeyCode::Left => {
                let _ = shell.handle_key("left");
                return DispatchResult::Continue;
            }
            KeyCode::Right | KeyCode::Enter => {
                let _ = shell.handle_key("right");
                return DispatchResult::Continue;
            }
            KeyCode::Char(' ') => {
                let _ = shell.handle_key(" ");
                return DispatchResult::Continue;
            }
            KeyCode::Char('j') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                let _ = shell.handle_key("j");
                return DispatchResult::Continue;
            }
            KeyCode::Char('k') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                let _ = shell.handle_key("k");
                return DispatchResult::Continue;
            }
            KeyCode::Char('h') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                let _ = shell.handle_key("h");
                return DispatchResult::Continue;
            }
            KeyCode::Char('l') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                let _ = shell.handle_key("l");
                return DispatchResult::Continue;
            }
            _ => {}
        }
    }

    // Work screen navigator: move / activate before generic mapping.
    if shell.screen == Screen::Work
        && shell.focus == PaneFocus::Navigator
        && shell.overlay.is_none()
    {
        match key.code {
            KeyCode::Up => {
                let _ = shell.handle_key("up");
                return DispatchResult::Continue;
            }
            KeyCode::Down => {
                let _ = shell.handle_key("down");
                return DispatchResult::Continue;
            }
            KeyCode::Enter => {
                let _ = shell.handle_key("enter");
                return DispatchResult::Continue;
            }
            KeyCode::Char('j') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                let _ = shell.handle_key("j");
                return DispatchResult::Continue;
            }
            KeyCode::Char('k') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                let _ = shell.handle_key("k");
                return DispatchResult::Continue;
            }
            KeyCode::Char('r') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                let _ = shell.handle_key("r");
                return DispatchResult::Continue;
            }
            _ => {}
        }
    }

    let code = match key.code {
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => "quit",
        KeyCode::Char('q') if shell.overlay.is_none() && shell.focus != PaneFocus::Chat => "quit",
        KeyCode::Char('?') => "help",
        KeyCode::F(1) => "help",
        KeyCode::Char(',') if key.modifiers.contains(KeyModifiers::CONTROL) => "settings",
        KeyCode::Char('m') if key.modifiers.contains(KeyModifiers::CONTROL) => "model",
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => "sessions",
        KeyCode::Char('i') if key.modifiers.contains(KeyModifiers::CONTROL) => "quick-idea",
        KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => "refresh-nav",
        KeyCode::Esc => "escape",
        KeyCode::Tab => "tab",
        KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
            let n = c.to_digit(10).unwrap_or(0) as u8;
            if shell.focus == PaneFocus::Chat {
                let action = shell.chat.activate_number(n);
                runtime::activate_hint(shell, agent_rt, agent_tx, action);
            }
            return DispatchResult::Continue;
        }
        _ => "pane-char",
    };

    match shell.handle_key(code) {
        KeyEffect::Quit => DispatchResult::Quit,
        KeyEffect::ProjectBound => DispatchResult::ProjectBound,
        _ => DispatchResult::Continue,
    }
}
