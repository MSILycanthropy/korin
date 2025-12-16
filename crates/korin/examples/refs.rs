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
    let first_input = NodeRef::new();
    let second_input = NodeRef::new();

    let value1 = RwSignal::new(String::new());
    let value2 = RwSignal::new(String::new());

    let first_ref = first_input.clone();
    let second_ref = second_input.clone();

    let app = || {
        view! {
            <Container style={Style::builder()
                .col()
                .w(full())
                .h(full())
                .gap(1)
                .p(1)
                .background(Color::DarkGray)
                .build()
            }>
                <Container style={Style::builder().h(3).w(full()).bordered().build()}>
                    "Refs Demo - Press J to focus first input, H for second"
                </Container>

                <Container style={Style::builder().col().gap(1).build()}>
                    "First input:"
                    <TextInput node_ref={first_ref} value={value1} placeholder={"Type here..."} />
                </Container>

                <Container style={Style::builder().col().gap(1).build()}>
                    "Second input:"
                    <TextInput node_ref={second_ref} value={value2} placeholder={"Or here..."} />
                </Container>
            </Container>
        }
    };

    runtime.mount(app).expect("failed to mount");

    loop {
        run_once(terminal, runtime, &first_input, &second_input)?;
        tick().await;
    }
}

fn run_once(
    terminal: &mut Terminal,
    runtime: &mut Runtime,
    first: &NodeRef,
    second: &NodeRef,
) -> io::Result<()> {
    terminal.render(runtime)?;
    terminal.flush()?;

    if let Some(event) = poll(Duration::from_millis(16)) {
        if let Event::Key(key) = &event {
            match key.code {
                KeyCode::Char('q') if key.ctrl() => {
                    terminal.restore()?;
                    std::process::exit(0)
                }
                KeyCode::Char(char) => {
                    if char == 'j' {
                        second.focus();
                        return Ok(());
                    }

                    if char == 'k' {
                        first.focus();
                        return Ok(());
                    }
                }

                _ => {}
            }
        }

        if let Event::Resize(resize) = event {
            terminal.resize(resize.width, resize.height);
        }

        dispatch(&event, runtime);
    }

    Ok(())
}
