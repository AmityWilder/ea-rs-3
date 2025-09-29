use crate::ivec::IVec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub(super) u128);

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "n{:x}", self.0)
    }
}

impl std::str::FromStr for NodeId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.strip_prefix('n')
            .ok_or(())
            .and_then(|x| u128::from_str_radix(x, 16).map_err(|_| ()))
            .map(Self)
    }
}

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

impl std::str::FromStr for Gate {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Or" => Ok(Gate::Or),
            "And" => Ok(Gate::And),
            "Nor" => Ok(Gate::Nor),
            "Xor" => Ok(Gate::Xor),
            "Resistor" => Ok(Gate::Resistor {}),
            "Capacitor" => Ok(Gate::Capacitor {}),
            "Led" => Ok(Gate::Led {}),
            "Delay" => Ok(Gate::Delay {}),
            "Battery" => Ok(Gate::Battery),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Node {
    pub(super) state: bool,
    id: NodeId,
    pub gate: Gate,
    pub position: IVec2,
}

impl Node {
    pub const fn new(id: NodeId, gate: Gate, position: IVec2) -> Self {
        Self {
            state: false,
            id,
            gate,
            position,
        }
    }

    pub const fn id(&self) -> NodeId {
        self.id
    }

    pub const fn state(&self) -> bool {
        self.state
    }
}
