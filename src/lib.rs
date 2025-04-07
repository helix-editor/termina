pub(crate) mod base64;
pub mod escape;
pub mod event;
pub(crate) mod parse;
pub mod style;
mod terminal;

pub use event::{stream::EventStream, Event};
pub use terminal::{PlatformTerminal, Terminal};
