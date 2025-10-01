use crate::{
    GRID_SIZE, IVec2, Theme,
    graph::{
        Graph,
        wire::{Flow, Wire},
    },
    icon_sheets::{NodeIconSheetId, NodeIconSheetSets},
    input::Inputs,
    ivec::{AsIVec2, Bounds},
    tool::{EditDragging, Tool},
    toolpane::ToolPane,
};
use raylib::prelude::*;
use std::sync::{RwLock, Weak};

#[derive(Debug)]
pub struct EditorTab {
    camera_target: Vector2,
    zoom_exp: f32,
    bounds: Bounds,
    grid: RenderTexture2D,
    dirty: bool,
    pub graph: Weak<RwLock<Graph>>,
}

impl EditorTab {
    pub fn new(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        bounds: Bounds,
        graph: Weak<RwLock<Graph>>,
    ) -> Result<Self, raylib::error::Error> {
        let grid = rl.load_render_texture(
            thread,
            u32::try_from((bounds.max.x - bounds.min.x).ceil() as i32).unwrap(),
            u32::try_from((bounds.max.y - bounds.min.y).ceil() as i32).unwrap(),
        )?;
        Ok(Self {
            camera_target: Vector2::zero(),
            zoom_exp: 0.0,
            bounds,
            grid,
            dirty: true,
            graph,
        })
    }

    pub const fn zoom_exp(&self) -> f32 {
        self.zoom_exp
    }

    pub fn camera(&self) -> Camera2D {
        Camera2D {
            offset: Vector2::zero(),
            target: self.camera_target,
            rotation: 0.0,
            zoom: 2.0f32.powf(self.zoom_exp),
        }
    }

    /// `pan_speed` is scaled by zoom (zoom applied first)
    pub fn zoom_and_pan(&mut self, origin: Vector2, pan: Vector2, zoom: f32, pan_speed: f32) {
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
    }

    pub const fn bounds(&self) -> &Bounds {
        &self.bounds
    }

    pub fn update_bounds(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        value: Bounds,
    ) -> Result<(), raylib::error::Error> {
        if self.bounds != value {
            self.bounds = value;
            self.grid = rl.load_render_texture(
                thread,
                u32::try_from((value.max.x - value.min.x).ceil() as i32).unwrap(),
                u32::try_from((value.max.y - value.min.y).ceil() as i32).unwrap(),
            )?;
            self.dirty = true;
        }
        Ok(())
    }

    pub fn refresh_grid(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, theme: &Theme) {
        if self.dirty {
            self.dirty = false;

            let camera = self.camera();

            let mut start = IVec2::from_vec2(rl.get_screen_to_world2D(self.bounds.min, camera));
            let mut end = IVec2::from_vec2(rl.get_screen_to_world2D(self.bounds.max, camera));

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

    pub fn grid_tex(&self) -> &WeakTexture2D {
        self.grid.texture()
    }

    pub fn screen_to_world(&self, screen_pos: Vector2) -> Vector2 {
        // SAFETY: GetScreenToWorld2D is a pure function with no preconditions
        unsafe { ffi::GetScreenToWorld2D(screen_pos.into(), self.camera().into()) }.into()
    }

    pub fn world_to_screen(&self, world_pos: Vector2) -> Vector2 {
        // SAFETY: GetWorldToScreen2D is a pure function with no preconditions
        unsafe { ffi::GetWorldToScreen2D(world_pos.into(), self.camera().into()) }.into()
    }

    pub fn draw<D: RaylibDraw>(
        &self,
        d: &mut D,
        theme: &Theme,
        input: &Inputs,
        toolpane: &ToolPane,
        node_icon_sheets: &NodeIconSheetSets,
    ) {
        let Rectangle {
            x,
            y,
            width,
            height,
        } = Rectangle::from(*self.bounds());
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
                        node_icon_sheets.draw(
                            &mut d,
                            zoom_exp,
                            NodeIconSheetId::Basic,
                            Rectangle {
                                x: temp_pos.x,
                                y: temp_pos.y,
                                width: GRID_SIZE.into(),
                                height: GRID_SIZE.into(),
                            },
                            node.gate,
                            Vector2::zero(),
                            0.0,
                            theme.special,
                        );
                    }
                }

                Tool::Interact {} => {}
            }

