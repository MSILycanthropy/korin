use std::{io, time::Duration};

use korin::prelude::*;
use korin_reactive::reactive_graph::traits::{Get, Set};
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
    let name = RwSignal::new(String::new());
    let greeting = RwSignal::new("World".to_string());

    let name_for_click = name;
    let greeting_for_click = greeting;

    let on_click = move |_: &EventContext<_>| {
        let n = name_for_click.get();
        if n.is_empty() {
            greeting_for_click.set("World".to_string());
        } else {
            greeting_for_click.set(n);
        }
    };

    let display = move || format!("Hello, {}!", greeting.get());

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
                <Container style={Style::builder().h(3).w(full()).build()}>
                    {display}
                </Container>

                <Container style={Style::builder().col().gap(1).build()}>
                    "Enter your name:"
                    <TextInput value={name} placeholder={"Type here..."} />
                </Container>

                <Button style={Style::builder()
                    .h(3)
                    .w(20)
                    .items(AlignItems::Center)
                    .justify(JustifyContent::Center)
                    .background(Color::Blue)
                    .on_focus(Style::builder().background(Color::Red))
                    .on_hover(Style::builder().background(Color::LightGreen))
                    .build()
                } on:click={on_click}>
                    "Greet"
                </Button>
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
