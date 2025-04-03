bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Modifiers: u8 {
        const NONE = 0;
        const SHIFT = 1 << 1;
        const ALT = 1 << 2;
        const CONTROL = 1 << 3;
        const SUPER = 1 << 4;
    }
}
