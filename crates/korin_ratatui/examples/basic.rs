use std::{io, time::Duration};

use korin_layout::{col, full, len, row};
use korin_ratatui::{Event, dispatch, poll, render};
use korin_runtime::Runtime;
use korin_style::Style;
use korin_view::container;
use ratatui::{
    DefaultTerminal,
    crossterm::{
        event::KeyCode,
        execute,
        terminal::{LeaveAlternateScreen, disable_raw_mode},
    },
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    let result = run(&mut terminal);

    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    result
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut runtime = Runtime::new();

    let view = container()
        .layout(col().w(full()).h(full()).gap(len(1.0)))
        .child(
            container()
                .layout(row().h(len(3.0)).w(full()))
                .style(Style::new().bordered())
                .child("Header"),
        )
        .child(
            container()
                .layout(row().grow(1.0).w(full()).gap(len(1.0)))
                .child(
                    container()
                        .layout(col().w(len(20.0)).h(full()))
                        .style(Style::new().bordered())
                        .child("Sidebar"),
                )
                .child(
                    container()
                        .layout(col().grow(1.0).h(full()))
                        .style(Style::new().bordered())
                        .child("Main Content"),
                ),
        )
        .child(
            container()
                .layout(row().h(len(3.0)).w(full()))
                .style(Style::new().bordered())
                .child("Footer - Press 'q' to quit"),
        );

    runtime.mount(view).expect("failed to mount");

    loop {
        let size = terminal.size()?;
        runtime
            .compute_layout(korin_layout::Size::new(
                f32::from(size.width),
                f32::from(size.height),
            ))
            .expect("layout failed");

        terminal.draw(|frame| {
            render(frame, &runtime);
        })?;

        if let Some(event) = poll(Duration::from_millis(16)) {
            dispatch(&event, &runtime);

            if let Event::Key(key) = &event
                && key.code == KeyCode::Char('q')
            {
                break;
            }
        }
    }

    Ok(())
}
