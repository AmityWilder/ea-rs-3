use arrayvec::ArrayString;
use nom::{
    IResult, Parser,
    bytes::complete::{is_not, tag},
    character::{char, complete::u8},
    combinator::fail,
    multi::{many0, separated_list1},
    sequence::{delimited, preceded},
};
use raylib::prelude::*;

// pub mod attempt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u32)]
enum ColorItem {
    #[default]
    Reset = 0,
    Bold = 1,
    Dim = 2,
    Underline = 4,
    ResetBoldDim = 22,
    ResetUnderline = 24,

    ForegroundBlack = 30,
    ForegroundRed = 31,
    ForegroundGreen = 32,
    ForegroundYellow = 33,
    ForegroundBlue = 34,
    ForegroundMagenta = 35,
    ForegroundCyan = 36,
    ForegroundWhite = 37,
    TruecolorForeground(u8, u8, u8), // 38
    ResetForeground = 39,

    BackgroundBlack = 40,
    BackgroundRed = 41,
    BackgroundGreen = 42,
    BackgroundYellow = 43,
    BackgroundBlue = 44,
    BackgroundMagenta = 45,
    BackgroundCyan = 46,
    BackgroundWhite = 47,
    TruecolorBackground(u8, u8, u8), // 48
    ResetBackground = 49,

    ForegroundBrightBlack = 90,
    ForegroundBrightRed = 91,
    ForegroundBrightGreen = 92,
    ForegroundBrightYellow = 93,
    ForegroundBrightBlue = 94,
    ForegroundBrightMagenta = 95,
    ForegroundBrightCyan = 96,
    ForegroundBrightWhite = 97,

