use std::{io, time::Duration};

use korin::prelude::*;
use korin_reactive::reactive_graph::traits::{Get, Update};
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
    let visible = RwSignal::new(true);

    let toggle = move |_: &EventContext<_>| {
        visible.update(|v| *v = !*v);
    };

    let style = move || {
        let style = Style::builder().col().w(full()).h(full()).gap(1).p(1);

        if visible.get() {
            style.background(Color::DarkGray)
        } else {
            style.background(Color::Gray)
        }
        .build()
    };

    let app = view! {
        <Container style={style}>
            <Container style={Style::builder().h(3).w(full()).build()}>
                "Show Demo - Press the button to toggle visibility"
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
            } on:click={toggle}>
                "Toggle"
            </Button>

            <Show when={move || visible.get()} fallback={|| "Content is hidden!"}>
                <Container style={Style::builder()
                    .h(5)
                    .w(30)
                    .bordered()
                    .background(Color::Green)
                    .build()
                }>
                    "I am visible!"
                </Container>
            </Show>
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
