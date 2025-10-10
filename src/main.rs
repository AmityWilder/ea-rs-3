#![deny(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]
#![allow(dead_code, reason = "for future use")]

use crate::{
    config::Config,
    console::{Console, LogType},
    graph::{GraphList, node::Gate, wire::Elbow},
    ivec::{Bounds, IVec2},
    properties::PropertiesPanel,
    tab::{EditorTab, Tab, TabList},
    theme::Theme,
    tool::Tool,
    toolpane::ToolPane,
    ui::{Anchoring, ExactSizing, NcSizing, Padding, Panel, PanelContent, Sizing},
};
use console::Logger;
use raylib::prelude::*;
use std::{
    io::Write,
    sync::{Arc, OnceLock},
    time::{Duration, Instant},
};

mod config;
mod console;
mod graph;
mod icon_sheets;
mod input;
mod ivec;
mod properties;
mod rich_text;
mod tab;
mod theme;
mod tool;
mod toolpane;
mod ui;

pub const GRID_SIZE: u8 = 8;

fn main() {
    let (mut console, mut logger) = Console::new(
        Panel::new(
            "Log",
            Anchoring::Bottom {
                h: Sizing::Exact(ExactSizing {
                    val: 150.0,
                    min: Some(|theme, _, _| {
                        Some(
                            theme.console_font.line_height()
                                + theme.console_font.line_spacing
                                + theme.console_padding.vertical(),
                        )
                    }),
                    max: Some(|_theme, container_size, _content_size| Some(container_size)),
                }),
            },
            |theme| theme.console_padding,
        ),
        4096 * 80,
    );

    {
        static RL_LOGGER: OnceLock<Logger> = OnceLock::new();
        RL_LOGGER.set(logger.clone()).unwrap();
        fn trace_log_callback(level: TraceLogLevel, msg: &str) {
            logln!(
                RL_LOGGER.get().cloned().unwrap(),
                match level {
                    TraceLogLevel::LOG_DEBUG => LogType::Debug,
                    TraceLogLevel::LOG_TRACE | TraceLogLevel::LOG_INFO => LogType::Info,
                    TraceLogLevel::LOG_WARNING => LogType::Warning,
                    TraceLogLevel::LOG_ERROR | TraceLogLevel::LOG_FATAL => LogType::Error,
                    TraceLogLevel::LOG_NONE | TraceLogLevel::LOG_ALL =>
                        unreachable!("not actual log levels, only for comparison"),
                },
                "Raylib: {msg}",
            )
        }
        if let Err(e) = set_trace_log_callback(trace_log_callback) {
            logln!(
                logger,
                LogType::Error,
                "failed to set Raylib tracelog callback: {e}"
            )
        }
    }

    let program_icon =
        Image::load_image_from_mem(".png", include_bytes!("../assets/program_icon32x.png")).ok();

    let (mut rl, thread) = init()
        .title("Electron Architect")
        .size(1280, 720)
        .resizable()
        .build();

    // SAFETY: raylib has been initialized
    unsafe {
        ffi::SetTraceLogLevel(ffi::TraceLogLevel::LOG_WARNING as i32);
    }

    rl.set_target_fps(
        get_monitor_refresh_rate(get_current_monitor())
            .try_into()
            .unwrap(),
    );

    rl.set_exit_key(None);

    if let Some(icon) = program_icon.as_ref() {
        rl.set_window_icon(icon);
    }

    const CONFIG_PATH: &str = "config.toml";
    logln!(
        logger,
        LogType::Attempt,
        "Loading config from {CONFIG_PATH}..."
    );

    // load preferences
    let Config {
        mut theme,
        mut binds,
    } = {
        match std::fs::read_to_string(CONFIG_PATH) {
            Ok(s) => match toml::from_str(&s) {
                Ok(config) => {
                    logln!(logger, LogType::Success, "Config loaded.");
                    config
                }
                Err(e) => {
                    logln!(logger, LogType::Error, "Failed to read config: {e}");
                    Config::default()
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                logln!(
                    logger,
                    LogType::Warning,
                    "Config does not exist. Generating default."
                );
                let config = Config::default();
                if let Err(e) = std::fs::File::create(CONFIG_PATH).and_then(|mut file| {
                    file.write_all(
                        toml::to_string_pretty(&config)
                            .expect("default config should be serializeable")
                            .as_bytes(),
                    )
                }) {
                    logln!(logger, LogType::Error, "Failed to generate file: {e}");
                }
                config
            }
            Err(e) => {
                logln!(logger, LogType::Error, "Failed to open config file: {e}");
                Config::default()
            }
        }
    };
    theme.reload_assets(&mut rl, &thread).unwrap();

    let mut graphs = GraphList::new();

    let mut tabs = TabList::with_tabs(
        Panel::new("Editor", Anchoring::Fill, |_| Padding::amount(0.0)),
        [Tab::Editor(
            EditorTab::new(
                &mut rl,
                &thread,
                1280,
                720,
                Arc::downgrade(graphs.create_graph()),
            )
            .unwrap(),
        )],
    );

    let mut toolpane = ToolPane::new(
        Panel::new(
            "",
            Anchoring::Floating {
                x: 3.0,
                y: 3.0,
                w: NcSizing::FitContent,
                h: NcSizing::FitContent,
            },
            |theme| theme.toolpane_padding,
        ),
        Tool::default(),
        Gate::default(),
        Elbow::default(),
        theme.toolpane_orientation,
        theme.toolpane_visibility,
        theme.button_icon_scale,
    );

    let mut properties = PropertiesPanel::new(Panel::new(
        "Properties",
        Anchoring::Right {
            w: Sizing::Exact(ExactSizing {
                val: 200.0,
                min: Some(|_, _, _| Some(0.0)),
                max: Some(|_, container_size, _content_size| Some(container_size)),
            }),
        },
        |theme| theme.properties_padding,
    ));

    let mut next_eval_tick = Instant::now();
    let eval_duration = Duration::from_millis(200);

    // initialize bounds
    {
        let mut container = Bounds::new(
            Vector2::zero(),
            rvec2(rl.get_screen_width(), rl.get_screen_height()),
        );

        tabs.update_bounds(&mut rl, &thread, &theme, &container)
            .unwrap();

        if let Some(new_container) =
            properties
                .panel
                .update_bounds(&theme, &container, Vector2::zero(/* TODO */))
        {
            container = new_container;
        }

        if let Some(new_container) =
            toolpane
                .panel
                .update_bounds(&theme, &container, toolpane.content_size(&theme))
        {
            container = new_container;
        }

        if let Some(new_container) =
            console
                .panel
                .update_bounds(&theme, &container, Vector2::zero(/* TODO */))
        {
            container = new_container;
        }

        _ = container;
    }

    logln!(logger, LogType::Success, "initialized");

    while !rl.window_should_close() {
        // Tick

        let input = binds.get_all(&rl);

        if rl.is_window_resized() {
            let window_width = rl.get_screen_width();
            let window_height = rl.get_screen_height();
            tabs.update_bounds(
                &mut rl,
                &thread,
                &theme,
                &Bounds::new(Vector2::zero(), rvec2(window_width, window_height)),
            )
            .unwrap();
            // TODO: refresh bounds on other panels
        }

        Panel::tick_resize_set(
            Bounds::new(
                Vector2::zero(),
                rvec2(rl.get_screen_width(), rl.get_screen_height()),
            ),
            &theme,
            &input,
            [
                // tabs only changes when window does, for now
                &mut properties,
                &mut console,
                &mut toolpane,
            ] as [&mut dyn PanelContent; _],
        );

        let focused_panel = {
            let panels = [
                &toolpane.panel,
                &properties.panel,
                &console.panel,
                tabs.panel(),
            ];
            panels
                .iter()
                .find(|panel| panel.is_dragging())
                .or_else(|| panels.iter().find(|panel| panel.interactable(input.cursor)))
                .map(|&panel| panel as *const Panel)
                .unwrap_or_else(std::ptr::null)
        };

        if std::ptr::eq(focused_panel, &toolpane.panel) {
            toolpane.tick(&mut logger, &theme, &input);
        } else if std::ptr::eq(focused_panel, &properties.panel) {
            properties.tick(&theme, |properties, bounds, theme| {
                let mut y = bounds.min.y;
                if let Tool::Edit {
                    target: Some(tool::EditDragging { id, .. }),
                } = &toolpane.tool
                    && let Some(Tab::Editor(tab)) = tabs.focused_tab()
                    && let Some(graph) = tab.graph.upgrade()
                    && let Ok(mut borrow) = graph.write()
                {
                    let node = borrow.node_mut(id).expect("edit target should be valid");
                    y = properties.tick_section(&mut rl, &thread, theme, &input, y, node);
                }
                y = properties.tick_section(&mut rl, &thread, theme, &input, y, &mut toolpane.tool);
                y = properties.tick_section(&mut rl, &thread, theme, &input, y, &mut toolpane.gate);
                _ = y;
            });
        } else if std::ptr::eq(focused_panel, &console.panel) {
            console.tick(&theme, &input, &graphs);
        } else if std::ptr::eq(focused_panel, tabs.panel()) {
            if let Some(tab) = tabs.focused_tab_mut() {
                match tab {
                    Tab::Editor(tab) => {
                        let is_dirty = tab.tick(&mut logger, &mut toolpane, &theme, &input);
                        if is_dirty {
                            // refresh immediately on change
                            next_eval_tick = Instant::now();
                        }
                    }
                }
            } else {
                // TODO: Hovering tabs without any focused tab (should that even be valid?)
            }
        }

        {
            let viewport = *tabs.panel().bounds();
            if let Some(focused_tab) = tabs.focused_tab_mut() {
                match focused_tab {
                    Tab::Editor(tab) => tab.refresh_grid(&mut rl, &thread, &theme, &viewport),
                }
            }
        }

        rl.set_mouse_cursor(
            [
                console.panel.hover.as_ref(),
                properties.panel.hover.as_ref(),
                toolpane.panel.hover.as_ref(),
                tabs.panel().hover.as_ref(),
            ]
            .into_iter()
            .flatten()
            .next()
            .map_or(MouseCursor::MOUSE_CURSOR_DEFAULT, |hover| {
                use ui::RectHoverRegion::*;
                match hover.region {
                    Left | Right => MouseCursor::MOUSE_CURSOR_RESIZE_EW,
                    Top | Bottom => MouseCursor::MOUSE_CURSOR_RESIZE_NS,
                    TopLeft | BottomRight => MouseCursor::MOUSE_CURSOR_RESIZE_NWSE,
                    TopRight | BottomLeft => MouseCursor::MOUSE_CURSOR_RESIZE_NESW,
                }
            }),
        );

        for mut graph in graphs.iter_mut().filter_map(|g| g.try_write().ok()) {
            if graph.is_eval_order_dirty() {
                graph.refresh_eval_order();
            }
            let now = Instant::now();
            while now >= next_eval_tick {
                graph.evaluate();
                next_eval_tick += eval_duration;
            }
        }

        console.update_recv();

        // Draw

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(theme.background);

        // tabs
        {
            if let Some(focused_tab) = tabs.focused_tab() {
                match focused_tab {
                    Tab::Editor(tab) => {
                        tab.draw(&mut d, tabs.panel().bounds(), &theme, &input, &toolpane);
                    }
                }
            }
        }

        // toolpane
        {
            toolpane.draw(&mut d, &input, &theme);
        }

        // console
        {
            console.draw(&mut d, &theme, &input, &graphs, &tabs, &toolpane);
        }

        // properties
        {
            properties.draw(&mut d, &theme, |properties, d, bounds, theme| {
                let mut y = bounds.min.y;
                if let Tool::Edit {
                    target: Some(tool::EditDragging { id, .. }),
                } = &toolpane.tool
                    && let Some(Tab::Editor(tab)) = tabs.focused_tab()
                    && let Some(graph) = tab.graph.upgrade()
                    && let Ok(borrow) = graph.read()
                {
                    let node = borrow.node(id).expect("edit target should be valid");
                    y = properties.draw_section(d, theme, bounds, y, node);
                }
                y = properties.draw_section(d, theme, bounds, y, &toolpane.tool);
                y = properties.draw_section(d, theme, bounds, y, &toolpane.gate);
                _ = y;
            });
        }
    }
}
