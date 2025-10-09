use crate::{
    graph::node::{Gate, Node},
    icon_sheets::{ButtonIconId, ButtonIconSheetId},
    input::Inputs,
    ivec::Bounds,
    theme::{Theme, ThemeFont},
    tool::Tool,
    ui::{Panel, PanelContent},
};
use raylib::prelude::*;

fn wrap_text(s: &str, container_width: f32, font: &ThemeFont) -> String {
    // size is not changed, some spaces are just replaced with newlines
    let mut string = String::with_capacity(s.len());
    let mut it = s.split(' ');
    if let Some(word) = it.next().as_ref() {
        let space_width = font.measure_text(" ").x + font.char_spacing * 2.0;
        let mut line_width = font.measure_text(word).x;
        string.push_str(word);
        for word in it {
            let word_width = font.measure_text(word).x;
            let new_line_width = line_width + space_width + word_width;
            let sep;
            (line_width, sep) = if new_line_width < container_width {
                (new_line_width, ' ')
            } else {
                (word_width, '\n')
            };
            string.push(sep);
            string.push_str(word);
        }
    }
    string
}

pub trait DrawPropertySection<D: RaylibDraw>: PropertySection {
    fn draw(&self, d: &mut D, container: Bounds, theme: &Theme);
}

pub trait PropertySection: std::fmt::Debug {
    fn title(&self) -> &str;
    fn content_height(&self, container_width: f32, theme: &Theme) -> f32;
    fn tick(
        &mut self,
        rl: &RaylibHandle,
        thread: &RaylibThread,
        container: Bounds,
        theme: &Theme,
        input: &Inputs,
    );
}

fn tool_data(tool: &Tool) -> (ButtonIconId, &'static str, &'static str) {
    match tool {
        Tool::Create { .. } => (
            ButtonIconId::Pen,
            "Create",
            "Place nodes with primary input. Placing or clicking a node automatically begins \
            creating a wire that will connect to the next placed or clicked node.",
        ),
        Tool::Erase { .. } => (
            ButtonIconId::Erase,
            "Erase",
            "Click nodes to delete them. A deleted node will delete all its wires as well.",
        ),
        Tool::Edit { .. } => (
            ButtonIconId::Edit,
            "Edit",
            "Drag nodes with primary input. Replace the gate of the selected node(s) with secondary input.",
        ),
        Tool::Interact { .. } => (
            ButtonIconId::Interact,
            "Interact",
            "Interact with input nodes using primary input to toggle them on and off",
        ),
    }
}

impl PropertySection for Tool {
    #[inline]
    fn title(&self) -> &str {
        "Tool"
    }

    fn content_height(&self, container_width: f32, theme: &Theme) -> f32 {
        let (_, name, desc) = tool_data(self);
        theme
            .general_font
            .measure_text(name)
            .y
            .max(ButtonIconSheetId::X32.icon_width() as f32)
            + theme
                .general_font
                .measure_text(&wrap_text(desc, container_width, &theme.general_font))
                .y
    }

    fn tick(
        &mut self,
        _rl: &RaylibHandle,
        _thread: &RaylibThread,
        _container: Bounds,
        _theme: &Theme,
        _input: &Inputs,
    ) {
        // TODO
    }
}

