use crate::{
    GRID_EXTENT, GRID_SIZE,
    console::attempt::*,
    graph::{
        Graph, OutOfIDsError,
        node::{Gate, GateId, GateInstance, Node, NodeId},
        wire::{Elbow, Flow, Wire},
    },
    icon_sheets::NodeIconSheetId,
    input::Inputs,
    ivec::{AsIVec2, Bounds, IBounds, IVec2},
    tab::EditorTab,
    theme::Theme,
    toolpane::ToolPane,
};
use arrayvec::ArrayString;
use raylib::prelude::*;
use rustc_hash::FxHashMap;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolId {
    #[default]
    Create,
    Erase,
    Edit,
    Interact,
}

impl std::fmt::Display for ToolId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolId::Create => "create",
            ToolId::Erase => "erase",
            ToolId::Edit => "edit",
            ToolId::Interact => "ineteract",
        }
        .fmt(f)
    }
}

impl std::str::FromStr for ToolId {
    type Err = ();

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "create" => Ok(ToolId::Create),
            "erase" => Ok(ToolId::Erase),
            "edit" => Ok(ToolId::Edit),
            "ineteract" => Ok(ToolId::Interact),
            _ => Err(()),
        }
    }
}

impl ToolId {
    #[inline]
    pub fn init(self) -> Tool {
        match self {
            ToolId::Create => Tool::Create(Create::default()),
            ToolId::Erase => Tool::Erase(Erase::default()),
            ToolId::Edit => Tool::Edit(Edit::default()),
            ToolId::Interact => Tool::Interact(Interact::default()),
        }
    }
}

