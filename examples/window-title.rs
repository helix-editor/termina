use std::io::{self, Write as _};

use termina::{
    escape::{
        csi::{self, Csi},
        osc::Osc,
    },
    PlatformTerminal, Terminal as _,
};

macro_rules! decset {
    ($mode:ident) => {
        Csi::Mode(csi::Mode::SetDecPrivateMode(csi::DecPrivateMode::Code(
            csi::DecPrivateModeCode::$mode,
        )))
    };
}
macro_rules! decreset {
    ($mode:ident) => {
        Csi::Mode(csi::Mode::ResetDecPrivateMode(csi::DecPrivateMode::Code(
            csi::DecPrivateModeCode::$mode,
        )))
    };
}

fn main() -> io::Result<()> {
    let mut terminal = PlatformTerminal::new()?;
    terminal.enter_raw_mode()?;

    write!(
        terminal,
        "{}{}{}{}Check the window/tab title of your terminal. Press any key to exit. ",
        decset!(ClearAndEnableAlternateScreen),
        // Save the current title to the terminal's stack.
        Csi::Window(Box::new(csi::Window::PushIconAndWindowTitle)),
        Osc::SetWindowTitle("Hello, world! - termina"),
        Csi::Cursor(csi::Cursor::Position {
            line: Default::default(),
            col: Default::default(),
        }),
    )?;
    terminal.flush()?;
    let _ = terminal.read(|event| matches!(event, termina::Event::Key(_)));

    write!(
        terminal,
        "{}{}",
        // Restore the title from the terminal's stack.
        Csi::Window(Box::new(csi::Window::PopIconAndWindowTitle)),
        decreset!(ClearAndEnableAlternateScreen),
    )?;

    Ok(())
}
