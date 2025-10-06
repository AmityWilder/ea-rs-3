use crate::{
    GRID_SIZE, IVec2, Theme,
    console::Console,
    graph::{
        Graph,
        node::GateNtd,
        wire::{Flow, Wire},
    },
    icon_sheets::{NodeIconSheetId, NodeIconSheetSetId},
    input::Inputs,
    ivec::{AsIVec2, Bounds},
    tool::{EditDragging, Tool},
    toolpane::ToolPane,
    ui::Panel,
};
use raylib::prelude::*;
use std::sync::{RwLock, Weak};

#[derive(Debug)]
pub struct EditorGrid {
    pub shader: Shader,
    offset_loc: i32,
    zoom_exp_loc: i32,
}

impl EditorGrid {
    pub fn new(shader: Shader) -> Self {
        Self {
            offset_loc: shader.get_shader_location("offset"),
            zoom_exp_loc: shader.get_shader_location("zoom_exp"),
            shader,
        }
    }

    #[inline]
    pub fn set_offset(&mut self, value: Vector2) {
        self.shader.set_shader_value(self.offset_loc, value);
    }

    #[inline]
    pub fn set_zoom_exp(&mut self, value: f32) {
        self.shader.set_shader_value(self.zoom_exp_loc, value);
    }
}

#[derive(Debug)]
pub struct EditorTab {
    camera_target: Vector2,
    zoom_exp: f32,
    grid: RenderTexture2D,
    dirty: bool,
    pub graph: Weak<RwLock<Graph>>,
}

impl EditorTab {
    pub fn new(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        width: u32,
        height: u32,
        graph: Weak<RwLock<Graph>>,
    ) -> Result<Self, raylib::error::Error> {
        let grid = rl.load_render_texture(thread, width, height)?;
        Ok(Self {
            camera_target: Vector2::zero(),
            zoom_exp: 0.0,
            grid,
            dirty: true,
            graph,
        })
    }

    #[inline]
    pub const fn zoom_exp(&self) -> f32 {
        self.zoom_exp
    }

    #[inline]
    pub fn camera(&self) -> Camera2D {
        Camera2D {
            offset: Vector2::zero(),
            target: self.camera_target,
            rotation: 0.0,
            zoom: 2.0f32.powf(self.zoom_exp),
        }
    }

    /// `pan_speed` is scaled by zoom (zoom applied first)
    pub fn zoom_and_pan(
        &mut self,
        origin: Vector2,
        pan: Vector2,
        zoom: f32,
        pan_speed: f32,
        editorgrid: &mut EditorGrid,
    ) {
        if zoom != 0.0 {
            let new_zoom = (self.zoom_exp + zoom).clamp(-3.0, 2.0);
            if self.zoom_exp != new_zoom {
                self.camera_target += origin / 2.0f32.powf(self.zoom_exp);
                self.zoom_exp = new_zoom;
                self.camera_target -= origin / 2.0f32.powf(self.zoom_exp);
                self.dirty = true;
            }
        }
        if pan.length_sqr() > 0.0 {
            const LO: f32 = (i32::MIN as f32).next_up();
            const HI: f32 = (i32::MAX as f32).next_down();
            #[allow(clippy::absurd_extreme_comparisons, reason = "outright untrue")]
            const _: () = {
                assert!((LO as i32) >= i32::MIN);
                assert!((HI as i32) <= i32::MAX);
            };
            let pan_speed = pan_speed * 2.0f32.powf(-self.zoom_exp);
            let new_pan = Vector2 {
                x: (self.camera_target.x + pan.x * pan_speed).clamp(LO, HI),
                y: (self.camera_target.y + pan.y * pan_speed).clamp(LO, HI),
            };
            if self.camera_target != new_pan {
                self.camera_target = new_pan;
                self.dirty = true;
            }
        }
        editorgrid.set_offset(self.camera_target);
        editorgrid.set_zoom_exp(self.zoom_exp);
    }

