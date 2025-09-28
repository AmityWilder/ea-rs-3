use raylib::prelude::*;

use crate::GRID_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IVec2 {
    pub x: i32,
    pub y: i32,
}

impl IVec2 {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub const fn zero() -> Self {
        Self { x: 0, y: 0 }
    }

    pub const fn as_vec2(self) -> Vector2 {
        Vector2 {
            x: self.x as f32,
            y: self.y as f32,
        }
    }

    pub const fn from_vec2(value: Vector2) -> Self {
        Self {
            x: value.x as i32,
            y: value.y as i32,
        }
    }

    pub const fn snap(self, grid_size: i32) -> Self {
        let x = self.x; // + self.x.signum() * grid_size / 2;
        let y = self.y; // + self.y.signum() * grid_size / 2;
        Self {
            x: x - (x % grid_size),
            y: y - (y % grid_size),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl IRect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    pub const fn as_rect(&self) -> Rectangle {
        Rectangle {
            x: self.x as f32,
            y: self.y as f32,
            width: self.w as f32,
            height: self.h as f32,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct IBounds {
    pub min: IVec2,
    pub max: IVec2,
}

impl From<IBounds> for IRect {
    fn from(value: IBounds) -> Self {
        IRect {
            x: value.min.x,
            y: value.min.y,
            w: value.max.x - value.min.x,
            h: value.max.y - value.min.y,
        }
    }
}

impl From<IRect> for IBounds {
    fn from(value: IRect) -> Self {
        IBounds {
            min: IVec2 {
                x: value.x,
                y: value.y,
            },
            max: IVec2 {
                x: value.x + value.w,
                y: value.y + value.h,
            },
        }
    }
}

impl IBounds {
    pub const fn new(min: IVec2, max: IVec2) -> Self {
        Self { min, max }
    }

    pub fn y(&self) -> std::ops::RangeInclusive<i32> {
        self.min.y..=self.max.y
    }

    pub fn x(&self) -> std::ops::RangeInclusive<i32> {
        self.min.x..=self.max.x
    }

    pub fn contains(&self, p: IVec2) -> bool {
        self.min.x <= p.x && p.x <= self.max.x && self.min.y <= p.y && p.y <= self.max.y
    }
}
