use std::{collections::VecDeque, str};

use crate::{
    event::InternalEvent,
    input::{
        KeyCode, KeyEvent, KeyEventKind, KeyEventState, MediaKeyCode, ModifierKeyCode, Modifiers,
    },
    Event,
};

#[derive(Debug)]
pub(crate) struct Parser {
    buffer: Vec<u8>,
    /// Events which have been parsed. Pop out with `Self::pop`.
    events: VecDeque<InternalEvent>,
}

impl Default for Parser {
    fn default() -> Self {
        Self {
            buffer: Vec::with_capacity(256),
            events: VecDeque::with_capacity(32),
        }
    }
}

impl Parser {
    /// Reads and removes a parsed event from the parser.
    pub fn pop(&mut self) -> Option<InternalEvent> {
        self.events.pop_front()
    }

    // NOTE: Windows is handled in the `windows` module below.
    #[cfg(unix)]
    pub fn parse(&mut self, bytes: &[u8], maybe_more: bool) {
        self.buffer.extend_from_slice(bytes);
        self.process_bytes(maybe_more);
    }

    fn process_bytes(&mut self, maybe_more: bool) {
        let mut start = 0;
        for n in 0..self.buffer.len() {
            let end = n + 1;
            match parse_event(
                &self.buffer[start..end],
                maybe_more || end < self.buffer.len(),
            ) {
                Ok(Some(event)) => {
                    self.events.push_back(event);
                    start = end;
                }
                Ok(None) => continue,
                Err(_) => start = end,
            }
        }
        self.advance(start);
    }

    fn advance(&mut self, len: usize) {
        if len == 0 {
            return;
        }
        let remain = self.buffer.len() - len;
        self.buffer.rotate_left(len);
        self.buffer.truncate(remain);
    }
}

#[cfg(windows)]
mod windows {
    use windows_sys::Win32::System::Console;

    use super::*;

    impl Parser {
        pub fn decode_input_records(&mut self, records: &[Console::INPUT_RECORD]) {
            for record in records {
                match record.EventType as u32 {
                    Console::KEY_EVENT => {
                        let record = unsafe { record.Event.KeyEvent };
                        // This skips 'down's. IIRC Termwiz skips 'down's and Crossterm skips
                        // 'up's. If we skip 'up's we don't seem to get key events at all.
                        if record.bKeyDown == 0 {
                            return;
                        }
                        // `read_console_input` uses `ReadConsoleInputA` so we should treat the
                        // key code as a byte and add it to the buffer.
                        self.buffer.push(unsafe { record.uChar.AsciiChar } as u8);
                    }
                    Console::WINDOW_BUFFER_SIZE_EVENT => {
                        let record = unsafe { record.Event.WindowBufferSizeEvent };
                        self.events
                            .push_back(InternalEvent::Event(crate::Event::WindowResized {
                                // Windows sizes are zero-indexed, Unix are 1-indexed. Normalize
                                // to Unix:
                                rows: (record.dwSize.Y + 1) as u16,
                                cols: (record.dwSize.X + 1) as u16,
                            }));
                    }
                    _ => (),
                }
            }
            self.process_bytes(false);
        }
    }
}

#[derive(Debug)]
struct MalformedSequenceError;

type Result<T> = std::result::Result<T, MalformedSequenceError>;

macro_rules! bail {
    () => {
        return Err(MalformedSequenceError)
    };
}

