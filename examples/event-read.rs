// CREDIT: This module is mostly based on crossterm's `event-read` example with minor
// modifications to adapt to the termina API.
// <https://github.com/crossterm-rs/crossterm/blob/36d95b26a26e64b0f8c12edfe11f410a6d56a812/examples/event-read.rs>
use std::{
    io::{self, Write as _},
    time::Duration,
};

use termina::{
    escape::csi::{self, KittyKeyboardFlags},
    event::{KeyCode, KeyEvent},
    Event, PlatformTerminal, Terminal, WindowSize,
};

const HELP: &str = r#"Blocking read()
 - Keyboard, mouse, focus and terminal resize events enabled
 - Hit "c" to print current cursor position
 - Use Esc to quit
"#;

macro_rules! decset {
    ($mode:ident) => {
        csi::Csi::Mode(csi::Mode::SetDecPrivateMode(csi::DecPrivateMode::Code(
            csi::DecPrivateModeCode::$mode,
        )))
    };
}
macro_rules! decreset {
    ($mode:ident) => {
        csi::Csi::Mode(csi::Mode::ResetDecPrivateMode(csi::DecPrivateMode::Code(
            csi::DecPrivateModeCode::$mode,
        )))
    };
}

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
        decset!(FocusTracking),
        decset!(BracketedPaste),
        decset!(MouseTracking),
        decset!(ButtonEventMouse),
        decset!(AnyEventMouse),
        decset!(RXVTMouse),
        decset!(SGRMouse),
    )?;
    terminal.flush()?;

    let mut size = terminal.get_dimensions()?;
    loop {
        let event = terminal.read(|event| !event.is_escape())?;

        println!("Event: {event:?}\r");

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Escape,
                ..
            }) => break,
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                ..
            }) => {
                write!(
                    terminal,
                    "{}",
                    csi::Csi::Cursor(csi::Cursor::RequestActivePositionReport),
                )?;
                terminal.flush()?;
                let filter = |event: &Event| {
                    matches!(
                        event,
                        Event::Csi(csi::Csi::Cursor(csi::Cursor::ActivePositionReport { .. }))
                    )
                };
                if terminal.poll(filter, Some(Duration::from_millis(50)))? {
                    let Event::Csi(csi::Csi::Cursor(csi::Cursor::ActivePositionReport {
                        line,
                        col,
                    })) = terminal.read(filter)?
                    else {
                        unreachable!()
                    };
                    println!(
                        "Cursor position: {:?}\r",
                        (line.get_zero_based(), col.get_zero_based())
                    );
                } else {
                    eprintln!("Failed to read the cursor position within 50msec\r");
                }
            }
            Event::WindowResized(dimensions) => {
                let new_size = flush_resize_events(&terminal, dimensions);
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
        decreset!(FocusTracking),
        decreset!(BracketedPaste),
        decreset!(MouseTracking),
        decreset!(ButtonEventMouse),
        decreset!(AnyEventMouse),
        decreset!(RXVTMouse),
        decreset!(SGRMouse),
    )?;

    Ok(())
}

fn flush_resize_events(terminal: &PlatformTerminal, original_size: WindowSize) -> WindowSize {
    let mut size = original_size;
    let filter = |event: &Event| matches!(event, Event::WindowResized { .. });
    while let Ok(true) = terminal.poll(filter, Some(Duration::from_millis(50))) {
        if let Ok(Event::WindowResized(dimensions)) = terminal.read(filter) {
            size = dimensions;
        }
    }
    size
}
