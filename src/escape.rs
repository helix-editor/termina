//! ANSI escape sequences.

// CREDIT: this tree of modules is mostly yanked from the equivalents in TermWiz with some
// stylistic edits and additions/subtractions of some escape sequences.

pub mod csi;
pub mod dcs;
pub mod osc;

// Originally yanked from <https://github.com/wezterm/wezterm/blob/a87358516004a652ad840bc1661bdf65ffc89b43/term/src/lib.rs#L131-L135>
pub const CSI: &str = "\x1b[";
pub const OSC: &str = "\x1b]";
pub const ST: &str = "\x1b\\";
pub const SS3: &str = "\x1bO";
pub const DCS: &str = "\x1bP";
