pub(crate) mod base64;
pub mod escape;
pub mod event;
pub(crate) mod parse;
pub mod style;
mod terminal;

pub use event::{reader::EventReader, Event};
pub use terminal::{PlatformHandle, PlatformTerminal, Terminal};

#[cfg(feature = "event-stream")]
pub use event::stream::EventStream;
