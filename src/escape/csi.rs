// CREDIT: This module was incrementally yanked from
// <https://github.com/wezterm/wezterm/blob/a87358516004a652ad840bc1661bdf65ffc89b43/termwiz/src/escape/osc.rs>.
// I've made stylistic changes like eliminating some traits (FromPrimitive, ToPrimitive) and only
// copied parts - TermWiz has a more complete set of OSC escapes. I have added some new escapes
// though, for example Contour's theme mode extension in `Mode::QueryTheme` and friends.

use std::fmt::{self, Display};

use crate::{
    escape::OneBased,
    event::Modifiers,
    style::{Blink, ColorSpec, CursorStyle, Font, Intensity, RgbaColor, Underline, VerticalAlign},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Csi {
    /// "Select Graphics Rendition" (SGR).
    /// These sequences affect how the cell is rendered by the terminal.
    Sgr(Sgr),
    Cursor(Cursor),
    Edit(Edit),
    Mode(Mode),
    Mouse(MouseReport),
    Keyboard(Keyboard),
    Device(Device),
    // TODO: Window(Box<Window>),
}

impl Display for Csi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // This here is the "control sequence introducer" (CSI):
        write!(f, "\x1b[")?;
        match self {
            Self::Sgr(sgr) => write!(f, "{sgr}m"),
            Self::Cursor(cursor) => cursor.fmt(f),
            Self::Edit(edit) => edit.fmt(f),
            Self::Mode(mode) => mode.fmt(f),
            Self::Mouse(report) => report.fmt(f),
            Self::Keyboard(keyboard) => keyboard.fmt(f),
            Self::Device(device) => device.fmt(f),
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
    Reverse(bool),
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
        fn write_true_color(
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
            Self::Reverse(true) => write!(f, "7")?,
            Self::Reverse(false) => write!(f, "27")?,
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
            Self::Foreground(ColorSpec::TrueColor(color)) => write_true_color(38, *color, f)?,
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
            Self::Background(ColorSpec::TrueColor(color)) => write_true_color(48, *color, f)?,
            Self::UnderlineColor(ColorSpec::Reset) => write!(f, "59")?,
            Self::UnderlineColor(ColorSpec::PaletteIndex(idx)) => write!(f, "58:5:{idx}")?,
            Self::UnderlineColor(ColorSpec::TrueColor(color)) => {
                write_true_color(58, *color, f)?;
            }
        }
        Ok(())
    }
}