fn parse_event(buffer: &[u8], maybe_more: bool) -> Result<Option<InternalEvent>> {
    // TODO: remove
    // eprintln!(
    //     "parsing buffer {buffer:?} ({:?})\r",
    //     str::from_utf8(buffer).ok()
    // );
    if buffer.is_empty() {
        return Ok(None);
    }

    match buffer[0] {
        b'\x1B' => {
            if buffer.len() == 1 {
                if maybe_more {
                    // Possible Esc sequence
                    Ok(None)
                } else {
                    Ok(Some(InternalEvent::Event(Event::Key(
                        KeyCode::Escape.into(),
                    ))))
                }
            } else {
                match buffer[1] {
                    b'O' => {
                        if buffer.len() == 2 {
                            Ok(None)
                        } else {
                            match buffer[2] {
                                b'D' => {
                                    Ok(Some(InternalEvent::Event(Event::Key(KeyCode::Left.into()))))
                                }
                                b'C' => Ok(Some(InternalEvent::Event(Event::Key(
                                    KeyCode::Right.into(),
                                )))),
                                b'A' => {
                                    Ok(Some(InternalEvent::Event(Event::Key(KeyCode::Up.into()))))
                                }
                                b'B' => {
                                    Ok(Some(InternalEvent::Event(Event::Key(KeyCode::Down.into()))))
                                }
                                b'H' => {
                                    Ok(Some(InternalEvent::Event(Event::Key(KeyCode::Home.into()))))
                                }
                                b'F' => {
                                    Ok(Some(InternalEvent::Event(Event::Key(KeyCode::End.into()))))
                                }
                                // F1-F4
                                val @ b'P'..=b'S' => Ok(Some(InternalEvent::Event(Event::Key(
                                    KeyCode::Function(1 + val - b'P').into(),
                                )))),
                                _ => bail!(),
                            }
                        }
                    }
                    b'[' => parse_csi(buffer),
                    b'\x1B' => Ok(Some(InternalEvent::Event(Event::Key(
                        KeyCode::Escape.into(),
                    )))),
                    _ => parse_event(&buffer[1..], maybe_more).map(|event_option| {
                        event_option.map(|event| {
                            if let InternalEvent::Event(Event::Key(key_event)) = event {
                                let mut alt_key_event = key_event;
                                alt_key_event.modifiers |= Modifiers::ALT;
                                InternalEvent::Event(Event::Key(alt_key_event))
                            } else {
                                event
                            }
                        })
                    }),
                }
            }
        }
        b'\r' => Ok(Some(InternalEvent::Event(Event::Key(
            KeyCode::Enter.into(),
        )))),
        b'\t' => Ok(Some(InternalEvent::Event(Event::Key(KeyCode::Tab.into())))),
        b'\x7F' => Ok(Some(InternalEvent::Event(Event::Key(
            KeyCode::Backspace.into(),
        )))),
        b'\0' => Ok(Some(InternalEvent::Event(Event::Key(KeyEvent::new(
            KeyCode::Char(' '),
            Modifiers::CONTROL,
        ))))),
        c @ b'\x01'..=b'\x1A' => Ok(Some(InternalEvent::Event(Event::Key(KeyEvent::new(
            KeyCode::Char((c - 0x1 + b'a') as char),
            Modifiers::CONTROL,
        ))))),
        c @ b'\x1C'..=b'\x1F' => Ok(Some(InternalEvent::Event(Event::Key(KeyEvent::new(
            KeyCode::Char((c - 0x1C + b'4') as char),
            Modifiers::CONTROL,
        ))))),
        _ => parse_utf8_char(buffer).map(|maybe_char| {
            maybe_char.map(|ch| {
                let modifiers = if ch.is_uppercase() {
                    Modifiers::SHIFT
                } else {
                    Modifiers::NONE
                };
                InternalEvent::Event(Event::Key(KeyEvent::new(KeyCode::Char(ch), modifiers)))
            })
        }),
    }
}

fn parse_utf8_char(buffer: &[u8]) -> Result<Option<char>> {
    assert!(!buffer.is_empty());
    match str::from_utf8(buffer) {
        Ok(s) => Ok(Some(s.chars().next().unwrap())),
        Err(_) => {
            // `from_utf8` failed but it could be because we don't have enough bytes to make a
            // valid UTF-8 codepoint. Check the validity of the bytes so far:
            let required_bytes = match buffer[0] {
                // https://en.wikipedia.org/wiki/UTF-8#Description
                (0x00..=0x7F) => 1, // 0xxxxxxx
                (0xC0..=0xDF) => 2, // 110xxxxx 10xxxxxx
                (0xE0..=0xEF) => 3, // 1110xxxx 10xxxxxx 10xxxxxx
                (0xF0..=0xF7) => 4, // 11110xxx 10xxxxxx 10xxxxxx 10xxxxxx
                (0x80..=0xBF) | (0xF8..=0xFF) => bail!(),
            };
            if required_bytes > 1 && buffer.len() > 1 {
                for byte in &buffer[1..] {
                    if byte & !0b0011_1111 != 0b1000_0000 {
                        bail!()
                    }
                }
            }
            if buffer.len() < required_bytes {
                Ok(None)
            } else {
                bail!()
            }
        }
    }
}

