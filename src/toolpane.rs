use crate::{
    graph::node::Gate,
    icon_sheets::{ButtonIconId, ButtonIconSheetId},
    ivec::{IBounds, IRect, IVec2},
    rich_text::ColorRef,
    theme::Theme,
    tool::Tool,
};
use raylib::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ButtonTemplate {
    pub text: Option<&'static str>,
    pub tooltip: Option<&'static str>,
    pub desc: Option<&'static str>,
    pub color: Option<ColorRef>,
    pub icon: Option<ButtonIconId>,
    pub rel_pos: IVec2,
    pub rel_ord: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Button {
    pub text: Option<&'static str>,
    pub tooltip: Option<&'static str>,
    pub desc: Option<&'static str>,
    pub color: ColorRef,
    pub icon: Option<ButtonIconId>,
    pub rec: IRect,
}

#[derive(Debug, Clone)]
pub struct ToolPane {
    pub tool: Tool,
    pub gate: Gate,
    pub anchoring: ToolPaneAnchoring,
    pub collapsed: bool,
    pub scale: ButtonIconSheetId,
}

impl ToolPane {
    pub fn new(
        tool: Tool,
        gate: Gate,
        anchoring: ToolPaneAnchoring,
        scale: ButtonIconSheetId,
    ) -> Self {
        Self {
            tool,
            gate,
            anchoring,
            collapsed: false,
            scale,
        }
    }

    pub fn buttons(&self, theme: &Theme) -> [Button; 26] {
        let scale = self.scale;
        let padding = match self.anchoring {
            ToolPaneAnchoring::Top { .. }
            | ToolPaneAnchoring::TopLeft
            | ToolPaneAnchoring::TopRight
            | ToolPaneAnchoring::TopFull => {
                IVec2::new(theme.toolpane_padding_along, theme.toolpane_padding_across)
            }
            ToolPaneAnchoring::Left { .. }
            | ToolPaneAnchoring::LeftTop
            | ToolPaneAnchoring::LeftBottom
            | ToolPaneAnchoring::LeftFull => {
                IVec2::new(theme.toolpane_padding_across, theme.toolpane_padding_along)
            }
        };
        let width = scale.icon_width();
        [
            ButtonTemplate {
                text: None,
                tooltip: None,
                desc: None,
                color: None,
                icon: Some(ButtonIconId::Pen),
                rel_pos: IVec2::new(0, 0),
                rel_ord: 0,
            },
            ButtonTemplate {
                text: None,
                tooltip: None,
                desc: None,
                color: None,
                icon: Some(ButtonIconId::Edit),
                rel_pos: IVec2::new(1, 0),
                rel_ord: 1,
            },
            ButtonTemplate {
                text: None,
                tooltip: None,
                desc: None,
                color: None,
                icon: Some(ButtonIconId::Erase),
                rel_pos: IVec2::new(2, 0),
                rel_ord: 2,
            },
            ButtonTemplate {
                text: None,
                tooltip: None,
                desc: None,
                color: None,
                icon: Some(ButtonIconId::BlueprintSelect),
                rel_pos: IVec2::new(0, 1),
                rel_ord: 3,
            },
            ButtonTemplate {
                text: None,
                tooltip: None,
                desc: None,
                color: None,
                icon: Some(ButtonIconId::Interact),
                rel_pos: IVec2::new(1, 1),
                rel_ord: 4,
            },
            ButtonTemplate {
                text: None,
                tooltip: None,
                desc: None,
                color: None,
                icon: Some(ButtonIconId::Clipboard),
                rel_pos: IVec2::new(2, 1),
                rel_ord: 5,
            },
            ButtonTemplate {
                text: None,
                tooltip: None,
                desc: None,
                color: None,
                icon: Some(ButtonIconId::Or),
                rel_pos: IVec2::new(0, 3),
                rel_ord: 7,
            },
            ButtonTemplate {
                text: None,
                tooltip: None,
                desc: None,
                color: None,
                icon: Some(ButtonIconId::And),
                rel_pos: IVec2::new(1, 3),
                rel_ord: 8,
            },
            ButtonTemplate {
                text: None,
                tooltip: None,
                desc: None,
                color: None,
                icon: Some(ButtonIconId::Nor),
                rel_pos: IVec2::new(2, 3),
                rel_ord: 9,
            },
            ButtonTemplate {
                text: None,
                tooltip: None,
                desc: None,
                color: None,
                icon: Some(ButtonIconId::Xor),
                rel_pos: IVec2::new(0, 4),
                rel_ord: 10,
            },
            ButtonTemplate {
                text: None,
                tooltip: None,
                desc: None,
                color: None,
                icon: Some(ButtonIconId::Resistor),
                rel_pos: IVec2::new(1, 4),
                rel_ord: 11,
            },
            ButtonTemplate {
                text: None,
                tooltip: None,
                desc: None,
                color: None,
                icon: Some(ButtonIconId::Capacitor),
                rel_pos: IVec2::new(2, 4),
                rel_ord: 12,
            },
            ButtonTemplate {
                text: None,
                tooltip: None,
                desc: None,
                color: None,
                icon: Some(ButtonIconId::Led),
                rel_pos: IVec2::new(0, 5),
                rel_ord: 13,
            },
            ButtonTemplate {
                text: None,
                tooltip: None,
                desc: None,
                color: None,
                icon: Some(ButtonIconId::Delay),
                rel_pos: IVec2::new(1, 5),
                rel_ord: 14,
            },
            ButtonTemplate {
                text: None,
                tooltip: None,
                desc: None,
                color: None,
                icon: Some(ButtonIconId::Battery),
                rel_pos: IVec2::new(2, 5),
                rel_ord: 15,
            },
            ButtonTemplate {
                text: Some("9"),
                tooltip: None,
                desc: None,
                color: Some(ColorRef::Exact(Color::WHITE)),
                icon: None,
                rel_pos: IVec2::new(2, 7),
                rel_ord: 17,
            },
            ButtonTemplate {
                text: Some("8"),
                tooltip: None,
                desc: None,
                color: Some(ColorRef::Exact(Color::GRAY)),
                icon: None,
                rel_pos: IVec2::new(1, 7),
                rel_ord: 18,
            },
            ButtonTemplate {
                text: Some("7"),
                tooltip: None,
                desc: None,
                color: Some(ColorRef::Exact(Color::PURPLE)),
                icon: None,
                rel_pos: IVec2::new(0, 7),
                rel_ord: 19,
            },
            ButtonTemplate {
                text: Some("6"),
                tooltip: None,
                desc: None,
                color: Some(ColorRef::Exact(Color::BLUE)),
                icon: None,
                rel_pos: IVec2::new(2, 8),
                rel_ord: 20,
            },
            ButtonTemplate {
                text: Some("5"),
                tooltip: None,
                desc: None,
                color: Some(ColorRef::Exact(Color::GREEN)),
                icon: None,
                rel_pos: IVec2::new(1, 8),
                rel_ord: 21,
            },
            ButtonTemplate {
                text: Some("4"),
                tooltip: None,
                desc: None,
                color: Some(ColorRef::Exact(Color::YELLOW)),
                icon: None,
                rel_pos: IVec2::new(0, 8),
                rel_ord: 22,
            },
            ButtonTemplate {
                text: Some("3"),
                tooltip: None,
                desc: None,
                color: Some(ColorRef::Exact(Color::ORANGE)),
                icon: None,
                rel_pos: IVec2::new(2, 9),
                rel_ord: 23,
            },
            ButtonTemplate {
                text: Some("2"),
                tooltip: None,
                desc: None,
                color: Some(ColorRef::Exact(Color::RED)),
                icon: None,
                rel_pos: IVec2::new(1, 9),
                rel_ord: 24,
            },
            ButtonTemplate {
                text: Some("1"),
                tooltip: None,
                desc: None,
                color: Some(ColorRef::Exact(Color::BROWN)),
                icon: None,
                rel_pos: IVec2::new(0, 9),
                rel_ord: 23,
            },
            ButtonTemplate {
                text: Some("0"),
                tooltip: None,
                desc: None,
                color: Some(ColorRef::Exact(Color::BLACK)),
                icon: None,
                rel_pos: IVec2::new(1, 10),
                rel_ord: 24,
            },
            ButtonTemplate {
                text: None,
                tooltip: None,
                desc: None,
                color: None,
                icon: Some(ButtonIconId::Settings),
                rel_pos: IVec2::new(1, 12),
                rel_ord: 26,
            },
        ]
        .map(
            |ButtonTemplate {
                 text,
                 tooltip,
                 desc,
                 color,
                 icon,
                 rel_pos,
                 rel_ord,
             }| {
                let pos = if self.collapsed {
                    match self.anchoring {
                        ToolPaneAnchoring::Top { .. }
                        | ToolPaneAnchoring::TopLeft
                        | ToolPaneAnchoring::TopRight
                        | ToolPaneAnchoring::TopFull => IVec2::new(rel_ord, 0),
                        ToolPaneAnchoring::Left { .. }
                        | ToolPaneAnchoring::LeftTop
                        | ToolPaneAnchoring::LeftBottom
                        | ToolPaneAnchoring::LeftFull => IVec2::new(0, rel_ord),
                    }
                } else {
                    rel_pos
                };
                Button {
                    text,
                    tooltip,
                    desc,
                    color: color.unwrap_or(ColorRef::Exact(Color::BLANK)),
                    icon,
                    rec: IRect {
                        x: padding.x + pos.x * width,
                        y: padding.y + pos.y * width,
                        w: width,
                        h: width,
                    },
                }
            },
        )
    }

    fn thickness(&self, theme: &Theme) -> i32 {
        self.scale.icon_width() * if self.collapsed { 1 } else { 3 }
            + 2 * theme.toolpane_padding_across
    }

    fn length(&self, theme: &Theme) -> i32 {
        self.scale.icon_width()
            * if self.collapsed {
                6 + 1 + 9 + 1 + 10
            } else {
                2 + 1 + 3 + 1 + 4
            }
            + 2 * theme.toolpane_padding_along
    }

    pub fn bounds(&self, container_width: i32, container_height: i32, theme: &Theme) -> IBounds {
        match self.anchoring {
            ToolPaneAnchoring::Top { left } => IBounds::new(
                IVec2::new(left, 0),
                IVec2::new(left + self.length(theme), self.thickness(theme)),
            ),
            ToolPaneAnchoring::TopLeft => IBounds::new(
                IVec2::new(0, 0),
                IVec2::new(self.length(theme), self.thickness(theme)),
            ),
            ToolPaneAnchoring::TopRight => IBounds::new(
                IVec2::new(container_width - self.length(theme), 0),
                IVec2::new(container_width, self.thickness(theme)),
            ),
            ToolPaneAnchoring::TopFull => IBounds::new(
                IVec2::new(0, 0),
                IVec2::new(container_width, self.thickness(theme)),
            ),
            ToolPaneAnchoring::Left { top } => IBounds::new(
                IVec2::new(0, top),
                IVec2::new(self.thickness(theme), top + self.length(theme)),
            ),
            ToolPaneAnchoring::LeftTop => IBounds::new(
                IVec2::new(0, 0),
                IVec2::new(self.thickness(theme), self.length(theme)),
            ),
            ToolPaneAnchoring::LeftBottom => IBounds::new(
                IVec2::new(0, container_height - self.length(theme)),
                IVec2::new(self.thickness(theme), container_height),
            ),
            ToolPaneAnchoring::LeftFull => IBounds::new(
                IVec2::new(0, 0),
                IVec2::new(self.thickness(theme), container_height),
            ),
        }
    }
}
