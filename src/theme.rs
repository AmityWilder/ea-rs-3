use std::fs::read_to_string;

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

// static LIGHT_THEME: Theme = Theme {
//     background: todo!(),
//     background1: todo!(),
//     background2: todo!(),
//     background3: todo!(),
//     foreground3: todo!(),
//     foreground2: todo!(),
//     foreground1: todo!(),
//     foreground: todo!(),
//     input: todo!(),
//     output: todo!(),
//     available: todo!(),
//     interact: todo!(),
//     active: todo!(),
//     error: todo!(),
//     destructive: todo!(),
//     special: todo!(),
//     hyperref: todo!(),
//     dead_link: todo!(),
//     caution: todo!(),
//     blueprints_background: todo!(),
//     resistance: [
//         Color::BLACK,
//         Color::BROWN,
//         Color::RED,
//         Color::ORANGE,
//         Color::YELLOW,
//         Color::GREEN,
//         Color::BLUE,
//         Color::PURPLE,
//         Color::GRAY,
//         Color::WHITE,
//     ],
//     console_font_size: todo!(),
//     console_char_spacing: todo!(),
//     console_line_spacing: todo!(),
//     console_padding_left: todo!(),
//     console_padding_top: todo!(),
//     console_padding_right: todo!(),
//     console_padding_bottom: todo!(),
//     title_padding_x: todo!(),
//     title_padding_y: todo!(),
//     toolpane_padding_across: todo!(),
//     toolpane_padding_along: todo!(),
// };

static DARK_THEME: Theme = Theme {
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
    hyperref: GLEEFUL_DUST,
    dead_link: HAUNTING_WHITE,
    caution: CAUTION_YELLOW,
    blueprints_background: Color::new(10, 15, 30, 255),
    resistance: [
        Color::BLACK,
        Color::BROWN,
        Color::RED,
        Color::ORANGE,
        Color::YELLOW,
        Color::GREEN,
        Color::BLUE,
        Color::PURPLE,
        Color::GRAY,
        Color::WHITE,
    ],
    console_font_size: 10,
    console_char_spacing: 1,
    console_line_spacing: 2,
    console_padding_left: 15,
    console_padding_top: 5,
    console_padding_right: 5,
    console_padding_bottom: 5,
    title_padding_x: 6,
    title_padding_y: 3,
    toolpane_padding_across: 3,
    toolpane_padding_along: 5,
};

// static AMOLED_THEME: Theme = Theme {
//     background: Color::BLACK,
//     background1: Color::BLACK,
//     background2: Color::BLACK,
//     background3: Color::BLACK,
//     foreground3: Color::LIGHTGRAY,
//     foreground2: Color::LIGHTGRAY,
//     foreground1: Color::LIGHTGRAY,
//     foreground: Color::LIGHTGRAY,
//     input: todo!(),
//     output: todo!(),
//     available: todo!(),
//     interact: todo!(),
//     active: todo!(),
//     error: todo!(),
//     destructive: todo!(),
//     special: todo!(),
//     hyperref: todo!(),
//     dead_link: todo!(),
//     caution: todo!(),
//     blueprints_background: todo!(),
//     resistance: [
//         Color::BLACK,
//         Color::BROWN,
//         Color::RED,
//         Color::ORANGE,
//         Color::YELLOW,
//         Color::GREEN,
//         Color::BLUE,
//         Color::PURPLE,
//         Color::GRAY,
//         Color::WHITE,
//     ],
//     console_font_size: todo!(),
//     console_char_spacing: todo!(),
//     console_line_spacing: todo!(),
//     console_padding_left: todo!(),
//     console_padding_top: todo!(),
//     console_padding_right: todo!(),
//     console_padding_bottom: todo!(),
//     title_padding_x: todo!(),
//     title_padding_y: todo!(),
//     toolpane_padding_across: todo!(),
//     toolpane_padding_along: todo!(),
// };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub hyperref: Color,
    pub dead_link: Color,
    pub caution: Color,
    pub blueprints_background: Color,
    pub resistance: [Color; 10],
    pub console_font_size: i32,
    pub console_char_spacing: i32,
    pub console_line_spacing: i32,
    pub console_padding_left: i32,
    pub console_padding_top: i32,
    pub console_padding_right: i32,
    pub console_padding_bottom: i32,
    pub title_padding_x: i32,
    pub title_padding_y: i32,
    pub toolpane_padding_across: i32,
    pub toolpane_padding_along: i32,
}

impl Default for Theme {
    fn default() -> Self {
        DARK_THEME
    }
}

