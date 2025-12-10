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
    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());

    let app = view! {
        <Container layout={Layout::col().w(full()).h(full()).gap(1)} style={Style::new().background(Color::DarkGray)}>
            <Container layout={Layout::row().h(3).w(full())} style={Style::new().bordered().background(Color::Blue)}>
                "Login Form"
            </Container>
            <Container layout={Layout::col().grow(1).w(full()).gap(0.5)}>
                <Container layout={Layout::col().gap(1)}>
                    "Username:"
                    <TextInput value={username} placeholder={"Enter username..."} />
                </Container>
                <Container layout={Layout::col().gap(0.5)}>
                    "Password:"
                    <TextInput value={password} placeholder={"Enter password..."} />
                </Container>
            </Container>
            <Container layout={Layout::row().h(3).w(full())} style={Style::new().bordered().background(Color::Magenta)}>
                "Press Tab to switch fields, Ctrl+Q to quit"
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
    let size = terminal.size()?;
    runtime.compute_layout(size.cast()).expect("layout failed");

    terminal.render(runtime);
    terminal.flush()?;

    if let Some(event) = poll(Duration::from_millis(16)) {
        if let Event::Key(key) = &event
            && key.code == KeyCode::Char('q')
            && key.ctrl()
        {
            terminal.restore()?;
            std::process::exit(0)
        }

        if let Event::Resize(w, h) = event {
            terminal.resize(w, h);
        }

        dispatch(&event, runtime);
    }

    Ok(())
}
