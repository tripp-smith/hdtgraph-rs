#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SplayToken {
    pub node: u32,
}

impl SplayToken {
    pub fn new(node: u32) -> Self {
        Self { node }
    }
}