fn parse_color(s: &str) -> Result<Color, ()> {
    if let Some(s) = s.strip_prefix('#') {
        Color::from_hex(s).map_err(|_| ())
    } else if let Some(s) = s.strip_prefix("rgba(").and_then(|s| s.strip_suffix(")")) {
        let mut it = s.splitn(4, ",").map(|item| {
            item.trim_start().parse::<u8>().ok().or_else(|| {
                item.parse::<f32>()
                    .ok()
                    .map(|x| (x.clamp(0.0, 1.0) * 255.0) as u8)
            })
        });
        Ok(Color {
            r: it.next().and_then(|x| x).ok_or(())?,
            g: it.next().and_then(|x| x).ok_or(())?,
            b: it.next().and_then(|x| x).ok_or(())?,
            a: it.next().and_then(|x| x).ok_or(())?,
        })
    } else {
        Err(())
    }
}

impl std::str::FromStr for Theme {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut theme = Self::default();
        theme.load_base(s).map(|()| theme)
    }
}

impl Theme {
    pub const fn console_line_height(&self) -> i32 {
        self.console_font_size + self.console_line_spacing
    }

    fn load_base(&mut self, s: &str) -> Result<(), ()> {
        for mut line in s.lines() {
            if let Some(n) = line.find("//") {
                line = &line[..n];
            }
            if let Some((mut key, mut val)) = line.split_once("=") {
                key = key.trim();
                val = val.trim();
                match key {
                    "base" => match val {
                        "dark" => self.clone_from(&DARK_THEME),
                        _ => self.load_base(read_to_string(val).map_err(|_| ())?.as_str())?,
                    },
                    "background" => self.background = parse_color(val)?,
                    "background1" => self.background1 = parse_color(val)?,
                    "background2" => self.background2 = parse_color(val)?,
                    "background3" => self.background3 = parse_color(val)?,
                    "foreground3" => self.foreground3 = parse_color(val)?,
                    "foreground2" => self.foreground2 = parse_color(val)?,
                    "foreground1" => self.foreground1 = parse_color(val)?,
                    "foreground" => self.foreground = parse_color(val)?,
                    "input" => self.input = parse_color(val)?,
                    "output" => self.output = parse_color(val)?,
                    "available" => self.available = parse_color(val)?,
                    "interact" => self.interact = parse_color(val)?,
                    "active" => self.active = parse_color(val)?,
                    "error" => self.error = parse_color(val)?,
                    "destructive" => self.destructive = parse_color(val)?,
                    "special" => self.special = parse_color(val)?,
                    "hyperref" => self.hyperref = parse_color(val)?,
                    "dead_link" => self.dead_link = parse_color(val)?,
                    "caution" => self.caution = parse_color(val)?,
                    "blueprints_background" => self.blueprints_background = parse_color(val)?,
                    "resistance0" => self.resistance[0] = parse_color(val)?,
                    "resistance1" => self.resistance[1] = parse_color(val)?,
                    "resistance2" => self.resistance[2] = parse_color(val)?,
                    "resistance3" => self.resistance[3] = parse_color(val)?,
                    "resistance4" => self.resistance[4] = parse_color(val)?,
                    "resistance5" => self.resistance[5] = parse_color(val)?,
                    "resistance6" => self.resistance[6] = parse_color(val)?,
                    "resistance7" => self.resistance[7] = parse_color(val)?,
                    "resistance8" => self.resistance[8] = parse_color(val)?,
                    "resistance9" => self.resistance[9] = parse_color(val)?,
                    "console_font_size" => self.console_font_size = val.parse().map_err(|_| ())?,
                    "console_char_spacing" => {
                        self.console_char_spacing = val.parse().map_err(|_| ())?
                    }
                    "console_line_spacing" => {
                        self.console_line_spacing = val.parse().map_err(|_| ())?
                    }
                    "console_padding_left" => {
                        self.console_padding_left = val.parse().map_err(|_| ())?
                    }
                    "console_padding_top" => {
                        self.console_padding_top = val.parse().map_err(|_| ())?
                    }
                    "console_padding_right" => {
                        self.console_padding_right = val.parse().map_err(|_| ())?
                    }
                    "console_padding_bottom" => {
                        self.console_padding_bottom = val.parse().map_err(|_| ())?
                    }
                    "title_padding_x" => self.title_padding_x = val.parse().map_err(|_| ())?,
                    "title_padding_y" => self.title_padding_y = val.parse().map_err(|_| ())?,
                    "toolpane_padding_across" => {
                        self.toolpane_padding_across = val.parse().map_err(|_| ())?
                    }
                    "toolpane_padding_along" => {
                        self.toolpane_padding_along = val.parse().map_err(|_| ())?
                    }
                    _ => {}
                }
            }
        }
        Ok(())
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
    HyperRef,
    DeadLink,
    Caution,
    BlueprintsBackground,
    Resistance0,
    Resistance1,
    Resistance2,
    Resistance3,
    Resistance4,
    Resistance5,
    Resistance6,
    Resistance7,
    Resistance8,
    Resistance9,
}

impl std::fmt::Display for ColorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorId::Background => "background",
            ColorId::Background1 => "background1",
            ColorId::Background2 => "background2",
            ColorId::Background3 => "background3",
            ColorId::Foreground3 => "foreground3",
            ColorId::Foreground2 => "foreground2",
            ColorId::Foreground1 => "foreground1",
            ColorId::Foreground => "foreground",
            ColorId::Input => "input",
            ColorId::Output => "output",
            ColorId::Available => "available",
            ColorId::Interact => "interact",
            ColorId::Active => "active",
            ColorId::Error => "error",
            ColorId::Destructive => "destructive",
            ColorId::Special => "special",
            ColorId::HyperRef => "hyper_ref",
            ColorId::DeadLink => "dead_link",
            ColorId::Caution => "caution",
            ColorId::BlueprintsBackground => "blueprints_background",
            ColorId::Resistance0 => "resistance0",
            ColorId::Resistance1 => "resistance1",
            ColorId::Resistance2 => "resistance2",
            ColorId::Resistance3 => "resistance3",
            ColorId::Resistance4 => "resistance4",
            ColorId::Resistance5 => "resistance5",
            ColorId::Resistance6 => "resistance6",
            ColorId::Resistance7 => "resistance7",
            ColorId::Resistance8 => "resistance8",
            ColorId::Resistance9 => "resistance9",
        }
        .fmt(f)
    }
}

