pub(crate) mod base64;
pub mod escape;
pub mod event;
pub(crate) mod parse;
pub mod style;
mod terminal;

use std::{fmt, num::NonZeroU16};

pub use event::{reader::EventReader, Event};
pub use terminal::{PlatformHandle, PlatformTerminal, Terminal};

#[cfg(feature = "event-stream")]
pub use event::stream::EventStream;

/// A helper type which avoids tripping over Unix terminal's one-indexed conventions.
///
/// Coordinates and terminal dimensions are one-based in Termina on both Unix and Windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// CREDIT: <https://github.com/wezterm/wezterm/blob/a87358516004a652ad840bc1661bdf65ffc89b43/termwiz/src/escape/mod.rs#L527-L588>.
// This can be seen as a reimplementation on top of NonZeroU16.
pub struct OneBased(NonZeroU16);

impl OneBased {
    pub const fn new(n: u16) -> Option<Self> {
        match NonZeroU16::new(n) {
            Some(n) => Some(Self(n)),
            None => None,
        }
    }

    pub const fn from_zero_based(n: u16) -> Self {
        Self(unsafe { NonZeroU16::new_unchecked(n + 1) })
    }

    pub const fn get(self) -> u16 {
        self.0.get()
    }

    pub const fn get_zero_based(self) -> u16 {
        self.get() - 1
    }
}

impl Default for OneBased {
    fn default() -> Self {
        Self(unsafe { NonZeroU16::new_unchecked(1) })
    }
}

impl fmt::Display for OneBased {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<NonZeroU16> for OneBased {
    fn from(n: NonZeroU16) -> Self {
        Self(n)
    }
}

/// The dimensions of a terminal screen.
///
/// For both Unix and Windows, Termina returns the width and height
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowSize {
    /// The width - the number of columns.
    #[doc(alias = "width")]
    pub cols: u16,
    /// The height - the number of rows.
    #[doc(alias = "height")]
    pub rows: u16,
}
