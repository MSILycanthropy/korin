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

    let app = || {
        view! {
            <Container style={Style::builder().col().w(full()).h(full()).gap(1).background(Color::DarkGray).overflow(Overflow::Scroll).build()}>
                <Container style={Style::builder().h(3).w(full()).bordered().background(Color::Blue).build()}>
                    "Login Form"
                </Container>
                <Container style={Style::builder().col().grow(1).w(full()).gap(0.5).build()}>
                    <Container style={Style::builder().col().gap(1).build()}>
                        "Username:"
                        <TextInput value={username} placeholder={"Enter username..."} />
                    </Container>
                    <Container style={Style::builder().col().gap(0.5).build()}>
                        "Password:"
                        <TextInput value={password} placeholder={"Enter password..."} />
                    </Container>
                </Container>
                <Container style={Style::builder().bordered().background(Color::Magenta).h(3).w(full()).build()}>
                    "Press Tab to switch fields, Ctrl+Q to quit"
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
