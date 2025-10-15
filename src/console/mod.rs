use crate::{
    GRID_SIZE,
    config::{
        input::Inputs,
        theme::{ColorId, Theme},
    },
    console::attempt::*,
    graph::{
        Graph, GraphList,
        id::{GraphId, NodeId, WireId},
        node::{Gate, Node},
        wire::Wire,
    },
    ivec::{AsIVec2, IBounds, IRect, IVec2},
    tab::TabList,
    tool::ToolId,
    toolpane::{ButtonAction, ToolPane},
    ui::{Panel, PanelContent},
};
use raylib::prelude::*;
use rich_text::{ColorAct, ColorRef, RichStr, RichString};
use std::{
    str::FromStr,
    sync::{
        Arc,
        mpsc::{Receiver, SendError, Sender, channel},
        nonpoison::{Mutex, RwLock, RwLockReadGuard},
    },
};
use thiserror::Error;

/// UNDER CONSTRUCTION
pub mod attempt;
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

impl From<LogType> for ColorRef {
    #[inline]
    fn from(value: LogType) -> Self {
        value.color()
    }
}

impl LogType {
    #[inline]
    pub const fn color(self) -> ColorRef {
        match self {
            LogType::Info => ColorRef::Theme(ColorId::Foreground3),
            LogType::Debug => ColorRef::Exact(Color::MAGENTA),
            LogType::Attempt => ColorRef::Theme(ColorId::Foreground2),
            LogType::Success => ColorRef::Theme(ColorId::Foreground1),
            LogType::Warning => ColorRef::Theme(ColorId::Caution),
            LogType::Error => ColorRef::Theme(ColorId::Error),
            LogType::Fatal => ColorRef::Theme(ColorId::Destructive),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct GateRef(pub Gate);

impl std::ops::Deref for GateRef {
    type Target = Gate;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for GateRef {
    #[inline]
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

impl FromStr for GateRef {
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

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ToolRef {
    #[inline]
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

impl FromStr for ToolRef {
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

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for PositionRef {
    #[inline]
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

impl FromStr for PositionRef {
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

impl GraphId {
    #[inline]
    pub const fn graph_ref(&self) -> GraphRef {
        GraphRef(*self)
    }
}

impl Graph {
    #[inline]
    pub const fn graph_ref(&self) -> GraphRef {
        self.id().graph_ref()
    }
}

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

impl FromStr for GraphRef {
    type Err = <GraphId as FromStr>::Err;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeRef(pub NodeId);

impl NodeId {
    #[inline]
    pub const fn node_ref(&self) -> NodeRef {
        NodeRef(*self)
    }
}

impl Node {
    #[inline]
    pub const fn node_ref(&self) -> NodeRef {
        self.id().node_ref()
    }
}

impl std::fmt::Display for NodeRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &Self(n) = self;
        write!(
            f,
            "{}{n}{}",
            ColorAct::Push(ColorRef::Theme(ColorId::HyperRef)),
            ColorAct::Pop
        )
    }
}

impl FromStr for NodeRef {
    type Err = <NodeId as FromStr>::Err;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl NodeRef {
    #[inline]
    pub fn deref_with<T, F>(self, graphs: &GraphList, f: F) -> Option<T>
    where
        F: for<'a> FnOnce(&'a Arc<RwLock<Graph>>, &RwLockReadGuard<'a, Graph>, &'a Node) -> T,
    {
        for graph in graphs.values() {
            if let Ok(borrow) = graph.try_read()
                && let Ok(node) = borrow.node(&self.0)
            {
                return Some(f(graph, &borrow, node));
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WireRef(pub WireId);

impl WireId {
    #[inline]
    pub const fn wire_ref(&self) -> WireRef {
        WireRef(*self)
    }
}

impl Wire {
    #[inline]
    pub const fn wire_ref(&self) -> WireRef {
        self.id().wire_ref()
    }
}

impl std::fmt::Display for WireRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &Self(w) = self;
        write!(
            f,
            "{}{w}{}",
            ColorAct::Push(ColorRef::Theme(ColorId::HyperRef)),
            ColorAct::Pop
        )
    }
}

impl FromStr for WireRef {
    type Err = <WireId as FromStr>::Err;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl WireRef {
    #[inline]
    pub fn deref_with<T, F>(self, graphs: &GraphList, f: F) -> Option<T>
    where
        F: for<'a> FnOnce(&'a Arc<RwLock<Graph>>, &RwLockReadGuard<'a, Graph>, &'a Wire) -> T,
    {
        for graph in graphs.values() {
            if let Ok(borrow) = graph.try_read()
                && let Ok(wire) = borrow.wire(&self.0)
            {
                return Some(f(graph, &borrow, wire));
            }
        }
        None
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
    #[inline]
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

#[derive(Debug, Error)]
#[error("no matching hyperref format")]
pub struct ParseHyperRefError;

impl FromStr for HyperRef {
    type Err = ParseHyperRefError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        (s.parse().ok().map(Self::Gate))
            .or_else(|| s.parse().ok().map(Self::Tool))
            .or_else(|| s.parse().ok().map(Self::Position))
            .or_else(|| s.parse().ok().map(Self::Graph))
            .or_else(|| s.parse().ok().map(Self::Node))
            .or_else(|| s.parse().ok().map(Self::Wire))
            .ok_or(ParseHyperRefError)
    }
}

impl HyperRef {
    fn draw_link<D>(
        &self,
        d: &mut D,
        rec: IRect,
        theme: &Theme,
        graphs: &GraphList,
        tabs: &TabList,
        toolpane: &ToolPane,
    ) where
        D: RaylibDraw,
    {
        const GRID_CENTER_OFFSET: Vector2 =
            Vector2::new((GRID_SIZE / 2) as f32, (GRID_SIZE / 2) as f32);

        // highlight ref text
        d.draw_rectangle(rec.x, rec.y, rec.w, rec.h, theme.hyperref.alpha(0.2));

        let link_anchor = Vector2::new(
            rec.x as f32 + rec.w as f32,
            rec.y as f32 + rec.h as f32 * 0.5,
        );

        match self {
            HyperRef::Gate(gate_ref) => {
                if let Some((rec, _)) =
                    toolpane
                        .buttons(Vector2::zero(), theme)
                        .find(|(_, button)| {
                            matches!(button.action,
                                ButtonAction::SetGate(id) if id == gate_ref.0.id()
                            )
                        })
                {
                    d.draw_line_v(
                        link_anchor,
                        Vector2::new(rec.x + 0.5 * rec.width, rec.y + 0.5 * rec.height),
                        theme.hyperref,
                    );
                }
            }

            HyperRef::Tool(tool_ref) => {
                // HACK: only matches against the icon of the button!
                if let Some((rec, _)) =
                    toolpane
                        .buttons(Vector2::zero(), theme)
                        .find(|(_, button)| {
                            matches!(button.action,
                                ButtonAction::SetTool(id) if id == tool_ref.0
                            )
                        })
                {
                    d.draw_line_v(
                        link_anchor,
                        Vector2::new(rec.x + 0.5 * rec.width, rec.y + 0.5 * rec.height),
                        theme.hyperref,
                    );
                }
            }

            HyperRef::Position(position_ref) => {
                for tab in tabs.editors() {
                    let pos = tab.world_to_screen(position_ref.as_vec2() + GRID_CENTER_OFFSET);
                    d.draw_line_v(link_anchor, pos, theme.hyperref);
                }
            }

            HyperRef::Graph(graph_ref) => {
                graph_ref.deref_with(graphs, |g, _borrow| {
                    for _tab in tabs.editors_of_graph(&Arc::downgrade(g)) {
                        // TODO
                    }
                });
            }

            HyperRef::Node(node_ref) => {
                node_ref.deref_with(graphs, |g, _borrow, node| {
                    for tab in tabs.editors_of_graph(&Arc::downgrade(g)) {
                        let pos =
                            tab.world_to_screen(node.position().as_vec2() + GRID_CENTER_OFFSET);
                        d.draw_line_v(link_anchor, pos, theme.hyperref);
                    }
                });
            }

            HyperRef::Wire(wire_ref) => {
                wire_ref.deref_with(graphs, |g, borrow, wire| {
                    for tab in tabs.editors_of_graph(&Arc::downgrade(g)) {
                        let (start, end) = borrow
                            .get_wire_nodes(wire)
                            .fatal("all wires should be valid");
                        let start_pos = start.position().as_vec2() + GRID_CENTER_OFFSET;
                        let end_pos = end.position().as_vec2() + GRID_CENTER_OFFSET;
                        let pos = tab.world_to_screen(wire.elbow.calculate(start_pos, end_pos));
                        d.draw_line_v(link_anchor, pos, theme.hyperref);
                    }
                });
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Logger(Sender<String>);

impl Logger {
    #[inline]
    pub fn push_log(
        &mut self,
        level: LogType,
        args: std::fmt::Arguments<'_>,
    ) -> Result<(), SendError<String>> {
        self.0.send(format!(
            "{}[{level}]: {args}{}\n",
            ColorAct::Push(level.into()),
            ColorAct::Pop,
        ))
    }
}

#[derive(Debug)]
pub struct Console {
    content: RichString,
    receiver: Receiver<String>,
    pub bottom_offset: f64,
    pub panel: Panel,
}

impl PanelContent for Console {
    #[inline]
    fn panel(&self) -> &Panel {
        &self.panel
    }

    #[inline]
    fn panel_mut(&mut self) -> &mut Panel {
        &mut self.panel
    }

    #[inline]
    fn content_size(&self, _theme: &Theme) -> Vector2 {
        Vector2::zero() // TODO
    }
}

impl Console {
    pub fn new(panel: Panel, capacity: usize) -> (Self, Logger) {
        let (sender, receiver) = channel();
        (
            Self {
                content: RichString::with_capacity(capacity),
                receiver,
                bottom_offset: 0.0,
                panel,
            },
            Logger(sender),
        )
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

static G_LOGGER: Mutex<Option<Logger>> = Mutex::new(None);

/// Global logger handle
///
/// Initializes the global [`Logger`] with [`Self::init`] and
/// releases it when the handle [`drop`]s.
pub struct GLoggerHandle(());

impl GLoggerHandle {
    pub fn init(logger: Logger) -> Self {
        *G_LOGGER.lock() = Some(logger);
        Self(())
    }
}

impl Drop for GLoggerHandle {
    fn drop(&mut self) {
        // Raylib will create extra messages when it closes.
        // Even if we never see them, its logger needs to still be valid or
        // the program will crash instead of closing successfully.
        // All resources must go out of scope before dropping the Raylib logger.
        G_LOGGER.lock().take();
    }
}

#[derive(Debug)]
pub enum GLogError {
    Unset,
    Send(SendError<String>),
}

impl From<SendError<String>> for GLogError {
    fn from(e: SendError<String>) -> Self {
        Self::Send(e)
    }
}

impl std::fmt::Display for GLogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GLogError::Unset => "G_LOGGER is None",
            GLogError::Send(_) => "formatting error",
        }
        .fmt(f)
    }
}

impl std::error::Error for GLogError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GLogError::Unset => None,
            GLogError::Send(e) => Some(e),
        }
    }
}

#[deny(
    clippy::correctness,
    clippy::suspicious,
    clippy::perf,
    clippy::pedantic,
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::unreachable,
    clippy::unimplemented,
    clippy::arithmetic_side_effects,
    reason = "RlLoggerHandle callback(s) will be executed in ffi, which cannot unwind"
)]
impl GLoggerHandle {
    pub fn try_log_str(level: LogType, msg: &str) -> Result<(), GLogError> {
        Self::try_log_fmt(level, format_args!("{msg}"))
    }

    pub fn try_log_fmt(level: LogType, args: std::fmt::Arguments<'_>) -> Result<(), GLogError> {
        let mut lock = G_LOGGER.lock();
        lock.as_mut().ok_or(GLogError::Unset).and_then(|g_logger| {
            // important messages should be duplicatively printed to stderr in case of crash
            if level >= LogType::Warning {
                eprintln!("{args}");
            }
            g_logger.push_log(level, args).map_err(Into::into)
        })
    }

    pub fn trace_log_callback(level: TraceLogLevel, msg: &str) {
        if let Err(e) = Self::try_log_str(
            match level {
                TraceLogLevel::LOG_DEBUG => LogType::Debug,
                TraceLogLevel::LOG_TRACE | TraceLogLevel::LOG_INFO => LogType::Info,
                TraceLogLevel::LOG_WARNING => LogType::Warning,
                TraceLogLevel::LOG_ERROR | TraceLogLevel::LOG_FATAL => LogType::Error,
                // not actual log levels; only exist for min log level
                TraceLogLevel::LOG_NONE | TraceLogLevel::LOG_ALL => return,
            },
            msg,
        ) {
            eprintln!("error: {e}; level: {level:?}, msg: {msg}");
        }
    }
}

/// Print to the global [`Logger`]
#[macro_export]
macro_rules! logln {
    ($level:expr, $($args:tt)+) => {{
        let level = {
            #[allow(unused_imports, clippy::enum_glob_use)]
            use $crate::console::LogType::*;
            $level
        };
        if let Err(e) = $crate::console::GLoggerHandle::try_log_fmt(level, format_args!($($args)+)) {
            eprintln!("logger error: {e}");
        }
    }};
}
