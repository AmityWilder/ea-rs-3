use crate::{
    graph::{node::Gate, wire::Elbow},
    icon_sheets::{ButtonIconId, ButtonIconSheetId, ButtonIconSheets},
    ivec::{IBounds, IRect, IVec2},
    rich_text::ColorRef,
    theme::Theme,
    tool::Tool,
};
use raylib::prelude::*;
use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ButtonGroup(Vec<Button>);

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
            Visibility::Expanded => self.0.len().div_ceil(3),
            Visibility::Collapsed => self.0.len(),
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
        match visibility {
            Visibility::Expanded => self.0.chunks(3),
            Visibility::Collapsed => self.0.chunks(1),
            Visibility::Hidden => [].as_slice().chunks(1),
        }
        .enumerate()
        .flat_map(move |(row, seg)| {
            seg.iter().enumerate().map(move |(col, button)| {
                let (x, y) = match orientation {
                    Orientation::Horizontal => (row, col),
                    Orientation::Vertical => (col, row),
                };
                (
                    IVec2::new(
                        i32::try_from(x).unwrap() * icon_width * (1 + gap) - gap,
                        i32::try_from(y).unwrap() * icon_width * (1 + gap) - gap,
                    ),
                    button,
                )
            })
        })
    }
}

#[derive(Debug, Clone)]
pub struct ToolPane {
    pub tool: Tool,
    pub gate: Gate,
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
            elbow,
            anchoring,
            visibility,
            scale,
            button_groups: vec![
                ButtonGroup(vec![
                    Button {
                        text: None,
                        tooltip: None,
                        desc: None,
                        color: None,
                        icon: Some(ButtonIconId::Pen),
                    },
                    Button {
                        text: None,
                        tooltip: None,
                        desc: None,
                        color: None,
                        icon: Some(ButtonIconId::Edit),
                    },
                    Button {
                        text: None,
                        tooltip: None,
                        desc: None,
                        color: None,
                        icon: Some(ButtonIconId::Erase),
                    },
                    Button {
                        text: None,
                        tooltip: None,
                        desc: None,
                        color: None,
                        icon: Some(ButtonIconId::BlueprintSelect),
                    },
                    Button {
                        text: None,
                        tooltip: None,
                        desc: None,
                        color: None,
                        icon: Some(ButtonIconId::Interact),
                    },
                    Button {
                        text: None,
                        tooltip: None,
                        desc: None,
                        color: None,
                        icon: Some(ButtonIconId::Clipboard),
                    },
                ]),
                ButtonGroup(vec![
                    Button {
                        text: None,
                        tooltip: None,
                        desc: None,
                        color: None,
                        icon: Some(ButtonIconId::Or),
                    },
                    Button {
                        text: None,
                        tooltip: None,
                        desc: None,
                        color: None,
                        icon: Some(ButtonIconId::And),
                    },
                    Button {
                        text: None,
                        tooltip: None,
                        desc: None,
                        color: None,
                        icon: Some(ButtonIconId::Nor),
                    },
                    Button {
                        text: None,
                        tooltip: None,
                        desc: None,
                        color: None,
                        icon: Some(ButtonIconId::Xor),
                    },
                    Button {
                        text: None,
                        tooltip: None,
                        desc: None,
                        color: None,
                        icon: Some(ButtonIconId::Resistor),
                    },
                    Button {
                        text: None,
                        tooltip: None,
                        desc: None,
                        color: None,
                        icon: Some(ButtonIconId::Capacitor),
                    },
                    Button {
                        text: None,
                        tooltip: None,
                        desc: None,
                        color: None,
                        icon: Some(ButtonIconId::Led),
                    },
                    Button {
                        text: None,
                        tooltip: None,
                        desc: None,
                        color: None,
                        icon: Some(ButtonIconId::Delay),
                    },
                    Button {
                        text: None,
                        tooltip: None,
                        desc: None,
                        color: None,
                        icon: Some(ButtonIconId::Battery),
                    },
                ]),
                ButtonGroup(vec![
                    Button {
                        text: Some("9"),
                        tooltip: None,
                        desc: None,
                        color: Some(ColorRef::Exact(Color::WHITE)),
                        icon: None,
                    },
                    Button {
                        text: Some("8"),
                        tooltip: None,
                        desc: None,
                        color: Some(ColorRef::Exact(Color::GRAY)),
                        icon: None,
                    },
                    Button {
                        text: Some("7"),
                        tooltip: None,
                        desc: None,
                        color: Some(ColorRef::Exact(Color::PURPLE)),
                        icon: None,
                    },
                    Button {
                        text: Some("6"),
                        tooltip: None,
                        desc: None,
                        color: Some(ColorRef::Exact(Color::BLUE)),
                        icon: None,
                    },
                    Button {
                        text: Some("5"),
                        tooltip: None,
                        desc: None,
                        color: Some(ColorRef::Exact(Color::GREEN)),
                        icon: None,
                    },
                    Button {
                        text: Some("4"),
                        tooltip: None,
                        desc: None,
                        color: Some(ColorRef::Exact(Color::YELLOW)),
                        icon: None,
                    },
                    Button {
                        text: Some("3"),
                        tooltip: None,
                        desc: None,
                        color: Some(ColorRef::Exact(Color::ORANGE)),
                        icon: None,
                    },
                    Button {
                        text: Some("2"),
                        tooltip: None,
                        desc: None,
                        color: Some(ColorRef::Exact(Color::RED)),
                        icon: None,
                    },
                    Button {
                        text: Some("1"),
                        tooltip: None,
                        desc: None,
                        color: Some(ColorRef::Exact(Color::BROWN)),
                        icon: None,
                    },
                    Button {
                        text: Some("0"),
                        tooltip: None,
                        desc: None,
                        color: Some(ColorRef::Exact(Color::BLACK)),
                        icon: None,
                    },
                ]),
                ButtonGroup(vec![Button {
                    text: None,
                    tooltip: None,
                    desc: None,
                    color: None,
                    icon: Some(ButtonIconId::Settings),
                }]),
            ],
        }
    }

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
        theme: &Theme,
        button_icon_sheets: &ButtonIconSheets,
    ) where
        D: RaylibDraw,
    {
        let IRect { x, y, w, h } = self.bounds(container_width, container_height, theme).into();

        d.draw_rectangle(x, y, w, h, theme.background2);
        d.draw_rectangle(x + 1, y + 1, w - 2, h - 2, theme.background1);

        for (button_rec, button) in self.buttons(IVec2::new(x, y), theme) {
            if let Some(icon) = button.icon {
                button_icon_sheets.draw(
                    d,
                    self.scale,
                    button_rec.as_rect(),
                    icon,
                    Vector2::zero(),
                    0.0,
                    theme.foreground,
                );
            } else {
                let IRect { x, y, w, h } = button_rec;
                d.draw_rectangle(x, y, w, h, theme.background2);
                if let Some(color) = button.color {
                    d.draw_rectangle(x + 1, y + 1, w - 2, h - 2, color.get(theme));
                }
            }
        }
    }
}
