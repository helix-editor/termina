use std::fmt::{self, Display};

use crate::style::{Blink, ColorSpec, Font, Intensity, RgbaColor, Underline, VerticalAlign};

pub(crate) const ENTER_ALTERNATE_SCREEN: Csi = Csi::Mode(Mode::SetDecPrivateMode(
    DecPrivateMode::Code(DecPrivateModeCode::ClearAndEnableAlternateScreen),
));

pub(crate) const EXIT_ALTERNATE_SCREEN: Csi = Csi::Mode(Mode::ResetDecPrivateMode(
    DecPrivateMode::Code(DecPrivateModeCode::ClearAndEnableAlternateScreen),
));

// TODO: no Copy?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Csi {
    /// "Set Graphics Rendition" (SGR).
    /// These sequences affect how the cell is rendered by the terminal.
    Sgr(Sgr),
    Mode(Mode),
    Keyboard(Keyboard),
    // TODO...
}

impl Display for Csi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // This here is the "control sequence introducer" (CSI):
        write!(f, "\x1b[")?;
        match self {
            Self::Sgr(sgr) => sgr.fmt(f),
            Self::Mode(mode) => mode.fmt(f),
            Self::Keyboard(keyboard) => keyboard.fmt(f),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sgr {
    /// Resets the graphics rendition to default.
    Reset,
    Intensity(Intensity),
    Underline(Underline),
    Blink(Blink),
    Italic(bool),
    Inverse(bool),
    Invisible(bool),
    StrikeThrough(bool),
    Overline(bool),
    Font(Font),
    VerticalAlign(VerticalAlign),
    Foreground(ColorSpec),
    Background(ColorSpec),
    UnderlineColor(ColorSpec),
}

impl Display for Sgr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn format_true_color(
            code: u8,
            RgbaColor {
                red,
                green,
                blue,
                alpha,
            }: RgbaColor,
            f: &mut fmt::Formatter,
        ) -> fmt::Result {
            if alpha == 255 {
                write!(f, "{code}:2::{red}:{green}:{blue}")
            } else {
                write!(f, "{code}:6::{red}:{green}:{blue}:{alpha}")
            }
        }

        // CSI <n> m
        match self {
            Self::Reset => write!(f, "0")?,
            Self::Intensity(Intensity::Normal) => write!(f, "22")?,
            Self::Intensity(Intensity::Bold) => write!(f, "1")?,
            Self::Intensity(Intensity::Dim) => write!(f, "2")?,
            Self::Underline(Underline::None) => write!(f, "24")?,
            Self::Underline(Underline::Single) => write!(f, "4")?,
            Self::Underline(Underline::Double) => write!(f, "21")?,
            Self::Underline(Underline::Curly) => write!(f, "4:3")?,
            Self::Underline(Underline::Dotted) => write!(f, "4:4")?,
            Self::Underline(Underline::Dashed) => write!(f, "4:5")?,
            Self::Blink(Blink::None) => write!(f, "25")?,
            Self::Blink(Blink::Slow) => write!(f, "5")?,
            Self::Blink(Blink::Rapid) => write!(f, "6")?,
            Self::Italic(true) => write!(f, "3")?,
            Self::Italic(false) => write!(f, "23")?,
            Self::Inverse(true) => write!(f, "7")?,
            Self::Inverse(false) => write!(f, "27")?,
            Self::Invisible(true) => write!(f, "8")?,
            Self::Invisible(false) => write!(f, "28")?,
            Self::StrikeThrough(true) => write!(f, "9")?,
            Self::StrikeThrough(false) => write!(f, "29")?,
            Self::Overline(true) => write!(f, "53")?,
            Self::Overline(false) => write!(f, "55")?,
            Self::Font(Font::Default) => write!(f, "10")?,
            Self::Font(Font::Alternate(1)) => write!(f, "11")?,
            Self::Font(Font::Alternate(2)) => write!(f, "12")?,
            Self::Font(Font::Alternate(3)) => write!(f, "13")?,
            Self::Font(Font::Alternate(4)) => write!(f, "14")?,
            Self::Font(Font::Alternate(5)) => write!(f, "15")?,
            Self::Font(Font::Alternate(6)) => write!(f, "16")?,
            Self::Font(Font::Alternate(7)) => write!(f, "17")?,
            Self::Font(Font::Alternate(8)) => write!(f, "18")?,
            Self::Font(Font::Alternate(9)) => write!(f, "19")?,
            Self::Font(_) => (),
            Self::VerticalAlign(VerticalAlign::BaseLine) => write!(f, "75")?,
            Self::VerticalAlign(VerticalAlign::SuperScript) => write!(f, "73")?,
            Self::VerticalAlign(VerticalAlign::SubScript) => write!(f, "74")?,
            Self::Foreground(ColorSpec::Reset) => write!(f, "39")?,
            Self::Foreground(ColorSpec::BLACK) => write!(f, "30")?,
            Self::Foreground(ColorSpec::RED) => write!(f, "31")?,
            Self::Foreground(ColorSpec::GREEN) => write!(f, "32")?,
            Self::Foreground(ColorSpec::YELLOW) => write!(f, "33")?,
            Self::Foreground(ColorSpec::BLUE) => write!(f, "34")?,
            Self::Foreground(ColorSpec::MAGENTA) => write!(f, "35")?,
            Self::Foreground(ColorSpec::CYAN) => write!(f, "36")?,
            Self::Foreground(ColorSpec::WHITE) => write!(f, "37")?,
            Self::Foreground(ColorSpec::BRIGHT_BLACK) => write!(f, "90")?,
            Self::Foreground(ColorSpec::BRIGHT_RED) => write!(f, "91")?,
            Self::Foreground(ColorSpec::BRIGHT_GREEN) => write!(f, "92")?,
            Self::Foreground(ColorSpec::BRIGHT_YELLOW) => write!(f, "93")?,
            Self::Foreground(ColorSpec::BRIGHT_BLUE) => write!(f, "94")?,
            Self::Foreground(ColorSpec::BRIGHT_MAGENTA) => write!(f, "95")?,
            Self::Foreground(ColorSpec::BRIGHT_CYAN) => write!(f, "96")?,
            Self::Foreground(ColorSpec::BRIGHT_WHITE) => write!(f, "97")?,
            Self::Foreground(ColorSpec::PaletteIndex(idx)) => write!(f, "38:5:{idx}")?,
            Self::Foreground(ColorSpec::TrueColor(color)) => format_true_color(38, *color, f)?,
            Self::Background(ColorSpec::Reset) => write!(f, "49")?,
            Self::Background(ColorSpec::BLACK) => write!(f, "40")?,
            Self::Background(ColorSpec::RED) => write!(f, "41")?,
            Self::Background(ColorSpec::GREEN) => write!(f, "42")?,
            Self::Background(ColorSpec::YELLOW) => write!(f, "43")?,
            Self::Background(ColorSpec::BLUE) => write!(f, "44")?,
            Self::Background(ColorSpec::MAGENTA) => write!(f, "45")?,
            Self::Background(ColorSpec::CYAN) => write!(f, "46")?,
            Self::Background(ColorSpec::WHITE) => write!(f, "47")?,
            Self::Background(ColorSpec::BRIGHT_BLACK) => write!(f, "100")?,
            Self::Background(ColorSpec::BRIGHT_RED) => write!(f, "101")?,
            Self::Background(ColorSpec::BRIGHT_GREEN) => write!(f, "102")?,
            Self::Background(ColorSpec::BRIGHT_YELLOW) => write!(f, "103")?,
            Self::Background(ColorSpec::BRIGHT_BLUE) => write!(f, "104")?,
            Self::Background(ColorSpec::BRIGHT_MAGENTA) => write!(f, "105")?,
            Self::Background(ColorSpec::BRIGHT_CYAN) => write!(f, "106")?,
            Self::Background(ColorSpec::BRIGHT_WHITE) => write!(f, "107")?,
            Self::Background(ColorSpec::PaletteIndex(idx)) => write!(f, "48:5:{idx}")?,
            Self::Background(ColorSpec::TrueColor(color)) => format_true_color(48, *color, f)?,
            Self::UnderlineColor(ColorSpec::Reset) => write!(f, "59")?,
            Self::UnderlineColor(ColorSpec::PaletteIndex(idx)) => write!(f, "58:5:{idx}")?,
            Self::UnderlineColor(ColorSpec::TrueColor(color)) => {
                format_true_color(58, *color, f)?;
            }
        }
        write!(f, "m")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    SetDecPrivateMode(DecPrivateMode),
    ResetDecPrivateMode(DecPrivateMode),
    SaveDecPrivateMode(DecPrivateMode),
    RestoreDecPrivateMode(DecPrivateMode),
    QueryDecPrivateMode(DecPrivateMode),
    SetMode(TerminalMode),
    ResetMode(TerminalMode),
    QueryMode(TerminalMode),
    XtermKeyMode {
        resource: XtermKeyModifierResource,
        value: Option<i64>,
    },
}

