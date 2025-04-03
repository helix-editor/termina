use std::collections::VecDeque;

use crate::event::InternalEvent;

#[derive(Debug)]
pub struct Parser {
    buffer: Vec<u8>,
    events: VecDeque<InternalEvent>,
}

impl Default for Parser {
    fn default() -> Self {
        Self {
            buffer: Vec::with_capacity(256),
            events: VecDeque::with_capacity(128),
        }
    }
}

impl Parser {
    pub fn next(&mut self) -> Option<InternalEvent> {
        self.events.pop_front()
    }
}
