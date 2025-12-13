use std::{io, time::Duration};

use korin::prelude::*;
use korin_tui::prelude::*;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut runtime = Runtime::new();

    run_tokio(async || {
        let mut terminal = Terminal::new()?;
        terminal.init()?;

        run(&mut runtime, &mut terminal).await
    })
    .await
}

async fn run(runtime: &mut Runtime, terminal: &mut Terminal) -> io::Result<()> {
    let app = view! {
        <Container style={Style::builder()
            .w(full())
            .h(full())
            .col()
            .gap(1)
            .p(1)
            .background(Color::DarkGray)
            .build()
        }>
            <Container style={Style::builder()
                .h(3)
                .w(full())
                .bordered()
                .build()
            }>
                "Scroll Demo - use mouse wheel inside the box below"
            </Container>

            <Container style={Style::builder()
                .w(40)
                .h(10)
                .bordered()
                .overflow(Overflow::Scroll)
                .background(Color::Blue)
                .build()
            }>
                <Container style={Style::builder()
                    .col()
                    .gap(0)
                    .build()
                }>
                    "Line 1"
                    "Line 2"
                    "Line 3"
                    "Line 4"
                    "Line 5"
                    "Line 6"
                    "Line 7"
                    "Line 8"
                    "Line 9"
                    "Line 10"
                    "Line 11"
                    "Line 12"
                    "Line 13"
                    "Line 14"
                    "Line 15"
                    "Line 16"
                    "Line 17"
                    "Line 18"
                    "Line 19"
                    "Line 20"
                </Container>
            </Container>
        </Container>
    };

    runtime.mount(app).expect("failed to mount");

    loop {
        run_once(terminal, runtime)?;
        tick().await;
    }
}

fn run_once(terminal: &mut Terminal, runtime: &mut Runtime) -> io::Result<()> {
    terminal.render(runtime)?;
    terminal.flush()?;

    if let Some(event) = poll(Duration::from_millis(16)) {
        if let Event::Key(key) = &event
            && key.code == KeyCode::Char('q')
            && key.ctrl()
        {
            terminal.restore()?;
            std::process::exit(0)
        }

        if let Event::Resize(resize) = event {
            terminal.resize(resize.width, resize.height);
        }

        dispatch(&event, runtime);
    }

    Ok(())
}