impl Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SetDecPrivateMode(mode) => write!(f, "?{mode}h"),
            Self::ResetDecPrivateMode(mode) => write!(f, "?{mode}l"),
            Self::SaveDecPrivateMode(mode) => write!(f, "?{mode}s"),
            Self::RestoreDecPrivateMode(mode) => write!(f, "?{mode}r"),
            Self::QueryDecPrivateMode(mode) => write!(f, "?{mode}$p"),
            Self::SetMode(mode) => write!(f, "{mode}h"),
            Self::ResetMode(mode) => write!(f, "{mode}l"),
            Self::QueryMode(mode) => write!(f, "?{mode}$p"),
            Self::XtermKeyMode { resource, value } => {
                write!(f, ">{}", *resource as u8)?;
                if let Some(value) = value {
                    write!(f, ";{}", value)?;
                } else {
                    write!(f, ";")?;
                }
                write!(f, "m")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecPrivateMode {
    Code(DecPrivateModeCode),
    Unspecified(u16),
}

impl Display for DecPrivateMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match *self {
            Self::Code(code) => code as u16,
            Self::Unspecified(code) => code,
        };
        write!(f, "{code}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecPrivateModeCode {
    /// https://vt100.net/docs/vt510-rm/DECCKM.html
    /// This mode is only effective when the terminal is in keypad application mode (see DECKPAM)
    /// and the ANSI/VT52 mode (DECANM) is set (see DECANM). Under these conditions, if the cursor
    /// key mode is reset, the four cursor function keys will send ANSI cursor control commands. If
    /// cursor key mode is set, the four cursor function keys will send application functions.
    ApplicationCursorKeys = 1,

    /// https://vt100.net/docs/vt510-rm/DECANM.html
    /// Behave like a vt52
    DecAnsiMode = 2,

    /// https://vt100.net/docs/vt510-rm/DECCOLM.html
    Select132Columns = 3,
    /// https://vt100.net/docs/vt510-rm/DECSCLM.html
    SmoothScroll = 4,
    /// https://vt100.net/docs/vt510-rm/DECSCNM.html
    ReverseVideo = 5,
    /// https://vt100.net/docs/vt510-rm/DECOM.html
    /// When OriginMode is enabled, cursor is constrained to the
    /// scroll region and its position is relative to the scroll
    /// region.
    OriginMode = 6,
    /// https://vt100.net/docs/vt510-rm/DECAWM.html
    /// When enabled, wrap to next line, Otherwise replace the last
    /// character
    AutoWrap = 7,
    /// https://vt100.net/docs/vt510-rm/DECARM.html
    AutoRepeat = 8,
    StartBlinkingCursor = 12,
    ShowCursor = 25,

    ReverseWraparound = 45,

    /// https://vt100.net/docs/vt510-rm/DECLRMM.html
    LeftRightMarginMode = 69,

    /// DECSDM - https://vt100.net/dec/ek-vt38t-ug-001.pdf#page=132
    SixelDisplayMode = 80,
    /// Enable mouse button press/release reporting
    MouseTracking = 1000,
    /// Warning: this requires a cooperative and timely response from
    /// the application otherwise the terminal can hang
    HighlightMouseTracking = 1001,
    /// Enable mouse button press/release and drag reporting
    ButtonEventMouse = 1002,
    /// Enable mouse motion, button press/release and drag reporting
    AnyEventMouse = 1003,
    /// Enable FocusIn/FocusOut events
    FocusTracking = 1004,
    Utf8Mouse = 1005,
    /// Use extended coordinate system in mouse reporting.  Does not
    /// enable mouse reporting itself, it just controls how reports
    /// will be encoded.
    SGRMouse = 1006,
    /// Use pixels rather than text cells in mouse reporting.  Does
    /// not enable mouse reporting itself, it just controls how
    /// reports will be encoded.
    SGRPixelsMouse = 1016,

    XTermMetaSendsEscape = 1036,
    XTermAltSendsEscape = 1039,

    /// Save cursor as in DECSC
    SaveCursor = 1048,
    ClearAndEnableAlternateScreen = 1049,
    EnableAlternateScreen = 47,
    OptEnableAlternateScreen = 1047,
    BracketedPaste = 2004,

    /// <https://github.com/contour-terminal/terminal-unicode-core/>
    /// Grapheme clustering mode
    GraphemeClustering = 2027,

    /// Applies to sixel and regis modes
    UsePrivateColorRegistersForEachGraphic = 1070,

    /// <https://gist.github.com/christianparpart/d8a62cc1ab659194337d73e399004036>
    SynchronizedOutput = 2026,

    MinTTYApplicationEscapeKeyMode = 7727,

    /// xterm: adjust cursor positioning after emitting sixel
    SixelScrollsRight = 8452,

    /// Windows Terminal: win32-input-mode
    /// <https://github.com/microsoft/terminal/blob/main/doc/specs/%234999%20-%20Improved%20keyboard%20handling%20in%20Conpty.md>
    Win32InputMode = 9001,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalMode {
    Code(TerminalModeCode),
    Unspecified(u16),
}

impl Display for TerminalMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match *self {
            Self::Code(code) => code as u16,
            Self::Unspecified(code) => code,
        };
        write!(f, "{code}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalModeCode {
    /// https://vt100.net/docs/vt510-rm/KAM.html
    KeyboardAction = 2,
    /// https://vt100.net/docs/vt510-rm/IRM.html
    Insert = 4,
    /// <https://terminal-wg.pages.freedesktop.org/bidi/recommendation/escape-sequences.html>
    BiDirectionalSupportMode = 8,
    /// https://vt100.net/docs/vt510-rm/SRM.html
    /// But in the MS terminal this is cursor blinking.
    SendReceive = 12,
    /// https://vt100.net/docs/vt510-rm/LNM.html
    AutomaticNewline = 20,
    /// MS terminal cursor visibility
    ShowCursor = 25,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XtermKeyModifierResource {
    Keyboard = 0,
    CursorKeys = 1,
    FunctionKeys = 2,
    OtherKeys = 4,
}

// --- Kitty keyboard protocol ---
//
// <https://sw.kovidgoyal.net/kitty/keyboard-protocol/>.

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct KittyKeyboardFlags: u8 {
        const NONE = 0;
        const DISAMBIGUATE_ESCAPE_CODES = 1;
        const REPORT_EVENT_TYPES = 2;
        const REPORT_ALTERNATE_KEYS = 4;
        const REPORT_ALL_KEYS_AS_ESCAPE_CODES = 8;
        const REPORT_ASSOCIATED_TEXT = 16;
    }
}

impl Display for KittyKeyboardFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.bits())
    }
}

