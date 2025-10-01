#![feature(impl_trait_in_assoc_type)]

use raylib::prelude::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(remote = "Vector2")]
struct Vector2Def {
    pub x: f32,
    pub y: f32,
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize)]
#[serde(remote = "KeyboardKey", rename_all = "snake_case")]
enum KeyboardKeyDef {
    #[serde(skip)]
    KEY_NULL,
    #[serde(rename = "'")]
    KEY_APOSTROPHE,
    #[serde(rename = ",")]
    KEY_COMMA,
    #[serde(rename = "-")]
    KEY_MINUS,
    #[serde(rename = ".")]
    KEY_PERIOD,
    #[serde(rename = "/")]
    KEY_SLASH,
    #[serde(rename = "0")]
    KEY_ZERO,
    #[serde(rename = "1")]
    KEY_ONE,
    #[serde(rename = "2")]
    KEY_TWO,
    #[serde(rename = "3")]
    KEY_THREE,
    #[serde(rename = "4")]
    KEY_FOUR,
    #[serde(rename = "5")]
    KEY_FIVE,
    #[serde(rename = "6")]
    KEY_SIX,
    #[serde(rename = "7")]
    KEY_SEVEN,
    #[serde(rename = "8")]
    KEY_EIGHT,
    #[serde(rename = "9")]
    KEY_NINE,
    #[serde(rename = ";")]
    KEY_SEMICOLON,
    #[serde(rename = "=")]
    KEY_EQUAL,
    KEY_A,
    KEY_B,
    KEY_C,
    KEY_D,
    KEY_E,
    KEY_F,
    KEY_G,
    KEY_H,
    KEY_I,
    KEY_J,
    KEY_K,
    KEY_L,
    KEY_M,
    KEY_N,
    KEY_O,
    KEY_P,
    KEY_Q,
    KEY_R,
    KEY_S,
    KEY_T,
    KEY_U,
    KEY_V,
    KEY_W,
    KEY_X,
    KEY_Y,
    KEY_Z,
    #[serde(rename = "[")]
    KEY_LEFT_BRACKET,
    #[serde(rename = "\\")]
    KEY_BACKSLASH,
    #[serde(rename = "]")]
    KEY_RIGHT_BRACKET,
    #[serde(rename = "`")]
    KEY_GRAVE,
    KEY_SPACE,
    #[serde(rename = "esc")]
    KEY_ESCAPE,
    KEY_ENTER,
    KEY_TAB,
    KEY_BACKSPACE,
    #[serde(rename = "ins")]
    KEY_INSERT,
    #[serde(rename = "del")]
    KEY_DELETE,
    KEY_RIGHT,
    KEY_LEFT,
    KEY_DOWN,
    KEY_UP,
    KEY_PAGE_UP,
    KEY_PAGE_DOWN,
    KEY_HOME,
    KEY_END,
    KEY_CAPS_LOCK,
    KEY_SCROLL_LOCK,
    KEY_NUM_LOCK,
    KEY_PRINT_SCREEN,
    KEY_PAUSE,
    KEY_F1,
    KEY_F2,
    KEY_F3,
    KEY_F4,
    KEY_F5,
    KEY_F6,
    KEY_F7,
    KEY_F8,
    KEY_F9,
    KEY_F10,
    KEY_F11,
    KEY_F12,
    #[serde(rename = "l_shift")]
    KEY_LEFT_SHIFT,
    #[serde(rename = "l_ctrl")]
    KEY_LEFT_CONTROL,
    #[serde(rename = "l_alt")]
    KEY_LEFT_ALT,
    #[serde(rename = "l_super")]
    KEY_LEFT_SUPER,
    #[serde(rename = "r_shift")]
    KEY_RIGHT_SHIFT,
    #[serde(rename = "r_ctrl")]
    KEY_RIGHT_CONTROL,
    #[serde(rename = "r_alt")]
    KEY_RIGHT_ALT,
    #[serde(rename = "r_super")]
    KEY_RIGHT_SUPER,
    KEY_KB_MENU,
    KEY_KP_0,
    KEY_KP_1,
    KEY_KP_2,
    KEY_KP_3,
    KEY_KP_4,
    KEY_KP_5,
    KEY_KP_6,
    KEY_KP_7,
    KEY_KP_8,
    KEY_KP_9,
    KEY_KP_DECIMAL,
    KEY_KP_DIVIDE,
    KEY_KP_MULTIPLY,
    KEY_KP_SUBTRACT,
    KEY_KP_ADD,
    KEY_KP_ENTER,
    KEY_KP_EQUAL,
    KEY_BACK,
    KEY_MENU,
    #[serde(rename = "vol_up")]
    KEY_VOLUME_UP,
    #[serde(rename = "vol_down")]
    KEY_VOLUME_DOWN,
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize)]
#[serde(remote = "MouseButton", rename_all = "snake_case")]
enum MouseButtonDef {
    #[serde(rename = "m1")]
    MOUSE_BUTTON_LEFT,
    #[serde(rename = "m2")]
    MOUSE_BUTTON_RIGHT,
    #[serde(rename = "m3")]
    MOUSE_BUTTON_MIDDLE,
    #[serde(rename = "m_side")]
    MOUSE_BUTTON_SIDE,
    #[serde(rename = "m_extra")]
    MOUSE_BUTTON_EXTRA,
    #[serde(rename = "m_forward")]
    MOUSE_BUTTON_FORWARD,
    #[serde(rename = "m_back")]
    MOUSE_BUTTON_BACK,
}

