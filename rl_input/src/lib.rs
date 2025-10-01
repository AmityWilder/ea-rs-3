#![feature(impl_trait_in_assoc_type)]

use raylib::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventSourceDef {
    Inactive,
    Starting,
    Active,
    Ending,

    #[serde(rename = "'")]
    Apostrophe,
    #[serde(rename = ",")]
    Comma,
    #[serde(rename = "-")]
    Minus,
    #[serde(rename = ".")]
    Period,
    #[serde(rename = "/")]
    Slash,
    #[serde(rename = "0")]
    Zero,
    #[serde(rename = "1")]
    One,
    #[serde(rename = "2")]
    Two,
    #[serde(rename = "3")]
    Three,
    #[serde(rename = "4")]
    Four,
    #[serde(rename = "5")]
    Five,
    #[serde(rename = "6")]
    Six,
    #[serde(rename = "7")]
    Seven,
    #[serde(rename = "8")]
    Eight,
    #[serde(rename = "9")]
    Nine,
    #[serde(rename = ";")]
    Semicolon,
    #[serde(rename = "=")]
    Equal,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    #[serde(rename = "[")]
    LeftBracket,
    #[serde(rename = "\\")]
    Backslash,
    #[serde(rename = "]")]
    RightBracket,
    #[serde(rename = "`")]
    Grave,
    Space,
    #[serde(rename = "esc")]
    Escape,
    Enter,
    Tab,
    Backspace,
    #[serde(rename = "ins")]
    Insert,
    #[serde(rename = "del")]
    Delete,
    Right,
    Left,
    Down,
    Up,
    PageUp,
    PageDown,
    Home,
    End,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    #[serde(rename = "l_shift")]
    LeftShift,
    #[serde(rename = "l_ctrl")]
    LeftControl,
    #[serde(rename = "l_alt")]
    LeftAlt,
    #[serde(rename = "l_super")]
    LeftSuper,
    #[serde(rename = "r_shift")]
    RightShift,
    #[serde(rename = "r_ctrl")]
    RightControl,
    #[serde(rename = "r_alt")]
    RightAlt,
    #[serde(rename = "r_super")]
    RightSuper,
    KbMenu,
    Kp0,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    KpDecimal,
    KpDivide,
    KpMultiply,
    KpSubtract,
    KpAdd,
    KpEnter,
    KpEqual,
    Back,
    Menu,
    #[serde(rename = "vol_up")]
    VolumeUp,
    #[serde(rename = "vol_down")]
    VolumeDown,

    #[serde(rename = "m1")]
    MouseLeft,
    #[serde(rename = "m2")]
    MouseRight,
    #[serde(rename = "m3")]
    MouseMiddle,
    #[serde(rename = "m_side")]
    MouseSide,
    #[serde(rename = "m_extra")]
    MouseExtra,
    #[serde(rename = "m_forward")]
    MouseForward,
    #[serde(rename = "m_back")]
    MouseBack,

    All(Box<[Self]>),
    Any(Box<[Self]>),
    Not(Box<Self>),
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

#[derive(Debug, Clone)]
pub enum EventSource {
    Constant(Event),
    Keyboard(KeyboardKey),
    Mouse(MouseButton),
    All(Box<[Self]>),
    Any(Box<[Self]>),
    Not(Box<Self>),
}

impl From<EventSource> for EventSourceDef {
    fn from(value: EventSource) -> Self {
        match value {
            EventSource::Constant(Event::Inactive) => EventSourceDef::Inactive,
            EventSource::Constant(Event::Starting) => EventSourceDef::Starting,
            EventSource::Constant(Event::Active) => EventSourceDef::Active,
            EventSource::Constant(Event::Ending) => EventSourceDef::Ending,
            EventSource::Keyboard(KeyboardKey::KEY_NULL) => unimplemented!(),
            EventSource::Keyboard(KeyboardKey::KEY_APOSTROPHE) => EventSourceDef::Apostrophe,
            EventSource::Keyboard(KeyboardKey::KEY_COMMA) => EventSourceDef::Comma,
            EventSource::Keyboard(KeyboardKey::KEY_MINUS) => EventSourceDef::Minus,
            EventSource::Keyboard(KeyboardKey::KEY_PERIOD) => EventSourceDef::Period,
            EventSource::Keyboard(KeyboardKey::KEY_SLASH) => EventSourceDef::Slash,
            EventSource::Keyboard(KeyboardKey::KEY_ZERO) => EventSourceDef::Zero,
            EventSource::Keyboard(KeyboardKey::KEY_ONE) => EventSourceDef::One,
            EventSource::Keyboard(KeyboardKey::KEY_TWO) => EventSourceDef::Two,
            EventSource::Keyboard(KeyboardKey::KEY_THREE) => EventSourceDef::Three,
            EventSource::Keyboard(KeyboardKey::KEY_FOUR) => EventSourceDef::Four,
            EventSource::Keyboard(KeyboardKey::KEY_FIVE) => EventSourceDef::Five,
            EventSource::Keyboard(KeyboardKey::KEY_SIX) => EventSourceDef::Six,
            EventSource::Keyboard(KeyboardKey::KEY_SEVEN) => EventSourceDef::Seven,
            EventSource::Keyboard(KeyboardKey::KEY_EIGHT) => EventSourceDef::Eight,
            EventSource::Keyboard(KeyboardKey::KEY_NINE) => EventSourceDef::Nine,
            EventSource::Keyboard(KeyboardKey::KEY_SEMICOLON) => EventSourceDef::Semicolon,
            EventSource::Keyboard(KeyboardKey::KEY_EQUAL) => EventSourceDef::Equal,
            EventSource::Keyboard(KeyboardKey::KEY_A) => EventSourceDef::A,
            EventSource::Keyboard(KeyboardKey::KEY_B) => EventSourceDef::B,
            EventSource::Keyboard(KeyboardKey::KEY_C) => EventSourceDef::C,
            EventSource::Keyboard(KeyboardKey::KEY_D) => EventSourceDef::D,
            EventSource::Keyboard(KeyboardKey::KEY_E) => EventSourceDef::E,
            EventSource::Keyboard(KeyboardKey::KEY_F) => EventSourceDef::F,
            EventSource::Keyboard(KeyboardKey::KEY_G) => EventSourceDef::G,
            EventSource::Keyboard(KeyboardKey::KEY_H) => EventSourceDef::H,
            EventSource::Keyboard(KeyboardKey::KEY_I) => EventSourceDef::I,
            EventSource::Keyboard(KeyboardKey::KEY_J) => EventSourceDef::J,
            EventSource::Keyboard(KeyboardKey::KEY_K) => EventSourceDef::K,
            EventSource::Keyboard(KeyboardKey::KEY_L) => EventSourceDef::L,
            EventSource::Keyboard(KeyboardKey::KEY_M) => EventSourceDef::M,
            EventSource::Keyboard(KeyboardKey::KEY_N) => EventSourceDef::N,
            EventSource::Keyboard(KeyboardKey::KEY_O) => EventSourceDef::O,
            EventSource::Keyboard(KeyboardKey::KEY_P) => EventSourceDef::P,
            EventSource::Keyboard(KeyboardKey::KEY_Q) => EventSourceDef::Q,
            EventSource::Keyboard(KeyboardKey::KEY_R) => EventSourceDef::R,
            EventSource::Keyboard(KeyboardKey::KEY_S) => EventSourceDef::S,
            EventSource::Keyboard(KeyboardKey::KEY_T) => EventSourceDef::T,
            EventSource::Keyboard(KeyboardKey::KEY_U) => EventSourceDef::U,
            EventSource::Keyboard(KeyboardKey::KEY_V) => EventSourceDef::V,
            EventSource::Keyboard(KeyboardKey::KEY_W) => EventSourceDef::W,
            EventSource::Keyboard(KeyboardKey::KEY_X) => EventSourceDef::X,
            EventSource::Keyboard(KeyboardKey::KEY_Y) => EventSourceDef::Y,
            EventSource::Keyboard(KeyboardKey::KEY_Z) => EventSourceDef::Z,
            EventSource::Keyboard(KeyboardKey::KEY_LEFT_BRACKET) => EventSourceDef::LeftBracket,
            EventSource::Keyboard(KeyboardKey::KEY_BACKSLASH) => EventSourceDef::Backslash,
            EventSource::Keyboard(KeyboardKey::KEY_RIGHT_BRACKET) => EventSourceDef::RightBracket,
            EventSource::Keyboard(KeyboardKey::KEY_GRAVE) => EventSourceDef::Grave,
            EventSource::Keyboard(KeyboardKey::KEY_SPACE) => EventSourceDef::Space,
            EventSource::Keyboard(KeyboardKey::KEY_ESCAPE) => EventSourceDef::Escape,
            EventSource::Keyboard(KeyboardKey::KEY_ENTER) => EventSourceDef::Enter,
            EventSource::Keyboard(KeyboardKey::KEY_TAB) => EventSourceDef::Tab,
            EventSource::Keyboard(KeyboardKey::KEY_BACKSPACE) => EventSourceDef::Backspace,
            EventSource::Keyboard(KeyboardKey::KEY_INSERT) => EventSourceDef::Insert,
            EventSource::Keyboard(KeyboardKey::KEY_DELETE) => EventSourceDef::Delete,
            EventSource::Keyboard(KeyboardKey::KEY_RIGHT) => EventSourceDef::Right,
            EventSource::Keyboard(KeyboardKey::KEY_LEFT) => EventSourceDef::Left,
            EventSource::Keyboard(KeyboardKey::KEY_DOWN) => EventSourceDef::Down,
            EventSource::Keyboard(KeyboardKey::KEY_UP) => EventSourceDef::Up,
            EventSource::Keyboard(KeyboardKey::KEY_PAGE_UP) => EventSourceDef::PageUp,
            EventSource::Keyboard(KeyboardKey::KEY_PAGE_DOWN) => EventSourceDef::PageDown,
            EventSource::Keyboard(KeyboardKey::KEY_HOME) => EventSourceDef::Home,
            EventSource::Keyboard(KeyboardKey::KEY_END) => EventSourceDef::End,
            EventSource::Keyboard(KeyboardKey::KEY_CAPS_LOCK) => EventSourceDef::CapsLock,
            EventSource::Keyboard(KeyboardKey::KEY_SCROLL_LOCK) => EventSourceDef::ScrollLock,
            EventSource::Keyboard(KeyboardKey::KEY_NUM_LOCK) => EventSourceDef::NumLock,
            EventSource::Keyboard(KeyboardKey::KEY_PRINT_SCREEN) => EventSourceDef::PrintScreen,
            EventSource::Keyboard(KeyboardKey::KEY_PAUSE) => EventSourceDef::Pause,
            EventSource::Keyboard(KeyboardKey::KEY_F1) => EventSourceDef::F1,
            EventSource::Keyboard(KeyboardKey::KEY_F2) => EventSourceDef::F2,
            EventSource::Keyboard(KeyboardKey::KEY_F3) => EventSourceDef::F3,
            EventSource::Keyboard(KeyboardKey::KEY_F4) => EventSourceDef::F4,
            EventSource::Keyboard(KeyboardKey::KEY_F5) => EventSourceDef::F5,
            EventSource::Keyboard(KeyboardKey::KEY_F6) => EventSourceDef::F6,
            EventSource::Keyboard(KeyboardKey::KEY_F7) => EventSourceDef::F7,
            EventSource::Keyboard(KeyboardKey::KEY_F8) => EventSourceDef::F8,
            EventSource::Keyboard(KeyboardKey::KEY_F9) => EventSourceDef::F9,
            EventSource::Keyboard(KeyboardKey::KEY_F10) => EventSourceDef::F10,
            EventSource::Keyboard(KeyboardKey::KEY_F11) => EventSourceDef::F11,
            EventSource::Keyboard(KeyboardKey::KEY_F12) => EventSourceDef::F12,
            EventSource::Keyboard(KeyboardKey::KEY_LEFT_SHIFT) => EventSourceDef::LeftShift,
            EventSource::Keyboard(KeyboardKey::KEY_LEFT_CONTROL) => EventSourceDef::LeftControl,
            EventSource::Keyboard(KeyboardKey::KEY_LEFT_ALT) => EventSourceDef::LeftAlt,
            EventSource::Keyboard(KeyboardKey::KEY_LEFT_SUPER) => EventSourceDef::LeftSuper,
            EventSource::Keyboard(KeyboardKey::KEY_RIGHT_SHIFT) => EventSourceDef::RightShift,
            EventSource::Keyboard(KeyboardKey::KEY_RIGHT_CONTROL) => EventSourceDef::RightControl,
            EventSource::Keyboard(KeyboardKey::KEY_RIGHT_ALT) => EventSourceDef::RightAlt,
            EventSource::Keyboard(KeyboardKey::KEY_RIGHT_SUPER) => EventSourceDef::RightSuper,
            EventSource::Keyboard(KeyboardKey::KEY_KB_MENU) => EventSourceDef::KbMenu,
            EventSource::Keyboard(KeyboardKey::KEY_KP_0) => EventSourceDef::Kp0,
            EventSource::Keyboard(KeyboardKey::KEY_KP_1) => EventSourceDef::Kp1,
            EventSource::Keyboard(KeyboardKey::KEY_KP_2) => EventSourceDef::Kp2,
            EventSource::Keyboard(KeyboardKey::KEY_KP_3) => EventSourceDef::Kp3,
            EventSource::Keyboard(KeyboardKey::KEY_KP_4) => EventSourceDef::Kp4,
            EventSource::Keyboard(KeyboardKey::KEY_KP_5) => EventSourceDef::Kp5,
            EventSource::Keyboard(KeyboardKey::KEY_KP_6) => EventSourceDef::Kp6,
            EventSource::Keyboard(KeyboardKey::KEY_KP_7) => EventSourceDef::Kp7,
            EventSource::Keyboard(KeyboardKey::KEY_KP_8) => EventSourceDef::Kp8,
            EventSource::Keyboard(KeyboardKey::KEY_KP_9) => EventSourceDef::Kp9,
            EventSource::Keyboard(KeyboardKey::KEY_KP_DECIMAL) => EventSourceDef::KpDecimal,
            EventSource::Keyboard(KeyboardKey::KEY_KP_DIVIDE) => EventSourceDef::KpDivide,
            EventSource::Keyboard(KeyboardKey::KEY_KP_MULTIPLY) => EventSourceDef::KpMultiply,
            EventSource::Keyboard(KeyboardKey::KEY_KP_SUBTRACT) => EventSourceDef::KpSubtract,
            EventSource::Keyboard(KeyboardKey::KEY_KP_ADD) => EventSourceDef::KpAdd,
            EventSource::Keyboard(KeyboardKey::KEY_KP_ENTER) => EventSourceDef::KpEnter,
            EventSource::Keyboard(KeyboardKey::KEY_KP_EQUAL) => EventSourceDef::KpEqual,
            EventSource::Keyboard(KeyboardKey::KEY_BACK) => EventSourceDef::Back,
            EventSource::Keyboard(KeyboardKey::KEY_MENU) => EventSourceDef::Menu,
            EventSource::Keyboard(KeyboardKey::KEY_VOLUME_UP) => EventSourceDef::VolumeUp,
            EventSource::Keyboard(KeyboardKey::KEY_VOLUME_DOWN) => EventSourceDef::VolumeDown,
            EventSource::Mouse(MouseButton::MOUSE_BUTTON_LEFT) => EventSourceDef::MouseLeft,
            EventSource::Mouse(MouseButton::MOUSE_BUTTON_RIGHT) => EventSourceDef::MouseRight,
            EventSource::Mouse(MouseButton::MOUSE_BUTTON_MIDDLE) => EventSourceDef::MouseMiddle,
            EventSource::Mouse(MouseButton::MOUSE_BUTTON_SIDE) => EventSourceDef::MouseSide,
            EventSource::Mouse(MouseButton::MOUSE_BUTTON_EXTRA) => EventSourceDef::MouseExtra,
            EventSource::Mouse(MouseButton::MOUSE_BUTTON_FORWARD) => EventSourceDef::MouseForward,
            EventSource::Mouse(MouseButton::MOUSE_BUTTON_BACK) => EventSourceDef::MouseBack,
            EventSource::All(x) => {
                EventSourceDef::All(x.into_iter().map(EventSourceDef::from).collect())
            }
            EventSource::Any(x) => {
                EventSourceDef::Any(x.into_iter().map(EventSourceDef::from).collect())
            }
            EventSource::Not(x) => EventSourceDef::Not(Box::from(EventSourceDef::from(*x))),
        }
    }
}

impl From<EventSourceDef> for EventSource {
    fn from(value: EventSourceDef) -> Self {
        match value {
            EventSourceDef::Inactive => EventSource::Constant(Event::Inactive),
            EventSourceDef::Starting => EventSource::Constant(Event::Starting),
            EventSourceDef::Active => EventSource::Constant(Event::Active),
            EventSourceDef::Ending => EventSource::Constant(Event::Ending),
            EventSourceDef::Apostrophe => EventSource::Keyboard(KeyboardKey::KEY_APOSTROPHE),
            EventSourceDef::Comma => EventSource::Keyboard(KeyboardKey::KEY_COMMA),
            EventSourceDef::Minus => EventSource::Keyboard(KeyboardKey::KEY_MINUS),
            EventSourceDef::Period => EventSource::Keyboard(KeyboardKey::KEY_PERIOD),
            EventSourceDef::Slash => EventSource::Keyboard(KeyboardKey::KEY_SLASH),
            EventSourceDef::Zero => EventSource::Keyboard(KeyboardKey::KEY_ZERO),
            EventSourceDef::One => EventSource::Keyboard(KeyboardKey::KEY_ONE),
            EventSourceDef::Two => EventSource::Keyboard(KeyboardKey::KEY_TWO),
            EventSourceDef::Three => EventSource::Keyboard(KeyboardKey::KEY_THREE),
            EventSourceDef::Four => EventSource::Keyboard(KeyboardKey::KEY_FOUR),
            EventSourceDef::Five => EventSource::Keyboard(KeyboardKey::KEY_FIVE),
            EventSourceDef::Six => EventSource::Keyboard(KeyboardKey::KEY_SIX),
            EventSourceDef::Seven => EventSource::Keyboard(KeyboardKey::KEY_SEVEN),
            EventSourceDef::Eight => EventSource::Keyboard(KeyboardKey::KEY_EIGHT),
            EventSourceDef::Nine => EventSource::Keyboard(KeyboardKey::KEY_NINE),
            EventSourceDef::Semicolon => EventSource::Keyboard(KeyboardKey::KEY_SEMICOLON),
            EventSourceDef::Equal => EventSource::Keyboard(KeyboardKey::KEY_EQUAL),
            EventSourceDef::A => EventSource::Keyboard(KeyboardKey::KEY_A),
            EventSourceDef::B => EventSource::Keyboard(KeyboardKey::KEY_B),
            EventSourceDef::C => EventSource::Keyboard(KeyboardKey::KEY_C),
            EventSourceDef::D => EventSource::Keyboard(KeyboardKey::KEY_D),
            EventSourceDef::E => EventSource::Keyboard(KeyboardKey::KEY_E),
            EventSourceDef::F => EventSource::Keyboard(KeyboardKey::KEY_F),
            EventSourceDef::G => EventSource::Keyboard(KeyboardKey::KEY_G),
            EventSourceDef::H => EventSource::Keyboard(KeyboardKey::KEY_H),
            EventSourceDef::I => EventSource::Keyboard(KeyboardKey::KEY_I),
            EventSourceDef::J => EventSource::Keyboard(KeyboardKey::KEY_J),
            EventSourceDef::K => EventSource::Keyboard(KeyboardKey::KEY_K),
            EventSourceDef::L => EventSource::Keyboard(KeyboardKey::KEY_L),
            EventSourceDef::M => EventSource::Keyboard(KeyboardKey::KEY_M),
            EventSourceDef::N => EventSource::Keyboard(KeyboardKey::KEY_N),
            EventSourceDef::O => EventSource::Keyboard(KeyboardKey::KEY_O),
            EventSourceDef::P => EventSource::Keyboard(KeyboardKey::KEY_P),
            EventSourceDef::Q => EventSource::Keyboard(KeyboardKey::KEY_Q),
            EventSourceDef::R => EventSource::Keyboard(KeyboardKey::KEY_R),
            EventSourceDef::S => EventSource::Keyboard(KeyboardKey::KEY_S),
            EventSourceDef::T => EventSource::Keyboard(KeyboardKey::KEY_T),
            EventSourceDef::U => EventSource::Keyboard(KeyboardKey::KEY_U),
            EventSourceDef::V => EventSource::Keyboard(KeyboardKey::KEY_V),
            EventSourceDef::W => EventSource::Keyboard(KeyboardKey::KEY_W),
            EventSourceDef::X => EventSource::Keyboard(KeyboardKey::KEY_X),
            EventSourceDef::Y => EventSource::Keyboard(KeyboardKey::KEY_Y),
            EventSourceDef::Z => EventSource::Keyboard(KeyboardKey::KEY_Z),
            EventSourceDef::LeftBracket => EventSource::Keyboard(KeyboardKey::KEY_LEFT_BRACKET),
            EventSourceDef::Backslash => EventSource::Keyboard(KeyboardKey::KEY_BACKSLASH),
            EventSourceDef::RightBracket => EventSource::Keyboard(KeyboardKey::KEY_RIGHT_BRACKET),
            EventSourceDef::Grave => EventSource::Keyboard(KeyboardKey::KEY_GRAVE),
            EventSourceDef::Space => EventSource::Keyboard(KeyboardKey::KEY_SPACE),
            EventSourceDef::Escape => EventSource::Keyboard(KeyboardKey::KEY_ESCAPE),
            EventSourceDef::Enter => EventSource::Keyboard(KeyboardKey::KEY_ENTER),
            EventSourceDef::Tab => EventSource::Keyboard(KeyboardKey::KEY_TAB),
            EventSourceDef::Backspace => EventSource::Keyboard(KeyboardKey::KEY_BACKSPACE),
            EventSourceDef::Insert => EventSource::Keyboard(KeyboardKey::KEY_INSERT),
            EventSourceDef::Delete => EventSource::Keyboard(KeyboardKey::KEY_DELETE),
            EventSourceDef::Right => EventSource::Keyboard(KeyboardKey::KEY_RIGHT),
            EventSourceDef::Left => EventSource::Keyboard(KeyboardKey::KEY_LEFT),
            EventSourceDef::Down => EventSource::Keyboard(KeyboardKey::KEY_DOWN),
            EventSourceDef::Up => EventSource::Keyboard(KeyboardKey::KEY_UP),
            EventSourceDef::PageUp => EventSource::Keyboard(KeyboardKey::KEY_PAGE_UP),
            EventSourceDef::PageDown => EventSource::Keyboard(KeyboardKey::KEY_PAGE_DOWN),
            EventSourceDef::Home => EventSource::Keyboard(KeyboardKey::KEY_HOME),
            EventSourceDef::End => EventSource::Keyboard(KeyboardKey::KEY_END),
            EventSourceDef::CapsLock => EventSource::Keyboard(KeyboardKey::KEY_CAPS_LOCK),
            EventSourceDef::ScrollLock => EventSource::Keyboard(KeyboardKey::KEY_SCROLL_LOCK),
            EventSourceDef::NumLock => EventSource::Keyboard(KeyboardKey::KEY_NUM_LOCK),
            EventSourceDef::PrintScreen => EventSource::Keyboard(KeyboardKey::KEY_PRINT_SCREEN),
            EventSourceDef::Pause => EventSource::Keyboard(KeyboardKey::KEY_PAUSE),
            EventSourceDef::F1 => EventSource::Keyboard(KeyboardKey::KEY_F1),
            EventSourceDef::F2 => EventSource::Keyboard(KeyboardKey::KEY_F2),
            EventSourceDef::F3 => EventSource::Keyboard(KeyboardKey::KEY_F3),
            EventSourceDef::F4 => EventSource::Keyboard(KeyboardKey::KEY_F4),
            EventSourceDef::F5 => EventSource::Keyboard(KeyboardKey::KEY_F5),
            EventSourceDef::F6 => EventSource::Keyboard(KeyboardKey::KEY_F6),
            EventSourceDef::F7 => EventSource::Keyboard(KeyboardKey::KEY_F7),
            EventSourceDef::F8 => EventSource::Keyboard(KeyboardKey::KEY_F8),
            EventSourceDef::F9 => EventSource::Keyboard(KeyboardKey::KEY_F9),
            EventSourceDef::F10 => EventSource::Keyboard(KeyboardKey::KEY_F10),
            EventSourceDef::F11 => EventSource::Keyboard(KeyboardKey::KEY_F11),
            EventSourceDef::F12 => EventSource::Keyboard(KeyboardKey::KEY_F12),
            EventSourceDef::LeftShift => EventSource::Keyboard(KeyboardKey::KEY_LEFT_SHIFT),
            EventSourceDef::LeftControl => EventSource::Keyboard(KeyboardKey::KEY_LEFT_CONTROL),
            EventSourceDef::LeftAlt => EventSource::Keyboard(KeyboardKey::KEY_LEFT_ALT),
            EventSourceDef::LeftSuper => EventSource::Keyboard(KeyboardKey::KEY_LEFT_SUPER),
            EventSourceDef::RightShift => EventSource::Keyboard(KeyboardKey::KEY_RIGHT_SHIFT),
            EventSourceDef::RightControl => EventSource::Keyboard(KeyboardKey::KEY_RIGHT_CONTROL),
            EventSourceDef::RightAlt => EventSource::Keyboard(KeyboardKey::KEY_RIGHT_ALT),
            EventSourceDef::RightSuper => EventSource::Keyboard(KeyboardKey::KEY_RIGHT_SUPER),
            EventSourceDef::KbMenu => EventSource::Keyboard(KeyboardKey::KEY_KB_MENU),
            EventSourceDef::Kp0 => EventSource::Keyboard(KeyboardKey::KEY_KP_0),
            EventSourceDef::Kp1 => EventSource::Keyboard(KeyboardKey::KEY_KP_1),
            EventSourceDef::Kp2 => EventSource::Keyboard(KeyboardKey::KEY_KP_2),
            EventSourceDef::Kp3 => EventSource::Keyboard(KeyboardKey::KEY_KP_3),
            EventSourceDef::Kp4 => EventSource::Keyboard(KeyboardKey::KEY_KP_4),
            EventSourceDef::Kp5 => EventSource::Keyboard(KeyboardKey::KEY_KP_5),
            EventSourceDef::Kp6 => EventSource::Keyboard(KeyboardKey::KEY_KP_6),
            EventSourceDef::Kp7 => EventSource::Keyboard(KeyboardKey::KEY_KP_7),
            EventSourceDef::Kp8 => EventSource::Keyboard(KeyboardKey::KEY_KP_8),
            EventSourceDef::Kp9 => EventSource::Keyboard(KeyboardKey::KEY_KP_9),
            EventSourceDef::KpDecimal => EventSource::Keyboard(KeyboardKey::KEY_KP_DECIMAL),
            EventSourceDef::KpDivide => EventSource::Keyboard(KeyboardKey::KEY_KP_DIVIDE),
            EventSourceDef::KpMultiply => EventSource::Keyboard(KeyboardKey::KEY_KP_MULTIPLY),
            EventSourceDef::KpSubtract => EventSource::Keyboard(KeyboardKey::KEY_KP_SUBTRACT),
            EventSourceDef::KpAdd => EventSource::Keyboard(KeyboardKey::KEY_KP_ADD),
            EventSourceDef::KpEnter => EventSource::Keyboard(KeyboardKey::KEY_KP_ENTER),
            EventSourceDef::KpEqual => EventSource::Keyboard(KeyboardKey::KEY_KP_EQUAL),
            EventSourceDef::Back => EventSource::Keyboard(KeyboardKey::KEY_BACK),
            EventSourceDef::Menu => EventSource::Keyboard(KeyboardKey::KEY_MENU),
            EventSourceDef::VolumeUp => EventSource::Keyboard(KeyboardKey::KEY_VOLUME_UP),
            EventSourceDef::VolumeDown => EventSource::Keyboard(KeyboardKey::KEY_VOLUME_DOWN),
            EventSourceDef::MouseLeft => EventSource::Mouse(MouseButton::MOUSE_BUTTON_LEFT),
            EventSourceDef::MouseRight => EventSource::Mouse(MouseButton::MOUSE_BUTTON_RIGHT),
            EventSourceDef::MouseMiddle => EventSource::Mouse(MouseButton::MOUSE_BUTTON_MIDDLE),
            EventSourceDef::MouseSide => EventSource::Mouse(MouseButton::MOUSE_BUTTON_SIDE),
            EventSourceDef::MouseExtra => EventSource::Mouse(MouseButton::MOUSE_BUTTON_EXTRA),
            EventSourceDef::MouseForward => EventSource::Mouse(MouseButton::MOUSE_BUTTON_FORWARD),
            EventSourceDef::MouseBack => EventSource::Mouse(MouseButton::MOUSE_BUTTON_BACK),
            EventSourceDef::All(x) => {
                EventSource::All(x.into_iter().map(EventSource::from).collect())
            }
            EventSourceDef::Any(x) => {
                EventSource::Any(x.into_iter().map(EventSource::from).collect())
            }
            EventSourceDef::Not(x) => EventSource::Not(Box::from(EventSource::from(*x))),
        }
    }
}

impl Serialize for EventSource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        EventSourceDef::serialize(&EventSourceDef::from(self.clone()), serializer)
    }
}

