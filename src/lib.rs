pub(crate) mod base64;
pub mod escape;
pub mod event;
pub(crate) mod parse;
pub mod style;
mod terminal;

pub use event::{
    stream::{DummyEventStream, EventStream},
    Event,
};
pub use terminal::{PlatformHandle, PlatformTerminal, Terminal};