    pub fn resize(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        new_width: i32,
        new_height: i32,
    ) -> Result<(), raylib::error::Error> {
        if new_width != self.grid.width() || new_height != self.grid.height() {
            self.grid = rl.load_render_texture(
                thread,
                new_width.try_into().unwrap(),
                new_height.try_into().unwrap(),
            )?;
            self.dirty = true;
        }
        Ok(())
    }

    pub fn refresh_grid(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        theme: &Theme,
        viewport: &Bounds,
    ) {
        if self.dirty {
            self.dirty = false;

            let camera = self.camera();

            let mut start = IVec2::from_vec2(rl.get_screen_to_world2D(viewport.min, camera));
            let mut end = IVec2::from_vec2(rl.get_screen_to_world2D(viewport.max, camera));

            start = start.snap(GRID_SIZE.into());
            start.x -= i32::from(GRID_SIZE);
            start.y -= i32::from(GRID_SIZE);

            end = end.snap(GRID_SIZE.into());
            end.x += i32::from(GRID_SIZE);
            end.y += i32::from(GRID_SIZE);

            let mut d = rl.begin_texture_mode(thread, &mut self.grid);
            d.clear_background(Color::BLANK);
            {
                let mut d = d.begin_mode2D(camera);
                if camera.zoom.recip() >= f32::from(GRID_SIZE) {
                    // size of 1 pixel is smaller than a grid
                    d.clear_background(theme.background1);
                } else {
                    for y in (start.y..=end.y).step_by(GRID_SIZE as usize) {
                        d.draw_line(start.x, y, end.x, y, theme.background1);
                    }
                    for x in (start.x..=end.x).step_by(GRID_SIZE as usize) {
                        d.draw_line(x, start.y, x, end.y, theme.background1);
                    }
                }
                d.draw_line(start.x, 0, end.x, 0, theme.background2);
                d.draw_line(0, start.y, 0, end.y, theme.background2);
            }
        }
    }

    #[inline]
    pub fn grid_tex(&self) -> &WeakTexture2D {
        self.grid.texture()
    }

    #[inline]
    pub fn screen_to_world(&self, screen_pos: Vector2) -> Vector2 {
        // SAFETY: GetScreenToWorld2D is a pure function with no preconditions
        unsafe { ffi::GetScreenToWorld2D(screen_pos.into(), self.camera().into()) }.into()
    }

    #[inline]
    pub fn world_to_screen(&self, world_pos: Vector2) -> Vector2 {
        // SAFETY: GetWorldToScreen2D is a pure function with no preconditions
        unsafe { ffi::GetWorldToScreen2D(world_pos.into(), self.camera().into()) }.into()
    }

