use std::io::{self, Write as _};

use termina::{
    escape::csi::{self, KittyKeyboardFlags},
    terminal::{PlatformTerminal, Terminal},
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
        "{}",
        csi::Csi::Keyboard(csi::Keyboard::PushFlags(
            KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES
                | KittyKeyboardFlags::REPORT_ALTERNATE_KEYS
        ))
    )?;

    print_events(terminal)?;

    Ok(())
}

fn print_events(terminal: PlatformTerminal) -> io::Result<()> {
    loop {
        let event = terminal.read()?;
        println!("Event: {event:?}");
    }

    // Ok(())
}
