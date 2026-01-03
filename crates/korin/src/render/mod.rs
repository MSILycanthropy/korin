use std::io;

use ratatui::crossterm::event::{self, Event, KeyCode};

use crate::Document;

mod paint;
mod terminal;

pub fn run_once(document: &Document) -> io::Result<()> {
    let writer = io::stdout();
    let mut terminal = terminal::setup(writer)?;

    terminal.draw(|frame| {
        paint::paint(document, frame);
    })?;

    loop {
        if let Event::Key(key) = event::read()?
            && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
        {
            break;
        }
    }

    let writer = io::stdout();
    terminal::restore(writer)?;
    Ok(())
}