impl<D: RaylibDraw> DrawPropertySection<D> for Tool {
    fn draw(&self, d: &mut D, container: Bounds, theme: &Theme) {
        let icon_scale = ButtonIconSheetId::X32;
        let icon_width = icon_scale.icon_width();
        let (icon_id, name, desc) = tool_data(self);
        let space_width = theme.general_font.measure_text(" ").x;
        let text_size = theme.general_font.measure_text(name);
        let rec = Rectangle::new(
            container.min.x,
            container.min.y,
            container.width(),
            text_size.y.max(icon_width as f32),
        );
        d.draw_rectangle_rec(rec, theme.background2);
        d.draw_texture_pro(
            &theme.button_icons[icon_scale],
            icon_id.icon_cell_irec(icon_width).as_rec(),
            Rectangle::new(
                container.min.x,
                container.min.y + 0.5 * (rec.height - icon_width as f32),
                icon_width as f32,
                icon_width as f32,
            ),
            Vector2::zero(),
            0.0,
            theme.foreground,
        );
        theme.general_font.draw_text(
            d,
            name,
            Vector2::new(
                container.min.x + space_width + icon_width as f32,
                container.min.y + 0.5 * (rec.height - text_size.y),
            ),
            theme.foreground,
        );
        theme.general_font.draw_text(
            d,
            &wrap_text(desc, container.width(), &theme.general_font),
            Vector2::new(
                container.min.x,
                container.min.y + rec.height + theme.general_font.line_spacing,
            ),
            theme.foreground,
        );
    }
}

fn gate_data(gate: &Gate) -> (ButtonIconId, &'static str, &'static str) {
    match gate {
        Gate::Or => (
            ButtonIconId::Or,
            "Or",
            "True when one input or more is true.",
        ),
        Gate::And => (
            ButtonIconId::And,
            "And",
            "True when every input is true and at least one input exists.",
        ),
        Gate::Nor => (ButtonIconId::Nor, "Nor", "True when every input is false."),
        Gate::Xor => (
            ButtonIconId::Xor,
            "Xor",
            "True when exactly one input is true.",
        ),
        Gate::Resistor { .. } => (
            ButtonIconId::Resistor,
            "Resistor",
            "True when the number of true inputs exceed the NTD value.",
        ),
        Gate::Capacitor { .. } => (
            ButtonIconId::Capacitor,
            "Capacitor",
            "Stores the quantity of true inputs up to a maximum of the NTD value, \
            losing charge every tick that no input is true. True as long as the charge is not zero.",
        ),
        Gate::Led { .. } => (
            ButtonIconId::Led,
            "Led",
            "Like Or, but in Inspect mode, fills its cell with the color of the NTD value when true.",
        ),
        Gate::Delay => (
            ButtonIconId::Delay,
            "Delay",
            "Like Or, but gives the previous output that would have been given the previous tick.",
        ),
        Gate::Battery => (ButtonIconId::Battery, "Battery", "Always true."),
    }
}

impl PropertySection for Gate {
    #[inline]
    fn title(&self) -> &str {
        "Gate"
    }

    fn content_height(&self, container_width: f32, theme: &Theme) -> f32 {
        let (_, name, desc) = gate_data(self);
        theme
            .general_font
            .measure_text(name)
            .y
            .max(ButtonIconSheetId::X32.icon_width() as f32)
            + theme
                .general_font
                .measure_text(&wrap_text(desc, container_width, &theme.general_font))
                .y
    }

    fn tick(
        &mut self,
        _rl: &RaylibHandle,
        _thread: &RaylibThread,
        _container: Bounds,
        _theme: &Theme,
        _input: &Inputs,
    ) {
        // TODO
    }
}

impl<D: RaylibDraw> DrawPropertySection<D> for Gate {
    fn draw(&self, d: &mut D, container: Bounds, theme: &Theme) {
        let icon_scale = ButtonIconSheetId::X32;
        let icon_width = icon_scale.icon_width();
        let (icon_id, name, desc) = gate_data(self);
        let space_width = theme.general_font.measure_text(" ").x;
        let text_size = theme.general_font.measure_text(name);
        let rec = Rectangle::new(
            container.min.x,
            container.min.y,
            container.width(),
            text_size.y.max(icon_width as f32),
        );
        d.draw_rectangle_rec(rec, theme.background2);
        d.draw_texture_pro(
            &theme.button_icons[icon_scale],
            icon_id.icon_cell_irec(icon_width).as_rec(),
            Rectangle::new(
                container.min.x,
                container.min.y + 0.5 * (rec.height - icon_width as f32),
                icon_width as f32,
                icon_width as f32,
            ),
            Vector2::zero(),
            0.0,
            theme.foreground,
        );
        theme.general_font.draw_text(
            d,
            name,
            Vector2::new(
                container.min.x + space_width + icon_width as f32,
                container.min.y + 0.5 * (rec.height - text_size.y),
            ),
            theme.foreground,
        );
        theme.general_font.draw_text(
            d,
            &wrap_text(desc, container.width(), &theme.general_font),
            Vector2::new(
                container.min.x,
                container.min.y + rec.height + theme.general_font.line_spacing,
            ),
            theme.foreground,
        );
    }
}

