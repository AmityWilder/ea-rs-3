use crate::theme::{ColorId, Theme};
use raylib::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RichStrError {
    InvalidEscapeCode,
}

impl std::fmt::Display for RichStrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RichStrError::InvalidEscapeCode => "escape code should match the pattern of `\\x1B{rgba(r,g,b,a)}` or `\\x1B{name}` \
                where `r`, `g`, `b`, and `a` are integers between 0 and 255 inclusively, and `name` is the name of a theme color".fmt(f),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorRef {
    Theme(ColorId),
    Exact(Color),
}

impl std::fmt::Display for ColorRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorRef::Theme(id) => write!(f, "{id}"),
            ColorRef::Exact(Color { r, g, b, a }) => write!(f, "rgba({r},{g},{b},{a})"),
        }
    }
}

impl std::str::FromStr for ColorRef {
    type Err = RichStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.strip_prefix("rgba(") {
            Some(s) => match s.strip_suffix(')') {
                Some(s) => {
                    let mut it = s.splitn(4, ',');
                    it.next()
                        .and_then(|x| x.parse().ok())
                        .zip(it.next().and_then(|x| x.parse().ok()))
                        .zip(it.next().and_then(|x| x.parse().ok()))
                        .zip(it.next().and_then(|x| x.parse().ok()))
                        .ok_or(RichStrError::InvalidEscapeCode)
                        .map(|(((r, g), b), a)| Self::Exact(Color::new(r, g, b, a)))
                }
                None => Err(RichStrError::InvalidEscapeCode),
            },
            None => s
                .parse::<ColorId>()
                .map(Self::Theme)
                .map_err(|_| RichStrError::InvalidEscapeCode),
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorAct {
    #[default]
    Pop,
    Repl(ColorRef),
    Push(ColorRef),
}

impl std::fmt::Display for ColorAct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorAct::Pop => write!(f, "\x1B{{pop}}"),
            ColorAct::Repl(c) => write!(f, "\x1B{{{c}}}"),
            ColorAct::Push(c) => write!(f, "\x1B{{push:{c}}}"),
        }
    }
}

impl std::str::FromStr for ColorAct {
    type Err = RichStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "pop" {
            Ok(Self::Pop)
        } else {
            let (c, wrapper): (&str, fn(ColorRef) -> Self) = s
                .strip_prefix("push:")
                .map_or((s, Self::Repl), |c| (c, Self::Push));
            c.parse().map(wrapper)
        }
    }
}

#[derive(Debug, Clone)]
pub struct RichStrIter<'a> {
    color_stack: Vec<ColorRef>,
    string: &'a str,
}

impl std::error::Error for RichStrError {}

impl<'a> Iterator for RichStrIter<'a> {
    type Item = Result<(Option<ColorRef>, &'a str), RichStrError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut s = self.string;
        if s.is_empty() {
            return None;
        }
        let act = match s.strip_prefix("\x1B{") {
            Some(string) => match string.split_once('}') {
                Some((code, rest)) => {
                    s = rest;
                    Some(code.parse::<ColorAct>())
                }
                None => {
                    s = &s["\x1B{".len()..];
                    Some(Err(RichStrError::InvalidEscapeCode))
                }
            },
            None => None,
        };
        let text;
        (text, self.string) = s.split_at(s.find("\x1B{").unwrap_or(s.len()));
        match act {
            Some(Ok(a)) => {
                match a {
                    ColorAct::Pop => _ = self.color_stack.pop(),
                    ColorAct::Repl(c) => match self.color_stack.last_mut() {
                        Some(back) => *back = c,
                        None => self.color_stack.push(c),
                    },
                    ColorAct::Push(c) => {
                        self.color_stack.push(c);
                    }
                }
                Some(Ok((self.color_stack.last().copied(), text)))
            }
            Some(Err(e)) => Some(Err(e)),
            None => Some(Ok((None, text))),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.len();
        (n, Some(n))
    }
}

impl<'a> DoubleEndedIterator for RichStrIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let mut s = self.string;
        if s.is_empty() {
            return None;
        }
        let color = match s.rsplit_once("\x1B{") {
            Some((pre, string)) => match string.split_once('}') {
                Some((code, text)) => {
                    self.string = pre;
                    s = text;
                    Some(code.parse::<ColorRef>())
                }
                None => Some(Err(RichStrError::InvalidEscapeCode)),
            },
            None => None,
        };
        match color {
            Some(Ok(c)) => Some(Ok((Some(c), s))),
            Some(Err(e)) => Some(Err(e)),
            None => Some(Ok((None, s))),
        }
    }
}

impl<'a> ExactSizeIterator for RichStrIter<'a> {
    fn len(&self) -> usize {
        self.string.split("\x1B{").count()
    }
}

impl std::iter::FusedIterator for RichStrIter<'_> {}

#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct RichStr(str);

impl std::ops::Deref for RichStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for RichStr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<str> for RichStr {
    fn as_ref(&self) -> &str {
        self
    }
}

impl AsMut<str> for RichStr {
    fn as_mut(&mut self) -> &mut str {
        self
    }
}

impl RichStr {
    pub const fn new(s: &str) -> &Self {
        // SAFETY: RichStr is a wrapper for str
        unsafe { std::mem::transmute(s) }
    }

    pub const fn new_mut(s: &mut str) -> &mut Self {
        // SAFETY: RichStr is a wrapper for str
        unsafe { std::mem::transmute(s) }
    }

    pub const fn iter(&self) -> RichStrIter<'_> {
        RichStrIter {
            color_stack: Vec::new(),
            string: &self.0,
        }
    }
}

impl<'a> IntoIterator for &'a RichStr {
    type Item = <RichStrIter<'a> as Iterator>::Item;
    type IntoIter = RichStrIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RichString(String);

impl<T: Into<String>> From<T> for RichString {
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl std::ops::Deref for RichString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for RichString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<String> for RichString {
    fn as_ref(&self) -> &String {
        self
    }
}

impl AsMut<String> for RichString {
    fn as_mut(&mut self) -> &mut String {
        self
    }
}

impl AsRef<str> for RichString {
    fn as_ref(&self) -> &str {
        self
    }
}

impl AsMut<str> for RichString {
    fn as_mut(&mut self) -> &mut str {
        self
    }
}

impl AsRef<RichStr> for RichString {
    fn as_ref(&self) -> &RichStr {
        self.as_rich_str()
    }
}

impl AsMut<RichStr> for RichString {
    fn as_mut(&mut self) -> &mut RichStr {
        self.as_mut_rich_str()
    }
}

impl RichString {
    pub const fn new() -> Self {
        Self(String::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(String::with_capacity(capacity))
    }

    pub const fn as_rich_str(&self) -> &RichStr {
        RichStr::new(self.0.as_str())
    }

    pub const fn as_mut_rich_str(&mut self) -> &mut RichStr {
        RichStr::new_mut(self.0.as_mut_str())
    }
}