    pub fn tick(
        &mut self,
        console: &mut Console,
        toolpane: &mut ToolPane,
        _theme: &Theme,
        input: &Inputs,
        editorgrid: &mut EditorGrid,
    ) -> bool {
        let mut is_dirty = false;

        if let Some(gate) = input.gate() {
            toolpane.set_gate(gate, console);
        }
        if let Some(tool) = input.tool() {
            toolpane.set_tool(tool, console);
        }

        self.zoom_and_pan(input.cursor, input.pan, input.zoom, 5.0, editorgrid);

        // `try_write`: if graph is being borrowed, don't edit it! it might be saving!
        if let Some(graph) = self.graph.upgrade()
            && let Ok(mut graph) = graph.try_write()
        {
            let pos = self
                .screen_to_world(input.cursor)
                .as_ivec2()
                .snap(GRID_SIZE.into());

            match &mut toolpane.tool {
                Tool::Create { current_node } => {
                    if input.primary.is_starting() {
                        if let Some(&id) = graph.find_node_at(pos) {
                            // existing node
                            if let Some(current_node) = *current_node
                                && current_node != id
                            {
                                _ = graph.create_wire(toolpane.elbow, current_node, id, console);
                            }
                            *current_node = Some(id);
                        } else {
                            // new node
                            let gate = toolpane.gate;
                            let new_node = graph
                                .create_node(gate, pos, console)
                                .expect("this branch implies the position is available");
                            let new_node_id = *new_node.id();
                            if let Some(current_node) = current_node.as_ref() {
                                _ = graph.create_wire(
                                    toolpane.elbow,
                                    *current_node,
                                    new_node_id,
                                    console,
                                );
                            }
                            *current_node = Some(new_node_id);
                        }
                        is_dirty = true;
                    }
                    if input.secondary.is_starting() {
                        *current_node = None;
                    }
                }

                Tool::Erase {} => {
                    if input.primary.is_starting()
                        && let Some(&id) = graph.find_node_at(pos)
                    {
                        graph
                            .destroy_node(&id, false, console)
                            .expect("cannot reach this branch if graph did not contain the node");
                        is_dirty = true;
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
                        let new_position = self
                            .screen_to_world(input.cursor)
                            .as_ivec2()
                            .snap(GRID_SIZE.into());
                        graph
                            .translate_node(&id, new_position, console)
                            .expect("edit mode target node should be valid");
                    }

                    if let Some(EditDragging { temp_pos, id: _ }) = target.as_mut() {
                        *temp_pos = self.screen_to_world(input.cursor)
                            - rvec2(GRID_SIZE / 2, GRID_SIZE / 2);
                    }
                }

                Tool::Interact {} => {
                    if input.primary.is_starting()
                        && let Some(&id) = graph.find_node_at(pos)
                        && graph.is_inputless(&id)
                    {
                        let node = graph.node_mut(&id).expect("all nodes should be valid");
                        match node.gate_ntd_mut() {
                            gate @ GateNtd::Or => {
                                *gate = GateNtd::Nor;
                                is_dirty = true;
                            }
                            gate @ GateNtd::Nor => {
                                *gate = GateNtd::Or;
                                is_dirty = true;
                            }
                            _ => {}
                        };
                    }
                }
            }
        }
        is_dirty
    }

