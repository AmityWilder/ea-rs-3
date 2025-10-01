use crate::{
    draw_hyper_ref_link,
    graph::{
        Graph, GraphId, GraphList,
        node::{GateId, Node, NodeId},
        wire::{Wire, WireId},
    },
    input::Inputs,
    ivec::{AsIVec2, Bounds, IBounds, IRect, IVec2},
    rich_text::{ColorAct, ColorRef, RichStr, RichString},
    tab::TabList,
    theme::{ColorId, Fonts, Theme},
    tool::ToolId,
    toolpane::ToolPane,
};
use raylib::prelude::*;
use std::sync::{Arc, RwLock, RwLockReadGuard};

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
pub struct GateRef(pub GateId);

impl std::ops::Deref for GateRef {
    type Target = GateId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for GateRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for GateRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &Self(g) = self;
        write!(
            f,
            "{}[{g}]{}",
            ColorAct::Push(ColorRef::Theme(ColorId::HyperRef)),
            ColorAct::Pop
        )
    }
}

impl std::str::FromStr for GateRef {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
            .and_then(|s| s.parse().ok())
            .map(Self)
            .ok_or(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ToolRef(pub ToolId);

impl std::ops::Deref for ToolRef {
    type Target = ToolId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ToolRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for ToolRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &Self(g) = self;
        write!(
            f,
            "{}[{g}]{}",
            ColorAct::Push(ColorRef::Theme(ColorId::HyperRef)),
            ColorAct::Pop
        )
    }
}

impl std::str::FromStr for ToolRef {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
            .and_then(|s| s.parse().ok())
            .map(Self)
            .ok_or(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PositionRef(pub IVec2);

impl std::ops::Deref for PositionRef {
    type Target = IVec2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for PositionRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Display for PositionRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &Self(IVec2 { x, y }) = self;
        write!(
            f,
            "{}({x},{y}){}",
            ColorAct::Push(ColorRef::Theme(ColorId::HyperRef)),
            ColorAct::Pop
        )
    }
}

impl std::str::FromStr for PositionRef {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .and_then(|s| s.split_once(','))
            .and_then(|(x, y)| x.parse().ok().zip(y.parse().ok()))
            .map(|(x, y)| Self(IVec2 { x, y }))
            .ok_or(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphRef(pub GraphId);

impl std::fmt::Display for GraphRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &Self(id) = self;
        write!(
            f,
            "{}{id}{}",
            ColorAct::Push(ColorRef::Theme(ColorId::HyperRef)),
            ColorAct::Pop
        )
    }
}

impl std::str::FromStr for GraphRef {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map_err(|_| ()).map(Self)
    }
}

impl GraphRef {
    pub fn deref_with<T, F>(self, graphs: &GraphList, f: F) -> Option<T>
    where
        F: for<'a> FnOnce(&'a Arc<RwLock<Graph>>, RwLockReadGuard<'a, Graph>) -> T,
    {
        if let Some(graph) = graphs.get(&self.0)
            && let Ok(borrow) = graph.try_read()
        {
            Some(f(graph, borrow))
        } else {
            None
        }
    }

    pub fn node(&self, node_id: NodeId) -> NodeRef {
        NodeRef(self.0, node_id)
    }

    pub fn wire(&self, wire_id: WireId) -> WireRef {
        WireRef(self.0, wire_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeRef(pub GraphId, pub NodeId);

impl std::fmt::Display for NodeRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &Self(g, n) = self;
        write!(
            f,
            "{}{g}-{n}{}",
            ColorAct::Push(ColorRef::Theme(ColorId::HyperRef)),
            ColorAct::Pop
        )
    }
}

impl std::str::FromStr for NodeRef {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split_once('-')
            .and_then(|(g, n)| g.parse().ok().zip(n.parse().ok()))
            .map(|(g, n)| Self(g, n))
            .ok_or(())
    }
}

impl NodeRef {
    pub fn deref_with<T, F>(self, graphs: &GraphList, f: F) -> Option<T>
    where
        F: for<'a> FnOnce(&'a Arc<RwLock<Graph>>, &RwLockReadGuard<'a, Graph>, &'a Node) -> T,
    {
        if let Some(graph) = graphs.get(&self.0)
            && let Ok(borrow) = graph.try_read()
            && let Some(node) = borrow.node(&self.1)
        {
            Some(f(graph, &borrow, node))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireRef(pub GraphId, pub WireId);

impl std::fmt::Display for WireRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &Self(g, w) = self;
        write!(
            f,
            "{}{g}-{w}{}",
            ColorAct::Push(ColorRef::Theme(ColorId::HyperRef)),
            ColorAct::Pop
        )
    }
}

impl std::str::FromStr for WireRef {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split_once('-')
            .and_then(|(g, w)| g.parse().ok().zip(w.parse().ok()))
            .map(|(g, n)| Self(g, n))
            .ok_or(())
    }
}

