use crate::{graph::node::GateId, tool::ToolId, ui::Visibility};
use raylib::prelude::*;
use rl_input::{
    AxisSource, BoolSource, Event, EventCombo, EventSource, SelectorItem, SelectorSource, Source,
    VectorSource,
};
use serde_derive::{Deserialize, Serialize};

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
    pub or_gate_hotkey: Event,
    pub and_gate_hotkey: Event,
    pub nor_gate_hotkey: Event,
    pub xor_gate_hotkey: Event,
    pub resistor_gate_hotkey: Event,
    pub capacitor_gate_hotkey: Event,
    pub led_gate_hotkey: Event,
    pub delay_gate_hotkey: Event,
    pub battery_gate_hotkey: Event,
    pub create_tool_hotkey: Event,
    pub erase_tool_hotkey: Event,
    pub edit_tool_hotkey: Event,
    pub interact_tool_hotkey: Event,
    pub hide_toolpane: Event,
    pub collapse_toolpane: Event,
    pub expand_toolpane: Event,
}

impl Inputs {
    pub fn gate(&self) -> Option<GateId> {
        [
            (self.or_gate_hotkey, GateId::Or),
            (self.and_gate_hotkey, GateId::And),
            (self.nor_gate_hotkey, GateId::Nor),
            (self.xor_gate_hotkey, GateId::Xor),
            (self.resistor_gate_hotkey, GateId::Resistor),
            (self.capacitor_gate_hotkey, GateId::Capacitor),
            (self.led_gate_hotkey, GateId::Led),
            (self.delay_gate_hotkey, GateId::Delay),
            (self.battery_gate_hotkey, GateId::Battery),
        ]
        .iter()
        .find(|(src, _)| src.is_starting())
        .map(|(_, gate)| *gate)
    }

    pub fn tool(&self) -> Option<ToolId> {
        [
            (self.create_tool_hotkey, ToolId::Create),
            (self.erase_tool_hotkey, ToolId::Erase),
            (self.edit_tool_hotkey, ToolId::Edit),
            (self.interact_tool_hotkey, ToolId::Interact),
        ]
        .iter()
        .find(|(src, _)| src.is_starting())
        .map(|(_, tool)| *tool)
    }

