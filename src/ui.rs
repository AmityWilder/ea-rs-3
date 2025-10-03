use crate::ivec::Bounds;
use raylib::prelude::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Anchoring {
    Left,
    Top,
    Right,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sizing {
    Min,
    #[default]
    Fill,
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
#[serde(rename_all = "lowercase")]
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
    pub const fn pad(&self, padding: Padding) -> Self {
        Self {
            min: Vector2::new(self.min.x + padding.left, self.min.y + padding.top),
            max: Vector2::new(self.max.x - padding.right, self.max.y - padding.bottom),
        }
    }
}