impl WireRef {
    pub fn deref_with<T, F>(self, graphs: &GraphList, f: F) -> Option<T>
    where
        F: for<'a> FnOnce(&'a Arc<RwLock<Graph>>, &RwLockReadGuard<'a, Graph>, &'a Wire) -> T,
    {
        if let Some(graph) = graphs.get(&self.0)
            && let Ok(borrow) = graph.try_read()
            && let Some(wire) = borrow.wire(&self.1)
        {
            Some(f(graph, &borrow, wire))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HyperRef {
    Gate(GateRef),
    Tool(ToolRef),
    Position(PositionRef),
    Graph(GraphRef),
    Node(NodeRef),
    Wire(WireRef),
}

impl std::fmt::Display for HyperRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HyperRef::Gate(x) => x.fmt(f),
            HyperRef::Tool(x) => x.fmt(f),
            HyperRef::Position(x) => x.fmt(f),
            HyperRef::Graph(x) => x.fmt(f),
            HyperRef::Node(x) => x.fmt(f),
            HyperRef::Wire(x) => x.fmt(f),
        }
    }
}

impl std::str::FromStr for HyperRef {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse()
            .map(Self::Gate)
            .or_else(|()| s.parse().map(Self::Tool))
            .or_else(|()| s.parse().map(Self::Position))
            .or_else(|()| s.parse().map(Self::Graph))
            .or_else(|()| s.parse().map(Self::Node))
            .or_else(|()| s.parse().map(Self::Wire))
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
    pub bounds: Bounds,
    pub anchoring: ConsoleAnchoring,
}

impl Console {
    pub fn new(capacity: usize, bounds: Bounds, anchoring: ConsoleAnchoring) -> Self {
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

    pub fn displayable_lines(&self, theme: &Theme) -> usize {
        (((self.bounds.max.y - theme.console_padding_bottom)
            - (self.bounds.min.y + theme.console_padding_top)
            + /* Off by one otherwise */ theme.console_line_spacing)
            / theme.console_line_height()) as usize
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

    #[allow(clippy::too_many_arguments)]
    pub fn draw<D>(
        &self,
        d: &mut D,
        theme: &Theme,
        fonts: &Fonts,
        input: &Inputs,
        graphs: &GraphList,
        tabs: &TabList,
        toolpane: &ToolPane,
    ) where
        D: RaylibDraw,
    {
        let Rectangle {
            x,
            y,
            width,
            height,
        } = Rectangle::from(self.bounds);

        // content
        {
            d.draw_rectangle_rec(Rectangle::new(x, y, width, height), theme.background2);
            d.draw_rectangle_rec(
                Rectangle::new(x + 1.0, y + 1.0, width - 2.0, height - 2.0),
                theme.background1,
            );

            let mut x = x + theme.console_padding_left;
            let mut y = self.bounds.max.y
                - theme.console_padding_bottom
                - self.displayable_lines(theme) as f32 * theme.console_line_height();
            let left = x;
            for (color, text) in self.visible_content(theme) {
                let size = fonts.console.measure_text(
                    text,
                    theme.console_font_size,
                    theme.console_char_spacing,
                );
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
                        draw_hyper_ref_link(d, hr, hyper_rec, theme, graphs, tabs, toolpane);
                    }

                    Some(is_live)
                } else {
                    None
                };
                d.draw_text_ex(
                    &fonts.console,
                    text,
                    rvec2(x, y),
                    theme.console_font_size,
                    theme.console_char_spacing,
                    if is_live.is_none_or(|x| x) {
                        color.get(theme)
                    } else {
                        theme.dead_link
                    },
                );
                if text.ends_with('\n') {
                    y += theme.console_line_height();
                    x = left;
                } else {
                    x += size.x;
                }
            }
        }

        // title
        {
            let title = "Log";
            let title_text_size = fonts.general.measure_text(
                title,
                theme.console_font_size,
                theme.console_char_spacing,
            );
            let title_width = title_text_size.x + 2.0 * theme.title_padding_x;
            let title_height = title_text_size.y + 2.0 * theme.title_padding_y;
            d.draw_rectangle_rec(
                Rectangle::new(
                    self.bounds.max.x - title_width,
                    self.bounds.min.y,
                    title_width,
                    title_height,
                ),
                theme.background2,
            );
            d.draw_text_ex(
                &fonts.console,
                title,
                Vector2::new(
                    self.bounds.max.x - title_width + theme.title_padding_x,
                    self.bounds.min.y + theme.title_padding_y,
                ),
                theme.console_font_size,
                theme.console_char_spacing,
                theme.foreground,
            );
        }
    }
}

#[macro_export]
macro_rules! logln {
    ($console:expr, $ty:expr, $($args:tt)+) => {
        $crate::console::Console::log(
            $console,
            format_args!("{}[{}]: {}{}\n",
                $crate::rich_text::ColorAct::Push(<$crate::rich_text::ColorRef as From<LogType>>::from($ty)),
                $ty,
                format_args!($($args)+),
                $crate::rich_text::ColorAct::Pop,
            )
        )
    };
}