pub trait Source {
    type Value<'a>: 'a
    where
        Self: 'a;

    fn get<'a>(&'a mut self, rl: &RaylibHandle) -> Self::Value<'a>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Event {
    Inactive,
    Starting,
    Active,
    Ending,
}

impl Event {
    #[inline]
    pub fn is_active(self) -> bool {
        matches!(self, Self::Active | Self::Starting)
    }

    #[inline]
    pub fn is_inactive(self) -> bool {
        matches!(self, Self::Inactive | Self::Ending)
    }

    #[inline]
    pub fn is_starting(self) -> bool {
        matches!(self, Self::Starting)
    }

    #[inline]
    pub fn is_ending(self) -> bool {
        matches!(self, Self::Ending)
    }

    #[inline]
    pub fn is_changing(self) -> bool {
        matches!(self, Self::Starting | Self::Ending)
    }

    /// Set to [`Event::Starting`] if currently [inactive](Self::is_inactive), and [`Event::Active`] otherwise
    #[inline]
    pub fn activate(&mut self) {
        *self = match *self {
            Self::Inactive | Self::Ending => Self::Starting,
            Self::Active | Self::Starting => Self::Active,
        };
    }

    /// Set to [`Event::Ending`] if currently [inactive](Self::is_active), and [`Event::Inactive`] otherwise
    #[inline]
    pub fn deactivate(&mut self) {
        *self = match *self {
            Self::Active | Self::Starting => Self::Ending,
            Self::Inactive | Self::Ending => Self::Inactive,
        };
    }

    /// Downgrades [`Event::Starting`] to [`Event::Active`] and [`Event::Ending`] to [`Event::Inactive`]
    #[inline]
    pub fn next(self) -> Self {
        match self {
            Self::Starting | Self::Active => Self::Active,
            Self::Ending | Self::Inactive => Self::Inactive,
        }
    }

    /// Sets to the output of [`Self::next`]
    #[inline]
    pub fn step(&mut self) {
        *self = self.next();
    }