impl<'de> Deserialize<'de> for EventSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        EventSourceDef::deserialize(deserializer).map(EventSource::from)
    }
}

impl EventSource {
    #[inline]
    pub fn is_active(&mut self, rl: &RaylibHandle) -> bool {
        match self {
            Self::Constant(event) => event.is_active(),
            Self::Keyboard(key) => rl.is_key_down(*key),
            Self::Mouse(button) => rl.is_mouse_button_down(*button),
            Self::All(items) => items.iter_mut().any(|x| x.is_active(rl)),
            Self::Any(items) => items.iter_mut().all(|x| x.is_active(rl)),
            Self::Not(item) => !item.is_active(rl),
        }
    }

    #[inline]
    pub fn is_starting(&mut self, rl: &RaylibHandle) -> bool {
        match self {
            Self::Constant(event) => event.is_starting(),
            Self::Keyboard(key) => rl.is_key_pressed(*key),
            Self::Mouse(button) => rl.is_mouse_button_pressed(*button),
            Self::All(items) => items.iter_mut().any(|x| x.is_starting(rl)),
            Self::Any(items) => {
                items.iter_mut().any(|x| x.is_starting(rl))
                    && items.iter_mut().all(|x| x.is_active(rl))
            }
            Self::Not(item) => !item.is_starting(rl),
        }
    }

    #[inline]
    pub fn is_ending(&mut self, rl: &RaylibHandle) -> bool {
        match self {
            Self::Constant(event) => event.is_ending(),
            Self::Keyboard(key) => rl.is_key_released(*key),
            Self::Mouse(button) => rl.is_mouse_button_released(*button),
            Self::All(items) => {
                items.iter_mut().any(|x| x.is_ending(rl))
                    && items.iter_mut().all(
                        |x| !x.is_active(rl), // assumes that if an item is ending, it is also inactive
                    )
            }
            Self::Any(items) => {
                items.iter_mut().any(|x| x.is_ending(rl))
                    && items.iter_mut().all(|x| x.is_active(rl) || x.is_ending(rl))
            }
            Self::Not(item) => !item.is_ending(rl),
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
    Constant(Vector2),
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