fn parse_csi(buffer: &[u8]) -> Result<Option<InternalEvent>> {
    assert!(buffer.starts_with(b"\x1B["));
    if buffer.len() == 2 {
        return Ok(None);
    }
    let maybe_event = match buffer[2] {
        b'[' => match buffer.get(3) {
            None => None,
            Some(b @ b'A'..=b'E') => Some(Event::Key(KeyCode::Function(1 + b - b'A').into())),
            Some(_) => bail!(),
        },
        b'D' => Some(Event::Key(KeyCode::Left.into())),
        b'C' => Some(Event::Key(KeyCode::Right.into())),
        b'A' => Some(Event::Key(KeyCode::Up.into())),
        b'B' => Some(Event::Key(KeyCode::Down.into())),
        b'H' => Some(Event::Key(KeyCode::Home.into())),
        b'F' => Some(Event::Key(KeyCode::End.into())),
        b'Z' => Some(Event::Key(KeyEvent {
            code: KeyCode::BackTab,
            modifiers: Modifiers::SHIFT,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })),
        b'M' => todo!("normal mouse"),
        b'<' => todo!("SGR mouse"),
        b'I' => Some(Event::FocusIn),
        b'O' => Some(Event::FocusOut),
        b';' => return parse_csi_modifier_key_code(buffer),
        // P, Q, and S for compatibility with Kitty keyboard protocol,
        // as the 1 in 'CSI 1 P' etc. must be omitted if there are no
        // modifiers pressed:
        // https://sw.kovidgoyal.net/kitty/keyboard-protocol/#legacy-functional-keys
        b'P' => Some(Event::Key(KeyCode::Function(1).into())),
        b'Q' => Some(Event::Key(KeyCode::Function(2).into())),
        b'S' => Some(Event::Key(KeyCode::Function(4).into())),
        // b'?' => match buffer[buffer.len() - 1] {
        //     b'u' => return parse_csi_keyboard_enhancement_flags(buffer),
        //     b'c' => return parse_csi_primary_device_attributes(buffer),
        //     b'n' => return parse_csi_theme_mode(buffer),
        //     b'y' => return parse_csi_synchronized_output_mode(buffer),
        //     _ => None,
        // },
        b'0'..=b'9' => {
            // Numbered escape code.
            if buffer.len() == 3 {
                None
            } else {
                // The final byte of a CSI sequence can be in the range 64-126, so
                // let's keep reading anything else.
                let last_byte = buffer[buffer.len() - 1];
                if !(64..=126).contains(&last_byte) {
                    None
                } else {
                    // if buffer.starts_with(b"\x1B[200~") {
                    //     return parse_csi_bracketed_paste(buffer);
                    // }
                    match last_byte {
                        // b'M' => return parse_csi_rxvt_mouse(buffer),
                        b'~' => return parse_csi_special_key_code(buffer),
                        b'u' => return parse_csi_u_encoded_key_code(buffer),
                        // b'R' => return parse_csi_cursor_position(buffer),
                        _ => return parse_csi_modifier_key_code(buffer),
                    }
                }
            }
        }
        _ => bail!(),
    };
    Ok(maybe_event.map(InternalEvent::Event))
}

fn next_parsed<T>(iter: &mut dyn Iterator<Item = &str>) -> Result<T>
where
    T: str::FromStr,
{
    iter.next()
        .ok_or(MalformedSequenceError)?
        .parse::<T>()
        .map_err(|_| MalformedSequenceError)
}

