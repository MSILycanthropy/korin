use std::{panic, sync::Once};

use crossterm::{
    cursor::Show,
    event::DisableMouseCapture,
    execute,
    terminal::{EnableLineWrap, LeaveAlternateScreen, disable_raw_mode},
};

static INSTALL_ONCE: Once = Once::new();

pub fn install_panic_hook() {
    INSTALL_ONCE.call_once(|| {
        let prev = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            restore_terminal();
            prev(info);
        }));
    });
}

fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(
        std::io::stderr(),
        Show,
        EnableLineWrap,
        DisableMouseCapture,
        LeaveAlternateScreen
    );
}
