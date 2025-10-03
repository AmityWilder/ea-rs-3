use crate::{
    console::{Console, GateRef, LogType, ToolRef},
    graph::{
        node::{Gate, GateId},
        wire::Elbow,
    },
    icon_sheets::{ButtonIconId, ButtonIconSheetId, ButtonIconSheets},
    input::Inputs,
    ivec::{Bounds, IVec2},
    logln,
    rich_text::ColorRef,
    theme::Theme,
    tool::{Tool, ToolId},
    ui::{Orientation, Visibility},
};
use raylib::prelude::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolPaneAnchoring {
    /// ```not_code
    /// .-.---------.-.
    /// | '---------' |
    /// |             |
    /// |             |
    /// '-------------'
    /// ```
    Top { left: f32 },
    /// ```not_code
    /// .-----------.-.
    /// |-----------' |
    /// |             |
    /// |             |
    /// '-------------'
    /// ```
    TopLeft,
    /// ```not_code
    /// .-.-----------.
    /// | '-----------|
    /// |             |
    /// |             |
    /// '-------------'
    /// ```
    TopRight,
    /// ```not_code
    /// .-------------.
    /// |-------------|
    /// |             |
    /// |             |
    /// '-------------'
    /// ```
    TopFull,
    /// ```not_code
    /// .-------------.
    /// |-.           |
    /// | |           |
    /// |-'           |
    /// '-------------'
    /// ```
    Left { top: f32 },
    /// ```not_code
    /// .-.-----------.
    /// | |           |
    /// | |           |
    /// |-'           |
    /// '-------------'
    /// ```
    LeftTop,
    /// ```not_code
    /// .-------------.
    /// |-.           |
    /// | |           |
    /// | |           |
    /// '-'-----------'
    /// ```
    LeftBottom,
    /// ```not_code
    /// .-.-----------.
    /// | |           |
    /// | |           |
    /// | |           |
    /// '-'-----------'
    /// ```
    #[default]
    LeftFull,
    /// ```not_code
    /// .-------------.
    /// |           .-|
    /// |           | |
    /// |           '-|
    /// '-------------'
    /// ```
    Right { top: f32 },
    /// ```not_code
    /// .-----------.-.
    /// |           | |
    /// |           | |
    /// |           '-|
    /// '-------------'
    /// ```
    RightTop,
    /// ```not_code
    /// .-------------.
    /// |           .-|
    /// |           | |
    /// |           | |
    /// '-----------'-'
    /// ```
    RightBottom,
    /// ```not_code
    /// .-----------.-.
    /// |           | |
    /// |           | |
    /// |           | |
    /// '-----------'-'
    /// ```
    RightFull,
}

