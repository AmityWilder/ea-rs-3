use crate::ivec::IVec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Gate {
    #[default]
    Or,
    And,
    Nor,
    Xor,
    Resistor {},
    Capacitor {},
    Led {},
    Delay {},
    Battery,
}

impl std::fmt::Display for Gate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Gate::Or => "or".fmt(f),
            Gate::And => "and".fmt(f),
            Gate::Nor => "nor".fmt(f),
            Gate::Xor => "xor".fmt(f),
            Gate::Resistor {} => write!(f, "resistor"),
            Gate::Capacitor {} => write!(f, "capacitor"),
            Gate::Led {} => write!(f, "led"),
            Gate::Delay {} => write!(f, "delay"),
            Gate::Battery => "battery".fmt(f),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Node {
    pub(super) state: bool,
    pub gate: Gate,
    pub position: IVec2,
}

impl Node {
    pub const fn new(gate: Gate, position: IVec2) -> Self {
        Self {
            state: false,
            gate,
            position,
        }
    }

    pub const fn state(&self) -> bool {
        self.state
    }
}
