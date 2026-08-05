mod action;
mod app;
mod config;
mod github;
mod ui;
mod watcher;

use app::App;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use std::io::stdout;
use tokio::sync::mpsc;
use watcher::WatcherEvent;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = config::load_config();

    // on_start hooks fire once, before the terminal takes over the screen.
    // They run detached with stdout/stderr swallowed, so this cannot block
    // or corrupt the TUI even if a hook is slow.
    for hook in &cfg.on_start {
        action::run_shell(&hook.command);
    }

    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    // Populate the manual-action menu labels from config.
    app.action_names = cfg.actions.iter().map(|a| a.name.clone()).collect();

    // Start watcher
    let (tx, mut rx) = mpsc::unbounded_channel();
    watcher::spawn_watcher(cfg.clone(), tx.clone());

    loop {
        // Draw
        terminal.draw(|f| ui::draw(f, &app))?;

        // Handle events with a short timeout so we can also check watcher messages
        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // When the manual-action popup is open, keys drive the menu.
                    if app.show_action_menu {
                        match key.code {
                            KeyCode::Char('j') | KeyCode::Down => app.action_menu_next(),
                            KeyCode::Char('k') | KeyCode::Up => app.action_menu_prev(),
                            KeyCode::Esc | KeyCode::Char('a') | KeyCode::Char('q') => {
                                app.close_action_menu();
                            }
                            KeyCode::Enter => {
                                let idx = app.action_selected;
                                if let (Some(pr), Some(act)) = (app.selected_pr(), cfg.actions.get(idx)) {
                                    action::run_command(&act.command, pr);
                                }
                                app.close_action_menu();
                            }
                            _ => {}
                        }
                        continue;
                    }
                    match key.code {
                        KeyCode::Char('q') => {
                            app.should_quit = true;
                        }
                        KeyCode::Char('j') | KeyCode::Down => app.next(),
                        KeyCode::Char('k') | KeyCode::Up => app.previous(),
                        KeyCode::Char('a') => app.open_action_menu(),
                        KeyCode::Enter => {
                            if let Some(pr) = app.selected_pr() {
                                if let Some(ref on_select) = cfg.on_select {
                                    action::run_command(&on_select.command, pr);
                                } else {
                                    // Default: open URL
                                    action::run_command("open {url}", pr);
                                }
                            }
                        }
                        KeyCode::Tab => app.toggle_filter(),
                        KeyCode::Char('r') => {
                            // Manual refresh: spawn a one-off fetch
                            app.refreshing = true;
                            let tx2 = tx.clone();
                            tokio::task::spawn_blocking(move || {
                                match github::fetch_review_requests() {
                                    Ok(prs) => {
                                        let _ = tx2.send(WatcherEvent::Updated(prs));
                                    }
                                    Err(e) => {
                                        let _ = tx2.send(WatcherEvent::Error(e));
                                    }
                                }
                            });
                        }
                        _ => {}
                    }
                }
            }
        }

        // Process watcher events
        while let Ok(ev) = rx.try_recv() {
            match ev {
                WatcherEvent::Updated(prs) => app.update_prs(prs),
                WatcherEvent::Error(e) => app.set_error(e),
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}
