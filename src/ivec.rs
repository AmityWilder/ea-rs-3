use ext_trait::extension;
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

impl Extend<Vector2> for Bounds {
    #[inline]
    fn extend<T: IntoIterator<Item = Vector2>>(&mut self, iter: T) {
        for p in iter {
            self.include_point(p);
        }
    }
}

impl Extend<Self> for Bounds {
    #[inline]
    fn extend<T: IntoIterator<Item = Self>>(&mut self, iter: T) {
        for item in iter {
            self.include_bounds(item);
        }
    }
}

impl Bounds {
    pub const fn new(min: Vector2, max: Vector2) -> Self {
        Self { min, max }
    }

    #[inline]
    pub const fn minmax_points(p1: Vector2, p2: Vector2) -> Self {
        let (xmin, xmax) = if p1.x < p2.x {
            (p1.x, p2.x)
        } else {
            (p2.x, p1.x)
        };
        let (ymin, ymax) = if p1.y < p2.y {
            (p1.y, p2.y)
        } else {
            (p2.y, p1.y)
        };
        Self {
            min: Vector2 { x: xmin, y: ymin },
            max: Vector2 { x: xmax, y: ymax },
        }
    }

    #[inline]
    pub const fn include_point(&mut self, p: Vector2) {
        if p.x < self.min.x {
            self.min.x = p.x;
        }
        if p.y < self.min.y {
            self.min.y = p.y;
        }
        if p.x > self.max.x {
            self.max.x = p.x;
        }
        if p.y > self.max.y {
            self.max.y = p.y;
        }
    }

    #[inline]
    pub const fn include_bounds(&mut self, other: Self) {
        debug_assert!(other.min.x <= other.max.x && other.min.y <= other.max.y);
        if other.min.x < self.min.x {
            self.min.x = other.min.x;
        }
        if other.min.y < self.min.y {
            self.min.y = other.min.y;
        }
        if other.max.x > self.max.x {
            self.max.x = other.max.x;
        }
        if other.max.y > self.max.y {
            self.max.y = other.max.y;
        }
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

impl std::ops::Add for IVec2 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::iter::Sum for IVec2 {
    #[inline]
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|acc, e| acc + e).unwrap_or_default()
    }
}

impl std::ops::AddAssign for IVec2 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl std::ops::Sub for IVec2 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::SubAssign for IVec2 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl std::ops::Mul for IVec2 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl std::iter::Product for IVec2 {
    #[inline]
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.reduce(|acc, e| acc * e).unwrap_or(Self::new(1, 1))
    }
}

impl std::ops::MulAssign for IVec2 {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl std::ops::Div for IVec2 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

impl std::ops::DivAssign for IVec2 {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs
    }
}

impl std::ops::Rem for IVec2 {
    type Output = Self;

    #[inline]
    fn rem(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x % rhs.x,
            y: self.y % rhs.y,
        }
    }
}

impl std::ops::RemAssign for IVec2 {
    #[inline]
    fn rem_assign(&mut self, rhs: Self) {
        *self = *self % rhs
    }
}

impl std::ops::Mul<i32> for IVec2 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::MulAssign<i32> for IVec2 {
    #[inline]
    fn mul_assign(&mut self, rhs: i32) {
        *self = *self * rhs
    }
}

impl std::ops::Div<i32> for IVec2 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl std::ops::DivAssign<i32> for IVec2 {
    #[inline]
    fn div_assign(&mut self, rhs: i32) {
        *self = *self / rhs
    }
}

impl std::ops::Rem<i32> for IVec2 {
    type Output = Self;

    #[inline]
    fn rem(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x % rhs,
            y: self.y % rhs,
        }
    }
}

impl std::ops::RemAssign<i32> for IVec2 {
    #[inline]
    fn rem_assign(&mut self, rhs: i32) {
        *self = *self % rhs
    }
}

impl std::hash::Hash for IVec2 {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (((self.x as u64) << 32) | (self.y as u64)).hash(state);
    }
}

#[extension(pub trait AsIVec2)]
impl Vector2 {
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

    pub const fn one() -> Self {
        Self { x: 1, y: 1 }
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

    #[inline]
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

#[allow(dead_code, reason = "reflexivity with as_rect()")]
#[extension(pub trait AsIRect)]
impl Rectangle {
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

impl Extend<IVec2> for IBounds {
    #[inline]
    fn extend<T: IntoIterator<Item = IVec2>>(&mut self, iter: T) {
        for IVec2 { x, y } in iter {
            if x < self.min.x {
                self.min.x = x;
            }
            if y < self.min.y {
                self.min.y = y;
            }
            if x > self.max.x {
                self.max.x = x;
            }
            if y > self.max.y {
                self.max.y = y;
            }
        }
    }
}

impl Extend<Self> for IBounds {
    #[inline]
    fn extend<T: IntoIterator<Item = Self>>(&mut self, iter: T) {
        for Self { min, max } in iter {
            debug_assert!(min.x <= max.x && min.y <= max.y);
            if min.x < self.min.x {
                self.min.x = min.x;
            }
            if min.y < self.min.y {
                self.min.y = min.y;
            }
            if max.x > self.max.x {
                self.max.x = max.x;
            }
            if max.y > self.max.y {
                self.max.y = max.y;
            }
        }
    }
}

impl IBounds {
    pub const fn new(min: IVec2, max: IVec2) -> Self {
        Self { min, max }
    }

    #[inline]
    pub const fn y(&self) -> std::ops::RangeInclusive<i32> {
        self.min.y..=self.max.y
    }

    #[inline]
    pub const fn x(&self) -> std::ops::RangeInclusive<i32> {
        self.min.x..=self.max.x
    }

    #[inline]
    pub const fn contains(&self, p: IVec2) -> bool {
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

    #[inline]
    pub const fn area(&self) -> i32 {
        self.width() * self.height()
    }
}