// Cursor

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cursor {
    /// CBT Moves cursor to the Ps tabs backward. The default value of Ps is 1.
    BackwardTabulation(u32),

    /// TBC - TABULATION CLEAR
    TabulationClear(TabulationClear),

    /// CHA: Moves cursor to the Ps-th column of the active line. The default
    /// value of Ps is 1.
    CharacterAbsolute(OneBased),

    /// HPA CHARACTER POSITION ABSOLUTE
    /// HPA Moves cursor to the Ps-th column of the active line. The default
    /// value of Ps is 1.
    CharacterPositionAbsolute(OneBased),

    /// HPB - CHARACTER POSITION BACKWARD
    /// HPB Moves cursor to the left Ps columns. The default value of Ps is 1.
    CharacterPositionBackward(u32),

    /// HPR - CHARACTER POSITION FORWARD
    /// HPR Moves cursor to the right Ps columns. The default value of Ps is 1.
    CharacterPositionForward(u32),

    /// HVP - CHARACTER AND LINE POSITION
    /// HVP Moves cursor to the Ps1-th line and to the Ps2-th column. The
    /// default value of Ps1 and Ps2 is 1.
    CharacterAndLinePosition {
        line: OneBased,
        col: OneBased,
    },

    /// VPA - LINE POSITION ABSOLUTE
    /// Move to the corresponding vertical position (line Ps) of the current
    /// column. The default value of Ps is 1.
    LinePositionAbsolute(u32),

    /// VPB - LINE POSITION BACKWARD
    /// Moves cursor up Ps lines in the same column. The default value of Ps is
    /// 1.
    LinePositionBackward(u32),

    /// VPR - LINE POSITION FORWARD
    /// Moves cursor down Ps lines in the same column. The default value of Ps
    /// is 1.
    LinePositionForward(u32),

    /// CHT
    /// Moves cursor to the Ps tabs forward. The default value of Ps is 1.
    ForwardTabulation(u32),

    /// CNL Moves cursor to the first column of Ps-th following line. The
    /// default value of Ps is 1.
    NextLine(u32),

    /// CPL Moves cursor to the first column of Ps-th preceding line. The
    /// default value of Ps is 1.
    PrecedingLine(u32),

    /// CPR - ACTIVE POSITION REPORT
    /// If the DEVICE COMPONENT SELECT MODE (DCSM)
    /// is set to PRESENTATION, CPR is used to report the active presentation
    /// position of the sending device as residing in the presentation
    /// component at the n-th line position according to the line progression
    /// and at the m-th character position according to the character path,
    /// where n equals the value of Pn1 and m equal s the value of Pn2.
    /// If the DEVICE COMPONENT SELECT MODE (DCSM) is set to DATA, CPR is used
    /// to report the active data position of the sending device as
    /// residing in the data component at the n-th line position according
    /// to the line progression and at the m-th character position
    /// according to the character progression, where n equals the value of
    /// Pn1 and m equals the value of Pn2. CPR may be solicited by a DEVICE
    /// STATUS REPORT (DSR) or be sent unsolicited .
    ActivePositionReport {
        line: OneBased,
        col: OneBased,
    },

    /// CPR: this is the request from the client.
    /// The terminal will respond with ActivePositionReport.
    RequestActivePositionReport,

    /// SCP - Save Cursor Position.
    /// Only works when DECLRMM is disabled
    SaveCursor,
    RestoreCursor,

    /// CTC - CURSOR TABULATION CONTROL
    /// CTC causes one or more tabulation stops to be set or cleared in the
    /// presentation component, depending on the parameter values.
    /// In the case of parameter values 0, 2 or 4 the number of lines affected
    /// depends on the setting of the TABULATION STOP MODE (TSM).
    TabulationControl(CursorTabulationControl),

    /// CUB - Cursor Left
    /// Moves cursor to the left Ps columns. The default value of Ps is 1.
    Left(u32),

    /// CUD - Cursor Down
    Down(u32),

    /// CUF - Cursor Right
    Right(u32),

    /// CUU - Cursor Up
    Up(u32),

    /// CUP - Cursor Position
    /// Moves cursor to the Ps1-th line and to the Ps2-th column. The default
    /// value of Ps1 and Ps2 is 1.
    Position {
        line: OneBased,
        col: OneBased,
    },

    /// CVT - Cursor Line Tabulation
    /// CVT causes the active presentation position to be moved to the
    /// corresponding character position of the line corresponding to the n-th
    /// following line tabulation stop in the presentation component, where n
    /// equals the value of Pn.
    LineTabulation(u32),

    /// DECSTBM - Set top and bottom margins.
    SetTopAndBottomMargins {
        top: OneBased,
        bottom: OneBased,
    },

    /// https://vt100.net/docs/vt510-rm/DECSLRM.html
    SetLeftAndRightMargins {
        left: OneBased,
        right: OneBased,
    },

    CursorStyle(CursorStyle),
}