impl std::str::FromStr for ColorId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "background" => Ok(ColorId::Background),
            "background1" => Ok(ColorId::Background1),
            "background2" => Ok(ColorId::Background2),
            "background3" => Ok(ColorId::Background3),
            "foreground3" => Ok(ColorId::Foreground3),
            "foreground2" => Ok(ColorId::Foreground2),
            "foreground1" => Ok(ColorId::Foreground1),
            "foreground" => Ok(ColorId::Foreground),
            "input" => Ok(ColorId::Input),
            "output" => Ok(ColorId::Output),
            "available" => Ok(ColorId::Available),
            "interact" => Ok(ColorId::Interact),
            "active" => Ok(ColorId::Active),
            "error" => Ok(ColorId::Error),
            "destructive" => Ok(ColorId::Destructive),
            "special" => Ok(ColorId::Special),
            "hyper_ref" => Ok(ColorId::HyperRef),
            "dead_link" => Ok(ColorId::DeadLink),
            "caution" => Ok(ColorId::Caution),
            "blueprints_background" => Ok(ColorId::BlueprintsBackground),
            "resistance0" => Ok(ColorId::Resistance0),
            "resistance1" => Ok(ColorId::Resistance1),
            "resistance2" => Ok(ColorId::Resistance2),
            "resistance3" => Ok(ColorId::Resistance3),
            "resistance4" => Ok(ColorId::Resistance4),
            "resistance5" => Ok(ColorId::Resistance5),
            "resistance6" => Ok(ColorId::Resistance6),
            "resistance7" => Ok(ColorId::Resistance7),
            "resistance8" => Ok(ColorId::Resistance8),
            "resistance9" => Ok(ColorId::Resistance9),
            _ => Err(()),
        }
    }
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
            ColorId::HyperRef => &self.hyperref,
            ColorId::DeadLink => &self.dead_link,
            ColorId::Caution => &self.caution,
            ColorId::BlueprintsBackground => &self.blueprints_background,
            ColorId::Resistance0 => &self.resistance[0],
            ColorId::Resistance1 => &self.resistance[1],
            ColorId::Resistance2 => &self.resistance[2],
            ColorId::Resistance3 => &self.resistance[3],
            ColorId::Resistance4 => &self.resistance[4],
            ColorId::Resistance5 => &self.resistance[5],
            ColorId::Resistance6 => &self.resistance[6],
            ColorId::Resistance7 => &self.resistance[7],
            ColorId::Resistance8 => &self.resistance[8],
            ColorId::Resistance9 => &self.resistance[9],
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
            ColorId::HyperRef => &mut self.hyperref,
            ColorId::DeadLink => &mut self.dead_link,
            ColorId::Caution => &mut self.caution,
            ColorId::BlueprintsBackground => &mut self.blueprints_background,
            ColorId::Resistance0 => &mut self.resistance[0],
            ColorId::Resistance1 => &mut self.resistance[1],
            ColorId::Resistance2 => &mut self.resistance[2],
            ColorId::Resistance3 => &mut self.resistance[3],
            ColorId::Resistance4 => &mut self.resistance[4],
            ColorId::Resistance5 => &mut self.resistance[5],
            ColorId::Resistance6 => &mut self.resistance[6],
            ColorId::Resistance7 => &mut self.resistance[7],
            ColorId::Resistance8 => &mut self.resistance[8],
            ColorId::Resistance9 => &mut self.resistance[9],
        }
    }
}
