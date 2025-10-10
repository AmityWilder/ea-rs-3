use crate::ivec::IVec2;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub(super) u128);

/// Defaults to [`Self::INVALID`].
impl Default for NodeId {
    #[inline]
    fn default() -> Self {
        Self::INVALID
    }
}

impl std::fmt::Display for NodeId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "n{:x}", self.0)
    }
}

impl std::fmt::Debug for NodeId {
    #[inline]
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

impl NodeId {
    pub const INVALID: Self = Self(!0);

    /// Returns the current value and increments `self`.
    /// Returns [`None`] if [`Self::INVALID`] would have been returned.
    /// Does not increment if `self` is [`Self::INVALID`].
    #[inline]
    pub const fn step(&mut self) -> Option<Self> {
        const INVALID: NodeId = NodeId::INVALID;
        match *self {
            INVALID => None,
            id => {
                self.0 += 1;
                Some(id)
            }
        }
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
    #[inline]
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

    #[inline]
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
    #[inline]
    pub const fn to_gate(self, ntd: Ntd) -> Gate {
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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(try_from = "u8", into = "u8")]
pub enum Ntd {
    #[default]
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

impl std::ops::Add for Ntd {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self::try_from(u8::from(self) + u8::from(rhs)).expect("attempted to add with overflow")
    }
}

impl Ntd {
    #[inline]
    pub fn saturating_sub(self, rhs: Self) -> Self {
        match Self::try_from(u8::from(self).saturating_sub(u8::from(rhs))) {
            Ok(n) => n,
            Err(_) => unreachable!(),
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct SaturatingNtd(pub Ntd);

impl std::ops::Deref for SaturatingNtd {
    type Target = Ntd;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SaturatingNtd {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::iter::Sum for SaturatingNtd {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        match Ntd::try_from(
            iter.map(|x| u8::from(x.0))
                .fold(0, |a, b| (a + b).clamp(0, 9)),
        ) {
            Ok(n) => Self(n),
            Err(_) => unreachable!(),
        }
    }
}

impl std::fmt::Display for Ntd {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Zero => '0',
            Self::One => '1',
            Self::Two => '2',
            Self::Three => '3',
            Self::Four => '4',
            Self::Five => '5',
            Self::Six => '6',
            Self::Seven => '7',
            Self::Eight => '8',
            Self::Nine => '9',
        }
        .fmt(f)
    }
}

impl std::str::FromStr for Ntd {
    type Err = ();

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Self::Zero),
            "1" => Ok(Self::One),
            "2" => Ok(Self::Two),
            "3" => Ok(Self::Three),
            "4" => Ok(Self::Four),
            "5" => Ok(Self::Five),
            "6" => Ok(Self::Six),
            "7" => Ok(Self::Seven),
            "8" => Ok(Self::Eight),
            "9" => Ok(Self::Nine),
            _ => Err(()),
        }
    }
}

impl TryFrom<u8> for Ntd {
    type Error = &'static str;

    #[inline]
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Zero),
            1 => Ok(Self::One),
            2 => Ok(Self::Two),
            3 => Ok(Self::Three),
            4 => Ok(Self::Four),
            5 => Ok(Self::Five),
            6 => Ok(Self::Six),
            7 => Ok(Self::Seven),
            8 => Ok(Self::Eight),
            9 => Ok(Self::Nine),
            _ => Err("NTD only supports the values 0-9"),
        }
    }
}

impl From<bool> for Ntd {
    fn from(value: bool) -> Self {
        match value {
            true => Ntd::One,
            false => Ntd::Zero,
        }
    }
}

impl From<Ntd> for u8 {
    fn from(value: Ntd) -> Self {
        value as u8
    }
}

