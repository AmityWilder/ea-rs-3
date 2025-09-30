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
    Resistor {
        resistance: u8,
    },
    Capacitor {
        capacity: u8,
    },
    Led {
        color: u8,
    },
    Delay {
        ticks: u8,
    },
    Battery,
}

impl std::fmt::Display for Gate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Gate::Or => "or".fmt(f),
            Gate::And => "and".fmt(f),
            Gate::Nor => "nor".fmt(f),
            Gate::Xor => "xor".fmt(f),
            Gate::Resistor { resistance } => write!(f, "resistor.{resistance}"),
            Gate::Capacitor { capacity } => write!(f, "capacitor.{capacity}"),
            Gate::Led { color } => write!(f, "led.{color}"),
            Gate::Delay { ticks } => write!(f, "delay.{ticks}"),
            Gate::Battery => "battery".fmt(f),
        }
    }
}

impl std::str::FromStr for Gate {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "or" => Ok(Gate::Or),
            "and" => Ok(Gate::And),
            "nor" => Ok(Gate::Nor),
            "xor" => Ok(Gate::Xor),
            "battery" => Ok(Gate::Battery),
            _ => s
                .split_once('.')
                .and_then(|(name, value)| value.parse().ok().map(|val| (name, val)))
                .and_then(|(name, value)| match name {
                    "resistor" => Some(Gate::Resistor { resistance: value }),
                    "capacitor" => Some(Gate::Capacitor { capacity: value }),
                    "led" => Some(Gate::Led { color: value }),
                    "delay" => Some(Gate::Delay { ticks: value }),
                    _ => None,
                })
                .ok_or(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Node {
    pub(super) state: u8,
    id: NodeId,
    pub gate: Gate,
    pub(super) position: IVec2,
}

impl Node {
    pub const fn new(id: NodeId, gate: Gate, position: IVec2) -> Self {
        Self {
            state: 0,
            id,
            gate,
            position,
        }
    }

    pub const fn id(&self) -> &NodeId {
        &self.id
    }

    pub const fn state(&self) -> u8 {
        self.state
    }

    pub const fn position(&self) -> IVec2 {
        self.position
    }
}