fn modifier_and_kind_parsed(iter: &mut dyn Iterator<Item = &str>) -> Result<(u8, u8)> {
    let mut sub_split = iter.next().ok_or(MalformedSequenceError)?.split(':');

    let modifier_mask = next_parsed::<u8>(&mut sub_split)?;

    if let Ok(kind_code) = next_parsed::<u8>(&mut sub_split) {
        Ok((modifier_mask, kind_code))
    } else {
        Ok((modifier_mask, 1))
    }
}

fn parse_csi_u_encoded_key_code(buffer: &[u8]) -> Result<Option<InternalEvent>> {
    assert!(buffer.starts_with(b"\x1B")); // CSI
    assert!(buffer.ends_with(b"u"));

    // This function parses `CSI … u` sequences. These are sequences defined in either
    // the `CSI u` (a.k.a. "Fix Keyboard Input on Terminals - Please", https://www.leonerd.org.uk/hacks/fixterms/)
    // or Kitty Keyboard Protocol (https://sw.kovidgoyal.net/kitty/keyboard-protocol/) specifications.
    // This CSI sequence is a tuple of semicolon-separated numbers.
    let s =
        std::str::from_utf8(&buffer[2..buffer.len() - 1]).map_err(|_| MalformedSequenceError)?;
    let mut split = s.split(';');

    // In `CSI u`, this is parsed as:
    //
    //     CSI codepoint ; modifiers u
    //     codepoint: ASCII Dec value
    //
    // The Kitty Keyboard Protocol extends this with optional components that can be
    // enabled progressively. The full sequence is parsed as:
    //
    //     CSI unicode-key-code:alternate-key-codes ; modifiers:event-type ; text-as-codepoints u
    let mut codepoints = split.next().ok_or(MalformedSequenceError)?.split(':');

    let codepoint = codepoints
        .next()
        .ok_or(MalformedSequenceError)?
        .parse::<u32>()
        .map_err(|_| MalformedSequenceError)?;

    let (mut modifiers, kind, state_from_modifiers) =
        if let Ok((modifier_mask, kind_code)) = modifier_and_kind_parsed(&mut split) {
            (
                parse_modifiers(modifier_mask),
                parse_key_event_kind(kind_code),
                parse_modifiers_to_state(modifier_mask),
            )
        } else {
            (Modifiers::NONE, KeyEventKind::Press, KeyEventState::NONE)
        };

    let (mut code, state_from_keycode) = {
        if let Some((special_key_code, state)) = translate_functional_key_code(codepoint) {
            (special_key_code, state)
        } else if let Some(c) = char::from_u32(codepoint) {
            (
                match c {
                    '\x1B' => KeyCode::Escape,
                    '\r' => KeyCode::Enter,
                    /*
                    // Issue #371: \n = 0xA, which is also the keycode for Ctrl+J. The only reason we get
                    // newlines as input is because the terminal converts \r into \n for us. When we
                    // enter raw mode, we disable that, so \n no longer has any meaning - it's better to
                    // use Ctrl+J. Waiting to handle it here means it gets picked up later
                    '\n' if !crate::terminal::sys::is_raw_mode_enabled() => KeyCode::Enter,
                    */
                    '\t' => {
                        if modifiers.contains(Modifiers::SHIFT) {
                            KeyCode::BackTab
                        } else {
                            KeyCode::Tab
                        }
                    }
                    '\x7F' => KeyCode::Backspace,
                    _ => KeyCode::Char(c),
                },
                KeyEventState::empty(),
            )
        } else {
            bail!();
        }
    };

    if let KeyCode::Modifier(modifier_keycode) = code {
        match modifier_keycode {
            ModifierKeyCode::LeftAlt | ModifierKeyCode::RightAlt => {
                modifiers.set(Modifiers::ALT, true)
            }
            ModifierKeyCode::LeftControl | ModifierKeyCode::RightControl => {
                modifiers.set(Modifiers::CONTROL, true)
            }
            ModifierKeyCode::LeftShift | ModifierKeyCode::RightShift => {
                modifiers.set(Modifiers::SHIFT, true)
            }
            ModifierKeyCode::LeftSuper | ModifierKeyCode::RightSuper => {
                modifiers.set(Modifiers::SUPER, true)
            }
            ModifierKeyCode::LeftHyper | ModifierKeyCode::RightHyper => {
                modifiers.set(Modifiers::HYPER, true)
            }
            ModifierKeyCode::LeftMeta | ModifierKeyCode::RightMeta => {
                modifiers.set(Modifiers::META, true)
            }
            _ => {}
        }
    }

    // When the "report alternate keys" flag is enabled in the Kitty Keyboard Protocol
    // and the terminal sends a keyboard event containing shift, the sequence will
    // contain an additional codepoint separated by a ':' character which contains
    // the shifted character according to the keyboard layout.
    if modifiers.contains(Modifiers::SHIFT) {
        if let Some(shifted_c) = codepoints
            .next()
            .and_then(|codepoint| codepoint.parse::<u32>().ok())
            .and_then(char::from_u32)
        {
            code = KeyCode::Char(shifted_c);
            modifiers.set(Modifiers::SHIFT, false);
        }
    }

    let input_event = Event::Key(KeyEvent {
        code,
        modifiers,
        kind,
        state: state_from_keycode | state_from_modifiers,
    });

    Ok(Some(InternalEvent::Event(input_event)))
}

