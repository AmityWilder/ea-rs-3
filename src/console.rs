use crate::{
    GRID_SIZE,
    graph::{
        Graph, GraphId, GraphList,
        node::{Gate, Node, NodeId},
        wire::{Wire, WireId},
    },
    input::Inputs,
    ivec::{AsIVec2, IBounds, IRect, IVec2},
    rich_text::{ColorAct, ColorRef, RichStr, RichString},
    tab::TabList,
    theme::{ColorId, Theme},
    tool::ToolId,
    toolpane::{ButtonAction, ToolPane},
    ui::{Panel, PanelContent},
};
use raylib::prelude::*;
use std::sync::{
    Arc, Mutex, RwLock, RwLockReadGuard,
    mpsc::{Receiver, Sender, channel},
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
    #[inline]
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
            LogType::Attempt => ColorRef::Theme(ColorId::Special),
            LogType::Success => ColorRef::Theme(ColorId::Foreground1),
            LogType::Warning => ColorRef::Theme(ColorId::Caution),
            LogType::Error => ColorRef::Theme(ColorId::Error),
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

    #[inline]
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

    #[inline]
    pub fn node(&self, node_id: NodeId) -> NodeRef {
        NodeRef(self.0, node_id)
    }

    #[inline]
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
                            .expect("all wires should be valid");
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ConsoleAnchoring {
    pub left: bool,
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
}

#[derive(Debug, Clone)]
pub struct Logger(Sender<String>);

impl std::fmt::Write for Logger {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.send(s.to_string()).map_err(|_| std::fmt::Error)
    }

    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> std::fmt::Result {
        self.0.send(args.to_string()).map_err(|_| std::fmt::Error)
    }
}

impl Logger {
    #[inline]
    pub const fn by_ref(&mut self) -> &mut Self {
        self
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

    pub fn update_recv(&mut self) {
        let mut it = std::iter::from_fn(|| self.receiver.try_recv().ok()).peekable();
        if it.peek().is_some() {
            self.push_log(it.collect::<String>().as_str());
        }
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

#[macro_export]
macro_rules! logln {
    ($logger:expr, $ty:expr, $($args:tt)+) => {
        <$crate::console::Logger as std::fmt::Write>::write_fmt(
            $logger.by_ref(),
            format_args!("{}[{}]: {}{}\n",
                $crate::rich_text::ColorAct::Push(<$crate::rich_text::ColorRef as From<LogType>>::from($ty)),
                $ty,
                format_args!($($args)+),
                $crate::rich_text::ColorAct::Pop,
            ),
        ).unwrap()
    };
}

static RL_LOGGER: Mutex<Option<Logger>> = Mutex::new(None);

pub struct RlLoggerHandle(());

impl RlLoggerHandle {
    pub fn init(logger: Logger) -> Self {
        *RL_LOGGER.lock().unwrap() = Some(logger);
        Self(())
    }
}

impl Drop for RlLoggerHandle {
    fn drop(&mut self) {
        // Raylib will create extra messages when it closes.
        // Even if we never see them, its logger needs to still be valid or
        // the program will crash instead of closing successfully.
        // All resources must go out of scope before dropping the Raylib logger.
        RL_LOGGER.lock().unwrap().take();
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
impl RlLoggerHandle {
    pub fn trace_log_callback(level: TraceLogLevel, msg: &str) {
        // important messages should be printed to stdout in case of crash
        if matches!(level, TraceLogLevel::LOG_ERROR | TraceLogLevel::LOG_FATAL) {
            eprintln!("{msg}");
        }

        if let Ok(mut lock) = RL_LOGGER.lock()
            && let Some(rl_logger) = lock.as_mut()
        {
            logln!(
                rl_logger,
                match level {
                    TraceLogLevel::LOG_DEBUG => LogType::Debug,
                    TraceLogLevel::LOG_TRACE | TraceLogLevel::LOG_INFO => LogType::Info,
                    TraceLogLevel::LOG_WARNING => LogType::Warning,
                    TraceLogLevel::LOG_ERROR | TraceLogLevel::LOG_FATAL => LogType::Error,
                    // not actual log levels; only exist for min log level
                    TraceLogLevel::LOG_NONE | TraceLogLevel::LOG_ALL => return,
                },
                "Raylib: {msg}",
            );
        } else {
            eprintln!("error: failed to lock RL_LOGGER; args: {level:?} {msg}");
        }
    }
}