            // nodes
            for node in graph.nodes_iter() {
                let node_position = node.position().as_vec2();
                node_icon_sheets.draw(
                    &mut d,
                    zoom_exp,
                    NodeIconSheetId::Basic,
                    Rectangle {
                        x: node_position.x,
                        y: node_position.y,
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
                node_icon_sheets.draw(
                    &mut d,
                    zoom_exp,
                    NodeIconSheetId::Highlight,
                    Rectangle {
                        x: node_position.x,
                        y: node_position.y,
                        width: GRID_SIZE.into(),
                        height: GRID_SIZE.into(),
                    },
                    node.gate,
                    Vector2::zero(),
                    0.0,
                    theme.special,
                );
            }
        }
    }
}

#[derive(Debug)]
pub enum Tab {
    Editor(EditorTab),
}

#[derive(Debug, Default)]
pub struct TabList {
    tabs: Vec<Tab>,
    /// ignore if `tabs` is empty
    focused: usize,
}

impl<T: Into<Vec<Tab>>> From<T> for TabList {
    fn from(value: T) -> Self {
        Self {
            tabs: value.into(),
            focused: 0,
        }
    }
}

impl FromIterator<Tab> for TabList {
    fn from_iter<T: IntoIterator<Item = Tab>>(iter: T) -> Self {
        Self {
            tabs: Vec::from_iter(iter),
            focused: 0,
        }
    }
}

impl Extend<Tab> for TabList {
    fn extend<T: IntoIterator<Item = Tab>>(&mut self, iter: T) {
        self.tabs.extend(iter);
    }
}

impl std::ops::Deref for TabList {
    type Target = [Tab];

    fn deref(&self) -> &Self::Target {
        self.tabs.as_slice()
    }
}

impl IntoIterator for TabList {
    type Item = Tab;
    type IntoIter = std::vec::IntoIter<Tab>;

    fn into_iter(self) -> Self::IntoIter {
        self.tabs.into_iter()
    }
}

impl<'a> IntoIterator for &'a TabList {
    type Item = &'a Tab;
    type IntoIter = std::slice::Iter<'a, Tab>;

    fn into_iter(self) -> Self::IntoIter {
        self.tabs.as_slice().iter()
    }
}

impl<'a> IntoIterator for &'a mut TabList {
    type Item = &'a mut Tab;
    type IntoIter = std::slice::IterMut<'a, Tab>;

    fn into_iter(self) -> Self::IntoIter {
        self.tabs.as_mut_slice().iter_mut()
    }
}

impl TabList {
    pub const fn new() -> Self {
        Self {
            tabs: Vec::new(),
            focused: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.tabs.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    pub const fn focused_tab(&self) -> Option<&Tab> {
        if self.tabs.is_empty() {
            None
        } else {
            Some(&self.tabs.as_slice()[self.focused])
        }
    }

    pub const fn focused_tab_mut(&mut self) -> Option<&mut Tab> {
        if self.tabs.is_empty() {
            None
        } else {
            Some(&mut self.tabs.as_mut_slice()[self.focused])
        }
    }

    /// Returns an error if `tab` is out of range
    pub const fn focus(&mut self, tab: usize) -> Result<(), ()> {
        if tab < self.tabs.len() {
            self.focused = tab;
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn push(&mut self, tab: Tab) {
        self.tabs.push(tab);
    }

    pub fn pop(&mut self) -> Option<Tab> {
        let popped = self.tabs.pop();
        if popped.is_some() && self.focused == self.tabs.len() {
            self.focused -= 1;
        }
        popped
    }

    pub fn insert(&mut self, index: usize, tab: Tab) {
        if self.focused >= index {
            self.focused += 1;
        }
        self.tabs.insert(index, tab);
    }

    pub fn remove(&mut self, index: usize) -> Tab {
        let removed = self.tabs.remove(index);
        if self.focused > index {
            self.focused -= 1;
        }
        removed
    }

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

    pub fn editors(&self) -> impl DoubleEndedIterator<Item = &EditorTab> + Clone {
        self.tabs.iter().map(|tab| match tab {
            Tab::Editor(tab) => tab,
            // _ => None,
        })
    }

    pub fn editors_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut EditorTab> {
        self.tabs.iter_mut().map(|tab| match tab {
            Tab::Editor(tab) => tab,
            // _ => None,
        })
    }

    pub fn editors_of_graph(
        &self,
        graph: &Weak<RwLock<Graph>>,
    ) -> impl DoubleEndedIterator<Item = &EditorTab> + Clone {
        self.tabs.iter().filter_map(|tab| match tab {
            Tab::Editor(tab) if tab.graph.ptr_eq(graph) => Some(tab),
            _ => None,
        })
    }

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