    /// Calls the corresponding "`is_`" methods rather than comparing with [`Eq`]
    fn is(&self, when: Event) -> bool {
        match when {
            Event::Inactive => self.is_inactive(),
            Event::Starting => self.is_starting(),
            Event::Active => self.is_active(),
            Event::Ending => self.is_ending(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventCombo {
    All(Box<[EventSource]>),
    Any(Box<[EventSource]>),
    Not(Box<EventSource>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventSource {
    Constant(Event),
    Keyboard(#[serde(with = "KeyboardKeyDef")] KeyboardKey),
    Mouse(#[serde(with = "MouseButtonDef")] MouseButton),
    Combo(EventCombo),
}

impl EventSource {
    #[inline]
    pub fn is_active(&mut self, rl: &RaylibHandle) -> bool {
        match self {
            Self::Constant(event) => event.is_active(),
            Self::Keyboard(key) => rl.is_key_down(*key),
            Self::Mouse(button) => rl.is_mouse_button_down(*button),
            Self::Combo(EventCombo::All(items)) => items.iter_mut().any(|x| x.is_active(rl)),
            Self::Combo(EventCombo::Any(items)) => items.iter_mut().all(|x| x.is_active(rl)),
            Self::Combo(EventCombo::Not(item)) => !item.is_active(rl),
        }
    }

    #[inline]
    pub fn is_starting(&mut self, rl: &RaylibHandle) -> bool {
        match self {
            Self::Constant(event) => event.is_starting(),
            Self::Keyboard(key) => rl.is_key_pressed(*key),
            Self::Mouse(button) => rl.is_mouse_button_pressed(*button),
            Self::Combo(EventCombo::All(items)) => items.iter_mut().any(|x| x.is_starting(rl)),
            Self::Combo(EventCombo::Any(items)) => {
                items.iter_mut().any(|x| x.is_starting(rl))
                    && items.iter_mut().all(|x| x.is_active(rl))
            }
            Self::Combo(EventCombo::Not(item)) => !item.is_starting(rl),
        }
    }

    #[inline]
    pub fn is_ending(&mut self, rl: &RaylibHandle) -> bool {
        match self {
            Self::Constant(event) => event.is_ending(),
            Self::Keyboard(key) => rl.is_key_released(*key),
            Self::Mouse(button) => rl.is_mouse_button_released(*button),
            Self::Combo(EventCombo::All(items)) => {
                items.iter_mut().any(|x| x.is_ending(rl))
                    && items.iter_mut().all(
                        |x| !x.is_active(rl), // assumes that if an item is ending, it is also inactive
                    )
            }
            Self::Combo(EventCombo::Any(items)) => {
                items.iter_mut().any(|x| x.is_ending(rl))
                    && items.iter_mut().all(|x| x.is_active(rl) || x.is_ending(rl))
            }
            Self::Combo(EventCombo::Not(item)) => !item.is_ending(rl),
        }
    }
}

impl Source for EventSource {
    type Value<'a> = Event;

    /// Prefer calling [`Self::is_active`], [`Self::is_starting`], or [`Self::is_ending`] if you only need one
    fn get(&mut self, rl: &RaylibHandle) -> Event {
        if let Self::Constant(event) = self {
            *event
        } else if self.is_active(rl) {
            if self.is_starting(rl) {
                Event::Starting
            } else {
                Event::Active
            }
        } else if self.is_ending(rl) {
            Event::Ending
        } else {
            Event::Inactive
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IntSource {
    Selector(SelectorSource<i32>),
    Sum(Box<[Self]>),
    Prod(Box<[Self]>),
    Diff(Box<[Self]>),
}

impl Source for IntSource {
    type Value<'a> = i32;

    fn get(&mut self, rl: &RaylibHandle) -> i32 {
        match self {
            Self::Selector(src) => src.get(rl).first().map(|x| **x).unwrap_or(0),
            Self::Sum(items) => items.iter_mut().map(|item| item.get(rl)).sum(),
            Self::Prod(items) => items.iter_mut().map(|item| item.get(rl)).product(),
            Self::Diff(items) => items
                .iter_mut()
                .map(|item| item.get(rl))
                .reduce(|a, b| a - b)
                .unwrap_or(0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSource<T> {
    pub index: IntSource,
    pub options: Box<[T]>,
}

impl<T> Source for IndexSource<T> {
    type Value<'a>
        = Option<&'a mut T>
    where
        Self: 'a;

    fn get<'a>(&'a mut self, rl: &RaylibHandle) -> Option<&'a mut T> {
        self.options
            .get_mut(usize::try_from(self.index.get(rl)).ok()?)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "std::cmp::Ordering")]
enum OrderingDef {
    #[serde(rename = "<")]
    Less = -1,
    #[serde(rename = "=")]
    Equal = 0,
    #[serde(rename = ">")]
    Greater = 1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BoolSource {
    Event {
        what: EventSource,
        when: Event,
    },
    Compare {
        src: AxisSource,
        #[serde(with = "OrderingDef")]
        cmp: std::cmp::Ordering,
        val: f32,
    },
    All(Box<[Self]>),
    Any(Box<[Self]>),
    Not(Box<Self>),
}

impl Source for BoolSource {
    type Value<'a> = bool;

    fn get(&mut self, rl: &RaylibHandle) -> bool {
        match self {
            Self::Event { what, when } => what.get(rl).is(*when),
            Self::Compare { src, cmp, val } => {
                src.get(rl).partial_cmp(val).is_some_and(|x| x == *cmp)
            }
            Self::All(items) => items.iter_mut().all(|item| item.get(rl)),
            Self::Any(items) => items.iter_mut().any(|item| item.get(rl)),
            Self::Not(item) => !item.get(rl),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorItem<T> {
    pub src: BoolSource,
    pub val: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectorSource<T>(Box<[SelectorItem<T>]>);

impl<T, U: Into<Box<[SelectorItem<T>]>>> From<U> for SelectorSource<T> {
    fn from(value: U) -> Self {
        Self(value.into())
    }
}

impl<T> Source for SelectorSource<T> {
    type Value<'a>
        = Box<[&'a mut T]>
    where
        Self: 'a;

    fn get<'a>(&'a mut self, rl: &RaylibHandle) -> Self::Value<'a> {
        self.0
            .iter_mut()
            .filter_map(|item| item.src.get(rl).then_some(&mut item.val))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AxisSource {
    #[serde(rename = "const")]
    Constant(f32),
    #[serde(rename = "scroll")]
    MouseWheelMove,
    EventMix(SelectorSource<AxisSource>),
    #[serde(rename = "+")]
    Sum(Box<[Self]>),
    #[serde(rename = "*")]
    Prod(Box<[Self]>),
    #[serde(rename = "-")]
    Neg(Box<Self>),
}

impl Source for AxisSource {
    type Value<'a>
        = f32
    where
        Self: 'a;

    fn get(&mut self, rl: &RaylibHandle) -> f32 {
        match self {
            Self::Constant(x) => *x,
            Self::MouseWheelMove => rl.get_mouse_wheel_move(),
            Self::EventMix(items) => items.get(rl).iter_mut().map(|x| x.get(rl)).sum(),
            Self::Sum(items) => items.iter_mut().map(|x| x.get(rl)).sum(),
            Self::Prod(items) => items.iter_mut().map(|x| x.get(rl)).product(),
            Self::Neg(item) => -item.get(rl),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VectorSource {
    #[serde(rename = "const")]
    Constant(#[serde(with = "Vector2Def")] Vector2),
    MousePosition,
    MouseDelta,
    EventMix(SelectorSource<VectorSource>),
    #[serde(rename = "xy")]
    AxisXY {
        x: AxisSource,
        y: AxisSource,
    },
    #[serde(rename = "+")]
    Sum(Box<[Self]>),
    #[serde(rename = "*")]
    Prod(Box<[Self]>),
    #[serde(rename = "-")]
    Neg(Box<Self>),
}

impl Source for VectorSource {
    type Value<'a>
        = Vector2
    where
        Self: 'a;

    #[inline]
    fn get(&mut self, rl: &RaylibHandle) -> Vector2 {
        match self {
            Self::Constant(v) => *v,
            Self::MousePosition => rl.get_mouse_position(),
            Self::MouseDelta => rl.get_mouse_delta(),
            Self::EventMix(items) => items
                .get(rl)
                .iter_mut()
                .map(|src| src.get(rl))
                .reduce(|a, b| a + b)
                .unwrap_or(Vector2::zero()),
            Self::AxisXY { x, y } => Vector2::new(x.get(rl), y.get(rl)),
            Self::Sum(items) => items
                .iter_mut()
                .map(|x| x.get(rl))
                .reduce(|a, b| a + b)
                .unwrap_or(Vector2::zero()),
            Self::Prod(items) => items
                .iter_mut()
                .map(|x| x.get(rl))
                .reduce(|a, b| a * b)
                .unwrap_or(Vector2::zero()),
            Self::Neg(item) => -item.get(rl),
        }
    }
}