fn parse_modifiers(mask: u8) -> Modifiers {
    let modifier_mask = mask.saturating_sub(1);
    let mut modifiers = Modifiers::empty();
    if modifier_mask & 1 != 0 {
        modifiers |= Modifiers::SHIFT;
    }
    if modifier_mask & 2 != 0 {
        modifiers |= Modifiers::ALT;
    }
    if modifier_mask & 4 != 0 {
        modifiers |= Modifiers::CONTROL;
    }
    if modifier_mask & 8 != 0 {
        modifiers |= Modifiers::SUPER;
    }
    if modifier_mask & 16 != 0 {
        modifiers |= Modifiers::HYPER;
    }
    if modifier_mask & 32 != 0 {
        modifiers |= Modifiers::META;
    }
    modifiers
}

fn parse_modifiers_to_state(mask: u8) -> KeyEventState {
    let modifier_mask = mask.saturating_sub(1);
    let mut state = KeyEventState::empty();
    if modifier_mask & 64 != 0 {
        state |= KeyEventState::CAPS_LOCK;
    }
    if modifier_mask & 128 != 0 {
        state |= KeyEventState::NUM_LOCK;
    }
    state
}

fn parse_key_event_kind(kind: u8) -> KeyEventKind {
    match kind {
        1 => KeyEventKind::Press,
        2 => KeyEventKind::Repeat,
        3 => KeyEventKind::Release,
        _ => KeyEventKind::Press,
    }
}

fn parse_csi_modifier_key_code(buffer: &[u8]) -> Result<Option<InternalEvent>> {
    assert!(buffer.starts_with(b"\x1B[")); // ESC [
                                           //
    let s =
        std::str::from_utf8(&buffer[2..buffer.len() - 1]).map_err(|_| MalformedSequenceError)?;
    let mut split = s.split(';');

    split.next();

    let (modifiers, kind) =
        if let Ok((modifier_mask, kind_code)) = modifier_and_kind_parsed(&mut split) {
            (
                parse_modifiers(modifier_mask),
                parse_key_event_kind(kind_code),
            )
        } else if buffer.len() > 3 {
            (
                parse_modifiers(
                    (buffer[buffer.len() - 2] as char)
                        .to_digit(10)
                        .ok_or(MalformedSequenceError)? as u8,
                ),
                KeyEventKind::Press,
            )
        } else {
            (Modifiers::NONE, KeyEventKind::Press)
        };
    let key = buffer[buffer.len() - 1];

    let code = match key {
        b'A' => KeyCode::Up,
        b'B' => KeyCode::Down,
        b'C' => KeyCode::Right,
        b'D' => KeyCode::Left,
        b'F' => KeyCode::End,
        b'H' => KeyCode::Home,
        b'P' => KeyCode::Function(1),
        b'Q' => KeyCode::Function(2),
        b'R' => KeyCode::Function(3),
        b'S' => KeyCode::Function(4),
        _ => bail!(),
    };

    let input_event = Event::Key(KeyEvent {
        code,
        modifiers,
        kind,
        state: KeyEventState::NONE,
    });

    Ok(Some(InternalEvent::Event(input_event)))
}

