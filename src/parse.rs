use crate::event::InternalEvent;

#[derive(Debug)]
pub struct Parser {
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
    pub fn parse<F>(&mut self, bytes: &[u8], f: F, maybe_more: bool)
    where
        F: FnMut(InternalEvent),
    {
        self.buffer.extend_from_slice(bytes);
        self.process_bytes(f, maybe_more);
    }

    fn process_bytes<F>(&mut self, mut f: F, maybe_more: bool)
    where
        F: FnMut(InternalEvent),
    {
        todo!()
    }
}
