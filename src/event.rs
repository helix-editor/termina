use crate::input::{KeyEvent, MouseEvent};

pub(crate) mod reader;
pub(crate) mod source;
pub(crate) mod stream;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    /// The window was resized to the given dimensions.
    WindowResized {
        rows: u16,
        cols: u16,
    },
    FocusIn,
    FocusOut,
    /// A "bracketed" paste.
    ///
    /// Normally pasting into a terminal with Ctrl+v (or Super+v) enters the pasted text as if
    /// you had typed the keys individually. Terminals commonly support ["bracketed
    /// paste"](https://en.wikipedia.org/wiki/Bracketed-paste) now however, which uses an escape
    /// sequence to deliver the entire pasted content.
    Paste(String),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum InternalEvent {
    Event(Event),
    CursorPosition(u16, u16),
}