    pub fn toolpane_vis(&self) -> Option<Visibility> {
        [
            (self.hide_toolpane, Visibility::Hidden),
            (self.collapse_toolpane, Visibility::Collapsed),
            (self.expand_toolpane, Visibility::Expanded),
        ]
        .iter()
        .find(|(src, _)| src.is_starting())
        .map(|(_, vis)| *vis)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bindings {
    pub primary: EventSource,
    pub secondary: EventSource,
    pub alternate: EventSource,
    pub parallel: EventSource,
    pub zoom: AxisSource,
    pub scroll_console: AxisSource,
    pub cursor: VectorSource,
    pub pan: VectorSource,
    pub or_gate_hotkey: EventSource,
    pub and_gate_hotkey: EventSource,
    pub nor_gate_hotkey: EventSource,
    pub xor_gate_hotkey: EventSource,
    pub resistor_gate_hotkey: EventSource,
    pub capacitor_gate_hotkey: EventSource,
    pub led_gate_hotkey: EventSource,
    pub delay_gate_hotkey: EventSource,
    pub battery_gate_hotkey: EventSource,
    pub create_tool_hotkey: EventSource,
    pub erase_tool_hotkey: EventSource,
    pub edit_tool_hotkey: EventSource,
    pub interact_tool_hotkey: EventSource,
    pub hide_toolpane: EventSource,
    pub collapse_toolpane: EventSource,
    pub expand_toolpane: EventSource,
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
                SelectorItem {
                    src: BoolSource::Event {
                        what: EventSource::Keyboard(KEY_D),
                        when: Event::Active,
                    },
                    val: VectorSource::Constant(rvec2(1, 0)),
                },
                SelectorItem {
                    src: BoolSource::Event {
                        what: EventSource::Keyboard(KEY_A),
                        when: Event::Active,
                    },
                    val: VectorSource::Constant(rvec2(-1, 0)),
                },
                SelectorItem {
                    src: BoolSource::Event {
                        what: EventSource::Keyboard(KEY_W),
                        when: Event::Active,
                    },
                    val: VectorSource::Constant(rvec2(0, -1)),
                },
                SelectorItem {
                    src: BoolSource::Event {
                        what: EventSource::Keyboard(KEY_S),
                        when: Event::Active,
                    },
                    val: VectorSource::Constant(rvec2(0, 1)),
                },
            ])),
            or_gate_hotkey: EventSource::Keyboard(KEY_ONE),
            and_gate_hotkey: EventSource::Keyboard(KEY_TWO),
            nor_gate_hotkey: EventSource::Keyboard(KEY_THREE),
            xor_gate_hotkey: EventSource::Keyboard(KEY_FOUR),
            resistor_gate_hotkey: EventSource::Keyboard(KEY_FIVE),
            capacitor_gate_hotkey: EventSource::Keyboard(KEY_SIX),
            led_gate_hotkey: EventSource::Keyboard(KEY_SEVEN),
            delay_gate_hotkey: EventSource::Keyboard(KEY_EIGHT),
            battery_gate_hotkey: EventSource::Keyboard(KEY_NINE),
            create_tool_hotkey: EventSource::Keyboard(KEY_B),
            erase_tool_hotkey: EventSource::Keyboard(KEY_X),
            edit_tool_hotkey: EventSource::Keyboard(KEY_V),
            interact_tool_hotkey: EventSource::Keyboard(KEY_F),
            hide_toolpane: EventSource::Combo(EventCombo::All(Box::from([
                EventSource::Combo(EventCombo::Any(Box::from([
                    EventSource::Keyboard(KEY_LEFT_CONTROL),
                    EventSource::Keyboard(KEY_RIGHT_CONTROL),
                ]))),
                EventSource::Keyboard(KEY_B),
            ]))),
            collapse_toolpane: EventSource::Combo(EventCombo::All(Box::from([
                EventSource::Combo(EventCombo::Any(Box::from([
                    EventSource::Keyboard(KEY_LEFT_CONTROL),
                    EventSource::Keyboard(KEY_RIGHT_CONTROL),
                ]))),
                EventSource::Keyboard(KEY_B),
            ]))),
            expand_toolpane: EventSource::Combo(EventCombo::All(Box::from([
                EventSource::Combo(EventCombo::Any(Box::from([
                    EventSource::Keyboard(KEY_LEFT_CONTROL),
                    EventSource::Keyboard(KEY_RIGHT_CONTROL),
                ]))),
                EventSource::Keyboard(KEY_B),
            ]))),
        }
    }
}

impl Bindings {
    pub fn get_all(&mut self, rl: &RaylibHandle) -> Inputs {
        Inputs {
            primary: self.primary.get(rl),
            secondary: self.secondary.get(rl),
            alternate: self.alternate.get(rl),
            parallel: self.parallel.get(rl),
            zoom: self.zoom.get(rl),
            scroll_console: self.scroll_console.get(rl),
            cursor: self.cursor.get(rl),
            pan: self.pan.get(rl),
            or_gate_hotkey: self.or_gate_hotkey.get(rl),
            and_gate_hotkey: self.and_gate_hotkey.get(rl),
            nor_gate_hotkey: self.nor_gate_hotkey.get(rl),
            xor_gate_hotkey: self.xor_gate_hotkey.get(rl),
            resistor_gate_hotkey: self.resistor_gate_hotkey.get(rl),
            capacitor_gate_hotkey: self.capacitor_gate_hotkey.get(rl),
            led_gate_hotkey: self.led_gate_hotkey.get(rl),
            delay_gate_hotkey: self.delay_gate_hotkey.get(rl),
            battery_gate_hotkey: self.battery_gate_hotkey.get(rl),
            create_tool_hotkey: self.create_tool_hotkey.get(rl),
            erase_tool_hotkey: self.erase_tool_hotkey.get(rl),
            edit_tool_hotkey: self.edit_tool_hotkey.get(rl),
            interact_tool_hotkey: self.interact_tool_hotkey.get(rl),
            hide_toolpane: self.hide_toolpane.get(rl),
            collapse_toolpane: self.collapse_toolpane.get(rl),
            expand_toolpane: self.expand_toolpane.get(rl),
        }
    }
}
