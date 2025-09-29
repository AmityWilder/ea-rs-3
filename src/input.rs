use crate::{graph::node::Gate, tool::ToolId};
use raylib::prelude::*;
use rl_input::{AxisSource, Event, EventSource, SelectorSource, VectorSource};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Inputs {
    pub primary: Event,
    pub secondary: Event,
    pub alternate: Event,
    pub parallel: Event,
    pub zoom: f32,
    pub scroll_console: f32,
    pub cursor: Vector2,
    pub pan: Vector2,
    pub gate_hotkey: Option<Gate>,
    pub tool_hotkey: Option<ToolId>,
}

#[derive(Debug, Clone)]
pub struct Bindings {
    pub primary: EventSource,
    pub secondary: EventSource,
    pub alternate: EventSource,
    pub parallel: EventSource,
    pub zoom: AxisSource,
    pub scroll_console: AxisSource,
    pub cursor: VectorSource,
    pub pan: VectorSource,
    pub gate_hotkey: SelectorSource<Gate>,
    pub tool_hotkey: SelectorSource<ToolId>,
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
            scroll_console: AxisSource::MouseWheelMove,
            cursor: VectorSource::MousePosition,
            pan: VectorSource::EventMix(SelectorSource::from([
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
            gate_hotkey: SelectorSource::from([
                (EventSource::Keyboard(KEY_ONE), Gate::Or),
                (EventSource::Keyboard(KEY_TWO), Gate::And),
                (EventSource::Keyboard(KEY_THREE), Gate::Nor),
                (EventSource::Keyboard(KEY_FOUR), Gate::Xor),
                (EventSource::Keyboard(KEY_FIVE), Gate::Resistor {}),
                (EventSource::Keyboard(KEY_SIX), Gate::Capacitor {}),
                (EventSource::Keyboard(KEY_SEVEN), Gate::Led {}),
                (EventSource::Keyboard(KEY_EIGHT), Gate::Delay {}),
                (EventSource::Keyboard(KEY_NINE), Gate::Battery),
            ]),
            tool_hotkey: SelectorSource::from([
                (EventSource::Keyboard(KEY_B), ToolId::Create),
                (EventSource::Keyboard(KEY_V), ToolId::Edit),
                (EventSource::Keyboard(KEY_X), ToolId::Erase),
            ]),
        }
    }
}

impl Bindings {
    pub fn get_all(&self, rl: &RaylibHandle) -> Inputs {
        Inputs {
            primary: self.primary.get(rl),
            secondary: self.secondary.get(rl),
            alternate: self.alternate.get(rl),
            parallel: self.parallel.get(rl),
            zoom: self.zoom.get(rl),
            scroll_console: self.scroll_console.get(rl),
            cursor: self.cursor.get(rl),
            pan: self.pan.get(rl),
            gate_hotkey: self.gate_hotkey.get_starting(rl).next().copied(),
            tool_hotkey: self.tool_hotkey.get_starting(rl).next().copied(),
        }
    }
}
