use std::io;

use korin_core::{
    Document,
    element::Element,
    error::KorinError,
    primitives::{div::Div, text::Text},
};
use korin_layout::{Size, col, full, len, row};
use ratatui::{
    DefaultTerminal, Terminal,
    backend::TestBackend,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    style::{Color, Style},
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut doc = build_ui().expect("document building failed");

    loop {
        terminal.draw(|frame| {
            let size = frame.area();

            doc.layout(Size::new(size.width, size.height))
                .expect("layout failed");
            doc.render(frame).expect("render failed");
        })?;

        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
            && key.code == KeyCode::Char('q')
        {
            return Ok(());
        }
    }
}

fn build_ui() -> Result<Document<'static>, KorinError> {
    let mut doc = Document::new();

    let root = doc.set_root(Element::div(
        Div::bordered()
            .layout(col().w(full()).h(full()).p(len(1.0)).gap(len(1.0)))
            .title("Basic Example")
            .style(Style::default().bg(Color::Black)),
    ))?;

    doc.append(
        root,
        Element::text(Text::new("Hello from Korin!").style(Style::default().fg(Color::Green))),
    )?;

    let nested = doc
        .append(
            root,
            Element::div(Div::bordered().layout(row().p(len(1.0))).title("Nested")),
        )
        .expect("append failed");

    doc.append(nested, Element::Text(Text::new("Press 'q' to quit")))?;

    let row_container = doc.append(
        root,
        Element::div(Div::new().layout(row().gap(len(1.0)).grow(1.0))),
    )?;

    doc.append(
        row_container,
        Element::div(
            Div::bordered()
                .layout(col().grow(1.0).p(len(1.0)))
                .title("Left")
                .style(Style::default().bg(Color::Blue)),
        ),
    )?;

    doc.append(
        row_container,
        Element::div(
            Div::bordered()
                .layout(col().grow(1.0).p(len(1.0)))
                .title("Right")
                .style(Style::default().bg(Color::Red)),
        ),
    )?;

    Ok(doc)
}
