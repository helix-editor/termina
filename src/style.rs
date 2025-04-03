//! Types for styling terminal cells.

/// Styling of a cell's underline.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
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

/// An 8-bit "256-color".
///
/// Colors 0-15 are the same as `AnsiColor`s (0-7 being normal colors and 8-15 being "bright").
/// Colors 16-231 make up a 6x6x6 "color cube." The remaining 232-255 colors define a
/// dark-to-light grayscale in 24 steps.
///
/// These are also known as "web-safe colors" or "X11 colors" historically, although the actual
/// colors varied somewhat between historical usages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// <https://en.wikipedia.org/wiki/ANSI_escape_code#8-bit>
pub struct WebColor(pub u8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl RgbColor {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbaColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    /// Also known as "opacity"
    pub alpha: u8,
}

impl From<RgbaColor> for RgbColor {
    fn from(color: RgbaColor) -> Self {
        Self {
            red: color.red,
            green: color.green,
            blue: color.blue,
        }
    }
}

impl From<RgbColor> for RgbaColor {
    fn from(color: RgbColor) -> Self {
        Self {
            red: color.red,
            green: color.green,
            blue: color.blue,
            alpha: 255,
        }
    }
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

pub type PaletteIndex = u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpec {
    Reset,
    PaletteIndex(PaletteIndex),
    TrueColor(RgbaColor),
}

impl ColorSpec {
    pub const BLACK: Self = Self::PaletteIndex(AnsiColor::Black as PaletteIndex);
    pub const RED: Self = Self::PaletteIndex(AnsiColor::Red as PaletteIndex);
    pub const GREEN: Self = Self::PaletteIndex(AnsiColor::Green as PaletteIndex);
    pub const YELLOW: Self = Self::PaletteIndex(AnsiColor::Yellow as PaletteIndex);
    pub const BLUE: Self = Self::PaletteIndex(AnsiColor::Blue as PaletteIndex);
    pub const MAGENTA: Self = Self::PaletteIndex(AnsiColor::Magenta as PaletteIndex);
    pub const CYAN: Self = Self::PaletteIndex(AnsiColor::Cyan as PaletteIndex);
    pub const WHITE: Self = Self::PaletteIndex(AnsiColor::White as PaletteIndex);
    pub const BRIGHT_BLACK: Self = Self::PaletteIndex(AnsiColor::BrightBlack as PaletteIndex);
    pub const BRIGHT_RED: Self = Self::PaletteIndex(AnsiColor::BrightRed as PaletteIndex);
    pub const BRIGHT_GREEN: Self = Self::PaletteIndex(AnsiColor::BrightGreen as PaletteIndex);
    pub const BRIGHT_YELLOW: Self = Self::PaletteIndex(AnsiColor::BrightYellow as PaletteIndex);
    pub const BRIGHT_BLUE: Self = Self::PaletteIndex(AnsiColor::BrightBlue as PaletteIndex);
    pub const BRIGHT_MAGENTA: Self = Self::PaletteIndex(AnsiColor::BrightMagenta as PaletteIndex);
    pub const BRIGHT_CYAN: Self = Self::PaletteIndex(AnsiColor::BrightCyan as PaletteIndex);
    pub const BRIGHT_WHITE: Self = Self::PaletteIndex(AnsiColor::BrightWhite as PaletteIndex);
}

impl From<AnsiColor> for ColorSpec {
    fn from(color: AnsiColor) -> Self {
        Self::PaletteIndex(color as u8)
    }
}

impl From<WebColor> for ColorSpec {
    fn from(color: WebColor) -> Self {
        Self::PaletteIndex(color.0)
    }
}

impl From<RgbColor> for ColorSpec {
    fn from(color: RgbColor) -> Self {
        Self::TrueColor(color.into())
    }
}

impl From<RgbaColor> for ColorSpec {
    fn from(color: RgbaColor) -> Self {
        Self::TrueColor(color)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Intensity {
    #[default]
    Normal,
    Bold,
    Dim,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Blink {
    #[default]
    None,
    Slow,
    Rapid,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Font {
    #[default]
    Default,
    /// An alternate font. Valid values are 1-9.
    Alternate(u8),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlign {
    #[default]
    BaseLine = 0,
    SuperScript = 1,
    SubScript = 2,
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
