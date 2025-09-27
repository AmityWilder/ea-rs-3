use crate::{GRID_SIZE, IBounds, IVec2, Theme};
use raylib::prelude::*;

#[derive(Debug)]
pub struct EditorTab {
    camera_target: Vector2,
    zoom_exp: f32,
    bounds: IBounds,
    grid: RenderTexture2D,
    dirty: bool,
}

impl EditorTab {
    pub fn new(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        bounds: IBounds,
    ) -> Result<Self, raylib::error::Error> {
        let grid = rl.load_render_texture(
            thread,
            bounds.max.x.abs_diff(bounds.min.x),
            bounds.max.y.abs_diff(bounds.min.y),
        )?;
        Ok(Self {
            camera_target: Vector2::zero(),
            zoom_exp: 0.0,
            bounds,
            grid,
            dirty: true,
        })
    }

    pub fn camera(&self) -> Camera2D {
        Camera2D {
            offset: Vector2::zero(),
            target: self.camera_target,
            rotation: 0.0,
            zoom: 2.0f32.powf(self.zoom_exp),
        }
    }

    pub const fn zoom(&mut self, amount: f32) {
        let new_zoom = (self.zoom_exp + amount).clamp(-3.0, 2.0);
        if self.zoom_exp != new_zoom {
            self.zoom_exp = new_zoom;
            self.dirty = true;
        }
    }

    pub const fn bounds(&self) -> &IBounds {
        &self.bounds
    }

    pub fn update_bounds(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        value: IBounds,
    ) -> Result<(), raylib::error::Error> {
        if self.bounds != value {
            self.bounds = value;
            self.grid = rl.load_render_texture(
                thread,
                value.max.x.abs_diff(value.min.x),
                value.max.y.abs_diff(value.min.y),
            )?;
            self.dirty = true;
        }
        Ok(())
    }

    pub fn refresh_grid(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread, theme: &Theme) {
        if self.dirty {
            self.dirty = false;

            let camera = self.camera();

            let mut start =
                IVec2::from_vec2(rl.get_screen_to_world2D(self.bounds.min.as_vec2(), camera));

            let mut end =
                IVec2::from_vec2(rl.get_screen_to_world2D(self.bounds.max.as_vec2(), camera));

            // snap grid
            start.x = (start.x.cast_unsigned())
                .saturating_sub(1)
                .next_multiple_of(GRID_SIZE.into())
                .cast_signed();
            start.y = (start.y.cast_unsigned())
                .saturating_sub(1)
                .next_multiple_of(GRID_SIZE.into())
                .cast_signed();

            end.x = (end.x.cast_unsigned())
                .next_multiple_of(GRID_SIZE.into())
                .cast_signed();
            end.y = (end.y.cast_unsigned())
                .next_multiple_of(GRID_SIZE.into())
                .cast_signed();

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
            }
            // d.draw_text(&format!("{}", d.get_time()), 0, 0, 10, Color::MAGENTA);
        }
    }

    pub fn grid_tex(&self) -> &WeakTexture2D {
        self.grid.texture()
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
}