impl From<Ntd> for usize {
    fn from(value: Ntd) -> Self {
        value as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Gate {
    #[default]
    #[serde(rename = "|")]
    Or,
    #[serde(rename = "&")]
    And,
    #[serde(rename = "!")]
    Nor,
    #[serde(rename = "^")]
    Xor,
    #[serde(rename = ">")]
    Resistor {
        #[serde(flatten)]
        resistance: Ntd,
    },
    #[serde(rename = "%")]
    Capacitor {
        #[serde(flatten)]
        capacity: Ntd,
    },
    #[serde(rename = "l")]
    Led {
        #[serde(flatten)]
        color: Ntd,
    },
    #[serde(rename = ";")]
    Delay,
    #[serde(rename = "T")]
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
    #[inline]
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

    #[inline]
    pub const fn ntd(self) -> Option<Ntd> {
        match self {
            Self::Or | Self::And | Self::Nor | Self::Xor | Self::Delay | Self::Battery => None,
            Self::Resistor { resistance: n }
            | Self::Capacitor { capacity: n }
            | Self::Led { color: n } => Some(n),
        }
    }

    #[inline]
    pub const fn with_ntd(self, value: Ntd) -> Self {
        match self {
            Self::Or | Self::And | Self::Nor | Self::Xor | Self::Delay | Self::Battery => self,
            Self::Resistor { .. } => Self::Resistor { resistance: value },
            Self::Capacitor { .. } => Self::Capacitor { capacity: value },
            Self::Led { .. } => Self::Led { color: value },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum GateInstance {
    #[default]
    Or,
    And,
    Nor,
    Xor,
    Resistor {
        resistance: Ntd,
    },
    Capacitor {
        capacity: Ntd,
        stored: Ntd,
    },
    Led {
        color: Ntd,
    },
    Delay {
        prev: bool,
    },
    Battery,
}

impl GateInstance {
    #[inline]
    pub const fn from_gate(gate: Gate) -> Self {
        match gate {
            Gate::Or => Self::Or,
            Gate::And => Self::And,
            Gate::Nor => Self::Nor,
            Gate::Xor => Self::Xor,
            Gate::Resistor { resistance } => Self::Resistor { resistance },
            Gate::Capacitor { capacity } => Self::Capacitor {
                capacity,
                stored: Ntd::Zero,
            },
            Gate::Led { color } => Self::Led { color },
            Gate::Delay => Self::Delay { prev: false },
            Gate::Battery => Self::Battery,
        }
    }

    #[inline]
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

    pub fn evaluate<I>(&mut self, inputs: I) -> bool
    where
        I: IntoIterator<Item = bool>,
    {
        let mut inputs = inputs.into_iter().peekable();
        match *self {
            GateInstance::Or | GateInstance::Led { .. } => inputs.any(|x| x),
            GateInstance::And => inputs.peek().is_some() && inputs.all(|x| x),
            GateInstance::Nor => !inputs.any(|x| x),
            GateInstance::Xor => inputs.filter(|&x| x).count() == 1,
            GateInstance::Resistor { resistance } => {
                *inputs
                    .map(Ntd::from)
                    .map(SaturatingNtd)
                    .sum::<SaturatingNtd>()
                    > resistance
            }
            GateInstance::Capacitor {
                capacity,
                ref mut stored,
            } => {
                let total = *inputs
                    .map(Ntd::from)
                    .map(SaturatingNtd)
                    .sum::<SaturatingNtd>();
                *stored = (*stored + total).min(capacity);
                total > Ntd::Zero || {
                    *stored = stored.saturating_sub(Ntd::One);
                    *stored > Ntd::Zero
                }
            }
            GateInstance::Delay { ref mut prev } => std::mem::replace(prev, inputs.any(|x| x)),
            GateInstance::Battery => true,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Node {
    pub(super) state: bool,
    id: NodeId,
    pub(super) gate: GateInstance,
    pub(super) position: IVec2,
}

impl Node {
    pub const fn new(id: NodeId, gate: Gate, position: IVec2, state: bool) -> Self {
        Self {
            state,
            id,
            gate: GateInstance::from_gate(gate),
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
    pub const fn gate(&self) -> &GateInstance {
        &self.gate
    }

    #[inline]
    pub const fn gate_mut(&mut self) -> &mut GateInstance {
        &mut self.gate
    }
}
