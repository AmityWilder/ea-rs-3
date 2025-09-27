#[derive(Debug, Clone)]
pub enum Tool {
    Create {},
    Erase {},
    Edit {},
}

impl Default for Tool {
    fn default() -> Self {
        Self::Create {}
    }
}
