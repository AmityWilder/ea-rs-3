#![deny(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
#![allow(dead_code, reason = "for future use")]

use crate::{
    config::Config,
    console::{Console, HyperRef, LogType},
    graph::{GraphList, node::Gate, wire::Elbow},
    icon_sheets::{ButtonIconSheets, NodeIconSheetSets},
    ivec::{AsIVec2, IVec2},
    tab::{EditorTab, Tab, TabList},
    theme::Theme,
    tool::{EditDragging, Tool},
    toolpane::ToolPane,
    ui::{NcSizing, Padding},
};
use ivec::Bounds;
use properties::{DrawPropertySection, PropertiesPanel};
use raylib::prelude::*;
use std::{
    io::Write,
    sync::Arc,
    time::{Duration, Instant},
};
use toolpane::ButtonAction;
use ui::{Anchoring, ExactSizing, Panel, Sizing};

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
        Panel::new(
            "Log",
            Anchoring::Bottom {
                h: Sizing::Exact(ExactSizing {
                    val: 150.0,
                    min: Some(|theme, _, _| {
                        Some(
                            theme.console_font.line_height()
                                + theme.console_font.line_spacing
                                + theme.console_padding.vertical(),
                        )
                    }),
                    max: Some(|_theme, container_size, _content_size| Some(container_size)),
                }),
            },
            |theme| theme.console_padding,
        ),
        4096 * 80,
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

    let mut tabs = TabList::with_tabs(
        Panel::new("Editor", Anchoring::Fill, |_| Padding::amount(0.0)),
        [Tab::Editor(
            EditorTab::new(
                &mut rl,
                &thread,
                1280,
                720,
                Arc::downgrade(graphs.create_graph()),
            )
            .unwrap(),
        )],
    );

    let mut toolpane = ToolPane::new(
        Panel::new(
            "",
            Anchoring::Floating {
                x: 3.0,
                y: 3.0,
                w: NcSizing::FitContent,
                h: NcSizing::FitContent,
            },
            |theme| theme.toolpane_padding,
        ),
        Tool::default(),
        Gate::default(),
        Elbow::default(),
        theme.toolpane_orientation,
        theme.toolpane_visibility,
        theme.button_icon_scale,
    );

    let mut properties = PropertiesPanel::new(Panel::new(
        "Properties",
        Anchoring::Right {
            w: Sizing::Exact(ExactSizing {
                val: 200.0,
                min: Some(|_, _, _| Some(0.0)),
                max: Some(|_, container_size, _content_size| Some(container_size)),
            }),
        },
        |theme| theme.properties_padding,
    ));

    let mut next_eval_tick = Instant::now();
    let eval_duration = Duration::from_millis(200);

    // initialize bounds
    {
        let mut container = Bounds::new(
            Vector2::zero(),
            rvec2(rl.get_screen_width(), rl.get_screen_height()),
        );

        tabs.update_bounds(&mut rl, &thread, &theme, &container)
            .unwrap();

        if let Some(new_container) =
            properties
                .panel
                .update_bounds(&theme, &container, Vector2::zero(/* TODO */))
        {
            container = new_container;
        }

        if let Some(new_container) =
            toolpane
                .panel
                .update_bounds(&theme, &container, toolpane.content_size(&theme))
        {
            container = new_container;
        }

        if let Some(new_container) =
            console
                .panel
                .update_bounds(&theme, &container, Vector2::zero(/* TODO */))
        {
            container = new_container;
        }

        _ = container;
    }

    let mut focused_panel;

    logln!(&mut console, LogType::Success, "initialized");

    while !rl.window_should_close() {
        // Tick

        let input = binds.get_all(&rl);

        if rl.is_window_resized() {
            let window_width = rl.get_screen_width();
            let window_height = rl.get_screen_height();
            tabs.update_bounds(
                &mut rl,
                &thread,
                &theme,
                &Bounds::new(Vector2::zero(), rvec2(window_width, window_height)),
            )
            .unwrap();
            // TODO: refresh bounds on other panels
        }

        {
            // tabs only changes when window does, for now

            let toolpane_content_size = toolpane.content_size(&theme);
            let panels = [
                (&mut properties.panel, Vector2::zero(/* TODO */)),
                (&mut console.panel, Vector2::zero(/* TODO */)),
                (&mut toolpane.panel, toolpane_content_size),
            ];
            let mut container = Bounds::new(
                Vector2::zero(),
                rvec2(rl.get_screen_width(), rl.get_screen_height()),
            );
            for (panel, content_size) in panels {
                panel.tick_resize(&theme, &input, &container, content_size);

                // bounds must update regardless of if *this panel* has been resized
                if let Some(new_container) = panel.update_bounds(&theme, &container, content_size) {
                    container = new_container;
                }
            }
        }

        focused_panel = [
            (&toolpane.panel, ("toolpane", 0)),
            (&properties.panel, ("properties", 0)),
            (&console.panel, ("console", 0)),
            (tabs.panel(), ("tabs", 0)),
        ]
        .into_iter()
        .find(|(panel, _)| {
            panel
                .bounds()
                .pad(&Padding::amount(-1.5))
                .contains(input.cursor)
        })
        .map(|(_, id)| id);

        if let Some(id) = focused_panel {
            match id {
                ("toolpane", _) => {
                    if input.primary.is_starting() {
                        let bounds = toolpane.panel.content_bounds(&theme);
                        let action = toolpane.buttons(bounds.min, &theme).find_map(
                            |(button_rec, button)| {
                                Bounds::from(button_rec)
                                    .contains(input.cursor)
                                    .then_some(button.action)
                            },
                        );
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
                }

                ("properties", _) => {
                    // TODO
                }

                ("console", _) => {
                    console.bottom_offset = (console.bottom_offset + input.scroll_console as f64)
                        .clamp(
                            0.0,
                            console
                                .content_str()
                                .lines()
                                .count()
                                .saturating_sub(console.displayable_lines(&theme))
                                as f64,
                        );

                    let Vector2 { mut x, mut y } = console.panel.content_bounds(&theme).min;
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
                }

                ("tabs", _) => {
                    if let Some(tab) = tabs.focused_tab_mut() {
                        match tab {
                            Tab::Editor(tab) => {
                                if let Some(gate) = input.gate() {
                                    toolpane.set_gate(gate, &mut console);
                                }
                                if let Some(tool) = input.tool() {
                                    toolpane.set_tool(tool, &mut console);
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
                                                    if let Some(current_node) =
                                                        current_node.as_ref()
                                                    {
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
                                                && let Some(EditDragging { temp_pos: _, id }) =
                                                    target.take()
                                            {
                                                let new_position = tab
                                                    .screen_to_world(input.cursor)
                                                    .as_ivec2()
                                                    .snap(GRID_SIZE.into());
                                                graph
                                                    .translate_node(&id, new_position, &mut console)
                                                    .expect(
                                                        "edit mode target node should be valid",
                                                    );
                                            }

                                            if let Some(EditDragging { temp_pos, id: _ }) =
                                                target.as_mut()
                                            {
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
                }

                _ => unreachable!(),
            }
        }

        {
            let viewport = *tabs.panel().bounds();
            if let Some(focused_tab) = tabs.focused_tab_mut() {
                match focused_tab {
                    Tab::Editor(tab) => tab.refresh_grid(&mut rl, &thread, &theme, &viewport),
                }
            }
        }

        rl.set_mouse_cursor(
            [
                console.panel.hover.as_ref(),
                properties.panel.hover.as_ref(),
                toolpane.panel.hover.as_ref(),
                tabs.panel().hover.as_ref(),
            ]
            .into_iter()
            .flatten()
            .next()
            .map_or(MouseCursor::MOUSE_CURSOR_DEFAULT, |hover| {
                use ui::RectHoverRegion::*;
                match hover.region {
                    Left | Right => MouseCursor::MOUSE_CURSOR_RESIZE_EW,
                    Top | Bottom => MouseCursor::MOUSE_CURSOR_RESIZE_NS,
                    TopLeft | BottomRight => MouseCursor::MOUSE_CURSOR_RESIZE_NWSE,
                    TopRight | BottomLeft => MouseCursor::MOUSE_CURSOR_RESIZE_NESW,
                }
            }),
        );

        for mut graph in graphs.iter_mut().filter_map(|g| g.try_write().ok()) {
            let now = Instant::now();
            while now >= next_eval_tick {
                graph.evaluate();
                next_eval_tick = now + eval_duration;
            }
        }

        // properties.tick(
        //     &mut rl,
        //     &thread,
        //     &theme,
        //     [/*&mut toolpane.tool, &mut toolpane.gate*/] as [&mut dyn PropertySection; _],
        // );

        // Draw

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(theme.background);

        // tabs
        {
            if let Some(focused_tab) = tabs.focused_tab() {
                match focused_tab {
                    Tab::Editor(tab) => {
                        tab.draw(
                            &mut d,
                            tabs.panel().bounds(),
                            &theme,
                            &input,
                            &toolpane,
                            &node_icon_sheets,
                        );
                    }
                }
            }
        }

        // toolpane
        {
            toolpane.draw(&mut d, &input, &theme, &button_icon_sheets);
        }

        // console
        {
            console.draw(&mut d, &theme, &input, &graphs, &tabs, &toolpane);
        }

        // properties
        {
            properties.draw(
                &mut d,
                &theme,
                [/*&toolpane.tool, &toolpane.gate*/] as [&dyn DrawPropertySection<_>; _],
            );
        }
    }
}
