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
    let lines: Vec<usize> = (1..=20).collect();

    let app = || {
        view! {
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
                    "Scroll Demo - use mouse wheel inside the boxes below"
                </Container>

                <Container style={Style::builder()
                    .row()
                    .gap(2)
                    .build()
                }>
                    // Vertical scroll
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
                            .build()
                        }>
                            <Each
                                each={move || lines.clone()}
                                key={|n| *n}
                                children={|n| format!("Line {n}")}
                            />
                        </Container>
                    </Container>

                    <Container style={Style::builder()
                        .w(30)
                        .h(5)
                        .bordered()
                        .overflow(Overflow::Scroll)
                        .background(Color::Green)
                        .build()
                    }>
                        <Container style={Style::builder()
                            .col()
                            .build()
                        }>
                            "This is a very long line of text that should overflow horizontally"
                            "Another super duper extra long line that keeps going and going"
                            "Short"
                            "Yet another extremely lengthy line to test horizontal scrolling"
                        </Container>
                    </Container>
                </Container>
            </Container>
        }
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
