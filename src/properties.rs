use crate::{
    console::{GateRef, GraphRef, NodeRef, PositionRef, ToolRef, WireRef},
    ivec::{IBounds, IRect, IVec2},
    theme::{Fonts, Theme},
};
use raylib::prelude::*;

#[derive(Debug, Clone)]
pub enum Value {
    Label,
    Text(String),
    Bool(bool),
    Signed(i128),
    Unsigned(u128),
    Float(f64),
    Color(Color),
    Point(PositionRef),
    Graph(GraphRef),
    Node(NodeRef),
    Wire(WireRef),
    Gate(GateRef),
    Tool(ToolRef),
}

macro_rules! into_value {
    ($($value:ident: $T:ty => $f:expr),* $(,)?) => {
        $(impl From<$T> for Value {
            fn from($value: $T) -> Self {
                use Value::*;
                $f
            }
        })*
    };
}

into_value! {
    s: String => Text(s),
    s: &str => Text(s.to_string()),
    b: bool => Bool(b),
    n: i128 => Signed(n),
    n: i64 => Signed(n.into()),
    n: i32 => Signed(n.into()),
    n: i16 => Signed(n.into()),
    n: i8 => Signed(n.into()),
    n: isize => Signed(n as i128),
    n: u128 => Unsigned(n),
    n: u64 => Unsigned(n.into()),
    n: u32 => Unsigned(n.into()),
    n: u16 => Unsigned(n.into()),
    n: u8 => Unsigned(n.into()),
    n: usize => Unsigned(n as u128),
    x: f64 => Float(x),
    x: f32 => Float(x.into()),
    c: Color => Color(c),
    p: PositionRef => Point(p),
    p: IVec2 => Point(PositionRef(p)),
    id: GraphRef => Graph(id),
    id: NodeRef => Node(id),
    id: WireRef => Wire(id),
    id: GateRef => Gate(id),
    id: ToolRef => Tool(id),
}

#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    pub value: Value,
}

impl Property {
    pub fn new<T: Into<Value>>(name: &str, value: T) -> Self {
        Self {
            name: name.replace('\n', "  "),
            value: value.into(),
        }
    }

    pub const fn label(name: String) -> Self {
        Self {
            name,
            value: Value::Label,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PropertyGroup {
    pub name: String,
    pub items: Vec<Property>,
}

impl PropertyGroup {
    pub fn new(name: &str) -> Self {
        Self::with_data(name, Vec::new())
    }

    pub fn with_data(name: &str, items: Vec<Property>) -> Self {
        Self {
            name: name.replace('\n', "  "),
            items,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PropertiesPanel {
    pub bounds: IBounds,
    pub data: Vec<PropertyGroup>,
}

impl PropertiesPanel {
    pub const fn new(bounds: IBounds) -> Self {
        Self {
            bounds,
            data: Vec::new(),
        }
    }

    pub const fn with_data(bounds: IBounds, data: Vec<PropertyGroup>) -> Self {
        Self { bounds, data }
    }

    pub fn draw<D>(&self, d: &mut D, theme: &Theme, fonts: &Fonts)
    where
        D: RaylibDraw,
    {
        let IRect { x, y, w, h } = self.bounds.into();
        d.draw_rectangle(x, y, w, h, theme.background2);
        d.draw_rectangle(x + 1, y + 1, w - 2, h - 2, theme.background1);
        let x = x as f32;
        let mut y = y as f32;

        // data
        {
            for group in &self.data {
                d.draw_text_ex(
                    &fonts.general,
                    &group.name,
                    rvec2(x, y),
                    theme.console_font_size,
                    theme.console_char_spacing,
                    theme.foreground,
                );
                y += theme.console_line_height() * group.name.lines().count() as f32;
                for item in &group.items {
                    d.draw_text_ex(
                        &fonts.general,
                        &item.name,
                        rvec2(x, y),
                        theme.console_font_size,
                        theme.console_char_spacing,
                        theme.foreground,
                    );

                    y += theme.console_line_height() * item.name.lines().count() as f32;
                }
            }
        }

        // title
        {
            let title = "Properties";
            let title_text_size = fonts.general.measure_text(
                title,
                theme.console_font_size,
                theme.console_char_spacing,
            );
            let title_width = title_text_size.x + 2.0 * theme.title_padding_x;
            let title_height = title_text_size.y + 2.0 * theme.title_padding_y;
            d.draw_rectangle_rec(
                Rectangle::new(
                    self.bounds.max.x as f32 - title_width,
                    self.bounds.min.y as f32,
                    title_width,
                    title_height,
                ),
                theme.background2,
            );
            d.draw_text_ex(
                &fonts.general,
                title,
                Vector2::new(
                    self.bounds.max.x as f32 - title_width + theme.title_padding_x,
                    self.bounds.min.y as f32 + theme.title_padding_y,
                ),
                theme.console_font_size,
                theme.console_char_spacing,
                theme.foreground,
            );
        }
    }
}
