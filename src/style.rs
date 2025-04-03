//! Types for styling terminal cells.

/// Styling of a cell's underline.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
// <https://sw.kovidgoyal.net/kitty/underlines/>
pub enum Underline {
    /// No underline
    #[default]
    None = 0,
    /// Straight underline
    Single = 1,
    /// Two underlines stacked on top of one another
    Double = 2,
    /// Curly / "squiggly" / "wavy" underline
    Curly = 3,
    /// Dotted underline
    Dotted = 4,
    /// Dashed underline
    Dashed = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// <https://en.wikipedia.org/wiki/ANSI_escape_code#Colors>
pub enum AnsiColor {
    Black = 0,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    /// "Bright" black (also known as "Gray")
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

/// A 256 "XTerm" color.
///
/// See <https://en.wikipedia.org/wiki/X11_color_names>.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct X11Color(pub u8);

/// 24-bit "true color" colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrueColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl TrueColor {
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    /// The floats are expected to be in the range `0.0..=1.0`.
    pub fn new_f32(red: f32, green: f32, blue: f32) -> Self {
        let red = (red * 255.) as u8;
        let green = (green * 255.) as u8;
        let blue = (blue * 255.) as u8;
        Self { red, green, blue }
    }

    pub const fn get_rgb(&self) -> (u8, u8, u8) {
        (self.red, self.green, self.blue)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Ansi(AnsiColor),
    X11(X11Color),
    TrueColor(TrueColor),
}

/*
mod representation {
    use std::{fmt, num::NonZeroU32};

    #[derive(Clone, Copy, PartialEq, Eq)]
    pub(super) struct NonMaxU32(NonZeroU32);

    impl NonMaxU32 {
        pub const fn new(val: u32) -> Option<Self> {
            match NonZeroU32::new(val ^ u32::MAX) {
                Some(nonzero) => Some(Self(nonzero)),
                None => None,
            }
        }

        pub const fn get(&self) -> u32 {
            self.0.get() ^ u32::MAX
        }
    }

    impl fmt::Debug for NonMaxU32 {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_tuple("NonMaxU32").field(&self.get()).finish()
        }
    }
}
*/
