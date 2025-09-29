#![deny(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]

use crate::{
    console::{
        Console, ConsoleAnchoring, GraphRef, HyperRef, LogType, NodeRef, PositionRef, WireRef,
    },
    graph::{Graph, GraphList, node::Gate},
    icon_sheets::{ButtonIconSheets, NodeIconSheetId, NodeIconSheetSets},
    input::Bindings,
    ivec::{IBounds, IRect, IVec2},
    rich_text::ColorAct,
    tab::{EditorTab, Tab, TabList},
    theme::{ColorId, Theme},
    tool::Tool,
    toolpane::{ToolPane, ToolPaneAnchoring},
};
use raylib::prelude::*;
use rl_input::Event;
use std::sync::{Arc, RwLock};

mod console;
mod graph;
mod icon_sheets;
mod input;
mod ivec;
mod rich_text;
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

    rl.set_exit_key(None);

    if let Some(icon) = program_icon.as_ref() {
        rl.set_window_icon(icon);
    }

    let theme = Theme::default();
    let binds = Bindings::default();

    let button_icon_sheets = ButtonIconSheets::load(&mut rl, &thread).unwrap();
    let node_icon_sheets = NodeIconSheetSets::load(&mut rl, &thread).unwrap();

    let mut graphs = GraphList::new();

    let mut tabs = TabList::from([Tab::Editor(
        EditorTab::new(
            &mut rl,
            &thread,
            IBounds::new(IVec2::zero(), IVec2::new(1280, 720)),
            Arc::downgrade(graphs.create_graph()),
        )
        .unwrap(),
    )]);

    let mut console = Console::new(
        327_680, // 4096 rows with 80 columns
        IBounds::new(IVec2::new(0, 570), IVec2::new(1280, 720)),
        ConsoleAnchoring {
            left: true,
            top: false,
            right: true,
            bottom: true,
        },
    );

    let mut toolpane = ToolPane::new(
        Tool::default(),
        Gate::default(),
        ToolPaneAnchoring::default(),
    );

    let mut hovering_console_top = Event::Inactive;
    let mut dragging_console_top = Event::Inactive;

    logln!(console, LogType::Success, "initialized");

    while !rl.window_should_close() {
        // Tick

        hovering_console_top.step();
        dragging_console_top.step();

        let input = binds.get_all(&rl);

        if rl.is_window_resized() {
            let window_width = rl.get_screen_width();
            let window_height = rl.get_screen_height();
            if console.anchoring.right {
                if console.anchoring.left {
                    console.bounds.max.x = window_width;
                } else {
                    let width = console.bounds.max.x - console.bounds.min.x;
                    console.bounds.min.x = window_width - width;
                    console.bounds.max.x = window_width;
                }
            }
            if console.anchoring.bottom {
                if console.anchoring.top {
                    console.bounds.max.y = window_width;
                } else {
                    let height = console.bounds.max.y - console.bounds.min.y;
                    console.bounds.min.y = window_height - height;
                    console.bounds.max.y = window_height;
                }
            }
        }

        if console.bounds.contains(IVec2::from_vec2(input.cursor))
            || dragging_console_top.is_active()
        {
            console.bottom_offset = (console.bottom_offset + input.scroll_console as f64).clamp(
                0.0,
                console
                    .content_str()
                    .lines()
                    .count()
                    .saturating_sub(console.displayable_lines(&theme).try_into().unwrap())
                    as f64,
            );

            let mut x = console.bounds.min.x + theme.console_padding_left;
            let mut y = console.bounds.min.y + theme.console_padding_top;
            let left = x;
            for (_, text) in console.visible_content(&theme) {
                if IBounds::from(IRect::new(
                    x,
                    y,
                    rl.measure_text(text, theme.console_font_size),
                    theme.console_font_size,
                ))
                .contains(IVec2::from_vec2(input.cursor))
                    && let Ok(x) = text.parse::<HyperRef>()
                {
                    match x {
                        HyperRef::Position(PositionRef(pos)) => {
                            // println!("{pos:?}");
                        }
                        HyperRef::Graph(GraphRef(g)) => {
                            let borrow;
                            let graph = match graphs.get_by_id(g) {
                                Some(g) => {
                                    borrow = g.read().unwrap();
                                    Some(&*borrow)
                                }
                                None => None,
                            };
                            // println!("{graph:?}");
                        }
                        HyperRef::Node(NodeRef(g, n)) => {
                            let borrow;
                            let node = match graphs.get_by_id(g) {
                                Some(g) => {
                                    borrow = g.read().unwrap();
                                    borrow.get_node_by_id(n)
                                }
                                None => None,
                            };
                            // println!("{node:?}");
                        }
                        HyperRef::Wire(WireRef(g, w)) => {
                            let borrow;
                            let wire = match graphs.get_by_id(g) {
                                Some(g) => {
                                    borrow = g.read().unwrap();
                                    borrow.get_wire_by_id(w)
                                }
                                None => None,
                            };
                            // println!("{wire:?}");
                        }
                    }
                }
                if text.ends_with('\n') {
                    y += theme.console_line_height();
                    x = left;
                } else {
                    x += rl.measure_text(text, theme.console_font_size);
                }
            }
        } else if let Some(tab) = tabs.focused_tab_mut() {
            match tab {
                Tab::Editor(tab) => {
                    if let Some(gate) = input.gate_hotkey {
                        toolpane.gate = gate;
                    }

                    if rl.is_window_resized() {
                        let bounds = IBounds::new(
                            IVec2::zero(),
                            IVec2::new(rl.get_screen_width(), rl.get_screen_height()),
                        );
                        tab.update_bounds(&mut rl, &thread, bounds).unwrap();
                    }

                    tab.zoom_and_pan(input.pan, input.zoom, 5.0);

                    if let Some(graph) = tab.graph.upgrade()
                        // if graph is being borrowed, don't edit it! it might be saving!
                        && let Ok(mut graph) = graph.try_write()
                    {
                        match toolpane.tool {
                            Tool::Create {} => {
                                if input.primary.is_starting() {
                                    let pos =
                                        tab.screen_to_world(input.cursor).snap(GRID_SIZE.into());
                                    if let Some(_idx) = graph.find_node_at_pos(pos) {
                                        // TODO
                                    } else {
                                        let gate = toolpane.gate;
                                        let (_, node) = graph.create_node(gate, pos);
                                        let node_id = node.id();
                                        logln!(
                                            console,
                                            LogType::Info,
                                            "create {}[{gate}]{} node {} at {}",
                                            ColorAct::Push(ColorId::Special.into()),
                                            ColorAct::Pop,
                                            NodeRef(graph.id(), node_id),
                                            PositionRef(pos),
                                        );
                                    }
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
            console.bounds.min.y = (input.cursor.y as i32).clamp(
                theme.console_padding_top,
                console.bounds.max.y
                    - theme.console_padding_bottom
                    - theme.console_padding_bottom
                    - theme.console_font_size,
            );
        }

        if hovering_console_top == Event::Starting {
            rl.set_mouse_cursor(MouseCursor::MOUSE_CURSOR_RESIZE_NS);
        } else if hovering_console_top == Event::Ending {
            rl.set_mouse_cursor(MouseCursor::MOUSE_CURSOR_DEFAULT);
        }

        for mut graph in graphs.iter_mut().filter_map(|g| g.try_write().ok()) {
            graph.evaluate();
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
                            node_icon_sheets.draw(
                                &mut d,
                                tab.zoom_exp().floor() as i32,
                                NodeIconSheetId::Basic,
                                Rectangle {
                                    x: node.position.x as f32,
                                    y: node.position.y as f32,
                                    width: GRID_SIZE.into(),
                                    height: GRID_SIZE.into(),
                                },
                                node.gate,
                                Vector2::zero(),
                                0.0,
                                if node.state() {
                                    theme.active
                                } else {
                                    theme.foreground
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
            let mut d = d.begin_scissor_mode(x, y, w, h);

            // content
            {
                d.draw_rectangle(x, y, w, h, theme.background2);
                d.draw_rectangle(x + 1, y + 1, w - 2, h - 2, theme.background1);
                let mut d = d.begin_scissor_mode(
                    x + theme.console_padding_left,
                    y + theme.console_padding_top,
                    w - theme.console_padding_left - theme.console_padding_right,
                    h - theme.console_padding_top - theme.console_padding_bottom,
                );

                let mut x = x + theme.console_padding_left;
                let mut y = y + theme.console_padding_top;
                let left = x;
                for (color, text) in console.visible_content(&theme) {
                    let width = d.measure_text(text, theme.console_font_size);
                    if IBounds::from(IRect::new(x, y, width, theme.console_font_size))
                        .contains(IVec2::from_vec2(input.cursor))
                        && let Ok(hr) = text.parse::<HyperRef>()
                    {
                        d.draw_rectangle(
                            x,
                            y,
                            width,
                            theme.console_font_size,
                            theme.special.alpha(0.2),
                        );
                        match hr {
                            HyperRef::Position(PositionRef(pos)) => {
                                let mut d = d.begin_scissor_mode(
                                    0,
                                    0,
                                    d.get_screen_width(),
                                    d.get_screen_height(),
                                );
                                for tab in &tabs {
                                    match tab {
                                        Tab::Editor(tab) => {
                                            d.draw_line_v(
                                                input.cursor,
                                                d.get_world_to_screen2D(
                                                    pos.as_vec2()
                                                        + rvec2(GRID_SIZE / 2, GRID_SIZE / 2),
                                                    tab.camera(),
                                                ),
                                                theme.special,
                                            );
                                        }
                                    }
                                }
                            }
                            HyperRef::Graph(g) => {
                                // TODO
                            }
                            HyperRef::Node(NodeRef(g, n)) => {
                                if let Some(g) = graphs.get_by_id(g) {
                                    let borrow = g.read().unwrap();
                                    if let Some(node) = borrow.get_node_by_id(n) {
                                        let mut d = d.begin_scissor_mode(
                                            0,
                                            0,
                                            d.get_screen_width(),
                                            d.get_screen_height(),
                                        );
                                        for tab in &tabs {
                                            match tab {
                                                Tab::Editor(tab) => {
                                                    if tab
                                                        .graph
                                                        .upgrade()
                                                        .is_some_and(|x| Arc::ptr_eq(&x, g))
                                                    {
                                                        d.draw_line_v(
                                                            input.cursor,
                                                            d.get_world_to_screen2D(
                                                                node.position.as_vec2()
                                                                    + rvec2(
                                                                        GRID_SIZE / 2,
                                                                        GRID_SIZE / 2,
                                                                    ),
                                                                tab.camera(),
                                                            ),
                                                            theme.special,
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                };
                            }
                            HyperRef::Wire(w) => {
                                // TODO
                            }
                        }
                    }
                    d.draw_text(text, x, y, theme.console_font_size, color.get(&theme));
                    if text.ends_with('\n') {
                        y += theme.console_line_height();
                        x = left;
                    } else {
                        x += width;
                    }
                }
            }

            // title
            {
                let title = "Log";
                let title_text_width = d.measure_text(title, theme.console_font_size);
                let title_width = title_text_width + 2 * theme.title_padding_x;
                let title_height = theme.console_font_size + 2 * theme.title_padding_y;
                d.draw_rectangle(
                    console.bounds.max.x - title_width,
                    console.bounds.min.y,
                    title_width,
                    title_height,
                    theme.background2,
                );
                d.draw_text(
                    title,
                    console.bounds.max.x - title_width + theme.title_padding_x,
                    console.bounds.min.y + theme.title_padding_y,
                    theme.console_font_size,
                    theme.foreground,
                );
            }
        }
    }
}
