use crate::{
    console::Console,
    input::Bindings,
    ivec::{IBounds, IRect, IVec2},
    tab::{EditorTab, Tab, TabList},
    theme::{ColorId, Theme},
};
use raylib::prelude::*;
use rl_input::Event;

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
    let binds = Bindings::default();

    let mut tabs = TabList::from([Tab::Editor(
        EditorTab::new(
            &mut rl,
            &thread,
            IBounds::new(IVec2::zero(), IVec2::new(1280, 720)),
        )
        .unwrap(),
    )]);

    let mut console = Console::new(
        4096,
        IBounds::new(IVec2::new(0, 520), IVec2::new(1280, 720)),
        true,
        false,
        true,
        true,
    );

    let mut hovering_console_top = Event::Inactive;
    let mut dragging_console_top = Event::Inactive;

    log!(
        console,
        (theme.caution, "squeak squeak\n:3"),
        (theme.input, " ee"),
        (ColorId::Input, "ee!\n"),
        (theme.special, "^w^"),
    )
    .unwrap();

    while !rl.window_should_close() {
        // Tick

        hovering_console_top.step();
        dragging_console_top.step();

        let input = binds.get_all(&rl);

        if rl.is_window_resized() {
            let window_width = rl.get_screen_width();
            let window_height = rl.get_screen_height();
            if console.right_anchored {
                if console.left_anchored {
                    console.bounds.max.x = window_width;
                } else {
                    let width = console.bounds.max.x - console.bounds.min.x;
                    console.bounds.min.x = window_width - width;
                    console.bounds.max.x = window_width;
                }
            }
            if console.bottom_anchored {
                if console.top_anchored {
                    console.bounds.max.y = window_width;
                } else {
                    let height = console.bounds.max.y - console.bounds.min.y;
                    console.bounds.min.y = window_height - height;
                    console.bounds.max.y = window_height;
                }
            }
        }

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

                    tab.zoom(input.zoom);
                }
            }
        }

        for tab in &mut tabs {
            match tab {
                Tab::Editor(tab) => tab.refresh_grid(&mut rl, &thread, &theme),
            }
        }

        // TODO: does it make more sense to have dedicated inputs for this?
        if (console.bounds.min.y..console.bounds.min.y + 3).contains(&(input.cursor.y as i32)) {
            hovering_console_top.activate();
            if input.primary.is_starting() {
                dragging_console_top.activate();
            }
        } else if dragging_console_top.is_inactive() {
            hovering_console_top.deactivate();
        }
        if dragging_console_top.is_active() && input.primary.is_ending() {
            dragging_console_top.deactivate();
        }
        if dragging_console_top.is_active() {
            console.bounds.min.y = input.cursor.y as i32;
        }

        if hovering_console_top == Event::Starting {
            rl.set_mouse_cursor(MouseCursor::MOUSE_CURSOR_RESIZE_NS);
        } else if hovering_console_top == Event::Ending {
            rl.set_mouse_cursor(MouseCursor::MOUSE_CURSOR_DEFAULT);
        }

        // Draw

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
            let IRect { x, y, w, h } = IRect::from(console.bounds);
            let mut d = d.begin_scissor_mode(x, y, w, h);
            d.clear_background(theme.background2);
            d.draw_rectangle(x + 2, y + 2, w - 4, h - 4, theme.background1);
            let mut x = x + 5 + 10;
            let mut y = y + 5;
            let left = x;
            for (color, text) in console.content() {
                d.draw_text(text, x, y, 10, color.get(&theme));
                if text.contains('\n') {
                    y += i32::try_from((text.split('\n').count() - 1) * 12).unwrap();
                    x = left + d.measure_text(text.split('\n').next_back().unwrap(), 10) + 1;
                } else {
                    x += d.measure_text(text, 10) + 1;
                }
            }
        }

        d.draw_text(
            &format!(
                "\
                input.primary:\n  {:?}\n  {:?}\n\
                input.secondary:\n  {:?}\n  {:?}\n\
                input.alternate:\n  {:?}\n  {:?}\n\
                input.parallel:\n  {:?}\n  {:?}\n\
                input.zoom:\n  {:?}\n  {:?}\n\
                input.cursor:\n  {:?}\n  {:?}\n\
                input.pan:\n  {:?}\n  {:?}\
                ",
                &binds.primary,
                input.primary,
                &binds.secondary,
                input.secondary,
                &binds.alternate,
                input.alternate,
                &binds.parallel,
                input.parallel,
                &binds.zoom,
                input.zoom,
                &binds.cursor,
                input.cursor,
                &binds.pan,
                input.pan,
            ),
            5,
            5,
            10,
            Color::MAGENTA,
        );
    }
}