impl ToolPaneAnchoring {
    pub const fn orientation(&self) -> Orientation {
        match self {
            Self::Top { .. }
            | Self::TopLeft { .. }
            | Self::TopRight { .. }
            | Self::TopFull { .. } => Orientation::Horizontal,
            Self::Left { .. }
            | Self::LeftTop { .. }
            | Self::LeftBottom { .. }
            | Self::LeftFull { .. }
            | Self::Right { .. }
            | Self::RightTop { .. }
            | Self::RightBottom { .. }
            | Self::RightFull { .. } => Orientation::Vertical,
        }
    }
}

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
    pub fn cols(&self, visibility: Visibility) -> usize {
        match visibility {
            Visibility::Expanded => 3,
            Visibility::Collapsed => 1,
            Visibility::Hidden => 0,
        }
    }

    pub fn rows(&self, visibility: Visibility) -> usize {
        match visibility {
            Visibility::Expanded => self.buttons.len().div_ceil(3),
            Visibility::Collapsed => self.buttons.len(),
            Visibility::Hidden => 0,
        }
    }

    pub fn positions(
        &self,
        icon_width: i32,
        gap: i32,
        visibility: Visibility,
        orientation: Orientation,
    ) -> impl Iterator<Item = (IVec2, &Button)> {
        let chunk_size = match visibility {
            Visibility::Expanded => 3,
            Visibility::Collapsed => 1,
            Visibility::Hidden => 1,
        };
        match visibility {
            Visibility::Expanded | Visibility::Collapsed => self.buttons.as_slice(),
            Visibility::Hidden => [].as_slice(),
        }
        .chunks(chunk_size)
        .enumerate()
        .flat_map(move |(row, seg)| {
            seg.iter().enumerate().map(move |(col, button)| {
                let col = i32::try_from(if self.rev_rows {
                    seg.len() - 1 - col
                } else {
                    col
                })
                .unwrap();
                let row = i32::try_from(row).unwrap();
                let chunk_size = i32::try_from(chunk_size).unwrap();
                let n = i32::try_from(seg.len()).unwrap();
                let across = col + (chunk_size - n) / 2;
                let (x, y) = match orientation {
                    Orientation::Horizontal => (row, across),
                    Orientation::Vertical => (across, row),
                };
                (
                    IVec2::new(
                        x * icon_width * (1 + gap) - gap,
                        y * icon_width * (1 + gap) - gap,
                    ),
                    button,
                )
            })
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonAction {
    SetTool(ToolId),
    SetGate(GateId),
    SetNtd(u8),
    Blueprints,
    Clipboard,
    Settings,
}

#[derive(Debug, Clone)]
pub struct ToolPane {
    pub tool: Tool,
    pub gate: Gate,
    pub ntd: u8,
    pub elbow: Elbow,
    pub anchoring: ToolPaneAnchoring,
    pub visibility: Visibility,
    pub scale: ButtonIconSheetId,
    pub button_groups: Vec<ButtonGroup>,
}

impl ToolPane {
    pub fn new(
        tool: Tool,
        gate: Gate,
        elbow: Elbow,
        anchoring: ToolPaneAnchoring,
        visibility: Visibility,
        scale: ButtonIconSheetId,
    ) -> Self {
        Self {
            tool,
            gate,
            ntd: 0,
            elbow,
            anchoring,
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
                            action: ButtonAction::SetNtd(9),
                        },
                        Button {
                            text: Some("8"),
                            tooltip: None,
                            desc: None,
                            color: Some(ColorRef::Exact(Color::GRAY)),
                            icon: None,
                            action: ButtonAction::SetNtd(8),
                        },
                        Button {
                            text: Some("7"),
                            tooltip: None,
                            desc: None,
                            color: Some(ColorRef::Exact(Color::PURPLE)),
                            icon: None,
                            action: ButtonAction::SetNtd(7),
                        },
                        Button {
                            text: Some("6"),
                            tooltip: None,
                            desc: None,
                            color: Some(ColorRef::Exact(Color::BLUE)),
                            icon: None,
                            action: ButtonAction::SetNtd(6),
                        },
                        Button {
                            text: Some("5"),
                            tooltip: None,
                            desc: None,
                            color: Some(ColorRef::Exact(Color::GREEN)),
                            icon: None,
                            action: ButtonAction::SetNtd(5),
                        },
                        Button {
                            text: Some("4"),
                            tooltip: None,
                            desc: None,
                            color: Some(ColorRef::Exact(Color::YELLOW)),
                            icon: None,
                            action: ButtonAction::SetNtd(4),
                        },
                        Button {
                            text: Some("3"),
                            tooltip: None,
                            desc: None,
                            color: Some(ColorRef::Exact(Color::ORANGE)),
                            icon: None,
                            action: ButtonAction::SetNtd(3),
                        },
                        Button {
                            text: Some("2"),
                            tooltip: None,
                            desc: None,
                            color: Some(ColorRef::Exact(Color::RED)),
                            icon: None,
                            action: ButtonAction::SetNtd(2),
                        },
                        Button {
                            text: Some("1"),
                            tooltip: None,
                            desc: None,
                            color: Some(ColorRef::Exact(Color::BROWN)),
                            icon: None,
                            action: ButtonAction::SetNtd(1),
                        },
                        Button {
                            text: Some("0"),
                            tooltip: None,
                            desc: None,
                            color: Some(ColorRef::Exact(Color::BLACK)),
                            icon: None,
                            action: ButtonAction::SetNtd(0),
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

    pub fn set_tool(&mut self, tool_id: ToolId, console: &mut Console) {
        if self.tool.id() != tool_id {
            self.tool = tool_id.init();
            logln!(console, LogType::Info, "set tool to {}", ToolRef(tool_id));
        }
    }

    pub fn set_gate(&mut self, gate_id: GateId, console: &mut Console) {
        if self.gate.id() != gate_id {
            self.gate = gate_id.to_gate(self.ntd);
            logln!(
                console,
                LogType::Info,
                "set gate to {}",
                GateRef(self.gate.id())
            );
        }
    }

    pub fn set_ntd(&mut self, data: u8, console: &mut Console) {
        if self.ntd != data {
            self.ntd = data;
            logln!(
                console,
                LogType::Info,
                "set non-transistor data to {}",
                self.ntd
            );
        }
    }

    /// get `position` from [`Self::bounds`]
    pub fn buttons(
        &self,
        position: Vector2,
        theme: &Theme,
    ) -> impl Iterator<Item = (Rectangle, &Button)> {
        let orientation = self.anchoring.orientation();
        let visibility = self.visibility;
        let scale = self.scale;
        let icon_width = scale.icon_width();
        let group_gap = match visibility {
            Visibility::Expanded => theme.toolpane_group_expanded_gap,
            Visibility::Collapsed => theme.toolpane_group_collapsed_gap,
            Visibility::Hidden => 0.0,
        };
        let (mut along, padding) = match orientation {
            Orientation::Horizontal => (
                position.x,
                Vector2::new(theme.toolpane_padding.top, theme.toolpane_padding.right),
            ),
            Orientation::Vertical => (
                position.y,
                Vector2::new(theme.toolpane_padding.left, theme.toolpane_padding.top),
            ),
        };
        self.button_groups
            .iter()
            .flat_map(move |group| {
                let it = std::iter::repeat(match orientation {
                    Orientation::Horizontal => {
                        (position.x + padding.x + along, position.y + padding.y)
                    }
                    Orientation::Vertical => {
                        (position.x + padding.x, position.y + padding.y + along)
                    }
                })
                .zip(group.positions(icon_width, 0, visibility, orientation));
                along += group.rows(visibility) as f32 * icon_width as f32 + group_gap;
                it
            })
            .map(move |((x, y), (v, btn))| {
                (
                    Rectangle::new(
                        v.x as f32 + x,
                        v.y as f32 + y,
                        icon_width as f32,
                        icon_width as f32,
                    ),
                    btn,
                )
            })
    }

    fn thickness(&self, theme: &Theme) -> f32 {
        self.scale.icon_width() as f32
            * match self.visibility {
                Visibility::Expanded => 3.0,
                Visibility::Collapsed => 1.0,
                Visibility::Hidden => 0.0,
            }
            + theme.toolpane_padding.horizontal()
    }

    fn length(&self, container_length: f32, theme: &Theme) -> f32 {
        match self.anchoring {
            ToolPaneAnchoring::TopFull | ToolPaneAnchoring::LeftFull => return container_length,
            _ => {}
        }
        let group_gap = match self.visibility {
            Visibility::Expanded => theme.toolpane_group_expanded_gap,
            Visibility::Collapsed => theme.toolpane_group_collapsed_gap,
            Visibility::Hidden => 0.0,
        };
        self.scale.icon_width() as f32
            * self
                .button_groups
                .iter()
                .map(|g| g.rows(self.visibility) as f32)
                .sum::<f32>()
            + group_gap * (self.button_groups.len() as f32 - 1.0)
            + theme.toolpane_padding.vertical()
    }

    pub fn bounds(&self, container_width: f32, container_height: f32, theme: &Theme) -> Bounds {
        match self.anchoring {
            ToolPaneAnchoring::Top { left } => Bounds::new(
                Vector2::new(left, 0.0),
                Vector2::new(
                    left + self.length(container_width, theme),
                    self.thickness(theme),
                ),
            ),
            ToolPaneAnchoring::TopLeft => Bounds::new(
                Vector2::new(0.0, 0.0),
                Vector2::new(self.length(container_width, theme), self.thickness(theme)),
            ),
            ToolPaneAnchoring::TopRight => Bounds::new(
                Vector2::new(container_width - self.length(container_width, theme), 0.0),
                Vector2::new(container_width, self.thickness(theme)),
            ),
            ToolPaneAnchoring::TopFull => Bounds::new(
                Vector2::new(0.0, 0.0),
                Vector2::new(container_width, self.thickness(theme)),
            ),
            ToolPaneAnchoring::Left { top } => Bounds::new(
                Vector2::new(0.0, top),
                Vector2::new(
                    self.thickness(theme),
                    top + self.length(container_height, theme),
                ),
            ),
            ToolPaneAnchoring::LeftTop => Bounds::new(
                Vector2::new(0.0, 0.0),
                Vector2::new(self.thickness(theme), self.length(container_height, theme)),
            ),
            ToolPaneAnchoring::LeftBottom => Bounds::new(
                Vector2::new(0.0, container_height - self.length(container_height, theme)),
                Vector2::new(self.thickness(theme), container_height),
            ),
            ToolPaneAnchoring::LeftFull => Bounds::new(
                Vector2::new(0.0, 0.0),
                Vector2::new(self.thickness(theme), container_height),
            ),
            ToolPaneAnchoring::Right { top } => Bounds::new(
                Vector2::new(container_width - self.thickness(theme), top),
                Vector2::new(container_width, top + self.length(container_height, theme)),
            ),
            ToolPaneAnchoring::RightTop => Bounds::new(
                Vector2::new(container_width - self.thickness(theme), 0.0),
                Vector2::new(container_width, self.length(container_height, theme)),
            ),
            ToolPaneAnchoring::RightBottom => Bounds::new(
                Vector2::new(
                    container_width - self.thickness(theme),
                    container_height - self.length(container_height, theme),
                ),
                Vector2::new(container_width, container_height),
            ),
            ToolPaneAnchoring::RightFull => Bounds::new(
                Vector2::new(container_width - self.thickness(theme), 0.0),
                Vector2::new(container_width, container_height),
            ),
        }
    }

    pub fn draw<D>(
        &self,
        d: &mut D,
        container_width: f32,
        container_height: f32,
        input: &Inputs,
        theme: &Theme,
        button_icon_sheets: &ButtonIconSheets,
    ) where
        D: RaylibDraw,
    {
        let Rectangle {
            x,
            y,
            width,
            height,
        } = self.bounds(container_width, container_height, theme).into();

        d.draw_rectangle_rec(Rectangle::new(x, y, width, height), theme.background2);
        d.draw_rectangle_rec(
            Rectangle::new(x + 1.0, y + 1.0, width - 2.0, height - 2.0),
            theme.background1,
        );

        for (button_rec, button) in self.buttons(Vector2::new(x, y), theme) {
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
                button_icon_sheets.draw(
                    d,
                    self.scale,
                    button_rec,
                    icon,
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
    }
}
