use crate::{
    graph::{Graph, node::Node},
    ivec::{IBounds, IVec2},
    rich_text::{ColorAct, ColorRef, RichStr, RichString},
    theme::{ColorId, Theme},
};
use raylib::prelude::*;
use std::{
    collections::VecDeque,
    fmt::Write,
    num::{Saturating, Wrapping},
};

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

impl std::fmt::Display for LogType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogType::Info => "info",
            LogType::Debug => "debug",
            LogType::Attempt => "attempt",
            LogType::Success => "success",
            LogType::Warning => "warning",
            LogType::Error => "error",
        }
        .fmt(f)
    }
}

impl From<LogType> for ColorRef {
    fn from(value: LogType) -> Self {
        value.color()
    }
}

impl LogType {
    pub const fn color(self) -> ColorRef {
        match self {
            LogType::Info => ColorRef::Theme(ColorId::Foreground3),
            LogType::Debug => ColorRef::Exact(Color::MAGENTA),
            LogType::Attempt => ColorRef::Theme(ColorId::Special),
            LogType::Success => ColorRef::Theme(ColorId::Foreground1),
            LogType::Warning => ColorRef::Theme(ColorId::Caution),
            LogType::Error => ColorRef::Theme(ColorId::Error),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PositionRef(pub IVec2);

impl std::fmt::Display for PositionRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &Self(IVec2 { x, y }) = self;
        write!(
            f,
            "{}({x}, {y}){}",
            ColorAct::Push(ColorRef::Theme(ColorId::Special)),
            ColorAct::Pop
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeRef<'a>(pub &'a Graph, pub &'a Node);

impl std::fmt::Display for NodeRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &Self(g, n) = self;
        write!(
            f,
            "{}g{g:p}-n{n:p}{}",
            ColorAct::Push(ColorRef::Theme(ColorId::Special)),
            ColorAct::Pop
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ConsoleAnchoring {
    pub left: bool,
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
}

#[derive(Debug)]
pub struct Console {
    content: RichString,
    pub bottom_offset: f64,
    pub bounds: IBounds,
    pub anchoring: ConsoleAnchoring,
}

impl Console {
    pub fn new(capacity: usize, bounds: IBounds, anchoring: ConsoleAnchoring) -> Self {
        Self {
            content: RichString::with_capacity(capacity),
            bounds,
            bottom_offset: 0.0,
            anchoring,
        }
    }

    /// NOTE: You will need to append with newline
    pub fn log(&mut self, text: std::fmt::Arguments<'_>) {
        let buf;
        let s = match text.as_str() {
            Some(s) => s,
            None => {
                buf = text.to_string();
                buf.as_str()
            }
        };
        for mut line in s.split_inclusive('\n') {
            if line.len() > self.content.capacity() {
                self.content.clear();
                line = &line[line.ceil_char_boundary(line.len() - self.content.capacity())..];
            } else {
                while self.content.len() + line.len() > self.content.capacity() {
                    debug_assert!(
                        !self.content.is_empty(),
                        "if `line` exceeds capacity all by itself, this branch shouldn't have been reached"
                    );
                    match self.content.find('\n') {
                        Some(n) => self.content.replace_range(..n + '\n'.len_utf8(), ""),
                        None => self.content.clear(),
                    }
                }
            }
            debug_assert!(
                self.content.len() + line.len() <= self.content.capacity(),
                "content should not grow"
            );
            self.content.push_str(line);
        }
        self.bottom_offset = 0.0;
    }

    pub const fn content_str(&self) -> &RichStr {
        self.content.as_rich_str()
    }

    pub fn displayable_lines(&self, theme: &Theme) -> i32 {
        ((self.bounds.max.y - theme.console_padding_bottom)
            - (self.bounds.min.y + theme.console_padding_top)
            + /* Off by one otherwise */ theme.console_line_spacing)
            / theme.console_line_height()
    }

    pub fn content(&self) -> impl Iterator<Item = (ColorRef, &str)> {
        let mut last_color = ColorRef::Theme(ColorId::Foreground);
        RichStr::new(self.content.as_str())
            .iter()
            .map(move |item| match item {
                Ok((color, text)) => {
                    if let Some(color) = color {
                        last_color = color;
                    }
                    (last_color, text)
                }
                Err(e) => panic!("{e}"),
            })
    }

    pub fn visible_content(&self, theme: &Theme) -> impl Iterator<Item = (ColorRef, &str)> {
        const MAX_ROW: f64 = (usize::MAX as f64).next_down();
        let mut last_color = ColorRef::Theme(ColorId::Foreground);
        self.content
            .split_inclusive('\n')
            .skip(
                self.content
                    .lines()
                    .count()
                    .saturating_sub(self.bottom_offset.trunc().clamp(0.0, MAX_ROW) as usize)
                    .saturating_sub(usize::try_from(self.displayable_lines(theme)).unwrap()),
            )
            .take(self.displayable_lines(theme).try_into().unwrap())
            .flat_map(|line| RichStr::new(line).iter())
            .map(move |item| match item {
                Ok((color, text)) => {
                    if let Some(color) = color {
                        last_color = color;
                    }
                    (last_color, text)
                }
                Err(e) => panic!("{e}"),
            })
    }
}

#[macro_export]
macro_rules! log {
    ($console:expr, $($args:tt)+) => {
        $crate::console::Console::log(
            &mut $console,
            format_args!($($args)+)
        )
    };
}

#[macro_export]
macro_rules! logln {
    ($console:expr, $ty:expr, $($args:tt)+) => {
        $crate::console::Console::log(
            &mut $console,
            format_args!("{}[{}]: {}{}\n",
                $crate::rich_text::ColorAct::Push(<$crate::rich_text::ColorRef as From<LogType>>::from($ty)),
                $ty,
                format_args!($($args)+),
                $crate::rich_text::ColorAct::Pop,
            )
        )
    };
}
