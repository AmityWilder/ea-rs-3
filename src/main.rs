use std::collections::LinkedList;

use raylib::prelude::*;

#[derive(Debug, Clone)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub warning: Color,
    pub danger: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: Color::BLACK,
            foreground: Color::GRAY,
            accent: Color::BLUE,
            warning: Color::ORANGE,
            danger: Color::RED,
        }
    }
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub enum Combinator {
//     /// Also represents `Or`
//     Add { dst: u8, a: u8, b: u8 },
//     /// Also represents `And`
//     Mul { dst: u8, a: u8, b: u8 },
//     /// Also represents `Not`
//     Neg { dst: u8, x: u8 },
// }

// impl Combinator {
//     fn dst(&self) -> u8 {
//         use Combinator::*;
//         match self {
//             Add { dst, .. } | Mul { dst, .. } | Neg { dst, .. } => *dst,
//         }
//     }

//     fn max_src(&self) -> u8 {
//         use Combinator::*;
//         match self {
//             Add { a, b, .. } | Mul { a, b, .. } => (*a).max(*b),
//             Neg { x, .. } => *x,
//         }
//     }

//     fn precedence(&self) -> u8 {
//         use Combinator::*;
//         match self {
//             Add { .. } => 2,
//             Mul { .. } => 1,
//             Neg { .. } => 0,
//         }
//     }
// }

// impl PartialOrd for Combinator {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         Some(self.cmp(other))
//     }
// }

// impl Ord for Combinator {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         match self.dst().cmp(&other.dst()) {
//             std::cmp::Ordering::Equal => {}
//             ord => return ord,
//         }
//         match self.max_src().cmp(&other.max_src()) {
//             std::cmp::Ordering::Equal => {}
//             ord => return ord,
//         }
//         self.precedence().cmp(&other.precedence())
//     }
// }

// pub struct Combination<T> {
//     items: Vec<(T, u8)>,
//     logic: Vec<Combinator>,
// }

// impl<T> Combination<T> {
//     fn sort_logic(&mut self) {
//         self.logic.sort();
//     }
// }

// impl Combination<EventBinding> {
//     fn combine(&self, mut state: [bool; 255]) -> bool {
//         let mut last_value = false;
//         for cmb in &self.logic {
//             let dst;
//             (dst, last_value) = match *cmb {
//                 Combinator::Add { dst, a, b } => (dst, state[a as usize] || state[b as usize]),
//                 Combinator::Mul { dst, a, b } => (dst, state[a as usize] && state[b as usize]),
//                 Combinator::Neg { dst, x } => (dst, !state[x as usize]),
//             };
//             state[dst as usize] = last_value;
//         }
//         last_value
//     }

//     pub fn is_active(&self, rl: &RaylibHandle) -> bool {
//         let mut state = [false; 255];
//         for (item, dst) in &self.items {
//             state[*dst as usize] = item.is_active(rl);
//         }
//         self.combine(state)
//     }

//     pub fn is_starting(&self, rl: &RaylibHandle) -> bool {}

//     pub fn is_ending(&self, rl: &RaylibHandle) -> bool {}
// }

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IVec2 {
    pub x: i32,
    pub y: i32,
}

impl IVec2 {
    pub const fn as_vec2(self) -> Vector2 {
        Vector2 {
            x: self.x as f32,
            y: self.y as f32,
        }
    }

    pub const fn from_vec2(value: Vector2) -> Self {
        Self {
            x: value.x as i32,
            y: value.y as i32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl IRect {
    pub fn as_rect(&self) -> Rectangle {
        Rectangle {
            x: self.x as f32,
            y: self.y as f32,
            width: self.w as f32,
            height: self.h as f32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IBounds {
    pub min: IVec2,
    pub max: IVec2,
}

impl From<IBounds> for IRect {
    fn from(value: IBounds) -> Self {
        IRect {
            x: value.min.x,
            y: value.min.y,
            w: value.max.x - value.min.x,
            h: value.max.y - value.min.y,
        }
    }
}

impl From<IRect> for IBounds {
    fn from(value: IRect) -> Self {
        IBounds {
            min: IVec2 {
                x: value.x,
                y: value.y,
            },
            max: IVec2 {
                x: value.x + value.w,
                y: value.y + value.h,
            },
        }
    }
}

#[derive(Debug)]
pub struct Tab {
    pub camera_target: Vector2,
    pub zoom_exp: f32,
    pub bounds: IBounds,
}

impl Tab {
    pub fn camera(&self) -> Camera2D {
        Camera2D {
            offset: Vector2::zero(),
            target: self.camera_target,
            rotation: 0.0,
            zoom: 2.0f32.powf(self.zoom_exp),
        }
    }
}

pub const GRID_SIZE: u8 = 8;

fn main() {
    let (mut rl, thread) = init().title("Electron Architect").resizable().build();

    rl.set_target_fps(
        get_monitor_refresh_rate(get_current_monitor())
            .try_into()
            .unwrap(),
    );

    let theme = Theme::default();
    let bindings = Bindings::default();
    let mut tabs = vec![Tab {
        camera_target: Vector2::zero(),
        zoom_exp: 0.0,
        bounds: IBounds {
            min: IVec2 { x: 0, y: 0 },
            max: IVec2 { x: 300, y: 300 },
        },
    }];
    let focused_tab = 0;

    while !rl.window_should_close() {
        assert!(!tabs.is_empty(), "tabs should never be empty");
        {
            let tab = &mut tabs[focused_tab];
            tab.zoom_exp = (tab.zoom_exp + bindings.zoom.get(&rl)).clamp(-3.0, 3.0);
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(theme.background);

        for tab in &tabs {
            let IRect { x, y, w, h } = tab.bounds.into();
            let camera = tab.camera();
            let start = IVec2::from_vec2(d.get_screen_to_world2D(tab.bounds.min.as_vec2(), camera));
            let end = IVec2::from_vec2(d.get_screen_to_world2D(tab.bounds.max.as_vec2(), camera));
            let mut d = d.begin_scissor_mode(x, y, w, h);
            let mut d = d.begin_mode2D(tab.camera());
            let grid_color = theme.foreground.alpha(camera.zoom.clamp(0.0, 1.0));
            if camera.zoom.recip() >= GRID_SIZE as f32 {
                // size of 1 pixel is smaller than a grid
                d.draw_rectangle(
                    start.x,
                    start.y,
                    end.x - start.x,
                    end.y - start.y,
                    grid_color,
                );
            } else {
                for y in (start.y..end.y).step_by(GRID_SIZE as usize) {
                    d.draw_line(start.x, y, end.x, y, grid_color);
                }
                for x in (start.x..end.x).step_by(GRID_SIZE as usize) {
                    d.draw_line(x, start.y, x, end.y, grid_color);
                }
            }
        }
    }
}
