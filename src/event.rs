pub(crate) mod source;

#[derive(Debug)]
pub enum InputEvent {
    WindowResized { rows: u16, cols: u16 },
}
