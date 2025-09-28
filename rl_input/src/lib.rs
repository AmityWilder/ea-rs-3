use raylib::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
}

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
    Constant(Event),
    Keyboard(KeyboardKey),
    Mouse(MouseButton),
    Combo(Combo<Self>),
}

impl EventSource {
    #[inline]
    pub fn is_active(&self, rl: &RaylibHandle) -> bool {
        match self {
            Self::Constant(event) => event.is_active(),
            Self::Keyboard(key) => rl.is_key_down(*key),
            Self::Mouse(button) => rl.is_mouse_button_down(*button),
            Self::Combo(Combo::Add(items)) => items.iter().any(|x| x.is_active(rl)),
            Self::Combo(Combo::Mul(items)) => items.iter().all(|x| x.is_active(rl)),
            Self::Combo(Combo::Neg(item)) => !item.is_active(rl),
        }
    }

    #[inline]
    pub fn is_starting(&self, rl: &RaylibHandle) -> bool {
        match self {
            Self::Constant(event) => event.is_starting(),
            Self::Keyboard(key) => rl.is_key_pressed(*key),
            Self::Mouse(button) => rl.is_mouse_button_pressed(*button),
            Self::Combo(Combo::Add(items)) => items.iter().any(|x| x.is_starting(rl)),
            Self::Combo(Combo::Mul(items)) => {
                items.iter().any(|x| x.is_starting(rl)) && items.iter().all(|x| x.is_active(rl))
            }
            Self::Combo(Combo::Neg(item)) => !item.is_starting(rl),
        }
    }

    #[inline]
    pub fn is_ending(&self, rl: &RaylibHandle) -> bool {
        match self {
            Self::Constant(event) => event.is_ending(),
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

    /// Prefer calling [`Self::is_active`], [`Self::is_starting`], or [`Self::is_ending`] if you only need one
    #[inline]
    pub fn get(&self, rl: &RaylibHandle) -> Event {
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

#[derive(Debug, Clone)]
pub struct SelectorSource<T>(pub Box<[(EventSource, T)]>);

impl<T, U: Into<Box<[(EventSource, T)]>>> From<U> for SelectorSource<T> {
    fn from(value: U) -> Self {
        Self(value.into())
    }
}

impl<T> SelectorSource<T> {
    pub fn get<'a, F>(&'a self, rl: &RaylibHandle, mut f: F) -> impl Iterator<Item = &'a T>
    where
        F: FnMut(&EventSource, &RaylibHandle) -> bool,
    {
        self.0
            .iter()
            .filter(move |(src, _)| f(src, rl))
            .map(|(_, val)| val)
    }

    pub fn get_active<'a>(&'a self, rl: &RaylibHandle) -> impl Iterator<Item = &'a T> {
        self.get(rl, EventSource::is_active)
    }

    pub fn get_starting<'a>(&'a self, rl: &RaylibHandle) -> impl Iterator<Item = &'a T> {
        self.get(rl, EventSource::is_starting)
    }
}

#[derive(Debug, Clone)]
pub enum AxisSource {
    Constant(f32),
    MouseWheelMove,
    EventMix(SelectorSource<AxisSource>),
    Combo(Combo<Self>),
}

impl AxisSource {
    #[inline]
    pub fn get(&self, rl: &RaylibHandle) -> f32 {
        match self {
            Self::Constant(x) => *x,
            Self::MouseWheelMove => rl.get_mouse_wheel_move(),
            Self::EventMix(items) => items.get_active(rl).map(|src| src.get(rl)).sum(),
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
    EventMix(SelectorSource<VectorSource>),
    AxisXY { x: AxisSource, y: AxisSource },
    Combo(Combo<Self>),
}

impl VectorSource {
    #[inline]
    pub fn get(&self, rl: &RaylibHandle) -> Vector2 {
        match self {
            Self::Constant(v) => *v,
            Self::MousePosition => rl.get_mouse_position(),
            Self::MouseDelta => rl.get_mouse_delta(),
            Self::EventMix(items) => items
                .get_active(rl)
                .map(|src| src.get(rl))
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
