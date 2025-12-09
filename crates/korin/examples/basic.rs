use std::{io, time::Duration};

use korin::prelude::*;
use korin_ratatui::prelude::*;
use ratatui::{Terminal, backend::TestBackend, prelude::Backend};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut runtime = Runtime::new();

    run_tokio(async || {
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
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());

    terminal.clear()?;
    terminal.hide_cursor()?;

    let app = view! {
        <Container layout={Layout::col().w(full()).h(full()).gap(1.0)} style={Style::new().background(Color::DarkGray)}>
            <Container layout={Layout::row().h(3.0).w(full())} style={Style::new().bordered().background(Color::Blue)}>
                "Login Form"
            </Container>
            <Container layout={Layout::col().grow(1.0).w(full()).gap(1.0)}>
                <Container layout={Layout::col().gap(0.5)}>
                    "Username:"
                    <TextInput value={username} placeholder={"Enter username..."} />
                </Container>
                <Container layout={Layout::col().gap(0.5)}>
                    "Password:"
                    <TextInput value={password} placeholder={"Enter password..."} />
                </Container>
            </Container>
            <Container layout={Layout::row().h(3.0).w(full())} style={Style::new().bordered().background(Color::Magenta)}>
                "Press Tab to switch fields, Ctrl+Q to quit"
            </Container>
        </Container>
    };

    runtime.mount(app).expect("failed to mount");

    if debug {
        run_once(terminal, runtime)?;
    } else {
        loop {
            run_once(terminal, runtime)?;
            tick().await;
        }
    }

    Ok(())
}

fn run_once<B: Backend>(terminal: &mut Terminal<B>, runtime: &mut Runtime) -> io::Result<()> {
    let size = terminal.size()?;
    runtime
        .compute_layout(Size::new(f32::from(size.width), f32::from(size.height)))
        .expect("layout failed");

    terminal.draw(|frame| {
        render(frame, runtime);
    })?;

    if let Some(event) = poll(Duration::from_millis(16)) {
        if let Event::Key(key) = &event
            && key.code == KeyCode::Char('q')
            && key.ctrl()
        {
            terminal.show_cursor()?;
            ratatui::restore();
            std::process::exit(0);
        }

        dispatch(&event, runtime);
    }

    Ok(())
}
