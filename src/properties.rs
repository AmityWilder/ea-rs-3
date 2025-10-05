use crate::{graph::node::Gate, input::Inputs, ivec::Bounds, theme::Theme, tool::Tool, ui::Panel};
use raylib::prelude::*;

pub trait DrawPropertySection<D: RaylibDraw>: PropertySection {
    fn draw(&self, d: &mut D, container: Bounds, theme: &Theme);
}

pub trait PropertySection: std::fmt::Debug {
    fn title(&self) -> &str;
    fn content_height(&self, theme: &Theme) -> f32;
    fn tick(&mut self, rl: &RaylibHandle, thread: &RaylibThread, container: Bounds, theme: &Theme);
}

impl PropertySection for Tool {
    fn title(&self) -> &str {
        "Tool"
    }

    fn content_height(&self, _theme: &Theme) -> f32 {
        todo!()
    }

    fn tick(
        &mut self,
        _rl: &RaylibHandle,
        _thread: &RaylibThread,
        _container: Bounds,
        _theme: &Theme,
    ) {
        todo!()
    }
}

impl<D: RaylibDraw> DrawPropertySection<D> for Tool {
    fn draw(&self, _d: &mut D, _container: Bounds, _theme: &Theme) {
        todo!()
    }
}

impl PropertySection for Gate {
    fn title(&self) -> &str {
        "Gate"
    }

    fn content_height(&self, _theme: &Theme) -> f32 {
        todo!()
    }

    fn tick(
        &mut self,
        _rl: &RaylibHandle,
        _thread: &RaylibThread,
        _container: Bounds,
        _theme: &Theme,
    ) {
        todo!()
    }
}

impl<D: RaylibDraw> DrawPropertySection<D> for Gate {
    fn draw(&self, _d: &mut D, _container: Bounds, _theme: &Theme) {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct PropertiesPanel {
    pub panel: Panel,
}

impl PropertiesPanel {
    pub const fn new(panel: Panel) -> Self {
        Self { panel }
    }

    pub fn tick<'a, I>(
        &mut self,
        rl: &'a mut RaylibHandle,
        thread: &'a RaylibThread,
        theme: &Theme,
        input: &Inputs,
        sections: I,
    ) where
        I: IntoIterator<Item = &'a mut dyn PropertySection>,
    {
        // self.panel.tick_resize(rl, theme, input);
        let bounds = self.panel.content_bounds(theme);
        let mut y = bounds.min.y;
        for section in sections {
            y += theme.console_font.line_height() * section.title().lines().count() as f32;
            let height = section.content_height(theme);
            section.tick(
                rl,
                thread,
                Bounds::new(
                    Vector2::new(bounds.max.x, y),
                    Vector2::new(bounds.min.x, y + height),
                ),
                theme,
            );
        }
    }

    pub fn draw<'a, D, I>(&self, d: &'a mut D, theme: &Theme, sections: I)
    where
        D: RaylibDraw,
        I: IntoIterator<Item = &'a dyn DrawPropertySection<D>>,
    {
        self.panel.draw(d, theme, |d, bounds, theme| {
            let Vector2 { x, mut y } = bounds.min;
            for section in sections {
                theme.general_font.draw_text(
                    d,
                    section.title(),
                    Vector2::new(x, y),
                    theme.foreground,
                );
                y += theme.console_font.line_height() * section.title().lines().count() as f32;
                let height = section.content_height(theme);
                section.draw(
                    d,
                    Bounds::new(
                        Vector2::new(bounds.max.x, y),
                        Vector2::new(bounds.min.x, y + height),
                    ),
                    theme,
                );
            }
        });
    }
}