impl Display for Cursor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn write_csi<T: Default + Eq + Display>(
            value: T,
            f: &mut fmt::Formatter<'_>,
            control: &str,
        ) -> fmt::Result {
            if value == T::default() {
                write!(f, "{control}")
            } else {
                write!(f, "{value}{control}")
            }
        }

        match self {
            Cursor::BackwardTabulation(n) => write_csi(*n, f, "Z"),
            Cursor::TabulationClear(n) => write_csi(*n, f, "g"),
            Cursor::CharacterAbsolute(n) => write_csi(*n, f, "G"),
            Cursor::CharacterPositionAbsolute(n) => write_csi(*n, f, "``"),
            Cursor::CharacterPositionBackward(n) => write_csi(*n, f, "j"),
            Cursor::CharacterPositionForward(n) => write_csi(*n, f, "a"),
            Cursor::CharacterAndLinePosition { line, col } => write!(f, "{line};{col}f"),
            Cursor::LinePositionAbsolute(n) => write_csi(*n, f, "d"),
            Cursor::LinePositionBackward(n) => write_csi(*n, f, "k"),
            Cursor::LinePositionForward(n) => write_csi(*n, f, "e"),
            Cursor::ForwardTabulation(n) => write_csi(*n, f, "I"),
            Cursor::NextLine(n) => write_csi(*n, f, "E"),
            Cursor::PrecedingLine(n) => write_csi(*n, f, "F"),
            Cursor::ActivePositionReport { line, col } => write!(f, "{line};{col}R"),
            Cursor::RequestActivePositionReport => write!(f, "6n"),
            Cursor::SaveCursor => write!(f, "s"),
            Cursor::RestoreCursor => write!(f, "u"),
            Cursor::TabulationControl(n) => write_csi(*n, f, "W"),
            Cursor::Left(n) => write_csi(*n, f, "D"),
            Cursor::Down(n) => write_csi(*n, f, "B"),
            Cursor::Right(n) => write_csi(*n, f, "C"),
            Cursor::Up(n) => write_csi(*n, f, "A"),
            Cursor::Position { line, col } => write!(f, "{line};{col}H"),
            Cursor::LineTabulation(n) => write_csi(*n, f, "Y"),
            Cursor::SetTopAndBottomMargins { top, bottom } => {
                if top.get() == 1 && bottom.get() == u16::MAX {
                    write!(f, "r")
                } else {
                    write!(f, "{top};{bottom}r")
                }
            }
            Cursor::SetLeftAndRightMargins { left, right } => {
                if left.get() == 1 && right.get() == u16::MAX {
                    write!(f, "s")
                } else {
                    write!(f, "{left};{right}s")
                }
            }
            Cursor::CursorStyle(style) => write!(f, "{} q", *style as u8),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CursorTabulationControl {
    #[default]
    SetCharacterTabStopAtActivePosition = 0,
    SetLineTabStopAtActiveLine = 1,
    ClearCharacterTabStopAtActivePosition = 2,
    ClearLineTabstopAtActiveLine = 3,
    ClearAllCharacterTabStopsAtActiveLine = 4,
    ClearAllCharacterTabStops = 5,
    ClearAllLineTabStops = 6,
}

impl Display for CursorTabulationControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TabulationClear {
    #[default]
    ClearCharacterTabStopAtActivePosition = 0,
    ClearLineTabStopAtActiveLine = 1,
    ClearCharacterTabStopsAtActiveLine = 2,
    ClearAllCharacterTabStops = 3,
    ClearAllLineTabStops = 4,
    ClearAllTabStops = 5,
}

impl Display for TabulationClear {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

// Edit

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    /// DCH - DELETE CHARACTER
    /// Deletes Ps characters from the cursor position to the right. The
    /// default value of Ps is 1. If the DEVICE COMPONENT SELECT MODE
    /// (DCSM) is set to PRESENTATION, DCH causes the contents of the
    /// active presentation position and, depending on the setting of the
    /// CHARACTER EDITING MODE (HEM), the contents of the n-1 preceding or
    /// following character positions to be removed from the presentation
    /// component, where n equals the value of Pn. The resulting gap is
    /// closed by shifting the contents of the adjacent character positions
    /// towards the active presentation position. At the other end of the
    /// shifted part, n character positions are put into the erased state.
    DeleteCharacter(u32),

    /// DL - DELETE LINE
    /// If the DEVICE COMPONENT SELECT MODE (DCSM) is set to PRESENTATION, DL
    /// causes the contents of the active line (the line that contains the
    /// active presentation position) and, depending on the setting of the
    /// LINE EDITING MODE (VEM), the contents of the n-1 preceding or
    /// following lines to be removed from the presentation component, where n
    /// equals the value of Pn. The resulting gap is closed by shifting the
    /// contents of a number of adjacent lines towards the active line. At
    /// the other end of the shifted part, n lines are put into the
    /// erased state.  The active presentation position is moved to the line
    /// home position in the active line. The line home position is
    /// established by the parameter value of SET LINE HOME (SLH). If the
    /// TABULATION STOP MODE (TSM) is set to SINGLE, character tabulation stops
    /// are cleared in the lines that are put into the erased state.  The
    /// extent of the shifted part is established by SELECT EDITING EXTENT
    /// (SEE).  Any occurrences of the start or end of a selected area, the
    /// start or end of a qualified area, or a tabulation stop in the shifted
    /// part, are also shifted.
    DeleteLine(u32),

    /// ECH - ERASE CHARACTER
    /// If the DEVICE COMPONENT SELECT MODE (DCSM) is set to PRESENTATION, ECH
    /// causes the active presentation position and the n-1 following
    /// character positions in the presentation component to be put into
    /// the erased state, where n equals the value of Pn.
    EraseCharacter(u32),

    /// EL - ERASE IN LINE
    /// If the DEVICE COMPONENT SELECT MODE (DCSM) is set to PRESENTATION, EL
    /// causes some or all character positions of the active line (the line
    /// which contains the active presentation position in the presentation
    /// component) to be put into the erased state, depending on the
    /// parameter values
    EraseInLine(EraseInLine),

    /// ICH - INSERT CHARACTER
    /// If the DEVICE COMPONENT SELECT MODE (DCSM) is set to PRESENTATION, ICH
    /// is used to prepare the insertion of n characters, by putting into the
    /// erased state the active presentation position and, depending on the
    /// setting of the CHARACTER EDITING MODE (HEM), the n-1 preceding or
    /// following character positions in the presentation component, where n
    /// equals the value of Pn. The previous contents of the active
    /// presentation position and an adjacent string of character positions are
    /// shifted away from the active presentation position. The contents of n
    /// character positions at the other end of the shifted part are removed.
    /// The active presentation position is moved to the line home position in
    /// the active line. The line home position is established by the parameter
    /// value of SET LINE HOME (SLH).
    InsertCharacter(u32),

    /// IL - INSERT LINE
    /// If the DEVICE COMPONENT SELECT MODE (DCSM) is set to PRESENTATION, IL
    /// is used to prepare the insertion of n lines, by putting into the
    /// erased state in the presentation component the active line (the
    /// line that contains the active presentation position) and, depending on
    /// the setting of the LINE EDITING MODE (VEM), the n-1 preceding or
    /// following lines, where n equals the value of Pn. The previous
    /// contents of the active line and of adjacent lines are shifted away
    /// from the active line. The contents of n lines at the other end of the
    /// shifted part are removed. The active presentation position is moved
    /// to the line home position in the active line. The line home
    /// position is established by the parameter value of SET LINE
    /// HOME (SLH).
    InsertLine(u32),

    /// SD - SCROLL DOWN
    /// SD causes the data in the presentation component to be moved by n line
    /// positions if the line orientation is horizontal, or by n character
    /// positions if the line orientation is vertical, such that the data
    /// appear to move down; where n equals the value of Pn. The active
    /// presentation position is not affected by this control function.
    ///
    /// Also known as Pan Up in DEC:
    /// https://vt100.net/docs/vt510-rm/SD.html
    ScrollDown(u32),

    /// SU - SCROLL UP
    /// SU causes the data in the presentation component to be moved by n line
    /// positions if the line orientation is horizontal, or by n character
    /// positions if the line orientation is vertical, such that the data
    /// appear to move up; where n equals the value of Pn. The active
    /// presentation position is not affected by this control function.
    ScrollUp(u32),

    /// ED - ERASE IN PAGE (XTerm calls this Erase in Display)
    EraseInDisplay(EraseInDisplay),

    /// REP - Repeat the preceding character n times
    Repeat(u32),
}

impl Display for Edit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn write_csi(param: u32, f: &mut fmt::Formatter<'_>, control: &str) -> fmt::Result {
            if param == 1 {
                write!(f, "{control}")
            } else {
                write!(f, "{param}{control}")
            }
        }

        match self {
            Self::DeleteCharacter(n) => write_csi(*n, f, "P"),
            Self::DeleteLine(n) => write_csi(*n, f, "M"),
            Self::EraseCharacter(n) => write_csi(*n, f, "X"),
            Self::EraseInLine(n) => write_csi(*n as u32, f, "K"),
            Self::InsertCharacter(n) => write_csi(*n, f, "@"),
            Self::InsertLine(n) => write_csi(*n, f, "L"),
            Self::ScrollDown(n) => write_csi(*n, f, "T"),
            Self::ScrollUp(n) => write_csi(*n, f, "S"),
            Self::EraseInDisplay(n) => write_csi(*n as u32, f, "J"),
            Self::Repeat(n) => write_csi(*n, f, "b"),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum EraseInLine {
    #[default]
    EraseToEndOfLine = 0,
    EraseToStartOfLine = 1,
    EraseLine = 2,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum EraseInDisplay {
    /// the active presentation position and the character positions up to the
    /// end of the page are put into the erased state
    #[default]
    EraseToEndOfDisplay = 0,
    /// the character positions from the beginning of the page up to and
    /// including the active presentation position are put into the erased
    /// state
    EraseToStartOfDisplay = 1,
    /// all character positions of the page are put into the erased state
    EraseDisplay = 2,
    /// Clears the scrollback.  This is an Xterm extension to ECMA-48.
    EraseScrollback = 3,
}

// Mode

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    SetDecPrivateMode(DecPrivateMode),
    ResetDecPrivateMode(DecPrivateMode),
    SaveDecPrivateMode(DecPrivateMode),
    RestoreDecPrivateMode(DecPrivateMode),
    // <https://vt100.net/docs/vt510-rm/DECRQM.html>
    QueryDecPrivateMode(DecPrivateMode),
    // <https://vt100.net/docs/vt510-rm/DECRPM.html>
    ReportDecPrivateMode {
        mode: DecPrivateMode,
        setting: DecModeSetting,
    },
    SetMode(TerminalMode),
    ResetMode(TerminalMode),
    QueryMode(TerminalMode),
    XtermKeyMode {
        resource: XtermKeyModifierResource,
        value: Option<i64>,
    },
    // <https://github.com/contour-terminal/contour/blob/master/docs/vt-extensions/color-palette-update-notifications.md>
    QueryTheme,
    ReportTheme(ThemeMode),
}

impl Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SetDecPrivateMode(mode) => write!(f, "?{mode}h"),
            Self::ResetDecPrivateMode(mode) => write!(f, "?{mode}l"),
            Self::SaveDecPrivateMode(mode) => write!(f, "?{mode}s"),
            Self::RestoreDecPrivateMode(mode) => write!(f, "?{mode}r"),
            Self::QueryDecPrivateMode(mode) => write!(f, "?{mode}$p"),
            Self::ReportDecPrivateMode { mode, setting } => {
                write!(f, "?{mode};{}$y", *setting as u8)
            }
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
            Self::QueryTheme => write!(f, "?996n"),
            Self::ReportTheme(mode) => write!(f, "?997;{}n", *mode as u8),
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
    RXVTMouse = 1015,
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

    /// <https://github.com/contour-terminal/contour/blob/master/docs/vt-extensions/color-palette-update-notifications.md>
    Theme = 2031,

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecModeSetting {
    NotRecognized = 0,
    Set = 1,
    Reset = 2,
    PermanentlySet = 3,
    PermanentlyReset = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark = 1,
    Light = 2,
}

// Mouse

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseReport {
    Sgr1006 {
        x: u16,
        y: u16,
        button: MouseButton,
        modifiers: Modifiers,
    },
    Sgr1016 {
        x_pixels: u16,
        y_pixels: u16,
        button: MouseButton,
        modifiers: Modifiers,
    },
}

impl Display for MouseReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MouseReport::Sgr1006 {
                x,
                y,
                button,
                modifiers,
            } => {
                let mut b = 0;
                // TODO: check this.
                if (*modifiers & Modifiers::SHIFT) != Modifiers::NONE {
                    b |= 4;
                }
                if (*modifiers & Modifiers::ALT) != Modifiers::NONE {
                    b |= 8;
                }
                if (*modifiers & Modifiers::CONTROL) != Modifiers::NONE {
                    b |= 16;
                }
                b |= match button {
                    MouseButton::Button1Press | MouseButton::Button1Release => 0,
                    MouseButton::Button2Press | MouseButton::Button2Release => 1,
                    MouseButton::Button3Press | MouseButton::Button3Release => 2,
                    MouseButton::Button4Press | MouseButton::Button4Release => 64,
                    MouseButton::Button5Press | MouseButton::Button5Release => 65,
                    MouseButton::Button6Press | MouseButton::Button6Release => 66,
                    MouseButton::Button7Press | MouseButton::Button7Release => 67,
                    MouseButton::Button1Drag => 32,
                    MouseButton::Button2Drag => 33,
                    MouseButton::Button3Drag => 34,
                    MouseButton::None => 35,
                };
                let trailer = match button {
                    MouseButton::Button1Press
                    | MouseButton::Button2Press
                    | MouseButton::Button3Press
                    | MouseButton::Button4Press
                    | MouseButton::Button5Press
                    | MouseButton::Button1Drag
                    | MouseButton::Button2Drag
                    | MouseButton::Button3Drag
                    | MouseButton::None => 'M',
                    _ => 'm',
                };
                write!(f, "<{b};{x};{y}{trailer}")
            }
            MouseReport::Sgr1016 {
                x_pixels,
                y_pixels,
                button,
                modifiers,
            } => {
                let mut b = 0;
                // TODO: check this.
                if (*modifiers & Modifiers::SHIFT) != Modifiers::NONE {
                    b |= 4;
                }
                if (*modifiers & Modifiers::ALT) != Modifiers::NONE {
                    b |= 8;
                }
                if (*modifiers & Modifiers::CONTROL) != Modifiers::NONE {
                    b |= 16;
                }
                b |= match button {
                    MouseButton::Button1Press | MouseButton::Button1Release => 0,
                    MouseButton::Button2Press | MouseButton::Button2Release => 1,
                    MouseButton::Button3Press | MouseButton::Button3Release => 2,
                    MouseButton::Button4Press | MouseButton::Button4Release => 64,
                    MouseButton::Button5Press | MouseButton::Button5Release => 65,
                    MouseButton::Button6Press | MouseButton::Button6Release => 66,
                    MouseButton::Button7Press | MouseButton::Button7Release => 67,
                    MouseButton::Button1Drag => 32,
                    MouseButton::Button2Drag => 33,
                    MouseButton::Button3Drag => 34,
                    MouseButton::None => 35,
                };
                let trailer = match button {
                    MouseButton::Button1Press
                    | MouseButton::Button2Press
                    | MouseButton::Button3Press
                    | MouseButton::Button4Press
                    | MouseButton::Button5Press
                    | MouseButton::Button1Drag
                    | MouseButton::Button2Drag
                    | MouseButton::Button3Drag
                    | MouseButton::None => 'M',
                    _ => 'm',
                };
                write!(f, "<{b};{x_pixels};{y_pixels}{trailer}")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Button1Press,
    Button2Press,
    Button3Press,
    Button4Press,
    Button5Press,
    Button6Press,
    Button7Press,
    Button1Release,
    Button2Release,
    Button3Release,
    Button4Release,
    Button5Release,
    Button6Release,
    Button7Release,
    Button1Drag,
    Button2Drag,
    Button3Drag,
    None,
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
    /// A report from the terminal declaring which flags are currently set.
    ReportFlags(KittyKeyboardFlags),
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
            // NOTE: this is sent by the terminal, not meant to be sent by the application.
            Self::ReportFlags(flags) => write!(f, "?{flags}u"),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Device {
    DeviceAttributes(()),
    /// DECSTR - https://vt100.net/docs/vt510-rm/DECSTR.html
    SoftReset,
    RequestPrimaryDeviceAttributes,
    RequestSecondaryDeviceAttributes,
    RequestTertiaryDeviceAttributes,
    StatusReport,
    /// https://github.com/mintty/mintty/issues/881
    /// https://gitlab.gnome.org/GNOME/vte/-/issues/235
    RequestTerminalNameAndVersion,
    RequestTerminalParameters(i64),
}

impl Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DeviceAttributes(_) => unimplemented!(),
            Self::SoftReset => write!(f, "!p"),
            Self::RequestPrimaryDeviceAttributes => write!(f, "c"),
            Self::RequestSecondaryDeviceAttributes => write!(f, ">c"),
            Self::RequestTertiaryDeviceAttributes => write!(f, "=c"),
            Self::StatusReport => write!(f, "5n"),
            Self::RequestTerminalNameAndVersion => write!(f, ">q"),
            Self::RequestTerminalParameters(n) => write!(f, "{};1;1;128;128;1;0x", n + 2),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const ENTER_ALTERNATE_SCREEN: Csi = Csi::Mode(Mode::SetDecPrivateMode(DecPrivateMode::Code(
        DecPrivateModeCode::ClearAndEnableAlternateScreen,
    )));

    const EXIT_ALTERNATE_SCREEN: Csi = Csi::Mode(Mode::ResetDecPrivateMode(DecPrivateMode::Code(
        DecPrivateModeCode::ClearAndEnableAlternateScreen,
    )));

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
