pub(crate) mod reader;
pub(crate) mod source;
pub(crate) mod stream;

#[derive(Debug, PartialEq, Eq)]
pub enum InputEvent {
    WindowResized { rows: u16, cols: u16 },
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum InternalEvent {
    InputEvent(InputEvent),
}
