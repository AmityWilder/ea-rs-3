use raylib::prelude::*;
use rl_input::{AxisSource, EventSource, VectorSource};

#[derive(Debug, Clone)]
pub struct Bindings {
    pub primary: EventSource,
    pub secondary: EventSource,
    pub alternate: EventSource,
    pub parallel: EventSource,
    pub zoom: AxisSource,
    pub cursor: VectorSource,
    pub pan: VectorSource,
}

impl Default for Bindings {
    fn default() -> Self {
        use KeyboardKey::*;
        use MouseButton::*;
        Self {
            primary: EventSource::Mouse(MOUSE_BUTTON_LEFT),
            secondary: EventSource::Mouse(MOUSE_BUTTON_RIGHT),
            alternate: EventSource::Keyboard(KEY_LEFT_CONTROL),
            parallel: EventSource::Keyboard(KEY_LEFT_SHIFT),
            zoom: AxisSource::MouseWheelMove,
            cursor: VectorSource::MousePosition,
            pan: VectorSource::EventMix(Box::from([
                (
                    EventSource::Keyboard(KEY_D),
                    VectorSource::Constant(rvec2(1, 0)),
                ),
                (
                    EventSource::Keyboard(KEY_A),
                    VectorSource::Constant(rvec2(-1, 0)),
                ),
                (
                    EventSource::Keyboard(KEY_W),
                    VectorSource::Constant(rvec2(0, -1)),
                ),
                (
                    EventSource::Keyboard(KEY_S),
                    VectorSource::Constant(rvec2(0, 1)),
                ),
            ])),
        }
    }
}
