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
            .background(Color::DarkGray)
            .position(Position::Relative)
            .build()
        }>
            // Normal flow content
            <Container style={Style::builder()
                .w(20)
                .h(5)
                .bordered()
                .background(Color::Blue)
                .build()
            }>
                "Normal box 1"
            </Container>

            <Container style={Style::builder()
                .w(20)
                .h(5)
                .bordered()
                .background(Color::Green)
                .build()
            }>
                "Normal box 2"
            </Container>

            // Absolute positioned - top right corner
            <Container style={Style::builder()
                .position(Position::Absolute)
                .top(1)
                .right(1)
                .w(15)
                .h(5)
                .bordered()
                .background(Color::Red)
                .z_index(10)
                .build()
            }>
                "Top Right"
            </Container>

            // Absolute positioned - bottom left
            <Container style={Style::builder()
                .position(Position::Absolute)
                .bottom(1)
                .left(1)
                .w(15)
                .h(5)
                .bordered()
                .background(Color::Magenta)
                .z_index(10)
                .build()
            }>
                "Bottom Left"
            </Container>

            // Absolute centered (using left/top with offset)
            <Container style={Style::builder()
                .position(Position::Absolute)
                .top(10)
                .left(20)
                .w(20)
                .h(7)
                .bordered()
                .background(Color::Cyan)
                .z_index(20)
                .build()
            }>
                "Floating Modal"
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
