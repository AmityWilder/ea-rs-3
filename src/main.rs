use crate::{
    input::Bindings,
    tab::{EditorTab, Tab, TabList},
    theme::Theme,
};
use raylib::prelude::*;

mod input;
mod tab;
mod theme;

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
pub struct Console {
    content: String,
    colors: Vec<(std::ops::Range<usize>, Color)>,
}

impl Console {
    pub fn text_blocks(
        &self,
    ) -> impl ExactSizeIterator<Item = (&str, Color)>
    + DoubleEndedIterator
    + Clone
    + std::iter::FusedIterator {
        self.colors
            .iter()
            .cloned()
            .map(|(range, color)| (&self.content[range], color))
    }
}

pub const GRID_SIZE: u8 = 8;

fn main() {
    let program_icon =
        Image::load_image_from_mem(".png", include_bytes!("../assets/program_icon32x.png")).ok();

    let (mut rl, thread) = init()
        .title("Electron Architect")
        .size(1280, 720)
        .resizable()
        .build();

    rl.set_target_fps(
        get_monitor_refresh_rate(get_current_monitor())
            .try_into()
            .unwrap(),
    );

    if let Some(icon) = program_icon.as_ref() {
        rl.set_window_icon(icon);
    }

    let theme = Theme::default();
    let bindings = Bindings::default();
    let mut tabs = TabList::from([Tab::Editor(
        EditorTab::new(
            &mut rl,
            &thread,
            IBounds {
                min: IVec2 { x: 0, y: 0 },
                max: IVec2 { x: 1280, y: 720 },
            },
        )
        .unwrap(),
    )]);

    while !rl.window_should_close() {
        if let Some(tab) = tabs.focused_tab_mut() {
            match tab {
                Tab::Editor(tab) => {
                    if rl.is_window_resized() {
                        let bounds = IBounds {
                            min: IVec2 { x: 0, y: 0 },
                            max: IVec2 {
                                x: rl.get_screen_width(),
                                y: rl.get_screen_height(),
                            },
                        };
                        tab.update_bounds(&mut rl, &thread, bounds).unwrap();
                    }

                    tab.zoom(bindings.zoom.get(&rl));
                }
            }
        }

        for tab in &mut tabs {
            match tab {
                Tab::Editor(tab) => tab.refresh_grid(&mut rl, &thread, &theme),
            }
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(theme.background);

        for tab in &tabs {
            match tab {
                Tab::Editor(tab) => {
                    let IRect { x, y, w, h } = IRect::from(*tab.bounds());
                    let mut d = d.begin_scissor_mode(x, y, w, h);
                    let tex = tab.grid_tex();
                    d.draw_texture_pro(
                        tex,
                        Rectangle::new(0.0, 0.0, tex.width() as f32, -tex.height() as f32),
                        Rectangle::new(0.0, 0.0, tex.width() as f32, tex.height() as f32),
                        Vector2::zero(),
                        0.0,
                        Color::WHITE,
                    );
                    let mut d = d.begin_mode2D(tab.camera());
                }
            }
        }
    }
}