fn draw_wires<D: RaylibDraw>(d: &mut D, graph: &Graph, active: Color, inactive: Color) {
    for wire in graph.wires_iter() {
        // SAFETY: wires provided by a graph will always be of that graph
        unsafe {
            let state = wire.state(graph).unwrap_unchecked();
            wire.draw(d, graph, GRID_EXTENT, if state { active } else { inactive })
                .unwrap_unchecked();
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_node<D: RaylibDraw>(
    d: &mut D,
    position: Vector2,
    gate_id: GateId,
    sheet_and_width: Option<(&Texture2D, i32)>,
    background: Color,
    highlight: Option<Color>,
    color: Color,
    ntd_color: Option<Color>,
) {
    let rec = Rectangle {
        x: position.x,
        y: position.y,
        width: GRID_SIZE.into(),
        height: GRID_SIZE.into(),
    };

    if let Some((sheet, icon_width)) = sheet_and_width {
        let cell = gate_id.icon_cell_irec(NodeIconSheetId::Background, icon_width);
        d.draw_texture_pro(sheet, cell.as_rec(), rec, Vector2::zero(), 0.0, background);

        let cell = gate_id.icon_cell_irec(NodeIconSheetId::Basic, icon_width);
        d.draw_texture_pro(sheet, cell.as_rec(), rec, Vector2::zero(), 0.0, color);

        if let Some(ntd_color) = ntd_color {
            let cell = gate_id.icon_cell_irec(NodeIconSheetId::Ntd, icon_width);
            d.draw_texture_pro(sheet, cell.as_rec(), rec, Vector2::zero(), 0.0, ntd_color);
        }

        if let Some(highlight) = highlight {
            let cell = gate_id.icon_cell_irec(NodeIconSheetId::Highlight, icon_width);
            d.draw_texture_pro(sheet, cell.as_rec(), rec, Vector2::zero(), 0.0, highlight);
        }
    } else {
        d.draw_rectangle_rec(rec, color);
    }
}

fn ntd_color(gate: GateInstance, theme: &Theme) -> Option<Color> {
    match gate {
        GateInstance::Resistor { resistance: n } | GateInstance::Led { color: n } => {
            Some(theme.resistance[n])
        }

        GateInstance::Capacitor { capacity, stored } => {
            let percent = f32::from(u8::from(stored)) / f32::from(u8::from(capacity));
            Some(theme.active.alpha(percent))
        }

        _ => None,
    }
}

#[derive(Debug, Clone, Default)]
pub struct Create {
    pub current_node: Option<NodeId>,
}

impl Create {
    pub fn tick(
        &mut self,
        toolpane_gate: Gate,
        toolpane_elbow: Elbow,
        input: &Inputs,
        graph: &mut Graph,
        _cursor_world_pos: Vector2,
        snapped_cursor_world_pos: IVec2,
    ) -> bool {
        let mut is_dirty = false;

        if input.primary.is_starting() {
            match graph.create_node(toolpane_gate, snapped_cursor_world_pos) {
                Ok(new_node) => {
                    // new node
                    let new_node_id = *new_node.id();
                    if let Some(current_node) = self.current_node.as_ref() {
                        let src = *current_node;
                        let dst = new_node_id;
                        is_dirty |= graph
                            .create_wire(toolpane_elbow, src, dst)
                            .err_info_with(|existing| {
                                format!(
                                    "wire from {} to {} already exists: wire {}",
                                    src.node_ref(),
                                    dst.node_ref(),
                                    existing.wire_ref(),
                                )
                            })
                            .is_ok();
                    }
                    self.current_node = Some(new_node_id);
                }
                Err(node) => {
                    let next_node = *node.id();
                    // existing node
                    if let Some(current_node) = self.current_node
                        && current_node != next_node
                    {
                        let src = current_node;
                        let dst = next_node;
                        is_dirty |= graph
                            .create_wire(toolpane_elbow, src, dst)
                            .err_info_with(|wire| {
                                format!(
                                    "wire from {} to {} already exists: wire {}",
                                    src.node_ref(),
                                    dst.node_ref(),
                                    wire.wire_ref(),
                                )
                            })
                            .is_ok();
                    }
                    self.current_node = Some(next_node);
                }
            }
        }

        if input.secondary.is_starting() {
            self.current_node = None;
        }

        is_dirty
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw<D: RaylibDraw>(
        &self,
        d: &mut D,
        theme: &Theme,
        input: &Inputs,
        toolpane: &ToolPane,
        graph: &Graph,
        tab: &EditorTab,
        sheet_and_width: Option<(&Texture2D, i32)>,
    ) {
        draw_wires(d, graph, theme.active, theme.foreground);

        // current wire
        if let Some(&current_node) = self.current_node.as_ref() {
            Wire::draw_immediate(
                d,
                graph
                    .node(&current_node)
                    .fatal("current node should always be valid")
                    .position()
                    .as_vec2()
                    + GRID_EXTENT,
                tab.screen_to_world(input.cursor),
                toolpane.elbow,
                theme.foreground,
            );
        }

        // nodes
        for node in graph.nodes_iter() {
            draw_node(
                d,
                node.position().as_vec2(),
                node.gate().as_gate().id(),
                sheet_and_width,
                theme.background,
                tab.selection
                    .contains_key(node.id())
                    .then_some(theme.interact),
                if node.state() {
                    theme.active
                } else {
                    theme.foreground
                },
                ntd_color(*node.gate(), theme),
            );
        }

        if input.show_details {
            use std::fmt::Write;
            const BUF_SIZE: usize = "[18446744073709551615] 18446744073709551615".len();

            let mut buf = ArrayString::<BUF_SIZE>::new();
            for node in graph.nodes_iter() {
                let (group, order) = graph.eval_order_of(node.id()).fatal(
                    "all nodes returned by node_iter() should be of the graph the iterator borrows, \
                    and eval_order_of should hold entries for every node in its graph",
                );
                buf.clear();
                write!(buf, "[{group}] {order}").unwrap();
                let pos = node.position().as_vec2() + rvec2(GRID_SIZE, GRID_SIZE);
                theme.general_font.draw_text(d, &buf, pos, theme.foreground);
            }
        }

        let cursor_world_pos = tab.screen_to_world(input.cursor);

        if let Some(node) = graph.node_at(Graph::world_to_grid(
            cursor_world_pos.as_ivec2().snap(GRID_SIZE.into()),
        )) {
            // hovered node
            let node_position = node.position().as_vec2();
            let rec = Rectangle {
                x: node_position.x,
                y: node_position.y,
                width: GRID_SIZE.into(),
                height: GRID_SIZE.into(),
            };
            let color = theme.interact;
            if let Some((sheet, icon_width)) = sheet_and_width {
                let gate_id = node.gate().as_gate().id();
                let cell = gate_id.icon_cell_irec(NodeIconSheetId::Highlight, icon_width);
                d.draw_texture_pro(sheet, cell.as_rec(), rec, Vector2::zero(), 0.0, color);
            } else {
                d.draw_rectangle_rec(rec, color);
            }
        } else {
            // node to be created
            draw_node(
                d,
                cursor_world_pos - GRID_EXTENT,
                toolpane.gate.id(),
                sheet_and_width,
                theme.background,
                None,
                theme.foreground.alpha(0.5),
                if let Gate::Resistor { resistance: n } | Gate::Led { color: n } = toolpane.gate {
                    Some(theme.resistance[n])
                } else {
                    None
                },
            );
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Erase {}

impl Erase {
    pub fn tick(
        &mut self,
        input: &Inputs,
        graph: &mut Graph,
        _cursor_world_pos: Vector2,
        snapped_cursor_world_pos: IVec2,
    ) -> bool {
        let mut is_dirty = false;

        if input.primary.is_starting()
            && let Some(node) = graph.node_at(Graph::world_to_grid(snapped_cursor_world_pos))
        {
            let to_remove = *node.id();
            graph
                .destroy_node(&to_remove, false)
                .issue_error("cannot reach this branch if graph did not contain the node");
            is_dirty = true;
        }

        is_dirty
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw<D: RaylibDraw>(
        &self,
        d: &mut D,
        theme: &Theme,
        input: &Inputs,
        _toolpane: &ToolPane,
        graph: &Graph,
        tab: &EditorTab,
        sheet_and_width: Option<(&Texture2D, i32)>,
    ) {
        draw_wires(d, graph, theme.active, theme.foreground);

        for node in graph.nodes_iter() {
            draw_node(
                d,
                node.position().as_vec2(),
                node.gate().as_gate().id(),
                sheet_and_width,
                theme.background,
                tab.selection
                    .contains_key(node.id())
                    .then_some(theme.interact),
                if node.state() {
                    theme.active
                } else {
                    theme.foreground
                },
                ntd_color(*node.gate(), theme),
            );
        }

        if let Some(node) = graph.node_at(Graph::world_to_grid(
            tab.screen_to_world(input.cursor)
                .as_ivec2()
                .snap(GRID_SIZE.into()),
        )) {
            // hovered node
            let node_position = node.position().as_vec2();
            let rec = Rectangle {
                x: node_position.x,
                y: node_position.y,
                width: GRID_SIZE.into(),
                height: GRID_SIZE.into(),
            };
            let color = theme.interact;
            if let Some((sheet, icon_width)) = sheet_and_width {
                let gate_id = node.gate().as_gate().id();
                let cell = gate_id.icon_cell_irec(NodeIconSheetId::Highlight, icon_width);
                d.draw_texture_pro(sheet, cell.as_rec(), rec, Vector2::zero(), 0.0, color);
            } else {
                d.draw_rectangle_rec(rec, color);
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EditDragging {
    Marquee { start: Vector2 },
    Translate { start: Vector2 },
}

#[derive(Debug, Clone, Default)]
pub struct Edit {
    pub dragging: Option<EditDragging>,
}

impl Edit {
    #[allow(clippy::too_many_arguments)]
    pub fn tick(
        &mut self,
        toolpane_gate: Gate,
        input: &Inputs,
        selection: &mut FxHashMap<NodeId, u8>,
        graph: &mut Graph,
        cursor_world_pos: Vector2,
        snapped_cursor_world_pos: IVec2,
    ) -> bool {
        let mut is_dirty = false;

        // set gate
        if input.secondary.is_starting()
            && let Some(node) = graph.node_mut_at(Graph::world_to_grid(snapped_cursor_world_pos))
        {
            *node.gate_mut() = GateInstance::from_gate(toolpane_gate);
            is_dirty = true;
        }

        // begin dragging
        if input.primary.is_starting() {
            if let Some(node) = graph.node_at(Graph::world_to_grid(snapped_cursor_world_pos)) {
                // translate nodes
                if !selection.contains_key(node.id()) {
                    // replace selection
                    selection.clear();
                    selection.insert(*node.id(), 0);
                }

                self.dragging = Some(EditDragging::Translate {
                    start: cursor_world_pos,
                });
            } else {
                // create marquee
                self.dragging = Some(EditDragging::Marquee {
                    start: cursor_world_pos,
                });
            }
        }

        // finish dragging
        if input.primary.is_ending()
            && let Some(dragging) = &self.dragging
        {
            match *dragging {
                EditDragging::Marquee { start } => {
                    // add overlapping nodes to selection and store rectangle for visuals
                    let bounds = Bounds::minmax_points(start, cursor_world_pos);

                    let grid_bounds = IBounds::new(
                        Graph::world_to_grid(bounds.min.as_ivec2()),
                        Graph::world_to_grid(bounds.max.as_ivec2()),
                    );

                    let max_group_id = selection.values().copied().max().unwrap_or(0);
                    let group_id = max_group_id
                        .checked_add(1)
                        .ok_or(OutOfIDsError)
                        .error("failed to create new group")
                        .err_info("appending new selection to last group")
                        .unwrap_or(max_group_id);

                    selection.extend(
                        graph
                            .nodes_in(grid_bounds)
                            .map(Node::id)
                            .copied()
                            .zip(std::iter::repeat(group_id)),
                    );
                }

                EditDragging::Translate { start } => {
                    // move all selected
                    let delta = cursor_world_pos - start;
                    let idelta = delta.as_ivec2().snap(GRID_SIZE.into());

                    for id in selection.keys() {
                        if let Ok(node) = graph.node(id).error("all selected nodes should be valid")
                        {
                            graph.translate_node(id, node.position() + idelta).unwrap(/* covered by if let Ok() */);
                        }
                    }
                }
            }
            self.dragging = None;
        }

        is_dirty
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw<D: RaylibDraw>(
        &self,
        d: &mut D,
        theme: &Theme,
        input: &Inputs,
        _toolpane: &ToolPane,
        graph: &Graph,
        tab: &EditorTab,
        sheet_and_width: Option<(&Texture2D, i32)>,
    ) {
        let cursor_world_pos = tab.screen_to_world(input.cursor);

        let mut selection_groups: [Option<(Bounds, bool)>; _] = [None; u8::MAX as usize];
        for (id, &group) in &tab.selection {
            if let Ok(node) = graph.node(id).error("all selected nodes should be valid") {
                let pos = node.position().as_vec2();
                let item = Bounds::new(pos, pos + rvec2(GRID_SIZE, GRID_SIZE));
                let bounds = &mut selection_groups[group as usize];
                if let Some((bounds, multiple)) = bounds {
                    bounds.include_bounds(item);
                    *multiple = true;
                } else {
                    *bounds = Some((item, false));
                }
            }
        }
        for bounds in selection_groups.into_iter().filter_map(|x| match x {
            Some((x, true)) => Some(x),
            _ => None,
        }) {
            d.draw_rectangle_rec(Rectangle::from(bounds), theme.interact.alpha(0.2));
        }

        draw_wires(d, graph, theme.active, theme.foreground);

        if let Some(dragging) = &self.dragging {
            match dragging {
                EditDragging::Marquee { start } => {
                    d.draw_rectangle_rec(
                        Rectangle::from(Bounds::minmax_points(*start, cursor_world_pos)),
                        theme.interact.alpha(0.2),
                    );
                }

                EditDragging::Translate { start: _ } => {
                    // for id in &self.selection_nodes {
                    //     for (wire, flow) in graph.wires_of(id) {
                    //         let (start_pos, end_pos) = match flow {
                    //             Flow::Input => (
                    //                 graph
                    //                     .node(wire.src())
                    //                     .fatal("wire src should always be valid")
                    //                     .position()
                    //                     .as_vec2()
                    //                     + GRID_EXTENT,
                    //                 *temp_pos + GRID_EXTENT,
                    //             ),
                    //             Flow::Output => (
                    //                 *temp_pos + GRID_EXTENT,
                    //                 graph
                    //                     .node(wire.dst())
                    //                     .fatal("wire dst should always be valid")
                    //                     .position()
                    //                     .as_vec2()
                    //                     + GRID_EXTENT,
                    //             ),
                    //             Flow::Loop => {
                    //                 todo!()
                    //             }
                    //         };
                    //         Wire::draw_immediate(d, start_pos, end_pos, wire.elbow, theme.special);
                    //     }
                    //     let node = graph.node(id).fatal("node being dragged should be valid");
                    //     let rec = Rectangle {
                    //         x: temp_pos.x,
                    //         y: temp_pos.y,
                    //         width: GRID_SIZE.into(),
                    //         height: GRID_SIZE.into(),
                    //     };
                    //     let color = theme.special;
                    //     if let Some((sheet, icon_width)) = sheet_and_width {
                    //         let gate_id = node.gate().as_gate().id();
                    //         let cell = gate_id.icon_cell_irec(NodeIconSheetId::Basic, icon_width);
                    //         d.draw_texture_pro(
                    //             sheet,
                    //             cell.as_rec(),
                    //             rec,
                    //             Vector2::zero(),
                    //             0.0,
                    //             color,
                    //         );
                    //     } else {
                    //         d.draw_rectangle_rec(rec, color);
                    //     }
                    // }
                }
            }
        }

        if let Some(hovered) = graph.node_at(Graph::world_to_grid(
            tab.screen_to_world(input.cursor)
                .as_ivec2()
                .snap(GRID_SIZE.into()),
        )) {
            for (wire, flow) in graph.wires_of(hovered.id()) {
                wire.draw(
                    d,
                    graph,
                    GRID_EXTENT,
                    match flow {
                        Flow::Input => theme.input,
                        Flow::Output => theme.output,
                        Flow::Loop => todo!(),
                    },
                )
                .fatal("all wires should be valid");
            }
        }

        let translate_delta = if let Some(EditDragging::Translate { start }) = self.dragging {
            Some(cursor_world_pos - start)
        } else {
            None
        };
        for node in graph.nodes_iter() {
            let is_selected = tab.selection.contains_key(node.id());
            let base_color = if node.state() {
                theme.active
            } else {
                theme.foreground
            };
            let (alpha, highlight) = match (is_selected, translate_delta.is_some()) {
                (true, true) => (Some(0.25), None),
                (true, false) => (None, Some(theme.interact)),
                (false, _) => (None, None),
            };
            draw_node(
                d,
                node.position().as_vec2(),
                node.gate().as_gate().id(),
                sheet_and_width,
                theme.background,
                highlight,
                alpha.map_or(base_color, |a| base_color.alpha(a)),
                ntd_color(*node.gate(), theme),
            );
            if is_selected && let Some(delta) = translate_delta {
                draw_node(
                    d,
                    node.position().as_vec2() + delta,
                    node.gate().as_gate().id(),
                    sheet_and_width,
                    theme.background,
                    Some(theme.interact),
                    base_color,
                    ntd_color(*node.gate(), theme),
                );
            }
        }

        if let Some(node) = graph.node_at(Graph::world_to_grid(
            tab.screen_to_world(input.cursor)
                .as_ivec2()
                .snap(GRID_SIZE.into()),
        )) {
            // hovered node
            let node_position = node.position().as_vec2();
            let rec = Rectangle {
                x: node_position.x,
                y: node_position.y,
                width: GRID_SIZE.into(),
                height: GRID_SIZE.into(),
            };
            let color = theme.interact;
            if let Some((sheet, icon_width)) = sheet_and_width {
                let gate_id = node.gate().as_gate().id();
                let cell = gate_id.icon_cell_irec(NodeIconSheetId::Highlight, icon_width);
                d.draw_texture_pro(sheet, cell.as_rec(), rec, Vector2::zero(), 0.0, color);
            } else {
                d.draw_rectangle_rec(rec, color);
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Interact {}

impl Interact {
    pub fn tick(
        &mut self,
        input: &Inputs,
        graph: &mut Graph,
        _cursor_world_pos: Vector2,
        snapped_cursor_world_pos: IVec2,
    ) -> bool {
        let mut is_dirty = false;

        if input.primary.is_starting()
            && let Some(&id) = graph
                .node_mut_at(Graph::world_to_grid(snapped_cursor_world_pos))
                .map(|node| node.id())
            && graph.is_inputless(&id)
        {
            let node = graph.node_mut(&id).unwrap();
            match node.gate_mut() {
                gate @ GateInstance::Or => {
                    *gate = GateInstance::Nor;
                    is_dirty = true;
                }

                gate @ GateInstance::Nor => {
                    *gate = GateInstance::Or;
                    is_dirty = true;
                }

                _ => {}
            };
        }

        is_dirty
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw<D: RaylibDraw>(
        &self,
        d: &mut D,
        theme: &Theme,
        input: &Inputs,
        _toolpane: &ToolPane,
        graph: &Graph,
        tab: &EditorTab,
        sheet_and_width: Option<(&Texture2D, i32)>,
    ) {
        draw_wires(d, graph, theme.active, theme.foreground);

        for node in graph.nodes_iter() {
            match node.gate() {
                GateInstance::Led { color } => {
                    let node_position = node.position().as_vec2();
                    let rec = Rectangle {
                        x: node_position.x,
                        y: node_position.y,
                        width: GRID_SIZE.into(),
                        height: GRID_SIZE.into(),
                    };
                    let (count, sum) = graph.inputs_to(node.id()).fold((0, 0), |(n, acc), wire| {
                        let state = graph
                            .node(wire.src())
                            .fatal("wire src should always be valid")
                            .state();
                        (n + 1, acc + usize::from(state))
                    });
                    let alpha = if count == 0 {
                        0.0
                    } else {
                        sum as f32 / count as f32
                    };
                    d.draw_rectangle_rec(
                        rec,
                        theme.background.lerp(theme.resistance[*color], alpha),
                    );
                }

                GateInstance::Or | GateInstance::Nor if graph.is_inputless(node.id()) => {
                    let node_position = node.position().as_vec2();
                    let rec = Rectangle {
                        x: node_position.x,
                        y: node_position.y,
                        width: GRID_SIZE.into(),
                        height: GRID_SIZE.into(),
                    };
                    let color = theme.available;
                    if let Some((sheet, icon_width)) = sheet_and_width {
                        let gate_id = node.gate().as_gate().id();
                        d.draw_texture_pro(
                            sheet,
                            gate_id
                                .icon_cell_irec(NodeIconSheetId::Background, icon_width)
                                .as_rec(),
                            rec,
                            Vector2::zero(),
                            0.0,
                            theme.background,
                        );
                        d.draw_texture_pro(
                            sheet,
                            gate_id
                                .icon_cell_irec(NodeIconSheetId::Basic, icon_width)
                                .as_rec(),
                            rec,
                            Vector2::zero(),
                            0.0,
                            color,
                        );
                    } else {
                        d.draw_rectangle_rec(rec, color);
                    }
                }

                _ => {
                    let node_position = node.position().as_vec2();
                    let rec = Rectangle {
                        x: node_position.x + f32::from(GRID_SIZE) * (0.5 - 0.25 * 0.5),
                        y: node_position.y + f32::from(GRID_SIZE) * (0.5 - 0.25 * 0.5),
                        width: f32::from(GRID_SIZE) * 0.25,
                        height: f32::from(GRID_SIZE) * 0.25,
                    };
                    let color = if node.state() {
                        theme.active
                    } else {
                        theme.foreground1
                    };
                    d.draw_rectangle_rec(rec, color);
                }
            }
        }

        if let Some(node) = graph.node_at(Graph::world_to_grid(
            tab.screen_to_world(input.cursor)
                .as_ivec2()
                .snap(GRID_SIZE.into()),
        )) && graph.is_inputless(node.id())
        {
            // hovered node
            let node_position = node.position().as_vec2();
            let rec = Rectangle {
                x: node_position.x,
                y: node_position.y,
                width: GRID_SIZE.into(),
                height: GRID_SIZE.into(),
            };
            let color = theme.interact;
            if let Some((sheet, icon_width)) = sheet_and_width {
                let gate_id = node.gate().as_gate().id();
                let cell = gate_id.icon_cell_irec(NodeIconSheetId::Highlight, icon_width);
                d.draw_texture_pro(sheet, cell.as_rec(), rec, Vector2::zero(), 0.0, color);
            } else {
                d.draw_rectangle_rec(rec, color);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum Tool {
    Create(Create),
    Erase(Erase),
    Edit(Edit),
    Interact(Interact),
}

impl Default for Tool {
    #[inline]
    fn default() -> Self {
        Self::Create(Create::default())
    }
}

impl Tool {
    #[inline]
    pub const fn id(&self) -> ToolId {
        match self {
            Tool::Create { .. } => ToolId::Create,
            Tool::Erase { .. } => ToolId::Erase,
            Tool::Edit { .. } => ToolId::Edit,
            Tool::Interact { .. } => ToolId::Interact,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn tick(
        &mut self,
        toolpane_gate: Gate,
        toolpane_elbow: Elbow,
        input: &Inputs,
        selection: &mut FxHashMap<NodeId, u8>,
        graph: &mut Graph,
        cursor_world_pos: Vector2,
        snapped_cursor_world_pos: IVec2,
    ) -> bool {
        match self {
            Tool::Create(tool) => tool.tick(
                toolpane_gate,
                toolpane_elbow,
                input,
                graph,
                cursor_world_pos,
                snapped_cursor_world_pos,
            ),
            Tool::Erase(tool) => {
                tool.tick(input, graph, cursor_world_pos, snapped_cursor_world_pos)
            }
            Tool::Edit(tool) => tool.tick(
                toolpane_gate,
                input,
                selection,
                graph,
                cursor_world_pos,
                snapped_cursor_world_pos,
            ),
            Tool::Interact(tool) => {
                tool.tick(input, graph, cursor_world_pos, snapped_cursor_world_pos)
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw<D: RaylibDraw>(
        &self,
        d: &mut D,
        theme: &Theme,
        input: &Inputs,
        toolpane: &ToolPane,
        graph: &Graph,
        tab: &EditorTab,
        sheet_and_width: Option<(&Texture2D, i32)>,
    ) {
        match self {
            Tool::Create(tool) => tool.draw(d, theme, input, toolpane, graph, tab, sheet_and_width),
            Tool::Erase(tool) => tool.draw(d, theme, input, toolpane, graph, tab, sheet_and_width),
            Tool::Edit(tool) => tool.draw(d, theme, input, toolpane, graph, tab, sheet_and_width),
            Tool::Interact(tool) => {
                tool.draw(d, theme, input, toolpane, graph, tab, sheet_and_width)
            }
        }
    }
}
