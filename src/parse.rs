use crate::event::InternalEvent;

#[derive(Debug)]
pub(crate) struct Parser {
    buffer: Vec<u8>,
}

impl Default for Parser {
    fn default() -> Self {
        Self {
            buffer: Vec::with_capacity(256),
        }
    }
}

impl Parser {
    pub fn parse<F: FnMut(InternalEvent)>(&mut self, bytes: &[u8], callback: F, maybe_more: bool) {
        self.buffer.extend_from_slice(bytes);
        self.process_bytes(callback, maybe_more);
    }

    fn process_bytes<F: FnMut(InternalEvent)>(&mut self, mut callback: F, maybe_more: bool) {
        todo!()
    }
}

#[cfg(windows)]
mod windows {
    use windows_sys::Win32::System::Console;

    use super::*;

    impl Parser {
        pub fn decode_input_records<F: FnMut(InternalEvent)>(
            &mut self,
            records: &[Console::INPUT_RECORD],
            mut callback: F,
        ) {
            for record in records {
                match record.EventType as u32 {
                    Console::KEY_EVENT => {
                        self.decode_key_record(unsafe { record.Event.KeyEvent }, &mut callback)
                    }
                    Console::WINDOW_BUFFER_SIZE_EVENT => self.decode_resize_record(
                        unsafe { record.Event.WindowBufferSizeEvent },
                        &mut callback,
                    ),
                    _ => (),
                }
            }
            self.process_bytes(callback, false);
        }

        fn decode_resize_record<F: FnMut(InternalEvent)>(
            &mut self,
            record: Console::WINDOW_BUFFER_SIZE_RECORD,
            mut callback: F,
        ) {
            callback(InternalEvent::Event(crate::Event::WindowResized {
                // Windows sizes are zero-indexed, Unix are 1-indexed. Normalize to Unix:
                rows: (record.dwSize.Y + 1) as u16,
                cols: (record.dwSize.X + 1) as u16,
            }));
        }

        fn decode_key_record<F: FnMut(InternalEvent)>(
            &mut self,
            record: Console::KEY_EVENT_RECORD,
            callback: F,
        ) {
            // This skips 'up's. IIRC Termwiz skips 'down's and Crossterm skips 'up's.
            if record.bKeyDown != 0 {
                return;
            }
            // `read_console_input` uses `ReadConsoleInputW` so this should always be valid
            // Unicode.
            match std::char::from_u32(unsafe { record.uChar.UnicodeChar } as u32) {
                Some(unicode) if unicode != '\0' => {
                    let mut buf = [0u8; 4];
                    self.buffer
                        .extend_from_slice(unicode.encode_utf8(&mut buf).as_bytes());
                    self.process_bytes(callback, true);
                }
                _ => (),
            }
        }
    }
}