    BackgroundBrightBlack = 100,
    BackgroundBrightRed = 101,
    BackgroundBrightGreen = 102,
    BackgroundBrightYellow = 103,
    BackgroundBrightBlue = 104,
    BackgroundBrightMagenta = 105,
    BackgroundBrightCyan = 106,
    BackgroundBrightWhite = 107,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Style {
    bold: bool,
    underline: bool,
    background: Option<Color>,
    foreground: Option<Color>,
}

impl FromIterator<ColorItem> for Style {
    fn from_iter<T: IntoIterator<Item = ColorItem>>(iter: T) -> Self {
        let mut style = Style::default();
        for item in iter {
            match item {
                ColorItem::Reset => style = Style::default(),
                ColorItem::Bold => style.bold = true,
                ColorItem::Dim => todo!(),
                ColorItem::Underline => style.underline = true,
                ColorItem::ResetBoldDim => style.bold = false,
                ColorItem::ResetUnderline => style.underline = false,

                ColorItem::ForegroundBlack => style.foreground = Some(Color::BLACK),
                ColorItem::ForegroundRed => style.foreground = Some(Color::RED),
                ColorItem::ForegroundGreen => style.foreground = Some(Color::GREEN),
                ColorItem::ForegroundYellow => style.foreground = Some(Color::YELLOW),
                ColorItem::ForegroundBlue => style.foreground = Some(Color::BLUE),
                ColorItem::ForegroundMagenta => style.foreground = Some(Color::MAGENTA),
                ColorItem::ForegroundCyan => style.foreground = Some(Color::CYAN),
                ColorItem::ForegroundWhite => style.foreground = Some(Color::WHITE),
                ColorItem::TruecolorForeground(r, g, b) => {
                    style.foreground = Some(Color::new(r, g, b, 255))
                }
                ColorItem::ResetForeground => style.foreground = None,

                ColorItem::BackgroundBlack => style.background = Some(Color::BLACK),
                ColorItem::BackgroundRed => style.background = Some(Color::RED),
                ColorItem::BackgroundGreen => style.background = Some(Color::GREEN),
                ColorItem::BackgroundYellow => style.background = Some(Color::YELLOW),
                ColorItem::BackgroundBlue => style.background = Some(Color::BLUE),
                ColorItem::BackgroundMagenta => style.background = Some(Color::MAGENTA),
                ColorItem::BackgroundCyan => style.background = Some(Color::CYAN),
                ColorItem::BackgroundWhite => style.background = Some(Color::WHITE),
                ColorItem::TruecolorBackground(r, g, b) => {
                    style.background = Some(Color::new(r, g, b, 255))
                }
                ColorItem::ResetBackground => style.background = None,

                ColorItem::ForegroundBrightBlack => style.foreground = Some(Color::BLACK),
                ColorItem::ForegroundBrightRed => style.foreground = Some(Color::RED),
                ColorItem::ForegroundBrightGreen => style.foreground = Some(Color::GREEN),
                ColorItem::ForegroundBrightYellow => style.foreground = Some(Color::YELLOW),
                ColorItem::ForegroundBrightBlue => style.foreground = Some(Color::BLUE),
                ColorItem::ForegroundBrightMagenta => style.foreground = Some(Color::MAGENTA),
                ColorItem::ForegroundBrightCyan => style.foreground = Some(Color::CYAN),
                ColorItem::ForegroundBrightWhite => style.foreground = Some(Color::WHITE),

                ColorItem::BackgroundBrightBlack => style.background = Some(Color::BLACK),
                ColorItem::BackgroundBrightRed => style.background = Some(Color::RED),
                ColorItem::BackgroundBrightGreen => style.background = Some(Color::GREEN),
                ColorItem::BackgroundBrightYellow => style.background = Some(Color::YELLOW),
                ColorItem::BackgroundBrightBlue => style.background = Some(Color::BLUE),
                ColorItem::BackgroundBrightMagenta => style.background = Some(Color::MAGENTA),
                ColorItem::BackgroundBrightCyan => style.background = Some(Color::CYAN),
                ColorItem::BackgroundBrightWhite => style.background = Some(Color::WHITE),
            }
        }
        style
    }
}

fn color_item(input: &str) -> IResult<&str, ColorItem> {
    let (input, byte) = u8(input)?;
    match byte {
        0 => Ok((input, ColorItem::Reset)),
        1 => Ok((input, ColorItem::Bold)),
        2 => Ok((input, ColorItem::Dim)),
        4 => Ok((input, ColorItem::Underline)),
        22 => Ok((input, ColorItem::ResetBoldDim)),
        24 => Ok((input, ColorItem::ResetUnderline)),

        30 => Ok((input, ColorItem::ForegroundBlack)),
        31 => Ok((input, ColorItem::ForegroundRed)),
        32 => Ok((input, ColorItem::ForegroundGreen)),
        33 => Ok((input, ColorItem::ForegroundYellow)),
        34 => Ok((input, ColorItem::ForegroundBlue)),
        35 => Ok((input, ColorItem::ForegroundMagenta)),
        36 => Ok((input, ColorItem::ForegroundCyan)),
        37 => Ok((input, ColorItem::ForegroundWhite)),
        38 => {
            let (input, (r, g, b)) = (
                preceded(char(';'), u8),
                preceded(char(';'), u8),
                preceded(char(';'), u8),
            )
                .parse(input)?;
            Ok((input, ColorItem::TruecolorForeground(r, g, b)))
        }
        39 => Ok((input, ColorItem::ResetForeground)),

        40 => Ok((input, ColorItem::BackgroundBlack)),
        41 => Ok((input, ColorItem::BackgroundRed)),
        42 => Ok((input, ColorItem::BackgroundGreen)),
        43 => Ok((input, ColorItem::BackgroundYellow)),
        44 => Ok((input, ColorItem::BackgroundBlue)),
        45 => Ok((input, ColorItem::BackgroundMagenta)),
        46 => Ok((input, ColorItem::BackgroundCyan)),
        47 => Ok((input, ColorItem::BackgroundWhite)),
        48 => {
            let (input, (r, g, b)) = (
                preceded(char(';'), u8),
                preceded(char(';'), u8),
                preceded(char(';'), u8),
            )
                .parse(input)?;
            Ok((input, ColorItem::TruecolorBackground(r, g, b)))
        }
        49 => Ok((input, ColorItem::ResetBackground)),

        90 => Ok((input, ColorItem::ForegroundBrightBlack)),
        91 => Ok((input, ColorItem::ForegroundBrightRed)),
        92 => Ok((input, ColorItem::ForegroundBrightGreen)),
        93 => Ok((input, ColorItem::ForegroundBrightYellow)),
        94 => Ok((input, ColorItem::ForegroundBrightBlue)),
        95 => Ok((input, ColorItem::ForegroundBrightMagenta)),
        96 => Ok((input, ColorItem::ForegroundBrightCyan)),
        97 => Ok((input, ColorItem::ForegroundBrightWhite)),

        100 => Ok((input, ColorItem::BackgroundBrightBlack)),
        101 => Ok((input, ColorItem::BackgroundBrightRed)),
        102 => Ok((input, ColorItem::BackgroundBrightGreen)),
        103 => Ok((input, ColorItem::BackgroundBrightYellow)),
        104 => Ok((input, ColorItem::BackgroundBrightBlue)),
        105 => Ok((input, ColorItem::BackgroundBrightMagenta)),
        106 => Ok((input, ColorItem::BackgroundBrightCyan)),
        107 => Ok((input, ColorItem::BackgroundBrightWhite)),

        _ => fail().parse(input),
    }
}

fn style(input: &str) -> IResult<&str, Style> {
    let (input, items) = separated_list1(char(';'), color_item).parse(input)?;
    Ok((input, Style::from_iter(items)))
}

fn style_escaped(input: &str) -> IResult<&str, Style> {
    delimited(tag("\x1b["), style, char('m')).parse(input)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum LogLevel {
    #[default]
    Info,
    Debug,
    Attempt,
    Success,
    /// Failed, but was optional or can resort to a default
    Warning,
    /// Failed, must cancel action
    Error,
    /// Failed, application cannot continue
    Fatal,
}

impl std::fmt::Display for LogLevel {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Attempt => "attempt",
            LogLevel::Success => "success",
            LogLevel::Warning => "warning",
            LogLevel::Error => "error",
            LogLevel::Fatal => "fatal",
        }
        .fmt(f)
    }
}