fn parse_csi_special_key_code(buffer: &[u8]) -> Result<Option<InternalEvent>> {
    assert!(buffer.starts_with(b"\x1B[")); // CSI
    assert!(buffer.ends_with(b"~"));

    let s =
        std::str::from_utf8(&buffer[2..buffer.len() - 1]).map_err(|_| MalformedSequenceError)?;
    let mut split = s.split(';');

    // This CSI sequence can be a list of semicolon-separated numbers.
    let first = next_parsed::<u8>(&mut split)?;

    let (modifiers, kind, state) =
        if let Ok((modifier_mask, kind_code)) = modifier_and_kind_parsed(&mut split) {
            (
                parse_modifiers(modifier_mask),
                parse_key_event_kind(kind_code),
                parse_modifiers_to_state(modifier_mask),
            )
        } else {
            (Modifiers::NONE, KeyEventKind::Press, KeyEventState::NONE)
        };

    let code = match first {
        1 | 7 => KeyCode::Home,
        2 => KeyCode::Insert,
        3 => KeyCode::Delete,
        4 | 8 => KeyCode::End,
        5 => KeyCode::PageUp,
        6 => KeyCode::PageDown,
        v @ 11..=15 => KeyCode::Function(v - 10),
        v @ 17..=21 => KeyCode::Function(v - 11),
        v @ 23..=26 => KeyCode::Function(v - 12),
        v @ 28..=29 => KeyCode::Function(v - 15),
        v @ 31..=34 => KeyCode::Function(v - 17),
        _ => bail!(),
    };

    let input_event = Event::Key(KeyEvent {
        code,
        modifiers,
        kind,
        state,
    });

    Ok(Some(InternalEvent::Event(input_event)))
}

