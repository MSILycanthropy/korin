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
            .w(20)
            .h(10)
            .bordered()
            .background(Color::Blue)
            .build()
        }>
            <Container style={Style::builder()
                .min_w(30)
                .w(180)
                .h(20)
                .background(Color::Red)
                .build()
            }>
                "This text is way too long to fit in the parent container"
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
