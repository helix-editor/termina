use std::collections::VecDeque;

use crate::event::InternalEvent;

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
        eprintln!("bytes ({maybe_more}): {:?}\r", self.buffer);
        // todo!("(more: {maybe_more}), bytes: {:?}", self.buffer);
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
                    other => eprintln!("skipping record type {other}"),
                }
            }
            self.process_bytes(false);
        }
    }
}