fn translate_functional_key_code(codepoint: u32) -> Option<(KeyCode, KeyEventState)> {
    if let Some(keycode) = match codepoint {
        57399 => Some(KeyCode::Char('0')),
        57400 => Some(KeyCode::Char('1')),
        57401 => Some(KeyCode::Char('2')),
        57402 => Some(KeyCode::Char('3')),
        57403 => Some(KeyCode::Char('4')),
        57404 => Some(KeyCode::Char('5')),
        57405 => Some(KeyCode::Char('6')),
        57406 => Some(KeyCode::Char('7')),
        57407 => Some(KeyCode::Char('8')),
        57408 => Some(KeyCode::Char('9')),
        57409 => Some(KeyCode::Char('.')),
        57410 => Some(KeyCode::Char('/')),
        57411 => Some(KeyCode::Char('*')),
        57412 => Some(KeyCode::Char('-')),
        57413 => Some(KeyCode::Char('+')),
        57414 => Some(KeyCode::Enter),
        57415 => Some(KeyCode::Char('=')),
        57416 => Some(KeyCode::Char(',')),
        57417 => Some(KeyCode::Left),
        57418 => Some(KeyCode::Right),
        57419 => Some(KeyCode::Up),
        57420 => Some(KeyCode::Down),
        57421 => Some(KeyCode::PageUp),
        57422 => Some(KeyCode::PageDown),
        57423 => Some(KeyCode::Home),
        57424 => Some(KeyCode::End),
        57425 => Some(KeyCode::Insert),
        57426 => Some(KeyCode::Delete),
        57427 => Some(KeyCode::KeypadBegin),
        _ => None,
    } {
        return Some((keycode, KeyEventState::KEYPAD));
    }

    if let Some(keycode) = match codepoint {
        57358 => Some(KeyCode::CapsLock),
        57359 => Some(KeyCode::ScrollLock),
        57360 => Some(KeyCode::NumLock),
        57361 => Some(KeyCode::PrintScreen),
        57362 => Some(KeyCode::Pause),
        57363 => Some(KeyCode::Menu),
        57376 => Some(KeyCode::Function(13)),
        57377 => Some(KeyCode::Function(14)),
        57378 => Some(KeyCode::Function(15)),
        57379 => Some(KeyCode::Function(16)),
        57380 => Some(KeyCode::Function(17)),
        57381 => Some(KeyCode::Function(18)),
        57382 => Some(KeyCode::Function(19)),
        57383 => Some(KeyCode::Function(20)),
        57384 => Some(KeyCode::Function(21)),
        57385 => Some(KeyCode::Function(22)),
        57386 => Some(KeyCode::Function(23)),
        57387 => Some(KeyCode::Function(24)),
        57388 => Some(KeyCode::Function(25)),
        57389 => Some(KeyCode::Function(26)),
        57390 => Some(KeyCode::Function(27)),
        57391 => Some(KeyCode::Function(28)),
        57392 => Some(KeyCode::Function(29)),
        57393 => Some(KeyCode::Function(30)),
        57394 => Some(KeyCode::Function(31)),
        57395 => Some(KeyCode::Function(32)),
        57396 => Some(KeyCode::Function(33)),
        57397 => Some(KeyCode::Function(34)),
        57398 => Some(KeyCode::Function(35)),
        57428 => Some(KeyCode::Media(MediaKeyCode::Play)),
        57429 => Some(KeyCode::Media(MediaKeyCode::Pause)),
        57430 => Some(KeyCode::Media(MediaKeyCode::PlayPause)),
        57431 => Some(KeyCode::Media(MediaKeyCode::Reverse)),
        57432 => Some(KeyCode::Media(MediaKeyCode::Stop)),
        57433 => Some(KeyCode::Media(MediaKeyCode::FastForward)),
        57434 => Some(KeyCode::Media(MediaKeyCode::Rewind)),
        57435 => Some(KeyCode::Media(MediaKeyCode::TrackNext)),
        57436 => Some(KeyCode::Media(MediaKeyCode::TrackPrevious)),
        57437 => Some(KeyCode::Media(MediaKeyCode::Record)),
        57438 => Some(KeyCode::Media(MediaKeyCode::LowerVolume)),
        57439 => Some(KeyCode::Media(MediaKeyCode::RaiseVolume)),
        57440 => Some(KeyCode::Media(MediaKeyCode::MuteVolume)),
        57441 => Some(KeyCode::Modifier(ModifierKeyCode::LeftShift)),
        57442 => Some(KeyCode::Modifier(ModifierKeyCode::LeftControl)),
        57443 => Some(KeyCode::Modifier(ModifierKeyCode::LeftAlt)),
        57444 => Some(KeyCode::Modifier(ModifierKeyCode::LeftSuper)),
        57445 => Some(KeyCode::Modifier(ModifierKeyCode::LeftHyper)),
        57446 => Some(KeyCode::Modifier(ModifierKeyCode::LeftMeta)),
        57447 => Some(KeyCode::Modifier(ModifierKeyCode::RightShift)),
        57448 => Some(KeyCode::Modifier(ModifierKeyCode::RightControl)),
        57449 => Some(KeyCode::Modifier(ModifierKeyCode::RightAlt)),
        57450 => Some(KeyCode::Modifier(ModifierKeyCode::RightSuper)),
        57451 => Some(KeyCode::Modifier(ModifierKeyCode::RightHyper)),
        57452 => Some(KeyCode::Modifier(ModifierKeyCode::RightMeta)),
        57453 => Some(KeyCode::Modifier(ModifierKeyCode::IsoLevel3Shift)),
        57454 => Some(KeyCode::Modifier(ModifierKeyCode::IsoLevel5Shift)),
        _ => None,
    } {
        return Some((keycode, KeyEventState::empty()));
    }

    None
}
