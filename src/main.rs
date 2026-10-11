use anyhow::Result;
use crossterm::{
    cursor::{Hide, Show},
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};
use std::time::Duration;

use sonosteg::app::App;
use sonosteg::ui;

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture, Hide)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        Show
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            Show
        );
        original_hook(panic_info);
    }));
}

fn main() -> Result<()> {
    // Install panic safety hook (Section 6.5)
    install_panic_hook();

    let mut terminal = setup_terminal()?;
    let mut app = App::new();

    let tick_rate = Duration::from_millis(50);

    let run_result = (|| -> Result<()> {
        while app.running {
            terminal.draw(|frame| ui::render(frame, &app))?;

            if event::poll(tick_rate)? {
                if let Event::Key(key) = event::read()? {
                    // Only process key press events (ignore key release events on Windows/Crossterm)
                    if key.kind == KeyEventKind::Press {
                        app.handle_key_event(key);
                    }
                }
            }
        }
        Ok(())
    })();

    // Always restore terminal safely even if an error occurred during execution
    let restore_result = restore_terminal(&mut terminal);

    run_result.and(restore_result)
}
