#![deny(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
#![allow(dead_code, reason = "for future use")]

use crate::{
    config::Config,
    console::{
        Console, ConsoleAnchoring, GateRef, GraphRef, HyperRef, LogType, PositionRef, ToolRef,
    },
    graph::{
        GraphList,
        node::{Gate, GateId},
        wire::Elbow,
    },
    icon_sheets::{ButtonIconId, ButtonIconSheets, NodeIconSheetSets},
    ivec::{AsIVec2, IBounds, IRect, IVec2},
    tab::{EditorTab, Tab, TabList},
    theme::Theme,
    tool::{EditDragging, Tool, ToolId},
    toolpane::ToolPane,
};
use raylib::prelude::*;
use rl_input::Event;
use std::{io::Write, sync::Arc};

mod config;
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

fn draw_hyper_ref_link<D>(
    d: &mut D,
    hyper_ref: HyperRef,
    rec: IRect,
    theme: &Theme,
    graphs: &GraphList,
    tabs: &TabList,
    toolpane: &ToolPane,
) where
    D: RaylibDraw,
{
    const GRID_CENTER_OFFSET: Vector2 =
        Vector2::new((GRID_SIZE / 2) as f32, (GRID_SIZE / 2) as f32);

    // highlight ref text
    d.draw_rectangle(rec.x, rec.y, rec.w, rec.h, theme.hyperref.alpha(0.2));

    let link_anchor = Vector2::new(
        rec.x as f32 + rec.w as f32,
        rec.y as f32 + rec.h as f32 * 0.5,
    );

    match hyper_ref {
        HyperRef::Gate(gate_ref) => {
            // HACK: only matches against the icon of the button!
            if let Some((rec, _)) = toolpane.buttons(IVec2::zero(), theme).find(|(_, button)| {
                matches!(
                    (button.icon, gate_ref.0),
                    (Some(ButtonIconId::Or), GateId::Or)
                        | (Some(ButtonIconId::And), GateId::And)
                        | (Some(ButtonIconId::Nor), GateId::Nor)
                        | (Some(ButtonIconId::Xor), GateId::Xor)
                        | (Some(ButtonIconId::Resistor), GateId::Resistor)
                        | (Some(ButtonIconId::Capacitor), GateId::Capacitor)
                        | (Some(ButtonIconId::Led), GateId::Led)
                        | (Some(ButtonIconId::Delay), GateId::Delay)
                        | (Some(ButtonIconId::Battery), GateId::Battery)
                )
            }) {
                d.draw_line_v(
                    link_anchor,
                    Vector2::new(
                        rec.x as f32 + 0.5 * rec.w as f32,
                        rec.y as f32 + 0.5 * rec.h as f32,
                    ),
                    theme.hyperref,
                );
            }
        }

        HyperRef::Tool(tool_ref) => {
            // HACK: only matches against the icon of the button!
            if let Some((rec, _)) = toolpane.buttons(IVec2::zero(), theme).find(|(_, button)| {
                matches!(
                    (button.icon, tool_ref.0),
                    (Some(ButtonIconId::Pen), ToolId::Create)
                        | (Some(ButtonIconId::Erase), ToolId::Erase)
                        | (Some(ButtonIconId::Edit), ToolId::Edit)
                        | (Some(ButtonIconId::Interact), ToolId::Interact)
                )
            }) {
                d.draw_line_v(
                    link_anchor,
                    Vector2::new(
                        rec.x as f32 + 0.5 * rec.w as f32,
                        rec.y as f32 + 0.5 * rec.h as f32,
                    ),
                    theme.hyperref,
                );
            }
        }

        HyperRef::Position(position_ref) => {
            for tab in tabs.editors() {
                let pos = tab.world_to_screen(position_ref.as_vec2() + GRID_CENTER_OFFSET);
                d.draw_line_v(link_anchor, pos, theme.hyperref);
            }
        }

        HyperRef::Graph(graph_ref) => {
            graph_ref.deref_with(graphs, |g, _borrow| {
                for _tab in tabs.editors_of_graph(&Arc::downgrade(g)) {
                    // TODO
                }
            });
        }

        HyperRef::Node(node_ref) => {
            node_ref.deref_with(graphs, |g, _borrow, node| {
                for tab in tabs.editors_of_graph(&Arc::downgrade(g)) {
                    let pos = tab.world_to_screen(node.position().as_vec2() + GRID_CENTER_OFFSET);
                    d.draw_line_v(link_anchor, pos, theme.hyperref);
                }
            });
        }

        HyperRef::Wire(wire_ref) => {
            wire_ref.deref_with(graphs, |g, borrow, wire| {
                for tab in tabs.editors_of_graph(&Arc::downgrade(g)) {
                    let (start, end) = borrow
                        .get_wire_nodes(wire)
                        .expect("all wires should be valid");
                    let start_pos = start.position().as_vec2() + GRID_CENTER_OFFSET;
                    let end_pos = end.position().as_vec2() + GRID_CENTER_OFFSET;
                    let pos = tab.world_to_screen(wire.elbow.calculate(start_pos, end_pos));
                    d.draw_line_v(link_anchor, pos, theme.hyperref);
                }
            });
        }
    }
}

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

    // load preferences
    let Config { theme, mut binds } = {
        const CONFIG_PATH: &str = "config.toml";
        match std::fs::read_to_string(CONFIG_PATH) {
            Ok(s) => toml::from_str(&s).unwrap(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let config = Config::default();
                std::fs::File::create(CONFIG_PATH)
                    .unwrap()
                    .write_all(toml::to_string_pretty(&config).unwrap().as_bytes())
                    .unwrap();
                config
            }
            Err(e) => {
                panic!("{e}");
            }
        }
    };

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
        Elbow::default(),
        theme.toolpane_anchoring,
        theme.toolpane_visibility,
        theme.button_icon_scale,
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

        if toolpane
            .bounds(
                rl.get_screen_width(),
                rl.get_screen_height().min(console.bounds.min.y),
                &theme,
            )
            .contains(input.cursor.as_ivec2())
        {
        } else if console.bounds.contains(input.cursor.as_ivec2())
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
                .contains(input.cursor.as_ivec2())
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
                    y += theme.console_line_height();
                    x = left;
                } else {
                    x += rl.measure_text(text, theme.console_font_size);
                }
            }
        } else if let Some(tab) = tabs.focused_tab_mut() {
            match tab {
                Tab::Editor(tab) => {
                    if let Some(gate) = input.gate() {
                        toolpane.gate = gate.to_gate(0);
                        logln!(
                            console,
                            LogType::Info,
                            "set gate to {}",
                            GateRef(toolpane.gate.id())
                        );
                    }
                    if let Some(tool_id) = input.tool()
                        && tool_id != toolpane.tool.id()
                    {
                        toolpane.tool = tool_id.init();
                        logln!(console, LogType::Info, "set tool to {}", ToolRef(tool_id));
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
                        let graph_ref = GraphRef(*graph.id());
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
                                            let src_ref = graph_ref.node(current_node);
                                            let dst_ref = graph_ref.node(id);
                                            let wire =
                                                graph.create_wire(toolpane.elbow, current_node, id);
                                            let wire_ref = graph_ref.wire(*wire.id());
                                            logln!(
                                                console,
                                                LogType::Info,
                                                "create wire {wire_ref} from {src_ref} to {dst_ref}"
                                            );
                                        }
                                        *current_node = Some(id);
                                    } else {
                                        // new node
                                        let gate = toolpane.gate;
                                        let new_node = graph.create_node(gate, pos).expect(
                                            "this branch implies the position is available",
                                        );
                                        let node_ref = graph_ref.node(*new_node.id());
                                        logln!(
                                            console,
                                            LogType::Info,
                                            "create {} node {node_ref} at {}",
                                            GateRef(gate.id()),
                                            PositionRef(pos),
                                        );
                                        let new_node_id = *new_node.id();
                                        if let Some(current_node) = current_node.as_ref() {
                                            let src_ref = graph_ref.node(*current_node);
                                            let dst_ref = node_ref;
                                            let wire = graph.create_wire(
                                                toolpane.elbow,
                                                *current_node,
                                                new_node_id,
                                            );
                                            let wire_ref = graph_ref.wire(*wire.id());
                                            logln!(
                                                console,
                                                LogType::Info,
                                                "create wire {wire_ref} from {src_ref} to {dst_ref}"
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
                                    let _ = graph.destroy_node(&id, false).expect("cannot reach this branch if graph did not contain the node");
                                    let node_ref = graph_ref.node(id);
                                    logln!(console, LogType::Info, "destroy node {node_ref}");
                                }
                            }

                            Tool::Edit { target } => {
                                if input.primary.is_starting()
                                    && let Some(&id) = graph.find_node_at(pos)
                                {
                                    *target = Some(EditDragging {
                                        start_pos: graph.node(&id).unwrap().position(),
                                        temp_pos: Vector2::default(),
                                        id,
                                    });
                                }
                                if input.primary.is_ending()
                                    && let Some(EditDragging {
                                        start_pos,
                                        temp_pos: _,
                                        id,
                                    }) = target.take()
                                {
                                    let graph_ref = GraphRef(*graph.id());
                                    let node_ref = graph_ref.node(id);
                                    let new_position = tab
                                        .screen_to_world(input.cursor)
                                        .as_ivec2()
                                        .snap(GRID_SIZE.into());
                                    graph
                                        .translate_node(&id, new_position)
                                        .expect("edit mode target node should be valid");
                                    logln!(
                                        console,
                                        LogType::Info,
                                        "move node {node_ref} from {} to {}",
                                        PositionRef(start_pos),
                                        PositionRef(new_position),
                                    );
                                }

                                if let Some(EditDragging {
                                    start_pos: _,
                                    temp_pos,
                                    id: _,
                                }) = target.as_mut()
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
                theme.console_padding_top, // arbitrary
                console.bounds.max.y
                    - theme.console_padding_bottom
                    - theme.console_padding_bottom
                    - theme.console_line_height(),
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
                    tab.draw(&mut d, &theme, &input, &toolpane, &node_icon_sheets);
                }
            }
        }

        // toolpane
        {
            let container_width = d.get_screen_width();
            let container_height = d.get_screen_height();
            toolpane.draw(
                &mut d,
                container_width,
                container_height,
                &input,
                &theme,
                &button_icon_sheets,
            );
        }

        // console
        {
            console.draw(
                &mut d,
                |d, text, font_size| d.measure_text(text, font_size),
                &theme,
                &input,
                &graphs,
                &tabs,
                &toolpane,
            );
        }
    }
}