/// CSI sequences for interacting with the [Kitty Keyboard
/// Protocol](https://sw.kovidgoyal.net/kitty/keyboard-protocol/).
///
/// Note that the Kitty Keyboard Protocol requires terminals to maintain different stacks for the
/// main and alternate screens. This means that applications which use alternate screens do not
/// need to pop flags (via `Self::PopFlags`) when exiting. By exiting entering the main screen the
/// flags must be automatically reset by the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyboard {
    /// Query the current values of the flags.
    QueryFlags,
    /// Pushes the given flags onto the terminal's stack.
    PushFlags(KittyKeyboardFlags),
    /// Pops the given number of stack entries from the terminal's stack.
    PopFlags(u8),
    /// Requests keyboard enhancement with the given flags according to the mode.
    ///
    /// Also see [SetKeyboardFlagsMode].
    ///
    /// Applications such as editors which enter the alternate screen
    /// [crate::Terminal::enter_alternate_screen] should prefer `PushFlags` because the flags
    /// will be automatically dropped by the terminal when entering the main screen.
    SetFlags {
        flags: KittyKeyboardFlags,
        mode: SetKeyboardFlagsMode,
    },
}

impl Display for Keyboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::QueryFlags => write!(f, "?u"),
            Self::PushFlags(flags) => write!(f, ">{flags}u"),
            Self::PopFlags(number) => write!(f, "<{number}u"),
            Self::SetFlags { flags, mode } => write!(f, "={flags};{mode}u"),
        }
    }
}

