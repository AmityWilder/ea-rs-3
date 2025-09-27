use crate::{
    console::Console,
    input::Bindings,
    ivec::{IBounds, IRect, IVec2},
    tab::{EditorTab, Tab, TabList},
    theme::Theme,
};
use raylib::prelude::*;

mod console;
mod input;
mod ivec;
mod tab;
mod theme;

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
            IBounds::new(IVec2::zero(), IVec2::new(1280, 720)),
        )
        .unwrap(),
    )]);

    let mut console = Console::new(
        IBounds::new(IVec2::new(0, 520), IVec2::new(1280, 720)),
        4096,
    );

    log!(
        console,
        (theme.caution, "squeak squeak\n:3"),
        (theme.input, " ee"),
        (theme.input, "ee!\n"),
        (theme.special, "^w^"),
    )
    .unwrap();

    while !rl.window_should_close() {
        if let Some(tab) = tabs.focused_tab_mut() {
            match tab {
                Tab::Editor(tab) => {
                    if rl.is_window_resized() {
                        let bounds = IBounds::new(
                            IVec2::zero(),
                            IVec2::new(rl.get_screen_width(), rl.get_screen_height()),
                        );
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
                    d.draw_texture_pro(
                        tab.grid_tex(),
                        rrect(x, y, w, -h),
                        rrect(x, y, w, h),
                        Vector2::zero(),
                        0.0,
                        Color::WHITE,
                    );
                    let mut d = d.begin_mode2D(tab.camera());
                }
            }
        }

        // console
        {
            let IRect { x, y, w, h } = IRect::from(*console.bounds());
            let mut d = d.begin_scissor_mode(x, y, w, h);
            d.clear_background(theme.background2);
            let mut x = x + 5;
            let mut y = y + 5;
            let left = x;
            for (color, text) in console.content() {
                d.draw_text(text, x, y, 10, color);
                if text.contains('\n') {
                    y += i32::try_from((text.split('\n').count() - 1) * 12).unwrap();
                    x = left + d.measure_text(text.split('\n').next_back().unwrap(), 10) + 1;
                } else {
                    x += d.measure_text(text, 10) + 1;
                }
            }
        }
    }
}
