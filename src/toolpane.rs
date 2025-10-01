use crate::{
    graph::{
        node::{Gate, GateId},
        wire::Elbow,
    },
    icon_sheets::{ButtonIconId, ButtonIconSheetId, ButtonIconSheets},
    input::Inputs,
    ivec::{AsIVec2, IBounds, IRect, IVec2},
    rich_text::ColorRef,
    theme::Theme,
    tool::{Tool, ToolId},
};
use raylib::prelude::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    #[default]
    Expanded,
    Collapsed,
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Orientation {
    Horizontal,
    #[default]
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolPaneAnchoring {
    /// ```not_code
    /// .-.---------.-.
    /// | '---------' |
    /// |             |
    /// |             |
    /// '-------------'
    /// ```
    Top { left: i32 },
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
    Left { top: i32 },
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
            | Self::LeftFull { .. } => Orientation::Vertical,
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

    /// get `position` from [`Self::bounds`]
    pub fn buttons(
        &self,
        position: IVec2,
        theme: &Theme,
    ) -> impl Iterator<Item = (IRect, &Button)> {
        let orientation = self.anchoring.orientation();
        let visibility = self.visibility;
        let scale = self.scale;
        let icon_width = scale.icon_width();
        let group_gap = match visibility {
            Visibility::Expanded => theme.toolpane_group_expanded_gap,
            Visibility::Collapsed => theme.toolpane_group_collapsed_gap,
            Visibility::Hidden => 0,
        };
        let (mut along, padding) = match orientation {
            Orientation::Horizontal => (
                position.x,
                IVec2::new(theme.toolpane_padding_along, theme.toolpane_padding_across),
            ),
            Orientation::Vertical => (
                position.y,
                IVec2::new(theme.toolpane_padding_across, theme.toolpane_padding_along),
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
                along += i32::try_from(group.rows(visibility)).unwrap() * icon_width + group_gap;
                it
            })
            .map(move |((x, y), (v, btn))| {
                (IRect::new(v.x + x, v.y + y, icon_width, icon_width), btn)
            })
    }

    fn thickness(&self, theme: &Theme) -> i32 {
        self.scale.icon_width()
            * match self.visibility {
                Visibility::Expanded => 3,
                Visibility::Collapsed => 1,
                Visibility::Hidden => 0,
            }
            + 2 * theme.toolpane_padding_across
    }

    fn length(&self, container_length: i32, theme: &Theme) -> i32 {
        match self.anchoring {
            ToolPaneAnchoring::TopFull | ToolPaneAnchoring::LeftFull => return container_length,
            _ => {}
        }
        let group_gap = match self.visibility {
            Visibility::Expanded => theme.toolpane_group_expanded_gap,
            Visibility::Collapsed => theme.toolpane_group_collapsed_gap,
            Visibility::Hidden => 0,
        };
        self.scale.icon_width()
            * self
                .button_groups
                .iter()
                .map(|g| i32::try_from(g.rows(self.visibility)).unwrap())
                .sum::<i32>()
            + group_gap * (i32::try_from(self.button_groups.len()).unwrap() - 1)
            + 2 * theme.toolpane_padding_along
    }

    pub fn bounds(&self, container_width: i32, container_height: i32, theme: &Theme) -> IBounds {
        match self.anchoring {
            ToolPaneAnchoring::Top { left } => IBounds::new(
                IVec2::new(left, 0),
                IVec2::new(
                    left + self.length(container_width, theme),
                    self.thickness(theme),
                ),
            ),
            ToolPaneAnchoring::TopLeft => IBounds::new(
                IVec2::new(0, 0),
                IVec2::new(self.length(container_width, theme), self.thickness(theme)),
            ),
            ToolPaneAnchoring::TopRight => IBounds::new(
                IVec2::new(container_width - self.length(container_width, theme), 0),
                IVec2::new(container_width, self.thickness(theme)),
            ),
            ToolPaneAnchoring::TopFull => IBounds::new(
                IVec2::new(0, 0),
                IVec2::new(container_width, self.thickness(theme)),
            ),
            ToolPaneAnchoring::Left { top } => IBounds::new(
                IVec2::new(0, top),
                IVec2::new(
                    self.thickness(theme),
                    top + self.length(container_height, theme),
                ),
            ),
            ToolPaneAnchoring::LeftTop => IBounds::new(
                IVec2::new(0, 0),
                IVec2::new(self.thickness(theme), self.length(container_height, theme)),
            ),
            ToolPaneAnchoring::LeftBottom => IBounds::new(
                IVec2::new(0, container_height - self.length(container_height, theme)),
                IVec2::new(self.thickness(theme), container_height),
            ),
            ToolPaneAnchoring::LeftFull => IBounds::new(
                IVec2::new(0, 0),
                IVec2::new(self.thickness(theme), container_height),
            ),
        }
    }

    pub fn draw<D>(
        &self,
        d: &mut D,
        container_width: i32,
        container_height: i32,
        input: &Inputs,
        theme: &Theme,
        button_icon_sheets: &ButtonIconSheets,
    ) where
        D: RaylibDraw,
    {
        let IRect { x, y, w, h } = self.bounds(container_width, container_height, theme).into();

        d.draw_rectangle(x, y, w, h, theme.background2);
        d.draw_rectangle(x + 1, y + 1, w - 2, h - 2, theme.background1);

        let cusor = input.cursor.as_ivec2();

        for (button_rec, button) in self.buttons(IVec2::new(x, y), theme) {
            if let Some(icon) = button.icon {
                let is_hovered = IBounds::from(button_rec).contains(cusor);
                let is_selected = match button.action {
                    ButtonAction::SetTool(tool_id) => tool_id == self.tool.id(),
                    ButtonAction::SetGate(gate_id) => gate_id == self.gate.id(),
                    ButtonAction::SetNtd(data) => data == self.ntd,
                    ButtonAction::Blueprints => false,
                    ButtonAction::Clipboard => false,
                    ButtonAction::Settings => false,
                };
                button_icon_sheets.draw(
                    d,
                    self.scale,
                    button_rec.as_rect(),
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
                let IRect { x, y, w, h } = button_rec;
                // d.draw_rectangle(x, y, w, h, theme.background2);
                if let Some(color) = button.color {
                    d.draw_rectangle(x, y, w - 1, h - 1, color.get(theme));
                }
            }
        }
    }
}