#[derive(Debug, Default)]
pub struct Terminal<const CAP: usize> {
    content: ArrayString<CAP>,
    pub bottom_offset: f64,
}

impl<const CAP: usize> Terminal<CAP> {
    pub fn new() -> Self {
        Self {
            content: ArrayString::new(),
            bottom_offset: 0.0,
        }
    }

    /// NOTE: You will need to append with newline
    pub fn push(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        for mut line in text.split_inclusive('\n') {
            if line.len() > self.content.capacity() {
                self.content.clear();
                line = &line[line.ceil_char_boundary(line.len() - self.content.capacity())..];
            } else {
                while self.content.len() + line.len() > self.content.capacity() {
                    debug_assert!(
                        !self.content.is_empty(),
                        "if `line` exceeds capacity all by itself, this branch shouldn't have been reached"
                    );
                    match self.content.find('\n').map(|n| n + '\n'.len_utf8()) {
                        Some(n) => {
                            unsafe { self.content.as_bytes_mut() }.rotate_left(n);
                            self.content.truncate(self.content.len() - n);
                        }
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

    #[inline]
    pub fn content_str(&self) -> &str {
        self.content.as_str()
    }

    #[inline]
    pub fn displayable_lines(height: f32, font_size: f32, line_spacing: f32) -> usize {
        ((height + /* Off by one otherwise */ line_spacing) / (font_size + line_spacing)) as usize
    }

    pub fn visible_content(
        &self,
        height: f32,
        font_size: f32,
        line_spacing: f32,
    ) -> impl Iterator<Item = (Style, &str)> {
        const MAX_ROW: f64 = (usize::MAX as f64).next_down();
        let lines = Self::displayable_lines(height, font_size, line_spacing);
        self.content
            .split_inclusive('\n')
            .skip(
                self.content
                    .lines()
                    .count()
                    .saturating_sub(self.bottom_offset.trunc().clamp(0.0, MAX_ROW) as usize)
                    .saturating_sub(lines),
            )
            .take(lines)
            .flat_map(|line| {
                many0((style_escaped, is_not(&['\x1b'][..])))
                    .parse(line)
                    .map(|(_, x)| x)
                    .unwrap_or_else(|e| panic!("{e}"))
            })
    }

    /// Only draws the text content, not the container
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        d: &mut impl RaylibDraw,
        font: &Font,
        font_size: f32,
        spacing: f32,
        line_spacing: f32,
        default_color: Color,
        bounds: Rectangle,
    ) {
        let mut x = bounds.x;
        let left = x;
        let mut y = bounds.y;
        for (style, text) in self.visible_content(bounds.height, font_size, line_spacing) {
            d.draw_text_ex(
                font,
                &self.content,
                Vector2::new(x, y),
                font_size,
                spacing,
                style.foreground.unwrap_or(default_color),
            );
            let (prev_lines, last_line) = match text.rsplit_once('\n') {
                Some((prev, last)) => (prev.lines().count(), last),
                None => (0, text),
            };
            let size = font.measure_text(last_line, font_size, spacing);
            if prev_lines != 0 {
                y += prev_lines as f32 * (font_size + line_spacing);
                x = left + size.x;
            } else {
                x += size.x;
            }
        }
    }
}
