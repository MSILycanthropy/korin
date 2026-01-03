use std::io::{self, Write};

use ratatui::{
    Terminal,
    crossterm::{
        cursor,
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    },
    prelude::CrosstermBackend,
};

type CrosstermTerminal<W> = Terminal<CrosstermBackend<W>>;

pub fn setup<W: Write>(mut writer: W) -> io::Result<CrosstermTerminal<W>> {
    terminal::enable_raw_mode()?;

    execute!(
        writer,
        EnterAlternateScreen,
        EnableMouseCapture,
        cursor::Hide
    )?;

    let backend = CrosstermBackend::new(writer);
    Terminal::new(backend)
}

pub fn restore<W: Write>(mut writer: W) -> io::Result<()> {
    terminal::disable_raw_mode()?;

    execute!(
        writer,
        LeaveAlternateScreen,
        DisableMouseCapture,
        cursor::Show,
    )?;

    Ok(())
}
