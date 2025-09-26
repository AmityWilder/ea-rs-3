use raylib::prelude::*;

#[derive(Debug, Clone)]
pub enum EventBinding {
    Keyboard(KeyboardKey),
    Mouse(MouseButton),
}

impl EventBinding {
    pub fn is_active(&self, rl: &RaylibHandle) -> bool {
        match self {
            Self::Keyboard(key) => rl.is_key_down(*key),
            Self::Mouse(button) => rl.is_mouse_button_down(*button),
        }
    }

    pub fn is_starting(&self, rl: &RaylibHandle) -> bool {
        match self {
            Self::Keyboard(key) => rl.is_key_pressed(*key),
            Self::Mouse(button) => rl.is_mouse_button_pressed(*button),
        }
    }

    pub fn is_ending(&self, rl: &RaylibHandle) -> bool {
        match self {
            Self::Keyboard(key) => rl.is_key_released(*key),
            Self::Mouse(button) => rl.is_mouse_button_released(*button),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AxisBinding {
    MouseWheelMove,
}

impl AxisBinding {
    pub fn get(&self, rl: &RaylibHandle) -> f32 {
        match self {
            Self::MouseWheelMove => rl.get_mouse_wheel_move(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum VectorBinding {
    MousePosition,
    MouseDelta,
}

impl VectorBinding {
    pub fn get(&self, rl: &RaylibHandle) -> Vector2 {
        match self {
            Self::MousePosition => rl.get_mouse_position(),
            Self::MouseDelta => rl.get_mouse_delta(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Bindings {
    pub primary: EventBinding,
    pub secondary: EventBinding,
    pub alternate: EventBinding,
    pub parallel: EventBinding,
    pub zoom: AxisBinding,
    pub cursor: VectorBinding,
    pub pan_left: EventBinding,
    pub pan_right: EventBinding,
    pub pan_up: EventBinding,
    pub pan_down: EventBinding,
}

impl Default for Bindings {
    fn default() -> Self {
        Self {
            primary: EventBinding::Mouse(MouseButton::MOUSE_BUTTON_LEFT),
            secondary: EventBinding::Mouse(MouseButton::MOUSE_BUTTON_RIGHT),
            alternate: EventBinding::Keyboard(KeyboardKey::KEY_LEFT_CONTROL),
            parallel: EventBinding::Keyboard(KeyboardKey::KEY_LEFT_SHIFT),
            zoom: AxisBinding::MouseWheelMove,
            cursor: VectorBinding::MousePosition,
            pan_left: EventBinding::Keyboard(KeyboardKey::KEY_A),
            pan_right: EventBinding::Keyboard(KeyboardKey::KEY_D),
            pan_up: EventBinding::Keyboard(KeyboardKey::KEY_W),
            pan_down: EventBinding::Keyboard(KeyboardKey::KEY_S),
        }
    }
}
