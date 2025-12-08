use std::{io, time::Duration};

use korin_layout::{col, full, len, row};
use korin_ratatui::{Event, dispatch, poll, render};
use korin_reactive::{
    reactive_graph::traits::{Get, Update},
    rw_signal,
};
use korin_runtime::Runtime;
use korin_style::{Color, Style};
use korin_view::container;
use ratatui::{Terminal, backend::TestBackend, crossterm::event::KeyCode, prelude::Backend};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut runtime = Runtime::new();

    korin_reactive::run_tokio(async || {
        let debug = std::env::args().any(|x| x == "--debug");

        if debug {
            let mut terminal = Terminal::new(TestBackend::new(97, 11))?;
            run(&mut runtime, &mut terminal, debug).await
        } else {
            let mut terminal = ratatui::init();

            run(&mut runtime, &mut terminal, debug).await
        }
    })
    .await
}

async fn run<B: Backend>(
    runtime: &mut Runtime,
    terminal: &mut Terminal<B>,
    debug: bool,
) -> io::Result<()> {
    let count = rw_signal(0isize);

    terminal.clear()?;
    terminal.hide_cursor()?;

    let view = container()
        .layout(col().w(full()).h(full()).gap(len(1.0)))
        .style(Style::new().background(Color::DarkGray))
        .child(
            container()
                .layout(row().h(len(3.0)).w(full()))
                .style(Style::new().bordered().background(Color::Blue))
                .child("Header"),
        )
        .child(
            container()
                .layout(row().grow(1.0).w(full()).gap(len(1.0)))
                .style(Style::new().background(Color::Black))
                .child(
                    container()
                        .layout(col().w(len(20.0)).h(full()))
                        .style(Style::new().bordered().background(Color::Green))
                        .child("Sidebar"),
                )
                .child(
                    container()
                        .layout(col().grow(1.0).h(full()))
                        .style(Style::new().bordered().background(Color::Red))
                        .child(move || {
                            let c = count.get();

                            format!("{c}")
                        }),
                ),
        )
        .child(
            container()
                .layout(row().h(len(3.0)).w(full()))
                .style(Style::new().bordered().background(Color::Magenta))
                .child("Footer - Press 'q' to quit"),
        )
        .on_event::<Event>(move |event| {
            if let Event::Key(key) = event {
                match key.code {
                    KeyCode::Char('j') => count.update(|c| *c += 1),
                    KeyCode::Char('k') => count.update(|c| *c = c.saturating_sub(1)),
                    _ => {}
                }
            }
        });

    runtime.mount(view).expect("failed to mount");

    if debug {
        run_once(terminal, runtime)?;
    } else {
        loop {
            run_once(terminal, runtime)?;

            korin_reactive::tick().await;
        }
    }

    Ok(())
}

fn run_once<B: Backend>(terminal: &mut Terminal<B>, runtime: &mut Runtime) -> io::Result<()> {
    let size = terminal.size()?;
    runtime
        .compute_layout(korin_layout::Size::new(
            f32::from(size.width),
            f32::from(size.height),
        ))
        .expect("layout failed");

    terminal.draw(|frame| {
        render(frame, runtime);
    })?;

    if let Some(event) = poll(Duration::from_millis(16)) {
        if let Event::Key(key) = &event
            && key.code == KeyCode::Char('q')
        {
            terminal.show_cursor()?;
            ratatui::restore();
            std::process::exit(0);
        }

        dispatch(&event, runtime);
    }

    Ok(())
}
