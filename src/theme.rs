use raylib::prelude::*;

pub const SPACE_GRAY: Color = Color::new(28, 26, 41, 255);
pub const LIFELESS_NEBULA: Color = Color::new(75, 78, 94, 255);
pub const HAUNTING_WHITE: Color = Color::new(148, 150, 166, 255);
pub const GLEEFUL_DUST: Color = Color::new(116, 125, 237, 255);
pub const INTERFERENCE_GRAY: Color = Color::new(232, 234, 255, 255);
pub const REDSTONE: Color = Color::new(212, 25, 25, 255);
pub const DESTRUCTIVE_RED: Color = Color::new(219, 18, 18, 255);
pub const DEAD_CABLE: Color = Color::new(122, 118, 118, 255);
pub const INPUT_LAVENDER: Color = Color::new(128, 106, 217, 255);
pub const OUTPUT_APRICOT: Color = Color::new(207, 107, 35, 255);
pub const WIP_BLUE: Color = Color::new(26, 68, 161, 255);
pub const CAUTION_YELLOW: Color = Color::new(250, 222, 37, 255);

#[derive(Debug, Clone)]
pub struct Theme {
    pub background: Color,
    pub background1: Color,
    pub background2: Color,
    pub background3: Color,
    pub foreground3: Color,
    pub foreground2: Color,
    pub foreground1: Color,
    pub foreground: Color,
    pub input: Color,
    pub output: Color,
    pub available: Color,
    pub interact: Color,
    pub active: Color,
    pub error: Color,
    pub destructive: Color,
    pub special: Color,
    pub caution: Color,
    pub blueprints_background: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: Color::BLACK,
            background1: SPACE_GRAY,
            background2: LIFELESS_NEBULA,
            background3: GLEEFUL_DUST,
            foreground3: DEAD_CABLE,
            foreground2: HAUNTING_WHITE,
            foreground1: INTERFERENCE_GRAY,
            foreground: Color::WHITE,
            input: INPUT_LAVENDER,
            output: OUTPUT_APRICOT,
            available: WIP_BLUE,
            interact: Color::YELLOW,
            active: REDSTONE,
            error: Color::MAGENTA,
            destructive: DESTRUCTIVE_RED,
            special: Color::VIOLET,
            caution: CAUTION_YELLOW,
            blueprints_background: Color::new(10, 15, 30, 255),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorId {
    Background,
    Background1,
    Background2,
    Background3,
    Foreground3,
    Foreground2,
    Foreground1,
    Foreground,
    Input,
    Output,
    Available,
    Interact,
    Active,
    Error,
    Destructive,
    Special,
    Caution,
    BlueprintsBackground,
}

impl std::ops::Index<ColorId> for Theme {
    type Output = Color;

    fn index(&self, index: ColorId) -> &Self::Output {
        match index {
            ColorId::Background => &self.background,
            ColorId::Background1 => &self.background1,
            ColorId::Background2 => &self.background2,
            ColorId::Background3 => &self.background3,
            ColorId::Foreground3 => &self.foreground3,
            ColorId::Foreground2 => &self.foreground2,
            ColorId::Foreground1 => &self.foreground1,
            ColorId::Foreground => &self.foreground,
            ColorId::Input => &self.input,
            ColorId::Output => &self.output,
            ColorId::Available => &self.available,
            ColorId::Interact => &self.interact,
            ColorId::Active => &self.active,
            ColorId::Error => &self.error,
            ColorId::Destructive => &self.destructive,
            ColorId::Special => &self.special,
            ColorId::Caution => &self.caution,
            ColorId::BlueprintsBackground => &self.blueprints_background,
        }
    }
}

impl std::ops::IndexMut<ColorId> for Theme {
    fn index_mut(&mut self, index: ColorId) -> &mut Self::Output {
        match index {
            ColorId::Background => &mut self.background,
            ColorId::Background1 => &mut self.background1,
            ColorId::Background2 => &mut self.background2,
            ColorId::Background3 => &mut self.background3,
            ColorId::Foreground3 => &mut self.foreground3,
            ColorId::Foreground2 => &mut self.foreground2,
            ColorId::Foreground1 => &mut self.foreground1,
            ColorId::Foreground => &mut self.foreground,
            ColorId::Input => &mut self.input,
            ColorId::Output => &mut self.output,
            ColorId::Available => &mut self.available,
            ColorId::Interact => &mut self.interact,
            ColorId::Active => &mut self.active,
            ColorId::Error => &mut self.error,
            ColorId::Destructive => &mut self.destructive,
            ColorId::Special => &mut self.special,
            ColorId::Caution => &mut self.caution,
            ColorId::BlueprintsBackground => &mut self.blueprints_background,
        }
    }
}