impl PropertySection for Node {
    #[inline]
    fn title(&self) -> &str {
        "Node"
    }

    fn content_height(&self, _container_width: f32, _theme: &Theme) -> f32 {
        0.0
    }

    fn tick(
        &mut self,
        _rl: &RaylibHandle,
        _thread: &RaylibThread,
        _container: Bounds,
        _theme: &Theme,
        _input: &Inputs,
    ) {
        // TODO
    }
}

impl<D: RaylibDraw> DrawPropertySection<D> for Node {
    fn draw(&self, _d: &mut D, _container: Bounds, _theme: &Theme) {}
}

#[derive(Debug, Clone)]
pub struct PropertiesPanel {
    pub panel: Panel,
}

impl PanelContent for PropertiesPanel {
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

impl PropertiesPanel {
    pub const fn new(panel: Panel) -> Self {
        Self { panel }
    }

    pub fn tick_section<T>(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        theme: &Theme,
        input: &Inputs,
        mut y: f32,
        section: &mut T,
    ) -> f32
    where
        T: PropertySection,
    {
        // self.panel.tick_resize(rl, theme, input);
        let bounds = self.panel.content_bounds(theme);
        y += theme.console_font.line_height() * section.title().lines().count() as f32;
        let height = section.content_height(bounds.width(), theme);
        section.tick(
            rl,
            thread,
            Bounds::new(
                Vector2::new(bounds.max.x, y),
                Vector2::new(bounds.min.x, y + height),
            ),
            theme,
            input,
        );
        y
    }

    pub fn tick<T, F>(&mut self, theme: &Theme, f: F) -> T
    where
        F: FnOnce(&mut Self, Bounds, &Theme) -> T,
    {
        let bounds = self.panel.content_bounds(theme);
        f(self, bounds, theme)
    }

    pub fn draw_section<D, T>(
        &self,
        d: &mut D,
        theme: &Theme,
        bounds: Bounds,
        mut y: f32,
        section: &T,
    ) -> f32
    where
        D: RaylibDraw,
        T: DrawPropertySection<D>,
    {
        let header_size = theme.properties_header_font.measure_text(section.title());
        theme.properties_header_font.draw_text(
            d,
            section.title(),
            Vector2::new(bounds.min.x, y),
            theme.foreground,
        );
        y += header_size.y;
        y += theme.properties_header_font.line_spacing;
        d.draw_rectangle_rec(
            Rectangle::new(bounds.min.x, y, bounds.width(), 1.0),
            theme.foreground2,
        );
        y += theme.properties_header_font.line_spacing + theme.general_font.line_spacing;
        let height = section.content_height(bounds.width(), theme);
        section.draw(
            d,
            Bounds::new(
                Vector2::new(bounds.min.x, y.clamp(bounds.min.y, bounds.max.y)),
                Vector2::new(bounds.max.x, (y + height).clamp(bounds.min.y, bounds.max.y)),
            ),
            theme,
        );
        y += height + theme.properties_section_gap;
        y
    }

    pub fn draw<D, F>(&self, d: &mut D, theme: &Theme, f: F)
    where
        D: RaylibDraw,
        F: FnOnce(&Self, &mut D, Bounds, &Theme),
    {
        self.panel
            .draw(d, theme, |d, bounds, theme| f(self, d, bounds, theme));
    }
}
