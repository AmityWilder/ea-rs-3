use crate::{
    console::{Console, GateRef, LogType, ToolRef},
    graph::{
        node::{Gate, GateId, Ntd},
        wire::Elbow,
    },
    icon_sheets::{ButtonIconId, ButtonIconSheetId},
    input::Inputs,
    ivec::Bounds,
    logln,
    rich_text::ColorRef,
    theme::Theme,
    tool::{Tool, ToolId},
    ui::{Orientation, Panel, Visibility},
};
use raylib::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Button {
    pub text: Option<&'static str>,
    pub tooltip: Option<&'static str>,
    pub desc: Option<&'static str>,
    pub color: Option<ColorRef>,
    pub icon: Option<ButtonIconId>,
    pub action: ButtonAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ButtonGroup {
    buttons: Vec<Button>,
    rev_rows: bool,
}

impl ButtonGroup {
    #[inline]
    pub fn cols(&self, visibility: Visibility) -> usize {
        match visibility {
            Visibility::Expanded => 3,
            Visibility::Collapsed => 1,
            Visibility::Hidden => 0,
        }
    }

    #[inline]
    pub fn rows(&self, visibility: Visibility) -> usize {
        match visibility {
            Visibility::Expanded => self.buttons.len().div_ceil(3),
            Visibility::Collapsed => self.buttons.len(),
            Visibility::Hidden => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonAction {
    SetTool(ToolId),
    SetGate(GateId),
    SetNtd(Ntd),
    Blueprints,
    Clipboard,
    Settings,
}

#[derive(Debug, Clone)]
pub struct ToolPane {
    pub panel: Panel,
    pub tool: Tool,
    pub gate: Gate,
    pub ntd: Ntd,
    pub elbow: Elbow,
    pub orientation: Orientation,
    pub visibility: Visibility,
    pub scale: ButtonIconSheetId,
    pub button_groups: Vec<ButtonGroup>,
}

impl ToolPane {
    pub fn new(
        panel: Panel,
        tool: Tool,
        gate: Gate,
        elbow: Elbow,
        orientation: Orientation,
        visibility: Visibility,
        scale: ButtonIconSheetId,
    ) -> Self {
        Self {
            panel,
            tool,
            ntd: gate.ntd().unwrap_or_default(),
            gate,
            elbow,
            orientation,
            visibility,
            scale,
            button_groups: vec![
                ButtonGroup {
                    rev_rows: false,
                    buttons: vec![
                        Button {
                            text: None,
                            tooltip: None,
                            desc: None,
                            color: None,
                            icon: Some(ButtonIconId::Pen),
                            action: ButtonAction::SetTool(ToolId::Create),
                        },
                        Button {
                            text: None,
                            tooltip: None,
                            desc: None,
                            color: None,
                            icon: Some(ButtonIconId::Edit),
                            action: ButtonAction::SetTool(ToolId::Edit),
                        },
                        Button {
                            text: None,
                            tooltip: None,
                            desc: None,
                            color: None,
                            icon: Some(ButtonIconId::Erase),
                            action: ButtonAction::SetTool(ToolId::Erase),
                        },
                        Button {
                            text: None,
                            tooltip: None,
                            desc: None,
                            color: None,
                            icon: Some(ButtonIconId::BlueprintSelect),
                            action: ButtonAction::Blueprints,
                        },
                        Button {
                            text: None,
                            tooltip: None,
                            desc: None,
                            color: None,
                            icon: Some(ButtonIconId::Interact),
                            action: ButtonAction::SetTool(ToolId::Interact),
                        },
                        Button {
                            text: None,
                            tooltip: None,
                            desc: None,
                            color: None,
                            icon: Some(ButtonIconId::Clipboard),
                            action: ButtonAction::Clipboard,
                        },
                    ],
                },
                ButtonGroup {
                    rev_rows: false,
                    buttons: vec![
                        Button {
                            text: None,
                            tooltip: None,
                            desc: None,
                            color: None,
                            icon: Some(ButtonIconId::Or),
                            action: ButtonAction::SetGate(GateId::Or),
                        },
                        Button {
                            text: None,
                            tooltip: None,
                            desc: None,
                            color: None,
                            icon: Some(ButtonIconId::And),
                            action: ButtonAction::SetGate(GateId::And),
                        },
                        Button {
                            text: None,
                            tooltip: None,
                            desc: None,
                            color: None,
                            icon: Some(ButtonIconId::Nor),
                            action: ButtonAction::SetGate(GateId::Nor),
                        },
                        Button {
                            text: None,
                            tooltip: None,
                            desc: None,
                            color: None,
                            icon: Some(ButtonIconId::Xor),
                            action: ButtonAction::SetGate(GateId::Xor),
                        },
                        Button {
                            text: None,
                            tooltip: None,
                            desc: None,
                            color: None,
                            icon: Some(ButtonIconId::Resistor),
                            action: ButtonAction::SetGate(GateId::Resistor),
                        },
                        Button {
                            text: None,
                            tooltip: None,
                            desc: None,
                            color: None,
                            icon: Some(ButtonIconId::Capacitor),
                            action: ButtonAction::SetGate(GateId::Capacitor),
                        },
                        Button {
                            text: None,
                            tooltip: None,
                            desc: None,
                            color: None,
                            icon: Some(ButtonIconId::Led),
                            action: ButtonAction::SetGate(GateId::Led),
                        },
                        Button {
                            text: None,
                            tooltip: None,
                            desc: None,
                            color: None,
                            icon: Some(ButtonIconId::Delay),
                            action: ButtonAction::SetGate(GateId::Delay),
                        },
                        Button {
                            text: None,
                            tooltip: None,
                            desc: None,
                            color: None,
                            icon: Some(ButtonIconId::Battery),
                            action: ButtonAction::SetGate(GateId::Battery),
                        },
                    ],
                },
                ButtonGroup {
                    rev_rows: true,
                    buttons: vec![
                        Button {
                            text: Some("9"),
                            tooltip: None,
                            desc: None,
                            color: Some(ColorRef::Exact(Color::WHITE)),
                            icon: None,
                            action: ButtonAction::SetNtd(Ntd::Nine),
                        },
                        Button {
                            text: Some("8"),
                            tooltip: None,
                            desc: None,
                            color: Some(ColorRef::Exact(Color::GRAY)),
                            icon: None,
                            action: ButtonAction::SetNtd(Ntd::Eight),
                        },
                        Button {
                            text: Some("7"),
                            tooltip: None,
                            desc: None,
                            color: Some(ColorRef::Exact(Color::PURPLE)),
                            icon: None,
                            action: ButtonAction::SetNtd(Ntd::Seven),
                        },
                        Button {
                            text: Some("6"),
                            tooltip: None,
                            desc: None,
                            color: Some(ColorRef::Exact(Color::BLUE)),
                            icon: None,
                            action: ButtonAction::SetNtd(Ntd::Six),
                        },
                        Button {
                            text: Some("5"),
                            tooltip: None,
                            desc: None,
                            color: Some(ColorRef::Exact(Color::GREEN)),
                            icon: None,
                            action: ButtonAction::SetNtd(Ntd::Five),
                        },
                        Button {
                            text: Some("4"),
                            tooltip: None,
                            desc: None,
                            color: Some(ColorRef::Exact(Color::YELLOW)),
                            icon: None,
                            action: ButtonAction::SetNtd(Ntd::Four),
                        },
                        Button {
                            text: Some("3"),
                            tooltip: None,
                            desc: None,
                            color: Some(ColorRef::Exact(Color::ORANGE)),
                            icon: None,
                            action: ButtonAction::SetNtd(Ntd::Three),
                        },
                        Button {
                            text: Some("2"),
                            tooltip: None,
                            desc: None,
                            color: Some(ColorRef::Exact(Color::RED)),
                            icon: None,
                            action: ButtonAction::SetNtd(Ntd::Two),
                        },
                        Button {
                            text: Some("1"),
                            tooltip: None,
                            desc: None,
                            color: Some(ColorRef::Exact(Color::BROWN)),
                            icon: None,
                            action: ButtonAction::SetNtd(Ntd::One),
                        },
                        Button {
                            text: Some("0"),
                            tooltip: None,
                            desc: None,
                            color: Some(ColorRef::Exact(Color::BLACK)),
                            icon: None,
                            action: ButtonAction::SetNtd(Ntd::Zero),
                        },
                    ],
                },
                ButtonGroup {
                    rev_rows: bool::default(), // only one item in row anyway
                    buttons: vec![Button {
                        text: None,
                        tooltip: None,
                        desc: None,
                        color: None,
                        icon: Some(ButtonIconId::Settings),
                        action: ButtonAction::Settings,
                    }],
                },
            ],
        }
    }

    #[inline]
    pub fn set_tool(&mut self, tool_id: ToolId, console: &mut Console) -> bool {
        let change = self.tool.id() != tool_id;
        if change {
            self.tool = tool_id.init();
            logln!(console, LogType::Info, "set tool to {}", ToolRef(tool_id));
        }
        change
    }

    #[inline]
    pub fn set_gate(&mut self, gate_id: GateId, console: &mut Console) -> bool {
        let change = self.gate.id() != gate_id;
        if change {
            self.gate = gate_id.to_gate(self.ntd);
            logln!(console, LogType::Info, "set gate to {}", GateRef(self.gate));
        }
        change
    }

    #[inline]
    pub fn set_ntd(&mut self, data: Ntd, console: &mut Console) -> bool {
        let change = self.ntd != data;
        if change {
            self.ntd = data;
            self.gate = self.gate.with_ntd(self.ntd);
            logln!(
                console,
                LogType::Info,
                "set non-transistor data to {}",
                self.ntd
            );
        }
        change
    }

    /// get `position` from [`Self::bounds`]
    pub fn buttons(
        &self,
        position: Vector2,
        theme: &Theme,
    ) -> impl Iterator<Item = (Rectangle, &Button)> {
        let orientation = self.orientation;
        let visibility = self.visibility;
        let scale = self.scale;
        let icon_width = scale.icon_width();
        let group_gap = match visibility {
            Visibility::Expanded => theme.toolpane_group_expanded_gap,
            Visibility::Collapsed => theme.toolpane_group_collapsed_gap,
            Visibility::Hidden => 0.0,
        };
        let button_gap = theme.toolpane_button_gap;
        let mut along = 0.0;
        self.button_groups.iter().flat_map(move |group| {
            let offset = match orientation {
                Orientation::Horizontal => Vector2::new(along, 0.0),
                Orientation::Vertical => Vector2::new(0.0, along),
            };
            let it = {
                let chunk_size = match visibility {
                    Visibility::Expanded => 3,
                    Visibility::Collapsed => 1,
                    Visibility::Hidden => 1,
                };
                match visibility {
                    Visibility::Expanded | Visibility::Collapsed => group.buttons.as_slice(),
                    Visibility::Hidden => [].as_slice(),
                }
                .chunks(chunk_size)
                .enumerate()
                .flat_map(move |(row, seg)| {
                    seg.iter().enumerate().map(move |(col, button)| {
                        let col = if group.rev_rows {
                            seg.len() - 1 - col // iterator would not run if seg was empty
                        } else {
                            col
                        };
                        let along = row as f32;
                        let across = col as f32 + 0.5 * (chunk_size - seg.len()) as f32;
                        let (x, y) = match orientation {
                            Orientation::Horizontal => (along, across),
                            Orientation::Vertical => (across, along),
                        };
                        (
                            Rectangle::new(
                                position.x
                                    + offset.x
                                    + x * icon_width as f32
                                    + (x - 1.0).max(0.0) * button_gap,
                                position.y
                                    + offset.y
                                    + y * icon_width as f32
                                    + (y - 1.0).max(0.0) * button_gap,
                                icon_width as f32,
                                icon_width as f32,
                            ),
                            button,
                        )
                    })
                })
            };
            along += group.rows(visibility) as f32 * icon_width as f32 + group_gap;
            it
        })
    }

    pub fn content_size(&self, theme: &Theme) -> Vector2 {
        let cols = match self.visibility {
            Visibility::Expanded => 3,
            Visibility::Collapsed => 1,
            Visibility::Hidden => 0,
        };
        let rows = self
            .button_groups
            .iter()
            .map(|g| g.rows(self.visibility))
            .sum::<usize>();
        let groups = self.button_groups.len();
        let button_width = usize::try_from(self.scale.icon_width()).unwrap();
        let group_gap = match self.visibility {
            Visibility::Expanded => theme.toolpane_group_expanded_gap,
            Visibility::Collapsed => theme.toolpane_group_collapsed_gap,
            Visibility::Hidden => 0.0,
        };
        let button_gap = theme.toolpane_button_gap;

        let thickness = (cols * button_width) as f32 + cols.saturating_sub(1) as f32 * button_gap;
        let length = (rows * button_width) as f32
            + rows.saturating_sub(1) as f32 * button_gap
            + groups.saturating_sub(1) as f32 * group_gap;

        match self.orientation {
            Orientation::Horizontal => Vector2::new(length, thickness),
            Orientation::Vertical => Vector2::new(thickness, length),
        }
    }

    pub fn tick(&mut self, console: &mut Console, theme: &Theme, input: &Inputs) {
        if input.primary.is_starting() {
            let bounds = self.panel.content_bounds(theme);
            let action = self
                .buttons(bounds.min, theme)
                .find_map(|(button_rec, button)| {
                    Bounds::from(button_rec)
                        .contains(input.cursor)
                        .then_some(button.action)
                });
            if let Some(action) = action {
                match action {
                    ButtonAction::SetTool(tool_id) => {
                        self.set_tool(tool_id, console);
                    }
                    ButtonAction::SetGate(gate_id) => {
                        self.set_gate(gate_id, console);
                    }
                    ButtonAction::SetNtd(data) => {
                        self.set_ntd(data, console);
                    }
                    ButtonAction::Blueprints => {
                        // TODO
                    }
                    ButtonAction::Clipboard => {
                        // TODO
                    }
                    ButtonAction::Settings => {
                        // TODO
                    }
                }
            }
        }
    }

    pub fn draw<D>(&self, d: &mut D, input: &Inputs, theme: &Theme)
    where
        D: RaylibDraw,
    {
        self.panel.draw(d, theme, |d, bounds, theme| {
            for (button_rec, button) in self.buttons(bounds.min, theme) {
                let is_hovered = Bounds::from(button_rec).contains(input.cursor);
                let is_selected = match button.action {
                    ButtonAction::SetTool(tool_id) => tool_id == self.tool.id(),
                    ButtonAction::SetGate(gate_id) => gate_id == self.gate.id(),
                    ButtonAction::SetNtd(data) => data == self.ntd,
                    ButtonAction::Blueprints => false,
                    ButtonAction::Clipboard => false,
                    ButtonAction::Settings => false,
                };
                if let Some(icon) = button.icon {
                    d.draw_texture_pro(
                        &theme.button_icons[self.scale],
                        icon.icon_cell_irec(self.scale.icon_width()).as_rec(),
                        button_rec,
                        Vector2::zero(),
                        0.0,
                        match (is_selected, is_hovered) {
                            (true, false) => theme.foreground,
                            (false, true) | (true, true) => theme.foreground1,
                            (false, false) => theme.foreground2,
                        },
                    );
                } else {
                    let Rectangle {
                        x,
                        y,
                        width,
                        height,
                    } = button_rec;
                    if let Some(outline) = match (is_selected, is_hovered) {
                        (true, false) => Some(theme.foreground),
                        (false, true) | (true, true) => Some(theme.foreground1),
                        (false, false) => None,
                    } {
                        d.draw_rectangle_rec(Rectangle::new(x, y, width, height), outline);
                    }
                    if let Some(color) = button.color {
                        d.draw_rectangle_rec(
                            Rectangle::new(x + 1.0, y + 1.0, width - 2.0, height - 2.0),
                            color.get(theme),
                        );
                    }
                }
            }
        })
    }
}
