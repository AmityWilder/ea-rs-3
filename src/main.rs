#![deny(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
#![allow(dead_code, reason = "for future use")]

use crate::{
    config::Config,
    console::{Console, ConsoleAnchoring, HyperRef, LogType},
    graph::{GraphList, node::Gate, wire::Elbow},
    icon_sheets::{ButtonIconSheets, NodeIconSheetSets},
    ivec::{AsIVec2, IVec2},
    tab::{EditorTab, Tab, TabList},
    theme::Theme,
    tool::{EditDragging, Tool},
    toolpane::ToolPane,
};
use ivec::Bounds;
use properties::PropertiesPanel;
use raylib::prelude::*;
use rl_input::Event;
use std::{
    io::Write,
    sync::Arc,
    time::{Duration, Instant},
};
use toolpane::ButtonAction;

mod config;
mod console;
mod graph;
mod icon_sheets;
mod input;
mod ivec;
mod properties;
mod rich_text;
mod tab;
mod theme;
mod tool;
mod toolpane;
mod ui;

pub const GRID_SIZE: u8 = 8;

fn main() {
    let mut console = Console::new(
        327_680, // 4096 rows with 80 columns
        Bounds::new(Vector2::new(0.0, 570.0), Vector2::new(1280.0, 720.0)),
        ConsoleAnchoring {
            left: true,
            top: false,
            right: true,
            bottom: true,
        },
    );

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

    const CONFIG_PATH: &str = "config.toml";
    logln!(
        &mut console,
        LogType::Attempt,
        "Loading config from {CONFIG_PATH}..."
    );

    // load preferences
    let Config {
        mut theme,
        mut binds,
    } = {
        match std::fs::read_to_string(CONFIG_PATH) {
            Ok(s) => match toml::from_str(&s) {
                Ok(config) => {
                    logln!(&mut console, LogType::Success, "Config loaded.");
                    config
                }
                Err(e) => {
                    logln!(&mut console, LogType::Error, "Failed to read config: {e}");
                    Config::default()
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                logln!(
                    &mut console,
                    LogType::Warning,
                    "Config does not exist. Generating default."
                );
                let config = Config::default();
                if let Err(e) = std::fs::File::create(CONFIG_PATH).and_then(|mut file| {
                    file.write_all(
                        toml::to_string_pretty(&config)
                            .expect("default config should be serializeable")
                            .as_bytes(),
                    )
                }) {
                    logln!(&mut console, LogType::Error, "Failed to generate file: {e}");
                }
                config
            }
            Err(e) => {
                logln!(
                    &mut console,
                    LogType::Error,
                    "Failed to open config file: {e}"
                );
                Config::default()
            }
        }
    };
    theme.reload_fonts(&mut rl, &thread);

    let button_icon_sheets = ButtonIconSheets::load(&mut rl, &thread).unwrap();
    let node_icon_sheets = NodeIconSheetSets::load(&mut rl, &thread).unwrap();

    let mut graphs = GraphList::new();

    let mut tabs = TabList::from([Tab::Editor(
        EditorTab::new(
            &mut rl,
            &thread,
            Bounds::new(Vector2::zero(), Vector2::new(1280.0, 720.0)),
            Arc::downgrade(graphs.create_graph()),
        )
        .unwrap(),
    )]);

    let mut toolpane = ToolPane::new(
        Tool::default(),
        Gate::default(),
        Elbow::default(),
        theme.toolpane_anchoring,
        theme.toolpane_visibility,
        theme.button_icon_scale,
    );

    let properties = PropertiesPanel::new(Bounds::new(
        Vector2::new(rl.get_screen_width() as f32 - 300.0, 0.0),
        Vector2::new(rl.get_screen_width() as f32, rl.get_screen_height() as f32),
    ));

    let mut hovering_console_top = Event::Inactive;
    let mut dragging_console_top = Event::Inactive;

    let mut next_eval_tick = Instant::now();
    let eval_duration = Duration::from_millis(200);

    logln!(&mut console, LogType::Success, "initialized");

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
                    console.bounds.max.x = window_width as f32;
                } else {
                    let width = console.bounds.max.x - console.bounds.min.x;
                    console.bounds.min.x = window_width as f32 - width;
                    console.bounds.max.x = window_width as f32;
                }
            }
            if console.anchoring.bottom {
                if console.anchoring.top {
                    console.bounds.max.y = window_width as f32;
                } else {
                    let height = console.bounds.max.y - console.bounds.min.y;
                    console.bounds.min.y = window_height as f32 - height;
                    console.bounds.max.y = window_height as f32;
                }
            }
        }

        if toolpane
            .bounds(
                rl.get_screen_width() as f32,
                (rl.get_screen_height() as f32).min(console.bounds.min.y),
                &theme,
            )
            .contains(input.cursor)
        {
            if input.primary.is_starting() {
                let bounds = toolpane.bounds(
                    rl.get_screen_width() as f32,
                    rl.get_screen_height() as f32,
                    &theme,
                );
                let action = toolpane
                    .buttons(Vector2::new(bounds.min.x, bounds.min.y), &theme)
                    .find_map(|(button_rec, button)| {
                        Bounds::from(button_rec)
                            .contains(input.cursor)
                            .then_some(button.action)
                    });
                if let Some(action) = action {
                    match action {
                        ButtonAction::SetTool(tool_id) => {
                            toolpane.set_tool(tool_id, &mut console);
                        }
                        ButtonAction::SetGate(gate_id) => {
                            toolpane.set_gate(gate_id, &mut console);
                        }
                        ButtonAction::SetNtd(data) => {
                            toolpane.set_ntd(data, &mut console);
                        }
                        ButtonAction::Blueprints => {
                            // TODO
                        }
                        ButtonAction::Clipboard => {
                            // TODO
                        }
                        ButtonAction::Settings => {
                            // TODO
                        }
                    }
                }
            }
        } else if console.bounds.contains(input.cursor) || dragging_console_top.is_active() {
            console.bottom_offset = (console.bottom_offset + input.scroll_console as f64).clamp(
                0.0,
                console
                    .content_str()
                    .lines()
                    .count()
                    .saturating_sub(console.displayable_lines(&theme)) as f64,
            );

            let mut x = console.bounds.min.x + theme.console_padding.left;
            let mut y = console.bounds.min.y + theme.console_padding.top;
            let left = x;
            for (_, text) in console.visible_content(&theme) {
                let text_size = theme.console_font.measure_text(text);
                if Rectangle::new(x, y, text_size.x, text_size.y)
                    .check_collision_point_rec(input.cursor)
                    && let Ok(hyper_ref) = text.parse::<HyperRef>()
                {
                    match hyper_ref {
                        HyperRef::Gate(_gate_ref) => {
                            // TODO
                        }

                        HyperRef::Tool(_tool_ref) => {
                            // TODO
                        }

                        HyperRef::Position(_position_ref) => {
                            // TODO
                        }

                        HyperRef::Graph(graph_ref) => {
                            graph_ref.deref_with(&graphs, |_g, _borrow| {
                                // TODO
                            });
                        }

                        HyperRef::Node(node_ref) => {
                            node_ref.deref_with(&graphs, |_g, _borrow, _node| {
                                // TODO
                            });
                        }

                        HyperRef::Wire(wire_ref) => {
                            wire_ref.deref_with(&graphs, |_g, _borrow, _wire| {
                                // TODO
                            });
                        }
                    }
                }
                if text.ends_with('\n') {
                    y += theme.console_font.line_height();
                    x = left;
                } else {
                    x += theme.console_font.measure_text(text).x;
                }
            }
        } else if let Some(tab) = tabs.focused_tab_mut() {
            match tab {
                Tab::Editor(tab) => {
                    if let Some(gate) = input.gate() {
                        toolpane.set_gate(gate, &mut console);
                    }
                    if let Some(tool) = input.tool() {
                        toolpane.set_tool(tool, &mut console);
                    }

                    if rl.is_window_resized() {
                        let bounds = Bounds::new(
                            Vector2::zero(),
                            rvec2(rl.get_screen_width(), rl.get_screen_height()),
                        );
                        tab.update_bounds(&mut rl, &thread, bounds).unwrap();
                    }

                    tab.zoom_and_pan(input.cursor, input.pan, input.zoom, 5.0);

                    if let Some(graph) = tab.graph.upgrade()
                        // if graph is being borrowed, don't edit it! it might be saving!
                        && let Ok(mut graph) = graph.try_write()
                    {
                        let pos = tab
                            .screen_to_world(input.cursor)
                            .as_ivec2()
                            .snap(GRID_SIZE.into());

                        match &mut toolpane.tool {
                            Tool::Create { current_node } => {
                                if input.primary.is_starting() {
                                    if let Some(&id) = graph.find_node_at(pos) {
                                        // existing node
                                        if let Some(current_node) = *current_node {
                                            graph.create_wire(
                                                toolpane.elbow,
                                                current_node,
                                                id,
                                                &mut console,
                                            );
                                        }
                                        *current_node = Some(id);
                                    } else {
                                        // new node
                                        let gate = toolpane.gate;
                                        let new_node =
                                            graph.create_node(gate, pos, &mut console).expect(
                                                "this branch implies the position is available",
                                            );
                                        let new_node_id = *new_node.id();
                                        if let Some(current_node) = current_node.as_ref() {
                                            graph.create_wire(
                                                toolpane.elbow,
                                                *current_node,
                                                new_node_id,
                                                &mut console,
                                            );
                                        }
                                        *current_node = Some(new_node_id);
                                    }
                                }
                                if input.secondary.is_starting() {
                                    *current_node = None;
                                }
                            }

                            Tool::Erase {} => {
                                if input.primary.is_starting()
                                    && let Some(&id) = graph.find_node_at(pos)
                                {
                                    graph.destroy_node(&id, false, &mut console).expect("cannot reach this branch if graph did not contain the node");
                                }
                            }

                            Tool::Edit { target } => {
                                if input.primary.is_starting()
                                    && let Some(&id) = graph.find_node_at(pos)
                                {
                                    *target = Some(EditDragging {
                                        temp_pos: Vector2::default(),
                                        id,
                                    });
                                }
                                if input.primary.is_ending()
                                    && let Some(EditDragging { temp_pos: _, id }) = target.take()
                                {
                                    let new_position = tab
                                        .screen_to_world(input.cursor)
                                        .as_ivec2()
                                        .snap(GRID_SIZE.into());
                                    graph
                                        .translate_node(&id, new_position, &mut console)
                                        .expect("edit mode target node should be valid");
                                }

                                if let Some(EditDragging { temp_pos, id: _ }) = target.as_mut() {
                                    *temp_pos = tab.screen_to_world(input.cursor)
                                        - rvec2(GRID_SIZE / 2, GRID_SIZE / 2);
                                }
                            }

                            Tool::Interact {} => {}
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
        if (console.bounds.min.y..console.bounds.min.y + 3.0).contains(&(input.cursor.y)) {
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
            console.bounds.min.y = input.cursor.y.clamp(
                theme.console_padding.top, // arbitrary
                console.bounds.max.y
                    - theme.console_padding.vertical()
                    - theme.console_font.font_size
                    - 2.0 * theme.console_font.line_spacing,
            );
        }

        if hovering_console_top == Event::Starting {
            rl.set_mouse_cursor(MouseCursor::MOUSE_CURSOR_RESIZE_NS);
        } else if hovering_console_top == Event::Ending {
            rl.set_mouse_cursor(MouseCursor::MOUSE_CURSOR_DEFAULT);
        }

        for mut graph in graphs.iter_mut().filter_map(|g| g.try_write().ok()) {
            let now = Instant::now();
            while now >= next_eval_tick {
                graph.evaluate();
                next_eval_tick = now + eval_duration;
            }
        }

        // Draw

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(theme.background);

        for tab in &tabs {
            match tab {
                Tab::Editor(tab) => {
                    tab.draw(&mut d, &theme, &input, &toolpane, &node_icon_sheets);
                }
            }
        }

        // toolpane
        {
            let container_width = d.get_screen_width() as f32;
            let container_height = d.get_screen_height() as f32;
            toolpane.draw(
                &mut d,
                container_width,
                container_height,
                &input,
                &theme,
                &button_icon_sheets,
            );
        }

        // properties
        {
            properties.draw(&mut d, &theme);
        }

        // console
        {
            console.draw(&mut d, &theme, &input, &graphs, &tabs, &toolpane);
        }
    }
}
