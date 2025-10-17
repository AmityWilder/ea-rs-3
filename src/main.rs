#![feature(nonpoison_mutex, sync_nonpoison, nonpoison_rwlock, push_mut)]
#![deny(clippy::missing_safety_doc, clippy::undocumented_unsafe_blocks)]

use crate::{
    config::{Config, theme::Theme},
    console::{Console, attempt::*},
    graph::{GraphList, node::Gate, wire::Elbow},
    ivec::{Bounds, IVec2},
    properties::PropertiesPanel,
    tab::{EditorTab, Tab, TabList},
    tool::Tool,
    toolpane::ToolPane,
    ui::{Anchoring, ExactSizing, NcSizing, Padding, Panel, PanelContent, Sizing},
};
use console::GLoggerHandle;
use raylib::prelude::*;
use std::{
    io::Write,
    sync::Arc,
    time::{Duration, Instant},
};

mod config;
mod console;
mod graph;
mod icon_sheets;
mod ivec;
mod properties;
mod tab;
mod tool;
mod toolpane;
mod ui;

pub const GRID_SIZE: u8 = 8;
pub const GRID_EXTENT: Vector2 = Vector2::new(0.5 * GRID_SIZE as f32, 0.5 * GRID_SIZE as f32);

fn main() {
    let (mut console, logger) = Console::new(
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

    let _logger = GLoggerHandle::init(logger);

    attempt!("initializing");

    // setup raylib logging
    set_trace_log_callback(GLoggerHandle::trace_log_callback)
        .warn_or("failed to set Raylib tracelog callback", ());

    let program_icon =
        Image::load_image_from_mem(".png", include_bytes!("../assets/program_icon32x.png"))
            .warn("failed to load program icon")
            .ok();

    let (mut rl, thread) = init()
        .title("Electron Architect")
        .size(1280, 720)
        .resizable()
        .build();

    rl.set_target_fps(
        get_monitor_refresh_rate(get_current_monitor())
            .try_into()
            .error("monitor refresh rate cannot be negative")
            .unwrap_or(60),
    );

    rl.set_exit_key(None);

    if let Some(icon) = program_icon.as_ref() {
        rl.set_window_icon(icon);
    }

    // load preferences
    let Config {
        mut theme,
        mut binds,
    } = {
        const CONFIG_PATH: &str = "config.toml";
        attempt!("loading config from {CONFIG_PATH}");
        match std::fs::read_to_string(CONFIG_PATH) {
            Ok(s) => {
                attempt!("parsing config");
                toml::from_str(&s)
                    .success("config loaded")
                    .warn_or_default("failed to read config")
            }

            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                logln!(Warning, "config does not exist.");
                let config = Config::default();
                attempt!("generating default file");
                std::fs::File::create(CONFIG_PATH)
                    .and_then(|mut file| {
                        file.write_all(
                            toml::to_string_pretty(&config)
                                .fatal("default config should be serializeable")
                                .as_bytes(),
                        )
                    })
                    .success(format_args!("default config file {CONFIG_PATH} generated"))
                    .warn_or("failed to generate file", ());
                config
            }

            Err(e) => {
                logln!(Warning, "failed to open config file: {e}");
                Config::default()
            }
        }
    };

    attempt!("loading theme assets");
    _ = theme
        .reload_assets(&mut rl, &thread)
        .success("theme assets loaded")
        .error("failed to load theme assets");

    let mut graphs = GraphList::new();

    let mut tabs = TabList::with_tabs(
        Panel::new("Editor", Anchoring::Fill, |_| Padding::amount(0.0)),
        EditorTab::new(
            &mut rl,
            &thread,
            1280,
            720,
            Arc::downgrade(graphs.create_graph()),
        )
        .error("failed to create editor tab")
        .map(Tab::Editor),
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

        _ = tabs
            .update_bounds(&mut rl, &thread, &theme, &container)
            .error("failed to update tab bounds");

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

    logln!(Success, "initialized");

    while !rl.window_should_close() {
        // Tick

        let input = binds.get_all(&rl);

        if rl.is_window_resized() {
            let window_width = rl.get_screen_width();
            let window_height = rl.get_screen_height();
            _ = tabs
                .update_bounds(
                    &mut rl,
                    &thread,
                    &theme,
                    &Bounds::new(Vector2::zero(), rvec2(window_width, window_height)),
                )
                .error("failed to update tab bounds");
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
            toolpane.tick(&theme, &input);
        } else if std::ptr::eq(focused_panel, &properties.panel) {
            properties.tick(&theme, |properties, bounds, theme| {
                let mut y = bounds.min.y;
                // if let Tool::Edit(tool::Edit {
                //     target: Some(tool::EditDragging { selected: id, .. }),
                // }) = &toolpane.tool
                //     && let Some(Tab::Editor(tab)) = tabs.focused_tab()
                //     && let Some(graph) = tab.graph.upgrade()
                // {
                //     let mut borrow = graph.write();
                //     if let Ok(node) = borrow
                //         .node_mut(id)
                //         .error("edit target should always be valid")
                //     {
                //         y = properties.tick_section(&mut rl, &thread, theme, &input, y, node);
                //     }
                // }
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
                        if tab.tick(&mut toolpane, &input) {
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

        for mut graph in graphs.values().filter_map(|g| g.try_write().ok()) {
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
                // if let Tool::Edit(tool::Edit {
                //     target: Some(tool::EditDragging { selected: id, .. }),
                // }) = &toolpane.tool
                //     && let Some(Tab::Editor(tab)) = tabs.focused_tab()
                //     && let Some(graph) = tab.graph.upgrade()
                // {
                //     let borrow = graph.read();
                //     if let Ok(node) = borrow.node(id).error("edit target should be valid") {
                //         y = properties.draw_section(d, theme, bounds, y, node);
                //     }
                // }
                y = properties.draw_section(d, theme, bounds, y, &toolpane.tool);
                y = properties.draw_section(d, theme, bounds, y, &toolpane.gate);
                _ = y;
            });
        }
    }
}
