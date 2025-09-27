use crate::ivec::IBounds;
use raylib::prelude::*;
use std::{collections::VecDeque, fmt::Write};

#[derive(Debug)]
pub struct Console {
    content: String,
    colors: VecDeque<(usize, Color)>,
    bounds: IBounds,
    capacity: usize,
}

impl Console {
    pub const fn new(bounds: IBounds, capacity: usize) -> Self {
        Self {
            content: String::new(),
            colors: VecDeque::new(),
            bounds,
            capacity,
        }
    }

    /// NOTE: You will need to append with newline
    pub fn log<'a>(
        &mut self,
        text: impl IntoIterator<Item = (Color, std::fmt::Arguments<'a>)>,
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

    pub fn content(&self) -> impl Iterator<Item = (Color, &str)> {
        self.colors.iter().scan(0, move |end, &(len, color)| {
            let start = *end;
            *end += len;
            Some((color, &self.content[start..*end]))
        })
    }

    pub fn bounds(&self) -> &IBounds {
        &self.bounds
    }
}

#[macro_export]
macro_rules! log {
    ($console:expr, $(($color:expr, $($args:tt)+)),+ $(,)?) => {
        $crate::Console::log(&mut $console, [$(($color, format_args!($($args)+))),+])
    };
}
