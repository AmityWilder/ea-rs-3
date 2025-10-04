use crate::ivec::IVec2;
use serde_derive::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum GateId {
    #[default]
    Or,
    And,
    Nor,
    Xor,
    Resistor,
    Capacitor,
    Led,
    Delay,
    Battery,
}

impl std::fmt::Display for GateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateId::Or => "or",
            GateId::And => "and",
            GateId::Nor => "nor",
            GateId::Xor => "xor",
            GateId::Resistor => "resistor",
            GateId::Capacitor => "capacitor",
            GateId::Led => "led",
            GateId::Delay => "delay",
            GateId::Battery => "battery",
        }
        .fmt(f)
    }
}

impl std::str::FromStr for GateId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "or" => Ok(GateId::Or),
            "and" => Ok(GateId::And),
            "nor" => Ok(GateId::Nor),
            "xor" => Ok(GateId::Xor),
            "resistor" => Ok(GateId::Resistor),
            "capacitor" => Ok(GateId::Capacitor),
            "led" => Ok(GateId::Led),
            "delay" => Ok(GateId::Delay),
            "battery" => Ok(GateId::Battery),
            _ => Err(()),
        }
    }
}

impl GateId {
    pub const fn to_gate(self, ntd: u8) -> Gate {
        match self {
            GateId::Or => Gate::Or,
            GateId::And => Gate::And,
            GateId::Nor => Gate::Nor,
            GateId::Xor => Gate::Xor,
            GateId::Resistor => Gate::Resistor { resistance: ntd },
            GateId::Capacitor => Gate::Capacitor { capacity: ntd },
            GateId::Led => Gate::Led { color: ntd },
            GateId::Delay => Gate::Delay,
            GateId::Battery => Gate::Battery,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
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
    Delay,
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
            Gate::Delay => write!(f, "delay"),
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
                    "delay" => Some(Gate::Delay),
                    _ => None,
                })
                .ok_or(()),
        }
    }
}

impl Gate {
    pub const fn id(self) -> GateId {
        match self {
            Gate::Or => GateId::Or,
            Gate::And => GateId::And,
            Gate::Nor => GateId::Nor,
            Gate::Xor => GateId::Xor,
            Gate::Resistor { .. } => GateId::Resistor,
            Gate::Capacitor { .. } => GateId::Capacitor,
            Gate::Led { .. } => GateId::Led,
            Gate::Delay => GateId::Delay,
            Gate::Battery => GateId::Battery,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum GateNtd {
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
        stored: u8,
    },
    Led {
        color: u8,
    },
    Delay {
        prev: bool,
    },
    Battery,
}

impl GateNtd {
    pub const fn from_gate(gate: Gate) -> Self {
        match gate {
            Gate::Or => Self::Or,
            Gate::And => Self::And,
            Gate::Nor => Self::Nor,
            Gate::Xor => Self::Xor,
            Gate::Resistor { resistance } => Self::Resistor { resistance },
            Gate::Capacitor { capacity } => Self::Capacitor {
                capacity,
                stored: 0,
            },
            Gate::Led { color } => Self::Led { color },
            Gate::Delay => Self::Delay { prev: false },
            Gate::Battery => Self::Battery,
        }
    }

    pub const fn as_gate(self) -> Gate {
        match self {
            Self::Or => Gate::Or {},
            Self::And => Gate::And {},
            Self::Nor => Gate::Nor {},
            Self::Xor => Gate::Xor {},
            Self::Resistor { resistance } => Gate::Resistor { resistance },
            Self::Capacitor {
                capacity,
                stored: _,
            } => Gate::Capacitor { capacity },
            Self::Led { color } => Gate::Led { color },
            Self::Delay { prev: _ } => Gate::Delay {},
            Self::Battery => Gate::Battery {},
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Node {
    pub(super) state: bool,
    id: NodeId,
    pub(super) gate: GateNtd,
    pub(super) position: IVec2,
}

impl Node {
    pub const fn new(id: NodeId, gate: Gate, position: IVec2) -> Self {
        Self {
            state: false,
            id,
            gate: GateNtd::from_gate(gate),
            position,
        }
    }

    #[inline]
    pub const fn id(&self) -> &NodeId {
        &self.id
    }

    #[inline]
    pub const fn state(&self) -> bool {
        self.state
    }

    #[inline]
    pub const fn position(&self) -> IVec2 {
        self.position
    }

    #[inline]
    pub const fn gate_ntd(&self) -> &GateNtd {
        &self.gate
    }
}
