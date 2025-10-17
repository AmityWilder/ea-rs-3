use raylib::prelude::*;
use rich_text::{ColorAct, ColorRef, RichStr, RichString};

pub mod rich_text;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum LogType {
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

impl std::fmt::Display for LogType {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogType::Info => "info",
            LogType::Debug => "debug",
            LogType::Attempt => "attempt",
            LogType::Success => "success",
            LogType::Warning => "warning",
            LogType::Error => "error",
            LogType::Fatal => "fatal",
        }
        .fmt(f)
    }
}

#[derive(Debug)]
pub struct Console {
    content: RichString,
    receiver: String,
    pub bottom_offset: f64,
}

impl Console {
    #[inline]
    fn content_size<T: AsRef<ffi::Font>>(
        &self,
        _font: T,
        _font_size: f32,
        _spacing: f32,
    ) -> Vector2 {
        Vector2::zero() // TODO
    }
}

impl Console {
    pub fn new(capacity: usize) -> Self {
        (Self {
            content: RichString::with_capacity(capacity),
            receiver,
            bottom_offset: 0.0,
        },)
    }

    /// NOTE: You will need to append with newline
    fn push_log(&mut self, text: &str) {
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

    #[inline]
    pub const fn content_str(&self) -> &RichStr {
        self.content.as_rich_str()
    }

    #[inline]
    pub fn displayable_lines(&self, theme: &Theme) -> usize {
        ((self.panel.content_bounds(theme).height()
            + /* Off by one otherwise */ theme.console_font.line_spacing)
            / theme.console_font.line_height()) as usize
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
                    .saturating_sub(self.displayable_lines(theme)),
            )
            .take(self.displayable_lines(theme))
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

    pub fn update_recv(&mut self) {
        self.push_log(self.receiver.try_iter().collect::<String>().as_str());
    }

    pub fn tick(&mut self, theme: &Theme, input: &Inputs, graphs: &GraphList) {
        self.bottom_offset = (self.bottom_offset + input.scroll_console as f64).clamp(
            0.0,
            self.content_str()
                .lines()
                .count()
                .saturating_sub(self.displayable_lines(theme)) as f64,
        );

        let Vector2 { mut x, mut y } = self.panel.content_bounds(theme).min;
        let left = x;
        for (_, text) in self.visible_content(theme) {
            let text_size = theme.console_font.measure_text(text);
            if Rectangle::new(x, y, text_size.x, text_size.y)
                .check_collision_point_rec(input.cursor)
                && let Ok(hyper_ref) = text.parse::<HyperRef>()
            {
                match hyper_ref {
                    HyperRef::Gate(_gate_ref) => {
                        // TODO
                    }

                    HyperRef::Tool(_tool_ref) => {
                        // TODO
                    }

                    HyperRef::Position(_position_ref) => {
                        // TODO
                    }

                    HyperRef::Graph(graph_ref) => {
                        graph_ref.deref_with(graphs, |_g, _borrow| {
                            // TODO
                        });
                    }

                    HyperRef::Node(node_ref) => {
                        node_ref.deref_with(graphs, |_g, _borrow, _node| {
                            // TODO
                        });
                    }

                    HyperRef::Wire(wire_ref) => {
                        wire_ref.deref_with(graphs, |_g, _borrow, _wire| {
                            // TODO
                        });
                    }
                }
            }
            if text.ends_with('\n') {
                y += theme.console_font.line_height();
                x = left;
            } else {
                x += theme.console_font.measure_text(text).x;
            }
        }
    }

    pub fn draw<D>(
        &self,
        d: &mut D,
        theme: &Theme,
        input: &Inputs,
        graphs: &GraphList,
        tabs: &TabList,
        toolpane: &ToolPane,
    ) where
        D: RaylibDraw,
    {
        self.panel.draw(d, theme, move |d, bounds, theme| {
            let mut x = bounds.min.x;
            let mut y = bounds.max.y
                - self.displayable_lines(theme) as f32 * theme.console_font.line_height();
            let left = x;
            for (color, text) in self.visible_content(theme) {
                let size = theme.console_font.measure_text(text);
                let hyper_rec = IRect::new(x as i32, y as i32, size.x as i32, size.y as i32);
                let is_live = if let Ok(hr) = text.parse::<HyperRef>() {
                    let is_live = match hr {
                        HyperRef::Gate(_) => Some(()),
                        HyperRef::Tool(_) => Some(()),
                        HyperRef::Position(_) => Some(()),
                        HyperRef::Graph(graph_ref) => graph_ref.deref_with(graphs, |_, _| {}),
                        HyperRef::Node(node_ref) => node_ref.deref_with(graphs, |_, _, _| {}),
                        HyperRef::Wire(wire_ref) => wire_ref.deref_with(graphs, |_, _, _| {}),
                    }
                    .is_some();

                    if is_live
                        && IBounds::from(hyper_rec).contains(input.cursor.as_ivec2())
                        && let Ok(hr) = text.parse::<HyperRef>()
                    {
                        hr.draw_link(d, hyper_rec, theme, graphs, tabs, toolpane);
                    }

                    Some(is_live)
                } else {
                    None
                };
                theme.console_font.draw_text(
                    d,
                    text,
                    rvec2(x, y),
                    if is_live.is_none_or(|x| x) {
                        color.get(theme)
                    } else {
                        theme.dead_link
                    },
                );
                if text.ends_with('\n') {
                    y += theme.console_font.line_height();
                    x = left;
                } else {
                    x += size.x;
                }
            }
        });
    }
}
