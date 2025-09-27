use raylib::prelude::*;

#[derive(Debug, Clone)]
pub enum Combo<T> {
    /// Equivalent to `Or`
    Add(Box<[T]>),
    /// Equivalent to `And`
    Mul(Box<[T]>),
    /// Equivalent to `Not`
    Neg(Box<T>),
}

#[derive(Debug, Clone)]
pub enum EventSource {
    Always,
    Keyboard(KeyboardKey),
    Mouse(MouseButton),
    Combo(Combo<Self>),
}

impl EventSource {
    pub fn is_active(&self, rl: &RaylibHandle) -> bool {
        match self {
            Self::Always => true,
            Self::Keyboard(key) => rl.is_key_down(*key),
            Self::Mouse(button) => rl.is_mouse_button_down(*button),
            Self::Combo(Combo::Add(items)) => items.iter().any(|x| x.is_active(rl)),
            Self::Combo(Combo::Mul(items)) => items.iter().all(|x| x.is_active(rl)),
            Self::Combo(Combo::Neg(item)) => !item.is_active(rl),
        }
    }

    pub fn is_starting(&self, rl: &RaylibHandle) -> bool {
        match self {
            Self::Always => false,
            Self::Keyboard(key) => rl.is_key_pressed(*key),
            Self::Mouse(button) => rl.is_mouse_button_pressed(*button),
            Self::Combo(Combo::Add(items)) => items.iter().any(|x| x.is_starting(rl)),
            Self::Combo(Combo::Mul(items)) => {
                items.iter().any(|x| x.is_starting(rl)) && items.iter().all(|x| x.is_active(rl))
            }
            Self::Combo(Combo::Neg(item)) => !item.is_starting(rl),
        }
    }

    pub fn is_ending(&self, rl: &RaylibHandle) -> bool {
        match self {
            Self::Always => false,
            Self::Keyboard(key) => rl.is_key_released(*key),
            Self::Mouse(button) => rl.is_mouse_button_released(*button),
            Self::Combo(Combo::Add(items)) => {
                items.iter().any(|x| x.is_ending(rl))
                    && items.iter().all(
                        |x| !x.is_active(rl), // assumes that if an item is ending, it is also inactive
                    )
            }
            Self::Combo(Combo::Mul(items)) => {
                items.iter().any(|x| x.is_ending(rl))
                    && items.iter().all(|x| x.is_active(rl) || x.is_ending(rl))
            }
            Self::Combo(Combo::Neg(item)) => !item.is_ending(rl),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AxisSource {
    Constant(f32),
    MouseWheelMove,
    EventMix(Box<[(EventSource, AxisSource)]>),
    Combo(Combo<Self>),
}

impl AxisSource {
    pub fn get(&self, rl: &RaylibHandle) -> f32 {
        match self {
            Self::Constant(x) => *x,
            Self::MouseWheelMove => rl.get_mouse_wheel_move(),
            Self::EventMix(items) => items
                .iter()
                .filter(|(src, _)| src.is_active(rl))
                .map(|(_, val)| val.get(rl))
                .sum(),
            Self::Combo(Combo::Add(items)) => items.iter().map(|x| x.get(rl)).sum(),
            Self::Combo(Combo::Mul(items)) => items.iter().map(|x| x.get(rl)).product(),
            Self::Combo(Combo::Neg(item)) => -item.get(rl),
        }
    }
}

#[derive(Debug, Clone)]
pub enum VectorSource {
    Constant(Vector2),
    MousePosition,
    MouseDelta,
    EventMix(Box<[(EventSource, VectorSource)]>),
    AxisXY { x: AxisSource, y: AxisSource },
    Combo(Combo<Self>),
}

impl VectorSource {
    pub fn get(&self, rl: &RaylibHandle) -> Vector2 {
        match self {
            Self::Constant(v) => *v,
            Self::MousePosition => rl.get_mouse_position(),
            Self::MouseDelta => rl.get_mouse_delta(),
            Self::EventMix(items) => items
                .iter()
                .filter(|(src, _)| src.is_active(rl))
                .map(|(_, val)| val.get(rl))
                .reduce(|a, b| a + b)
                .unwrap_or(Vector2::zero()),
            Self::AxisXY { x, y } => Vector2::new(x.get(rl), y.get(rl)),
            Self::Combo(Combo::Add(items)) => items
                .iter()
                .map(|x| x.get(rl))
                .reduce(|a, b| a + b)
                .unwrap_or(Vector2::zero()),
            Self::Combo(Combo::Mul(items)) => items
                .iter()
                .map(|x| x.get(rl))
                .reduce(|a, b| a * b)
                .unwrap_or(Vector2::zero()),
            Self::Combo(Combo::Neg(item)) => -item.get(rl),
        }
    }
}
