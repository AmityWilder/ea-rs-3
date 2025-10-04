use crate::{input::Inputs, ivec::Bounds, theme::Theme};
use raylib::prelude::*;
use serde_derive::{Deserialize, Serialize};

pub type SizingBound = fn(&Theme, f32, f32) -> Option<f32>;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ExactSizing {
    pub val: f32,
    /// f(theme, container_size, content_size)
    #[serde(skip)]
    pub min: Option<SizingBound>,
    /// f(theme, container_size, content_size)
    #[serde(skip)]
    pub max: Option<SizingBound>,
}

impl ExactSizing {
    pub fn set_clamped(
        &mut self,
        theme: &Theme,
        container_size: f32,
        content_size: f32,
        mut value: f32,
    ) {
        if let Some(lower) = self
            .min
            .and_then(|f| f(theme, container_size, content_size))
            && value < lower
        {
            value = lower;
        }

        if let Some(upper) = self
            .min
            .and_then(|f| f(theme, container_size, content_size))
            && value > upper
        {
            value = upper;
        }

        self.val = value;
    }
}

/// No-container sizing
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NcSizing {
    #[default]
    FitContent,
    Exact(ExactSizing),
}

impl NcSizing {
    pub const fn get(self, content_size: f32) -> f32 {
        match self {
            Self::FitContent => content_size,
            Self::Exact(x) => x.val,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sizing {
    FitContent,
    Exact(ExactSizing),
    #[default]
    Fill,
}

impl Sizing {
    pub const fn get(self, container_size: f32, content_size: f32) -> f32 {
        match self {
            Self::FitContent => content_size,
            Self::Exact(x) => x.val,
            Self::Fill => container_size,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Anchoring {
    Left {
        w: Sizing,
    },
    TopLeft {
        w: Sizing,
        h: Sizing,
    },
    Top {
        h: Sizing,
    },
    TopRight {
        w: Sizing,
        h: Sizing,
    },
    Right {
        w: Sizing,
    },
    BottomRight {
        w: Sizing,
        h: Sizing,
    },
    Bottom {
        h: Sizing,
    },
    BottomLeft {
        w: Sizing,
        h: Sizing,
    },
    #[serde(untagged)]
    Floating {
        x: f32,
        y: f32,
        w: NcSizing,
        h: NcSizing,
    },
}

impl Anchoring {
    /// `(self bounds, remaining container bounds)`
    ///
    /// remaining container bounds is [`None`] if `self` is floating or doesn't split the container
    pub const fn bounds(
        &self,
        container: &Bounds,
        content_size: Vector2,
    ) -> (Bounds, Option<Bounds>) {
        match *self {
            Self::Left { w } => {
                let (left, right) = container
                    .split_left_right(container.min.x + w.get(container.width(), content_size.x));
                (left, Some(right))
            }

            Self::TopLeft { w, h } => (
                Bounds::new(
                    container.min,
                    Vector2::new(
                        container.min.x + w.get(container.width(), content_size.x),
                        container.min.y + h.get(container.height(), content_size.y),
                    ),
                ),
                None,
            ),

            Self::Top { h } => {
                let (top, bottom) = container
                    .split_top_bottom(container.min.y + h.get(container.height(), content_size.y));
                (top, Some(bottom))
            }

            Self::TopRight { w, h } => (
                Bounds::new(
                    Vector2::new(
                        container.max.x - w.get(container.width(), content_size.x),
                        container.min.y,
                    ),
                    Vector2::new(
                        container.max.x,
                        container.min.y + h.get(container.height(), content_size.y),
                    ),
                ),
                None,
            ),

            Self::Right { w } => {
                let (left, right) = container
                    .split_left_right(container.max.x - w.get(container.width(), content_size.x));
                (right, Some(left))
            }

            Self::BottomRight { w, h } => (
                Bounds::new(
                    Vector2::new(
                        container.max.x - w.get(container.width(), content_size.x),
                        container.max.y - h.get(container.height(), content_size.y),
                    ),
                    container.max,
                ),
                None,
            ),

            Self::Bottom { h } => {
                let (top, bottom) = container
                    .split_top_bottom(container.max.y - h.get(container.height(), content_size.y));
                (bottom, Some(top))
            }

            Self::BottomLeft { w, h } => (
                Bounds::new(
                    Vector2::new(
                        container.min.x,
                        container.max.y - h.get(container.height(), content_size.y),
                    ),
                    Vector2::new(
                        container.min.x + w.get(container.width(), content_size.x),
                        container.max.y,
                    ),
                ),
                None,
            ),

            Self::Floating { x, y, w, h } => (
                Bounds::new(
                    Vector2::new(x, y),
                    Vector2::new(x + w.get(content_size.x), y + h.get(content_size.y)),
                ),
                None,
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    #[default]
    Forward,
    Reverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    #[default]
    Expanded,
    Collapsed,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Orientation {
    Horizontal,
    #[default]
    Vertical,
}

/// May be relative to [`Orientation`]
/// - [`Orientation::Vertical`] - top/bottom = y, left/right = x
/// - [`Orientation::Horizontal`] - top/bottom = x, left/right = y
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Padding {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Padding {
    pub const fn amount(padding: f32) -> Self {
        Self {
            left: padding,
            top: padding,
            right: padding,
            bottom: padding,
        }
    }

    pub const fn block(x: f32, y: f32) -> Self {
        Self {
            left: x,
            top: y,
            right: x,
            bottom: y,
        }
    }

    pub const fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    pub const fn vertical(&self) -> f32 {
        self.top + self.bottom
    }

    #[inline]
    pub const fn rotate_cc(self) -> Self {
        Self {
            left: self.top,
            top: self.right,
            right: self.bottom,
            bottom: self.left,
        }
    }

    #[inline]
    pub const fn rotate_cw(self) -> Self {
        Self {
            left: self.bottom,
            top: self.left,
            right: self.top,
            bottom: self.right,
        }
    }

    #[inline]
    pub const fn rotate_180(self) -> Self {
        Self {
            left: self.right,
            top: self.bottom,
            right: self.left,
            bottom: self.top,
        }
    }
}

impl Bounds {
    pub const fn pad(&self, padding: &Padding) -> Self {
        Self {
            min: Vector2::new(self.min.x + padding.left, self.min.y + padding.top),
            max: Vector2::new(self.max.x - padding.right, self.max.y - padding.bottom),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Panel {
    pub title: &'static str,
    pub anchoring: Anchoring,
    pub padding: fn(&Theme) -> Padding,
    bounds: Bounds,
}

impl Panel {
    pub fn new(title: &'static str, anchoring: Anchoring, padding: fn(&Theme) -> Padding) -> Self {
        Self {
            title,
            anchoring,
            padding,
            bounds: Bounds::default(),
        }
    }

    /// returns new container bounds, if split
    pub fn update_bounds(
        &mut self,
        theme: &Theme,
        container: &Bounds,
        content_size: Vector2,
    ) -> Option<Bounds> {
        let padding = (self.padding)(theme);
        let (bounds, new_container) = self.anchoring.bounds(
            container,
            content_size + Vector2::new(padding.horizontal(), padding.vertical()),
        );
        self.bounds = bounds;
        new_container
    }

    pub fn bounds(&self) -> &Bounds {
        &self.bounds
    }

    pub fn content_bounds(&self, theme: &Theme) -> Bounds {
        self.bounds.pad(&(self.padding)(theme))
    }

    pub fn tick<T, F>(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        theme: &Theme,
        input: &Inputs,
        content: F,
    ) -> T
    where
        F: FnOnce(&mut RaylibHandle, &RaylibThread, Bounds, &Theme, &Inputs) -> T,
    {
        // todo: handle resizing unanchored bounds
        content(
            rl,
            thread,
            self.bounds.pad(&(self.padding)(theme)),
            theme,
            input,
        )
    }

    pub fn draw<T, D, F>(&self, d: &mut D, theme: &Theme, content: F) -> T
    where
        D: RaylibDraw,
        F: FnOnce(&mut D, Bounds, &Theme) -> T,
    {
        // background
        {
            let rec = Rectangle::from(self.bounds);
            d.draw_rectangle_rec(rec, theme.background2);
            d.draw_rectangle_rec(
                Rectangle {
                    x: rec.x + 1.0,
                    y: rec.y + 1.0,
                    width: rec.width - 2.0,
                    height: rec.height - 2.0,
                },
                theme.background1,
            );
        }

        // content
        let res = content(d, self.bounds.pad(&(self.padding)(theme)), theme);

        // title
        if !self.title.is_empty() {
            let title_text_size = theme.title_font.measure_text(self.title);
            let title_width = title_text_size.x + theme.title_padding.horizontal();
            let title_height = title_text_size.y + theme.title_padding.vertical();
            d.draw_rectangle_rec(
                Rectangle::new(
                    self.bounds.max.x - title_width,
                    self.bounds.min.y,
                    title_width,
                    title_height,
                ),
                theme.background2,
            );
            theme.title_font.draw_text(
                d,
                self.title,
                Vector2::new(
                    self.bounds.max.x - title_width + theme.title_padding.left,
                    self.bounds.min.y + theme.title_padding.top,
                ),
                theme.foreground,
            );
        }

        res
    }
}
