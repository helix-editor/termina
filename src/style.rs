//! Types for styling terminal cells.

// CREDIT: This is shared almost fairly between crossterm and termwiz. SGR properties like
// `Underline`, `CursorStyle` and `Intensity` are from termwiz. The `StyleExt` trait is similar
// to a crossterm trait.

use std::{
    borrow::Cow,
    fmt::{self, Display},
    sync::atomic::{AtomicBool, Ordering},
};

use crate::escape::{
    self,
    csi::{Csi, Sgr},
};

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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    #[default]
    Default = 0,
    BlinkingBlock = 1,
    SteadyBlock = 2,
    BlinkingUnderline = 3,
    SteadyUnderline = 4,
    BlinkingBar = 5,
    SteadyBar = 6,
}

impl Display for CursorStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u8)
    }
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

/// A helper type for conveniently rendering styled content to the terminal.
///
/// This is meant to be used instead of `PlatformTerminal` and proper CSI SGR codes when printing
/// basic text, for example a CLI's help string.
///
/// Instead of using this type directly, `use` the `StyleExt` trait and the helper functions
/// attached to strings:
///
/// ```
/// use termina::style::StyleExt as _;
/// assert_eq!("green".green().to_string(), "\x1b[0;32mgreen\x1b[m");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stylized<'a> {
    pub content: Cow<'a, str>,
    styles: Vec<Sgr>,
}

static INITIALIZER: parking_lot::Once = parking_lot::Once::new();
static NO_COLOR: AtomicBool = AtomicBool::new(false);

impl Stylized<'_> {
    /// Checks whether ANSI color sequences where turned off in the environment.
    ///
    /// See <https://no-color.org/>: if the `NO_COLOR` environment variable is present and
    /// non-empty, color escape sequences will be omitted when rendering this struct. This
    /// behavior can be overridden with [Self::force_ansi_color].
    pub fn is_ansi_color_disabled() -> bool {
        // <https://no-color.org/>
        INITIALIZER.call_once(|| {
            NO_COLOR.store(
                std::env::var("NO_COLOR").is_ok_and(|e| !e.is_empty()),
                Ordering::SeqCst,
            );
        });
        NO_COLOR.load(Ordering::SeqCst)
    }

    /// Overrides detection of the `NO_COLOR` environment variable.
    ///
    /// Pass `true` to ensure that ANSI color codes are always included when displaying this type
    /// or `false` to ensure ANSI color codes are never included.
    pub fn force_ansi_color(enable_color: bool) {
        // Run the `Once` first so this override is not later overwritten by the `Once` fn.
        let _ = Self::is_ansi_color_disabled();
        NO_COLOR.store(!enable_color, Ordering::SeqCst);
    }
}

impl Display for Stylized<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let no_color = Self::is_ansi_color_disabled();
        let mut styles = self
            .styles
            .iter()
            .filter(|sgr| {
                !(no_color
                    && matches!(
                        sgr,
                        Sgr::Foreground(_) | Sgr::Background(_) | Sgr::UnderlineColor(_)
                    ))
            })
            .peekable();

        if styles.peek().is_none() {
            write!(f, "{}", self.content)?;
        } else {
            write!(f, "{}0", escape::CSI)?;
            for sgr in styles {
                write!(f, ";{sgr}")?;
            }
            write!(f, "m{}{}", self.content, Csi::Sgr(Sgr::Reset))?;
        }
        Ok(())
    }
}

pub trait StyleExt<'a>: Sized {
    fn stylized(self) -> Stylized<'a>;

    fn foreground(self, color: impl Into<ColorSpec>) -> Stylized<'a> {
        let mut this = self.stylized();
        this.styles.push(Sgr::Foreground(color.into()));
        this
    }
    fn red(self) -> Stylized<'a> {
        self.foreground(ColorSpec::RED)
    }
    fn yellow(self) -> Stylized<'a> {
        self.foreground(ColorSpec::YELLOW)
    }
    fn green(self) -> Stylized<'a> {
        self.foreground(ColorSpec::GREEN)
    }
    fn underlined(self) -> Stylized<'a> {
        let mut this = self.stylized();
        this.styles.push(Sgr::Underline(Underline::Single));
        this
    }
    fn bold(self) -> Stylized<'a> {
        let mut this = self.stylized();
        this.styles.push(Sgr::Intensity(Intensity::Bold));
        this
    }
}

impl<'a> StyleExt<'a> for Cow<'a, str> {
    fn stylized(self) -> Stylized<'a> {
        Stylized {
            content: self,
            styles: Vec::with_capacity(2),
        }
    }
}

impl<'a> StyleExt<'a> for &'a str {
    fn stylized(self) -> Stylized<'a> {
        Cow::Borrowed(self).stylized()
    }
}

impl StyleExt<'static> for String {
    fn stylized(self) -> Stylized<'static> {
        Cow::<str>::Owned(self).stylized()
    }
}

// NOTE: this allows chaining like `"hello".green().bold()`.
impl<'a> StyleExt<'a> for Stylized<'a> {
    fn stylized(self) -> Stylized<'a> {
        self
    }
}
