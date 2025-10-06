use raylib::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Bounds {
    pub min: Vector2,
    pub max: Vector2,
}

impl From<Bounds> for Rectangle {
    fn from(value: Bounds) -> Self {
        Rectangle {
            x: value.min.x,
            y: value.min.y,
            width: value.width(),
            height: value.height(),
        }
    }
}

impl From<Rectangle> for Bounds {
    fn from(value: Rectangle) -> Self {
        Bounds {
            min: Vector2 {
                x: value.x,
                y: value.y,
            },
            max: Vector2 {
                x: value.x + value.width,
                y: value.y + value.height,
            },
        }
    }
}

impl Bounds {
    pub const fn new(min: Vector2, max: Vector2) -> Self {
        Self { min, max }
    }

    #[inline]
    pub const fn contains(&self, p: Vector2) -> bool {
        self.min.x <= p.x && p.x < self.max.x && self.min.y <= p.y && p.y < self.max.y
    }

    #[inline]
    pub const fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    #[inline]
    pub const fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    #[inline]
    pub const fn split_left_right(self, x: f32) -> (Self, Self) {
        (
            Bounds::new(self.min, Vector2::new(x, self.max.y)),
            Bounds::new(Vector2::new(x, self.min.y), self.max),
        )
    }

    #[inline]
    pub const fn split_top_bottom(self, y: f32) -> (Self, Self) {
        (
            Bounds::new(self.min, Vector2::new(self.max.x, y)),
            Bounds::new(Vector2::new(self.min.x, y), self.max),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IVec2 {
    pub x: i32,
    pub y: i32,
}

impl std::hash::Hash for IVec2 {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (((self.x as u64) << 32) | (self.y as u64)).hash(state);
    }
}

pub trait AsIVec2 {
    fn as_ivec2(&self) -> IVec2;
}

impl AsIVec2 for Vector2 {
    #[inline]
    fn as_ivec2(&self) -> IVec2 {
        IVec2::from_vec2(*self)
    }
}

impl IVec2 {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub const fn zero() -> Self {
        Self { x: 0, y: 0 }
    }

    #[inline]
    pub const fn as_vec2(self) -> Vector2 {
        Vector2 {
            x: self.x as f32,
            y: self.y as f32,
        }
    }

    #[inline]
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

pub trait AsIRect {
    fn as_irect(&self) -> IRect;
}

impl AsIRect for Rectangle {
    #[inline]
    fn as_irect(&self) -> IRect {
        IRect {
            x: self.x as i32,
            y: self.y as i32,
            w: self.width as i32,
            h: self.height as i32,
        }
    }
}

impl IRect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }

    #[inline]
    pub const fn as_rec(&self) -> Rectangle {
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
    #[inline]
    fn from(value: IBounds) -> Self {
        IRect {
            x: value.min.x,
            y: value.min.y,
            w: value.width(),
            h: value.height(),
        }
    }
}

impl From<IRect> for IBounds {
    #[inline]
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

    #[inline]
    pub fn y(&self) -> std::ops::RangeInclusive<i32> {
        self.min.y..=self.max.y
    }

    #[inline]
    pub fn x(&self) -> std::ops::RangeInclusive<i32> {
        self.min.x..=self.max.x
    }

    #[inline]
    pub fn contains(&self, p: IVec2) -> bool {
        self.min.x <= p.x && p.x < self.max.x && self.min.y <= p.y && p.y < self.max.y
    }

    #[inline]
    pub const fn width(&self) -> i32 {
        self.max.x - self.min.x
    }

    #[inline]
    pub const fn height(&self) -> i32 {
        self.max.y - self.min.y
    }
}
