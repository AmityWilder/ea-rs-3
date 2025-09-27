use std::sync::{Arc, RwLock};

use crate::{
    console::{Console, RichChunk},
    graph::{Gate, Graph},
    input::Bindings,
    ivec::{IBounds, IRect, IVec2},
    tab::{EditorTab, Tab, TabList},
    theme::{ColorId, Theme},
    tool::Tool,
    toolpane::{ToolPane, ToolPaneAnchoring},
};
use raylib::prelude::*;
use rl_input::Event;

mod console;
mod graph;
mod input;
mod ivec;
mod tab;
mod theme;
mod tool;
mod toolpane;

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

    let mut graphs = vec![Arc::new(RwLock::new(Graph::new()))];

    let mut tabs = TabList::from([Tab::Editor(
        EditorTab::new(
            &mut rl,
            &thread,
            IBounds::new(IVec2::zero(), IVec2::new(1280, 720)),
            Arc::downgrade(&graphs[0]),
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

    let mut toolpane = ToolPane::new(
        Tool::default(),
        Gate::default(),
        ToolPaneAnchoring::default(),
    );

    let mut hovering_console_top = Event::Inactive;
    let mut dragging_console_top = Event::Inactive;

    log!(
        console,
        rl,
        theme,
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

                    tab.zoom_and_pan(input.pan, input.zoom, 5.0);

                    if let Some(graph) = tab.graph.upgrade()
                        // if graph is being borrowed, don't edit it!
                        && let Ok(mut graph) = graph.try_write()
                    {
                        match toolpane.tool {
                            Tool::Create {} => {
                                if input.primary.is_starting() {
                                    graph.create_node(
                                        toolpane.gate,
                                        tab.screen_to_world(input.cursor).snap(GRID_SIZE.into()),
                                    );
                                }
                            }
                            Tool::Erase {} => todo!(),
                            Tool::Edit {} => todo!(),
                        }
                    }
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
            console.bounds.min.y = (input.cursor.y as i32).min(console.bounds.max.y - 20);
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
                    if let Some(graph) = tab.graph.upgrade() {
                        let graph = graph.read().unwrap();
                        for node in graph.nodes_iter() {
                            d.draw_circle(
                                node.position.x,
                                node.position.y,
                                f32::from(GRID_SIZE / 2),
                                if node.state() {
                                    theme.active
                                } else {
                                    theme.background3
                                },
                            );
                        }
                    }
                }
            }
        }

        // console
        {
            let IRect { x, y, w, h } = IRect::from(console.bounds);
            d.draw_rectangle(x, y, w, h, theme.background2);
            d.draw_rectangle(x + 1, y + 1, w - 2, h - 2, theme.background1);
            let mut x = x + 15;
            let mut y = y + 5;
            let mut d = d.begin_scissor_mode(x, y, w - 30, h - 10);
            for RichChunk {
                text,
                color,
                x_change,
                height,
                is_y_changing,
            } in console.content()
            {
                // assumes h is never negative and bounds.max >= bounds.min
                if y + h >= console.bounds.min.y + 5 && y <= console.bounds.max.y - 5 {
                    d.draw_text(text, x, y, theme.console_font_size, color.get(&theme));
                }
                if is_y_changing {
                    y += height
                }
                x += x_change;
            }
        }
    }
}
