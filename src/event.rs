// CREDIT: Most event code is adapted from crossterm. The main difference is that I include escape
// sequences like CSI and DCS in the `Event` struct and do not make a distinction between
// `InternalEvent` and `Event`. Otherwise all `KeyEvent` code is nearly identical to crossterm.

use crate::{
    escape::{csi::Csi, dcs::Dcs},
    WindowSize,
};

pub(crate) mod reader;
pub(crate) mod source;
#[cfg(feature = "event-stream")]
pub(crate) mod stream;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    /// The window was resized to the given dimensions.
    WindowResized(WindowSize),
    FocusIn,
    FocusOut,
    /// A "bracketed" paste.
    ///
    /// Normally pasting into a terminal with Ctrl+v (or Super+v) enters the pasted text as if
    /// you had typed the keys individually. Terminals commonly support ["bracketed
    /// paste"](https://en.wikipedia.org/wiki/Bracketed-paste) now however, which uses an escape
    /// sequence to deliver the entire pasted content.
    Paste(String),
    /// A parsed escape sequence starting with CSI (control sequence introducer).
    Csi(Csi),
    Dcs(Dcs),
}

impl Event {
    #[inline]
    pub fn is_escape(&self) -> bool {
        matches!(self, Self::Csi(_) | Self::Dcs(_))
    }
}

// CREDIT: <https://github.com/crossterm-rs/crossterm/blob/36d95b26a26e64b0f8c12edfe11f410a6d56a812/src/event.rs#L777-L1158>
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub kind: KeyEventKind,
    pub modifiers: Modifiers,
    pub state: KeyEventState,
}

impl KeyEvent {
    pub const fn new(code: KeyCode, modifiers: Modifiers) -> Self {
        Self {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }
}

impl From<KeyCode> for KeyEvent {
    fn from(code: KeyCode) -> Self {
        Self {
            code,
            kind: KeyEventKind::Press,
            modifiers: Modifiers::NONE,
            state: KeyEventState::NONE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEventKind {
    Press,
    Release,
    Repeat,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Modifiers: u8 {
        const NONE = 0;
        const SHIFT = 1 << 1;
        const ALT = 1 << 2;
        const CONTROL = 1 << 3;
        const SUPER = 1 << 4;
        const HYPER = 1 << 5;
        const META = 1 << 5;
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct KeyEventState: u8 {
        const NONE = 0;
        const KEYPAD = 1 << 1;
        const CAPS_LOCK = 1 << 2;
        const NUM_LOCK = 1 << 3;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Enter,
    Backspace,
    Tab,
    Escape,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    BackTab,
    PageUp,
    PageDown,
    Insert,
    Delete,
    KeypadBegin,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    Menu,
    Null,
    /// F1-F35 "function" keys
    Function(u8),
    Modifier(ModifierKeyCode),
    Media(MediaKeyCode),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierKeyCode {
    /// Left Shift key.
    LeftShift,
    /// Left Control key. (Control on macOS, Ctrl on other platforms)
    LeftControl,
    /// Left Alt key. (Option on macOS, Alt on other platforms)
    LeftAlt,
    /// Left Super key. (Command on macOS, Windows on Windows, Super on other platforms)
    LeftSuper,
    /// Left Hyper key.
    LeftHyper,
    /// Left Meta key.
    LeftMeta,
    /// Right Shift key.
    RightShift,
    /// Right Control key. (Control on macOS, Ctrl on other platforms)
    RightControl,
    /// Right Alt key. (Option on macOS, Alt on other platforms)
    RightAlt,
    /// Right Super key. (Command on macOS, Windows on Windows, Super on other platforms)
    RightSuper,
    /// Right Hyper key.
    RightHyper,
    /// Right Meta key.
    RightMeta,
    /// Iso Level3 Shift key.
    IsoLevel3Shift,
    /// Iso Level5 Shift key.
    IsoLevel5Shift,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKeyCode {
    /// Play media key.
    Play,
    /// Pause media key.
    Pause,
    /// Play/Pause media key.
    PlayPause,
    /// Reverse media key.
    Reverse,
    /// Stop media key.
    Stop,
    /// Fast-forward media key.
    FastForward,
    /// Rewind media key.
    Rewind,
    /// Next-track media key.
    TrackNext,
    /// Previous-track media key.
    TrackPrevious,
    /// Record media key.
    Record,
    /// Lower-volume media key.
    LowerVolume,
    /// Raise-volume media key.
    RaiseVolume,
    /// Mute media key.
    MuteVolume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseEvent {
    /// The kind of mouse event that was caused.
    pub kind: MouseEventKind,
    /// The column that the event occurred on.
    pub column: u16,
    /// The row that the event occurred on.
    pub row: u16,
    /// The key modifiers active when the event occurred.
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEventKind {
    /// Pressed mouse button. Contains the button that was pressed.
    Down(MouseButton),
    /// Released mouse button. Contains the button that was released.
    Up(MouseButton),
    /// Moved the mouse cursor while pressing the contained mouse button.
    Drag(MouseButton),
    /// Moved the mouse cursor while not pressing a mouse button.
    Moved,
    /// Scrolled mouse wheel downwards (towards the user).
    ScrollDown,
    /// Scrolled mouse wheel upwards (away from the user).
    ScrollUp,
    /// Scrolled mouse wheel left (mostly on a laptop touchpad).
    ScrollLeft,
    /// Scrolled mouse wheel right (mostly on a laptop touchpad).
    ScrollRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// Left mouse button.
    Left,
    /// Right mouse button.
    Right,
    /// Middle mouse button.
    Middle,
}
