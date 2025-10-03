use std::borrow::Cow;

use crate::{
    console::{GateRef, NodeRef, ToolRef, WireRef},
    ivec::Bounds,
    theme::Theme,
};
use raylib::prelude::*;

pub trait DrawPropertyGroup<D: RaylibDraw> {
    fn draw(&self, d: &mut D, bounds: Bounds, theme: &Theme);
}

pub trait PropertyGroup {
    fn title(&self) -> Cow<'_, str>;
    fn content_height(&self, theme: &Theme) -> f32;
}

#[derive(Debug, Clone)]
pub enum PropertySection {
    Tool(ToolRef),
    Gate(GateRef),
    Node(NodeRef),
    Wire(WireRef),
}

impl PropertyGroup for PropertySection {
    fn title(&self) -> Cow<'_, str> {
        match self {
            Self::Tool(_) => Cow::Borrowed("Tool"),
            Self::Gate(_) => Cow::Borrowed("Gate"),
            Self::Node(_) => Cow::Borrowed("Node"),
            Self::Wire(_) => Cow::Borrowed("Wire"),
        }
    }

    fn content_height(&self, theme: &Theme) -> f32 {
        theme.console_font.line_height()
    }
}

impl<D: RaylibDraw> DrawPropertyGroup<D> for PropertySection {
    fn draw(&self, d: &mut D, bounds: Bounds, theme: &Theme) {
        theme
            .general_font
            .draw_text(d, "eee", bounds.min, theme.foreground);
    }
}

#[derive(Debug, Clone)]
pub struct PropertiesPanel {
    pub bounds: Bounds,
    pub sections: Vec<PropertySection>,
}

impl PropertiesPanel {
    pub const fn new(bounds: Bounds) -> Self {
        Self {
            bounds,
            sections: Vec::new(),
        }
    }

    pub fn draw<D>(&self, d: &mut D, theme: &Theme)
    where
        D: RaylibDraw,
    {
        let Rectangle {
            x,
            mut y,
            width,
            height,
        } = self.bounds.into();
        d.draw_rectangle_rec(Rectangle::new(x, y, width, height), theme.background2);
        d.draw_rectangle_rec(
            Rectangle::new(x + 1.0, y + 1.0, width - 2.0, height - 2.0),
            theme.background1,
        );

        // data
        {
            for group in &self.sections {
                theme.general_font.draw_text(
                    d,
                    group.title().as_ref(),
                    rvec2(x, y),
                    theme.foreground,
                );
                y += theme.console_font.line_height() * group.title().lines().count() as f32;
                let height = group.content_height(theme);
                group.draw(
                    d,
                    Rectangle::new(x, y, self.bounds.max.x - self.bounds.min.x, height).into(),
                    theme,
                );
            }
        }

        // title
        {
            let title = "Properties";
            let title_text_size = theme.general_font.measure_text(title);
            let title_width = title_text_size.x + theme.title_padding.horizontal();
            let title_height = title_text_size.y + theme.title_padding.vertical();
            d.draw_rectangle_rec(
                Rectangle::new(
                    self.bounds.max.x - title_width,
                    self.bounds.min.y,
                    title_width,
                    title_height,
                ),
                theme.background2,
            );
            theme.general_font.draw_text(
                d,
                title,
                Vector2::new(
                    self.bounds.max.x - title_width + theme.title_padding.left,
                    self.bounds.min.y + theme.title_padding.top,
                ),
                theme.foreground,
            );
        }
    }
}
