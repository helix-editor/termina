use std::fmt::{self, Display};

pub(crate) const ENTER_ALTERNATE_SCREEN: Csi = Csi::Mode(Mode::SetDecPrivateMode(
    DecPrivateMode::Code(DecPrivateModeCode::ClearAndEnableAlternateScreen),
));

pub(crate) const EXIT_ALTERNATE_SCREEN: Csi = Csi::Mode(Mode::ResetDecPrivateMode(
    DecPrivateMode::Code(DecPrivateModeCode::ClearAndEnableAlternateScreen),
));

// TODO: no Copy?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Csi {
    Mode(Mode),
    Keyboard(Keyboard),
    // TODO...
}

impl Display for Csi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // This here is the "control sequence introducer" (CSI):
        write!(f, "\x1b[")?;
        match self {
            Self::Mode(mode) => mode.fmt(f),
            Self::Keyboard(keyboard) => keyboard.fmt(f),
        }
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
        // <https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences#alternate-screen-buffer>
        assert_eq!("\x1b[?1049h", ENTER_ALTERNATE_SCREEN.to_string());
        assert_eq!("\x1b[?1049l", EXIT_ALTERNATE_SCREEN.to_string());

        // Kitty keyboard flags used by Helix and Kakoune at time of writing
        assert_eq!(
            "\x1b[>5u",
            Csi::Keyboard(Keyboard::PushFlags(
                KittyKeyboardFlags::DISAMBIGUATE_ESCAPE_CODES
                    | KittyKeyboardFlags::REPORT_ALTERNATE_KEYS
            ))
            .to_string()
        );
    }
}