/// Controls how the flags passed in [Keyboard::SetFlags] are interpreted by the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetKeyboardFlagsMode {
    /// Request any of the given flags and reset any flags which are not given.
    AssignAll = 1,
    /// Request the given flags and ignore any flags which are not given.
    SetSpecified = 2,
    /// Clear the given flags and ignore any flags which are not given.
    ClearSpecified = 3,
}

impl Display for SetKeyboardFlagsMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn encoding() {
        // Enter the alternate screen using the mode part of CSI.
        // <https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences#alternate-screen-buffer>
        assert_eq!("\x1b[?1049h", ENTER_ALTERNATE_SCREEN.to_string());
        assert_eq!("\x1b[?1049l", EXIT_ALTERNATE_SCREEN.to_string());

        // Push Kitty keyboard flags used by Helix and Kakoune at time of writing.
        assert_eq!(
            "\x1b[>5u",
            Csi::Keyboard(Keyboard::PushFlags(
                KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES
                    | KittyKeyboardFlags::REPORT_ALTERNATE_KEYS
            ))
            .to_string()
        );

        // Common SGR: turn the text (i.e. foreground) green
        assert_eq!(
            "\x1b[32m",
            Csi::Sgr(Sgr::Foreground(ColorSpec::GREEN)).to_string(),
        );
        // ... and then reset to turn off the green.
        assert_eq!(
            "\x1b[39m",
            Csi::Sgr(Sgr::Foreground(ColorSpec::Reset)).to_string(),
        );
    }
}
