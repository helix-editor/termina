//! ANSI escape sequences.

pub mod csi;
pub mod dcs;
pub mod osc;

pub const CSI: &str = "\x1b[";
pub const DCS: &str = "\x1bP";
pub const ST: &str = "\x1b\\";
pub const OSC: &str = "\x1b]";
pub const SS3: &str = "\x1bO";
