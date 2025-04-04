use std::{
    io::{self, Write as _},
    time::Duration,
};

use termina::{
    escape::csi::{self, KittyKeyboardFlags},
    event::{KeyCode, KeyEvent},
    terminal::{PlatformTerminal, Terminal},
    Event,
};

const HELP: &str = r#"Blocking read()
 - Keyboard, mouse, focus and terminal resize events enabled
 - Hit "c" to print current cursor position
 - Use Esc to quit
"#;

fn main() -> io::Result<()> {
    println!("{HELP}");

    let mut terminal = PlatformTerminal::new()?;
    terminal.enter_raw_mode()?;

    write!(
        terminal,
        "{}{}{}{}{}{}{}{}",
        csi::Csi::Keyboard(csi::Keyboard::PushFlags(
            KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES
                | KittyKeyboardFlags::REPORT_ALTERNATE_KEYS
        )),
        csi::Csi::Mode(csi::Mode::SetDecPrivateMode(csi::DecPrivateMode::Code(
            csi::DecPrivateModeCode::FocusTracking
        ))),
        csi::Csi::Mode(csi::Mode::SetDecPrivateMode(csi::DecPrivateMode::Code(
            csi::DecPrivateModeCode::BracketedPaste
        ))),
        csi::Csi::Mode(csi::Mode::SetDecPrivateMode(csi::DecPrivateMode::Code(
            csi::DecPrivateModeCode::MouseTracking
        ))),
        csi::Csi::Mode(csi::Mode::SetDecPrivateMode(csi::DecPrivateMode::Code(
            csi::DecPrivateModeCode::ButtonEventMouse
        ))),
        csi::Csi::Mode(csi::Mode::SetDecPrivateMode(csi::DecPrivateMode::Code(
            csi::DecPrivateModeCode::AnyEventMouse
        ))),
        csi::Csi::Mode(csi::Mode::SetDecPrivateMode(csi::DecPrivateMode::Code(
            csi::DecPrivateModeCode::RXVTMouse
        ))),
        csi::Csi::Mode(csi::Mode::SetDecPrivateMode(csi::DecPrivateMode::Code(
            csi::DecPrivateModeCode::SGRMouse
        ))),
    )?;
    terminal.flush()?;

    let mut size = terminal.get_dimensions()?;
    loop {
        let event = terminal.read()?;

        println!("Event: {event:?}\r");

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Escape,
                ..
            }) => break,
            Event::WindowResized { rows, cols } => {
                let new_size = flush_resize_events(&terminal, (rows, cols));
                println!("Resize from {size:?} to {new_size:?}\r");
                size = new_size;
            }
            _ => (),
        }
    }

    write!(
        terminal,
        "{}{}{}{}{}{}{}{}",
        csi::Csi::Keyboard(csi::Keyboard::PopFlags(1)),
        csi::Csi::Mode(csi::Mode::ResetDecPrivateMode(csi::DecPrivateMode::Code(
            csi::DecPrivateModeCode::FocusTracking
        ))),
        csi::Csi::Mode(csi::Mode::ResetDecPrivateMode(csi::DecPrivateMode::Code(
            csi::DecPrivateModeCode::BracketedPaste
        ))),
        csi::Csi::Mode(csi::Mode::ResetDecPrivateMode(csi::DecPrivateMode::Code(
            csi::DecPrivateModeCode::MouseTracking
        ))),
        csi::Csi::Mode(csi::Mode::ResetDecPrivateMode(csi::DecPrivateMode::Code(
            csi::DecPrivateModeCode::ButtonEventMouse
        ))),
        csi::Csi::Mode(csi::Mode::ResetDecPrivateMode(csi::DecPrivateMode::Code(
            csi::DecPrivateModeCode::AnyEventMouse
        ))),
        csi::Csi::Mode(csi::Mode::ResetDecPrivateMode(csi::DecPrivateMode::Code(
            csi::DecPrivateModeCode::RXVTMouse
        ))),
        csi::Csi::Mode(csi::Mode::ResetDecPrivateMode(csi::DecPrivateMode::Code(
            csi::DecPrivateModeCode::SGRMouse
        ))),
    )?;

    Ok(())
}

fn flush_resize_events(terminal: &PlatformTerminal, original_size: (u16, u16)) -> (u16, u16) {
    let mut size = original_size;
    while let Ok(true) = terminal.poll(Some(Duration::from_millis(50))) {
        if let Ok(Event::WindowResized { rows, cols }) = terminal.read() {
            size = (rows, cols)
        }
    }
    size
}
