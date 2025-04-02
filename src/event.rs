pub(crate) mod source;

#[derive(Debug, PartialEq, Eq)]
pub enum InputEvent {
    WindowResized { rows: u16, cols: u16 },
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum InternalEvent {
    InputEvent(InputEvent),
    Wake,
}
