use crate::{
    ivec::IBounds,
    theme::{ColorId, Theme},
};
use raylib::prelude::*;
use std::{collections::VecDeque, fmt::Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorRef {
    Theme(ColorId),
    Exact(Color),
}

impl ColorRef {
    pub fn get(self, theme: &Theme) -> Color {
        match self {
            Self::Theme(id) => theme[id],
            Self::Exact(color) => color,
        }
    }
}

impl From<ColorId> for ColorRef {
    fn from(value: ColorId) -> Self {
        Self::Theme(value)
    }
}

impl From<Color> for ColorRef {
    fn from(value: Color) -> Self {
        Self::Exact(value)
    }
}

#[derive(Debug)]
pub struct Console {
    content: String,
    colors: VecDeque<(usize, ColorRef)>,
    capacity: usize,
    pub bounds: IBounds,
    pub left_anchored: bool,
    pub top_anchored: bool,
    pub right_anchored: bool,
    pub bottom_anchored: bool,
}

impl Console {
    pub const fn new(
        capacity: usize,
        bounds: IBounds,
        left_anchored: bool,
        top_anchored: bool,
        right_anchored: bool,
        bottom_anchored: bool,
    ) -> Self {
        Self {
            content: String::new(),
            colors: VecDeque::new(),
            capacity,
            bounds,
            left_anchored,
            top_anchored,
            right_anchored,
            bottom_anchored,
        }
    }

    /// NOTE: You will need to append with newline
    pub fn log<'a>(
        &mut self,
        text: impl IntoIterator<Item = (ColorRef, std::fmt::Arguments<'a>)>,
    ) -> std::fmt::Result {
        let it = text.into_iter();
        let (size_min, size_max) = it.size_hint();
        self.colors.reserve(size_max.unwrap_or(size_min));
        // cant reserve content because we dont know the len of each element without consuming the iterator
        for (color, args) in it {
            let start = self.content.len();
            self.content.write_fmt(args)?;
            let end = self.content.len();
            self.colors.push_back((end - start, color));
            let mut total_size = 0;
            let (sum, count) = self
                .colors
                .iter()
                .rev()
                .map(|(size, _)| *size)
                .skip_while(|&size| {
                    total_size += size;
                    total_size <= self.capacity
                })
                .fold((0, 0), |(sum, count), size| (sum + size, count + 1));
            self.content.replace_range(..sum, "");
            for _ in 0..count {
                self.colors.pop_front();
            }
        }
        Ok(())
    }

    pub fn content(&self) -> impl Iterator<Item = (ColorRef, &str)> {
        self.colors.iter().scan(0, move |end, &(len, color)| {
            let start = *end;
            *end += len;
            Some((color, &self.content[start..*end]))
        })
    }
}

#[macro_export]
macro_rules! log {
    ($console:expr, $(($color:expr, $($args:tt)+)),+ $(,)?) => {
        $crate::console::Console::log(&mut $console, [$(
            ($crate::console::ColorRef::from($color), format_args!($($args)+))
        ),+])
    };
}
