#![deny(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]

use crate::{
    console::{
        Console, ConsoleAnchoring, GateRef, HyperRef, LogType, NodeRef, PositionRef, ToolRef,
    },
    graph::{GraphList, node::Gate},
    icon_sheets::{ButtonIconSheets, NodeIconSheetId, NodeIconSheetSets},
    input::Bindings,
    ivec::{IBounds, IRect, IVec2},
    tab::{EditorTab, Tab, TabList},
    theme::Theme,
    tool::Tool,
    toolpane::{ToolPane, ToolPaneAnchoring},
};
use console::GraphRef;
use raylib::prelude::*;
use rl_input::Event;
use std::sync::Arc;

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
        HyperRef::Gate(_gate_ref) => {
            // TODO
        }

        HyperRef::Tool(_tool_ref) => {
            // TODO
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
                    let pos = tab.world_to_screen(node.position.as_vec2() + GRID_CENTER_OFFSET);
                    d.draw_line_v(link_anchor, pos, theme.hyperref);
                }
            });
        }

        HyperRef::Wire(wire_ref) => {
            wire_ref.deref_with(graphs, |g, _borrow, _wire| {
                for _tab in tabs.editors_of_graph(&Arc::downgrade(g)) {
                    // TODO
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

    let theme = Theme::default();
    let binds = Bindings::default();

    let _button_icon_sheets = ButtonIconSheets::load(&mut rl, &thread).unwrap();
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
                    if let Some(gate) = input.gate_hotkey {
                        toolpane.gate = gate;
                        logln!(console, LogType::Info, "set gate to {}", GateRef(gate));
                    }
                    if let Some(tool_id) = input.tool_hotkey {
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
                        let pos = IVec2::from_vec2(tab.screen_to_world(input.cursor))
                            .snap(GRID_SIZE.into());

                        match &mut toolpane.tool {
                            Tool::Create { current_node } => {
                                if input.primary.is_starting() {
                                    if let Some(&id) = graph.find_node_at_pos(pos) {
                                        // existing node
                                        if let Some(current_node) = *current_node {
                                            let src_ref = graph_ref.node(current_node);
                                            let dst_ref = graph_ref.node(id);
                                            let wire =
                                                graph.create_wire(IVec2::zero(), current_node, id);
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
                                        let new_node = graph.create_node(gate, pos);
                                        let node_ref = graph_ref.node(*new_node.id());
                                        logln!(
                                            console,
                                            LogType::Info,
                                            "create {} node {node_ref} at {}",
                                            GateRef(gate),
                                            PositionRef(pos),
                                        );
                                        let new_node_id = *new_node.id();
                                        if let Some(current_node) = current_node.as_ref() {
                                            let src_ref = graph_ref.node(*current_node);
                                            let dst_ref = node_ref;
                                            let wire = graph.create_wire(
                                                IVec2::zero(),
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
                                    && let Some(&id) = graph.find_node_at_pos(pos)
                                {
                                    let _ = graph.destroy_node(&id).expect("cannot reach this branch if graph did not contain the node");
                                    let node_ref = graph_ref.node(id);
                                    logln!(console, LogType::Info, "destroy node {node_ref}");
                                }
                            }

                            Tool::Edit { target } => {
                                if input.primary.is_starting()
                                    && let Some(id) = graph.find_node_at_pos(pos)
                                {
                                    *target = Some((graph.node(id).unwrap().position, *id));
                                }
                                if input.primary.is_ending() {
                                    if let Some((start_pos, id)) = target {
                                        let graph_ref = GraphRef(*graph.id());
                                        let node_ref = graph_ref.node(*id);
                                        let node = graph.node_mut(id).unwrap();
                                        node.position =
                                            IVec2::from_vec2(tab.screen_to_world(input.cursor))
                                                .snap(GRID_SIZE.into());
                                        logln!(
                                            console,
                                            LogType::Info,
                                            "move node {node_ref} from {} to {}",
                                            PositionRef(*start_pos),
                                            PositionRef(node.position),
                                        );
                                    }
                                    *target = None;
                                }

                                if let Some((_, id)) = target.as_ref() {
                                    let node = graph.node_mut(id).unwrap();
                                    node.position = IVec2::from_vec2(
                                        tab.screen_to_world(input.cursor)
                                            - rvec2(GRID_SIZE / 2, GRID_SIZE / 2),
                                    );
                                }
                            }
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
                        // wires
                        for wire in graph.wires_iter() {
                            let (src, dst) = graph
                                .get_wire_nodes(wire)
                                .expect("all wires should be valid");
                            d.draw_line_v(
                                src.position.as_vec2() + rvec2(GRID_SIZE / 2, GRID_SIZE / 2),
                                dst.position.as_vec2() + rvec2(GRID_SIZE / 2, GRID_SIZE / 2),
                                theme.foreground,
                            );
                        }
                        match &toolpane.tool {
                            Tool::Create { current_node } => {
                                if let Some(&current_node) = current_node.as_ref() {
                                    d.draw_line_v(
                                        graph
                                            .node(&current_node)
                                            .expect("current node should always be valid")
                                            .position
                                            .as_vec2()
                                            + rvec2(GRID_SIZE / 2, GRID_SIZE / 2),
                                        tab.screen_to_world(input.cursor),
                                        theme.foreground,
                                    );
                                }
                            }
                            Tool::Erase {} => {}
                            Tool::Edit { target: _ } => {}
                        }

                        // nodes
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
                                if node.state() != 0 {
                                    theme.active
                                } else {
                                    theme.foreground
                                },
                            );
                        }
                        match &toolpane.tool {
                            Tool::Create { current_node: _ } => {}
                            Tool::Erase {} => {}
                            Tool::Edit { target: _ } => {}
                        }
                    }
                }
            }
        }

        // console
        {
            let IRect { x, y, w, h } = IRect::from(console.bounds);
            // let mut d = d.begin_scissor_mode(x, y, w, h);

            // content
            {
                d.draw_rectangle(x, y, w, h, theme.background2);
                d.draw_rectangle(x + 1, y + 1, w - 2, h - 2, theme.background1);
                // let mut d = d.begin_scissor_mode(
                //     x + theme.console_padding_left,
                //     y + theme.console_padding_top,
                //     w - theme.console_padding_left - theme.console_padding_right,
                //     h - theme.console_padding_top - theme.console_padding_bottom,
                // );

                let mut x = x + theme.console_padding_left;
                let mut y = y + theme.console_padding_top;
                let left = x;
                for (color, text) in console.visible_content(&theme) {
                    let width = d.measure_text(text, theme.console_font_size);
                    let hyper_rec = IRect::new(x, y, width, theme.console_font_size);
                    let is_live = if let Ok(hr) = text.parse::<HyperRef>() {
                        let is_live = match hr {
                            HyperRef::Gate(_) => Some(()),
                            HyperRef::Tool(_) => Some(()),
                            HyperRef::Position(_) => Some(()),
                            HyperRef::Graph(graph_ref) => graph_ref.deref_with(&graphs, |_, _| {}),
                            HyperRef::Node(node_ref) => node_ref.deref_with(&graphs, |_, _, _| {}),
                            HyperRef::Wire(wire_ref) => wire_ref.deref_with(&graphs, |_, _, _| {}),
                        }
                        .is_some();

                        if is_live
                            && IBounds::from(hyper_rec).contains(IVec2::from_vec2(input.cursor))
                            && let Ok(hr) = text.parse::<HyperRef>()
                        {
                            draw_hyper_ref_link(&mut d, hr, hyper_rec, &theme, &graphs, &tabs);
                        }

                        Some(is_live)
                    } else {
                        None
                    };
                    d.draw_text(
                        text,
                        x,
                        y,
                        theme.console_font_size,
                        if is_live.is_none_or(|x| x) {
                            color.get(&theme)
                        } else {
                            theme.dead_link
                        },
                    );
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