    pub fn draw<D: RaylibDraw>(
        &self,
        d: &mut D,
        bounds: &Bounds,
        theme: &Theme,
        input: &Inputs,
        toolpane: &ToolPane,
        _editorgrid: &mut EditorGrid,
    ) {
        let Rectangle {
            x,
            y,
            width,
            height,
        } = Rectangle::from(*bounds);
        #[cfg(false)]
        {
            let mut _d = d.begin_shader_mode(&mut editorgrid.shader);
            // SAFETY: exclusive access to RaylibDraw guarantees all rlgl requirements are met
            unsafe {
                ffi::rlBegin(ffi::RL_QUADS as i32);
                {
                    ffi::rlColor4ub(255, 255, 255, 255);
                    ffi::rlTexCoord2f(0.0, 0.0);
                    ffi::rlVertex2f(x, y);
                    ffi::rlTexCoord2f(0.0, 1.0);
                    ffi::rlVertex2f(x, y + height);
                    ffi::rlTexCoord2f(1.0, 1.0);
                    ffi::rlVertex2f(x + width, y + height);
                    ffi::rlTexCoord2f(1.0, 0.0);
                    ffi::rlVertex2f(x + width, y);
                }
                ffi::rlEnd();
            }
        }
        let mut d = d.begin_scissor_mode(x as i32, y as i32, width as i32, height as i32);
        d.draw_texture_pro(
            self.grid_tex(),
            Rectangle::new(x, y, width, -height),
            Rectangle::new(x, y, width, height),
            Vector2::zero(),
            0.0,
            Color::WHITE,
        );
        let mut d = d.begin_mode2D(self.camera());
        let zoom_exp = self.zoom_exp().ceil() as i32;
        let scale_and_width =
            NodeIconSheetSetId::from_zoom_exp(zoom_exp).map(|scale| (scale, scale.icon_width()));
        if let Some(graph) = self.graph.upgrade() {
            let graph = graph.try_read().unwrap();

            // tool - background layer
            match &toolpane.tool {
                Tool::Create { current_node: _ } => {}
                Tool::Erase {} => {}
                Tool::Edit { target: _ } => {}
                Tool::Interact {} => {}
            }

            // wires
            for wire in graph.wires_iter() {
                wire.draw(
                    &mut d,
                    &graph,
                    rvec2(GRID_SIZE / 2, GRID_SIZE / 2),
                    theme.foreground,
                )
                .expect("all wires should be valid");
            }

            // tool - wire layer
            match &toolpane.tool {
                Tool::Create { current_node } => {
                    if let Some(&current_node) = current_node.as_ref() {
                        Wire::draw_immediate(
                            &mut d,
                            graph
                                .node(&current_node)
                                .expect("current node should always be valid")
                                .position()
                                .as_vec2()
                                + rvec2(GRID_SIZE / 2, GRID_SIZE / 2),
                            self.screen_to_world(input.cursor),
                            toolpane.elbow,
                            theme.foreground,
                        );
                    }
                }

                Tool::Erase {} => {}

                Tool::Edit { target } => {
                    if let Some(EditDragging { temp_pos, id }) = target {
                        for (_, wire, flow) in graph.wires_of(id) {
                            let (start_pos, end_pos) = match flow {
                                Flow::Input => (
                                    graph
                                        .node(wire.src())
                                        .expect("all wires should be valid")
                                        .position()
                                        .as_vec2()
                                        + rvec2(GRID_SIZE / 2, GRID_SIZE / 2),
                                    *temp_pos + rvec2(GRID_SIZE / 2, GRID_SIZE / 2),
                                ),
                                Flow::Output => (
                                    *temp_pos + rvec2(GRID_SIZE / 2, GRID_SIZE / 2),
                                    graph
                                        .node(wire.dst())
                                        .expect("all wires should be valid")
                                        .position()
                                        .as_vec2()
                                        + rvec2(GRID_SIZE / 2, GRID_SIZE / 2),
                                ),
                                Flow::Loop => {
                                    todo!()
                                }
                            };
                            Wire::draw_immediate(
                                &mut d,
                                start_pos,
                                end_pos,
                                wire.elbow,
                                theme.special,
                            );
                        }
                        let node = graph.node(id).expect("node being dragged should be valid");
                        let rec = Rectangle {
                            x: temp_pos.x,
                            y: temp_pos.y,
                            width: GRID_SIZE.into(),
                            height: GRID_SIZE.into(),
                        };
                        let color = theme.special;
                        if let Some((scale, icon_width)) = scale_and_width {
                            d.draw_texture_pro(
                                &theme.node_icons[scale][NodeIconSheetId::Basic],
                                node.gate_ntd()
                                    .as_gate()
                                    .id()
                                    .icon_cell_irec(icon_width)
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
                }

                Tool::Interact {} => {}
            }

            // nodes
            for node in graph.nodes_iter() {
                let node_position = node.position().as_vec2();
                let rec = Rectangle {
                    x: node_position.x,
                    y: node_position.y,
                    width: GRID_SIZE.into(),
                    height: GRID_SIZE.into(),
                };
                let color = if node.state() {
                    theme.active
                } else {
                    theme.foreground
                };
                if let Some((scale, icon_width)) = scale_and_width {
                    let src_rec = node
                        .gate_ntd()
                        .as_gate()
                        .id()
                        .icon_cell_irec(icon_width)
                        .as_rec();
                    d.draw_texture_pro(
                        &theme.node_icons[scale][NodeIconSheetId::Background],
                        src_rec,
                        rec,
                        Vector2::zero(),
                        0.0,
                        theme.background,
                    );
                    d.draw_texture_pro(
                        &theme.node_icons[scale][NodeIconSheetId::Basic],
                        src_rec,
                        rec,
                        Vector2::zero(),
                        0.0,
                        color,
                    );
                    if let Some(color) = match *node.gate_ntd() {
                        GateNtd::Or
                        | GateNtd::And
                        | GateNtd::Nor
                        | GateNtd::Xor
                        | GateNtd::Battery
                        | GateNtd::Delay { .. } => None,
                        GateNtd::Resistor { resistance: n } | GateNtd::Led { color: n } => Some(
                            theme
                                .resistance
                                .get(n as usize)
                                .copied()
                                .expect("gate should never contain invalid NT data"),
                        ),
                        GateNtd::Capacitor { capacity, stored } => {
                            Some(theme.active.alpha(stored as f32 / capacity as f32))
                        }
                    } {
                        d.draw_texture_pro(
                            &theme.node_icons[scale][NodeIconSheetId::Ntd],
                            src_rec,
                            rec,
                            Vector2::zero(),
                            0.0,
                            color,
                        );
                    }
                } else {
                    d.draw_rectangle_rec(rec, color);
                }
            }

            // tool - nodes layer
            match &toolpane.tool {
                Tool::Create { current_node: _ } => {}
                Tool::Erase {} => {}
                Tool::Edit { target: _ } => {}
                Tool::Interact {} => {}
            }

            if let Some(id) = graph.find_node_at(
                self.screen_to_world(input.cursor)
                    .as_ivec2()
                    .snap(GRID_SIZE.into()),
            ) {
                let node = graph
                    .node(id)
                    .expect("find_node_at should never return an invalid node");
                let node_position = node.position().as_vec2();
                let rec = Rectangle {
                    x: node_position.x,
                    y: node_position.y,
                    width: GRID_SIZE.into(),
                    height: GRID_SIZE.into(),
                };
                let color = theme.special;
                if let Some((scale, icon_width)) = scale_and_width {
                    d.draw_texture_pro(
                        &theme.node_icons[scale][NodeIconSheetId::Highlight],
                        node.gate_ntd()
                            .as_gate()
                            .id()
                            .icon_cell_irec(icon_width)
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
        }
    }
}

#[derive(Debug)]
pub enum Tab {
    Editor(EditorTab),
}

#[derive(Debug)]
pub struct TabList {
    panel: Panel,
    tabs: Vec<Tab>,
    /// ignore if `tabs` is empty
    focused: usize,
}

impl Extend<Tab> for TabList {
    #[inline]
    fn extend<T: IntoIterator<Item = Tab>>(&mut self, iter: T) {
        self.tabs.extend(iter);
    }
}

impl std::ops::Deref for TabList {
    type Target = [Tab];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.tabs.as_slice()
    }
}

impl IntoIterator for TabList {
    type Item = Tab;
    type IntoIter = std::vec::IntoIter<Tab>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.tabs.into_iter()
    }
}

impl<'a> IntoIterator for &'a TabList {
    type Item = &'a Tab;
    type IntoIter = std::slice::Iter<'a, Tab>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.tabs.as_slice().iter()
    }
}

impl<'a> IntoIterator for &'a mut TabList {
    type Item = &'a mut Tab;
    type IntoIter = std::slice::IterMut<'a, Tab>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.tabs.as_mut_slice().iter_mut()
    }
}

impl TabList {
    pub const fn new(panel: Panel) -> Self {
        Self {
            panel,
            tabs: Vec::new(),
            focused: 0,
        }
    }

    pub fn with_tabs<I>(panel: Panel, tabs: I) -> Self
    where
        I: IntoIterator<Item = Tab>,
    {
        Self {
            panel,
            tabs: Vec::from_iter(tabs),
            focused: 0,
        }
    }

    #[inline]
    pub const fn panel(&self) -> &Panel {
        &self.panel
    }

    pub fn update_bounds(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        theme: &Theme,
        container: &Bounds,
    ) -> Result<Option<Bounds>, raylib::error::Error> {
        let res = self
            .panel
            .update_bounds(theme, container, Vector2::zero(/* todo */));
        let new_width = self.panel.bounds().width().ceil() as i32;
        let new_height = self.panel.bounds().height().ceil() as i32;
        for tab in &mut self.tabs {
            match tab {
                Tab::Editor(tab) => tab.resize(rl, thread, new_width, new_height)?,
            }
        }
        Ok(res)
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.tabs.len()
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    #[inline]
    pub const fn focused_tab(&self) -> Option<&Tab> {
        if self.tabs.is_empty() {
            None
        } else {
            Some(&self.tabs.as_slice()[self.focused])
        }
    }

    #[inline]
    pub const fn focused_tab_mut(&mut self) -> Option<&mut Tab> {
        if self.tabs.is_empty() {
            None
        } else {
            Some(&mut self.tabs.as_mut_slice()[self.focused])
        }
    }

    /// Returns an error if `tab` is out of range
    #[inline]
    pub const fn focus(&mut self, tab: usize) -> Result<(), ()> {
        if tab < self.tabs.len() {
            self.focused = tab;
            Ok(())
        } else {
            Err(())
        }
    }

    #[inline]
    pub fn push(&mut self, tab: Tab) {
        self.tabs.push(tab);
    }

    #[inline]
    pub fn pop(&mut self) -> Option<Tab> {
        let popped = self.tabs.pop();
        if popped.is_some() && self.focused == self.tabs.len() {
            self.focused -= 1;
        }
        popped
    }

    #[inline]
    pub fn insert(&mut self, index: usize, tab: Tab) {
        if self.focused >= index {
            self.focused += 1;
        }
        self.tabs.insert(index, tab);
    }

    #[inline]
    pub fn remove(&mut self, index: usize) -> Tab {
        let removed = self.tabs.remove(index);
        if self.focused > index {
            self.focused -= 1;
        }
        removed
    }

    #[inline]
    pub fn retain<F: FnMut(&Tab) -> bool>(&mut self, mut f: F) {
        let mut i = 0;
        let mut shift = 0;
        self.tabs.retain_mut(|tab| {
            let keep = f(tab);
            if i <= self.focused {
                if i < self.focused && !keep {
                    shift += 1;
                }
                i += 1;
            }
            keep
        });
        self.focused -= shift;
    }

    #[inline]
    pub fn retain_mut<F: FnMut(&mut Tab) -> bool>(&mut self, mut f: F) {
        let mut i = 0;
        let mut shift = 0;
        self.tabs.retain_mut(|tab| {
            let keep = f(tab);
            if i <= self.focused {
                if i < self.focused && !keep {
                    shift += 1;
                }
                i += 1;
            }
            keep
        });
        self.focused -= shift;
    }

    /// Returns an error if `from_index` or `to_index` is out of range
    #[inline]
    pub fn reorder(&mut self, from_index: usize, to_index: usize) -> Result<(), ()> {
        use std::cmp::Ordering::*;
        if from_index < self.tabs.len() && to_index < self.tabs.len() {
            let (dir, range, rotate): (_, _, fn(&mut [Tab], usize)) =
                match from_index.cmp(&to_index) {
                    Less => (-1, from_index..to_index, <[_]>::rotate_left),
                    Equal => return Ok(()),
                    Greater => (1, to_index..from_index, <[_]>::rotate_right),
                };

            let slice = &mut self.tabs[range.clone()];
            rotate(slice, 1);
            if self.focused == from_index {
                self.focused = to_index;
            } else if range.contains(&self.focused) {
                self.focused = self.focused.strict_add_signed(dir);
            }

            Ok(())
        } else {
            Err(())
        }
    }

    #[inline]
    pub fn editors(&self) -> impl DoubleEndedIterator<Item = &EditorTab> + Clone {
        self.tabs.iter().map(|tab| match tab {
            Tab::Editor(tab) => tab,
            // _ => None,
        })
    }

    #[inline]
    pub fn editors_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut EditorTab> {
        self.tabs.iter_mut().map(|tab| match tab {
            Tab::Editor(tab) => tab,
            // _ => None,
        })
    }

    #[inline]
    pub fn editors_of_graph(
        &self,
        graph: &Weak<RwLock<Graph>>,
    ) -> impl DoubleEndedIterator<Item = &EditorTab> + Clone {
        self.tabs.iter().filter_map(|tab| match tab {
            Tab::Editor(tab) if tab.graph.ptr_eq(graph) => Some(tab),
            _ => None,
        })
    }

    #[inline]
    pub fn editors_of_graph_mut(
        &mut self,
        graph: &Weak<RwLock<Graph>>,
    ) -> impl DoubleEndedIterator<Item = &mut EditorTab> {
        self.tabs.iter_mut().filter_map(|tab| match tab {
            Tab::Editor(tab) if tab.graph.ptr_eq(graph) => Some(tab),
            _ => None,
        })
    }
}
