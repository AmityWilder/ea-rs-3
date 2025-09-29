#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WireId(pub(super) u128);

impl std::fmt::Display for WireId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "w{:x}", self.0)
    }
}

impl std::str::FromStr for WireId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.strip_prefix('w')
            .ok_or(())
            .and_then(|x| u128::from_str_radix(x, 16).map_err(|_| ()))
            .map(Self)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Wire {
    id: WireId,
    pub(super) src: usize,
    pub(super) dst: usize,
}

impl Wire {
    pub const fn new(id: WireId, src: usize, dst: usize) -> Self {
        Self { id, src, dst }
    }

    pub const fn id(&self) -> WireId {
        self.id
    }
}
