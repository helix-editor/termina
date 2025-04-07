//! ANSI escape sequences.

// CREDIT: this tree of modules is mostly yanked from the equivalents in TermWiz with some
// stylistic edits and additions/subtractions of some escape sequences.

use std::{
    fmt::{self, Display},
    num::NonZeroU16,
};

pub mod csi;
pub mod dcs;
pub mod osc;

// Originally yanked from <https://github.com/wezterm/wezterm/blob/a87358516004a652ad840bc1661bdf65ffc89b43/term/src/lib.rs#L131-L135>
pub const CSI: &str = "\x1b[";
pub const OSC: &str = "\x1b]";
pub const ST: &str = "\x1b\\";
pub const SS3: &str = "\x1bO";
pub const DCS: &str = "\x1bP";

/// A helper type which avoids tripping over Unix terminal's one-indexed conventions.
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

impl Display for OneBased {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<NonZeroU16> for OneBased {
    fn from(n: NonZeroU16) -> Self {
        Self(n)
    }
}
