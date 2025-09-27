use crate::{
    ivec::{IBounds, IVec2},
    theme::{ColorId, Theme},
};
use raylib::prelude::*;
use std::{collections::VecDeque, fmt::Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum LogType {
    #[default]
    Info,
    Debug,
    Attempt,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorRef {
    Trace(LogType),
    Theme(ColorId),
    Exact(Color),
}

impl ColorRef {
    pub fn get(self, theme: &Theme) -> Color {
        match self {
            Self::Trace(level) => match level {
                LogType::Info => theme.foreground,
                LogType::Debug => Color::MAGENTA,
                LogType::Attempt => theme.special,
                LogType::Success => theme.foreground1,
                LogType::Warning => theme.caution,
                LogType::Error => theme.error,
            },
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

impl From<LogType> for ColorRef {
    fn from(value: LogType) -> Self {
        Self::Trace(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RichChunk<'a> {
    pub text: &'a str,
    pub color: ColorRef,
    pub x_change: i32,
    pub height: i32,
    /// Whether to add `height` to your `y`
    pub is_y_changing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RichChunkData {
    pub len: usize,
    pub color: ColorRef,
    pub x_change: i32,
    pub height: i32,
    /// Whether to add `height` to your `y`
    pub is_y_changing: bool,
}

#[derive(Debug)]
pub struct Console {
    content: String,
    colors: VecDeque<RichChunkData>,
    capacity: usize,
    end_x: i32,
    /// In units of cols/rows
    pub scroll: IVec2,
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
            end_x: 0,
            bounds,
            scroll: IVec2::zero(),
            left_anchored,
            top_anchored,
            right_anchored,
            bottom_anchored,
        }
    }

    /// NOTE: You will need to append with newline
    /// NOTE: Remember to call [`Self::refresh_chunk_sizes`] if the font size has changed
    pub fn log<'a>(
        &mut self,
        rl: &RaylibHandle,
        theme: &Theme,
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
            let (x_change, height, is_y_changing) =
                Self::measure_chunk(rl, theme, &self.content[start..end], self.end_x);
            self.end_x += x_change;
            self.colors.push_back(RichChunkData {
                len: end - start,
                color,
                x_change,
                height,
                is_y_changing,
            });
            let mut total_size = 0;
            let (sum, count) = self
                .colors
                .iter()
                .rev()
                .map(|chunk| chunk.len)
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

    /// `(x_change, height, is_y_changing)`
    fn measure_chunk(rl: &RaylibHandle, theme: &Theme, s: &str, x: i32) -> (i32, i32, bool) {
        if s.contains('\n') {
            (
                rl.measure_text(s.split('\n').next_back().unwrap(), theme.console_font_size) + 1
                    - x,
                i32::try_from((s.split('\n').count() - 1) * 12).unwrap(),
                true,
            )
        } else {
            (rl.measure_text(s, theme.console_font_size) + 1, 12, false)
        }
    }

    /// Call this when font size changes
    pub fn refresh_chunk_sizes(&mut self, rl: &RaylibHandle, theme: &Theme) {
        self.end_x = 0;
        for (chunk, s) in self.colors.iter_mut().scan(0, |end, chunk| {
            let start = *end;
            *end += chunk.len;
            Some((chunk, &self.content[start..*end]))
        }) {
            let (x_change, height, is_y_changing) = Self::measure_chunk(rl, theme, s, self.end_x);
            self.end_x += x_change;
            chunk.x_change = x_change;
            chunk.height = height;
            chunk.is_y_changing = is_y_changing;
        }
    }

    pub fn content(&self) -> impl Iterator<Item = RichChunk<'_>> {
        self.colors.iter().scan(0, |end, &chunk| {
            let start = *end;
            *end += chunk.len;
            Some(RichChunk {
                text: &self.content[start..*end],
                color: chunk.color,
                x_change: chunk.x_change,
                height: chunk.height,
                is_y_changing: chunk.is_y_changing,
            })
        })
    }
}

#[macro_export]
macro_rules! log {
    ($console:expr, $rl:expr, $theme:expr, $(($color:expr, $($args:tt)+)),+ $(,)?) => {
        $crate::console::Console::log(
            &mut $console,
            &$rl,
            &$theme,
            [$( ($crate::console::ColorRef::from($color), format_args!($($args)+)) ),+]
        )
    };
}
