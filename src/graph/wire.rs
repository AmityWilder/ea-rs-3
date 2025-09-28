#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Wire {
    pub(super) src: usize,
    pub(super) dst: usize,
}

impl Wire {
    pub const fn new(src: usize, dst: usize) -> Self {
        Self { src, dst }
    }
}
