use crate::{
    icon_sheets::ButtonIconSheetId,
    toolpane::{ToolPaneAnchoring, Visibility},
};
use raylib::prelude::*;
use serde_derive::{Deserialize, Serialize};

mod color {
    use raylib::color::Color;

    struct ColorVisitor;

    struct HexCode;

    impl serde::de::Expected for HexCode {
        fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("3, 4, or 8 digits 0-F")
        }
    }

    impl<'de> serde::de::Visitor<'de> for ColorVisitor {
        type Value = Color;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str(
                "a color hexcode starting with '#' or a \"rgb(...)\" containing the rgb values",
            )
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if let Some(v) = v.strip_prefix('#') {
                Ok(match v.len() {
                    6 => {
                        let [_, r, g, b] = u32::from_str_radix(v, 16)
                            .map_err(|_| E::custom("invalid number"))?
                            .to_be_bytes();
                        Color::new(r, g, b, 255)
                    }
                    8 => {
                        let [r, g, b, a] = u32::from_str_radix(v, 16)
                            .map_err(|_| E::custom("invalid number"))?
                            .to_be_bytes();
                        Color::new(r, g, b, a)
                    }
                    len => Err(E::invalid_length(len, &HexCode))?,
                })
            } else if let Some(v) = v.strip_prefix("rgb(").and_then(|v| v.strip_suffix(')')) {
                let mut it = v.split(',');
                let r = it
                    .next()
                    .ok_or(E::custom("missing"))
                    .and_then(|x| x.parse().map_err(E::custom))?;
                let g = it
                    .next()
                    .ok_or(E::custom("missing"))
                    .and_then(|x| x.parse().map_err(E::custom))?;
                let b = it
                    .next()
                    .ok_or(E::custom("missing"))
                    .and_then(|x| x.parse().map_err(E::custom))?;
                Ok(Color::new(r, g, b, 255))
            } else if let Some(v) = v.strip_prefix("rgba(").and_then(|v| v.strip_suffix(')')) {
                let mut it = v.split(',');
                let r = it
                    .next()
                    .ok_or(E::custom("missing"))
                    .and_then(|x| x.parse().map_err(E::custom))?;
                let g = it
                    .next()
                    .ok_or(E::custom("missing"))
                    .and_then(|x| x.parse().map_err(E::custom))?;
                let b = it
                    .next()
                    .ok_or(E::custom("missing"))
                    .and_then(|x| x.parse().map_err(E::custom))?;
                let a = it
                    .next()
                    .ok_or(E::custom("missing"))
                    .and_then(|x| x.parse::<f32>().map_err(E::custom))?;
                Ok(Color::new(r, g, b, (a * 255.0).clamp(0.0, 255.0) as u8))
            } else {
                Err(E::custom("unknown color format"))
            }
        }
    }

    pub fn serialize<S>(value: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("#{:02X}{:02X}{:02X}", value.r, value.g, value.b))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(ColorVisitor)
    }
}

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
    resistance0: Color::BLACK,
    resistance1: Color::BROWN,
    resistance2: Color::RED,
    resistance3: Color::ORANGE,
    resistance4: Color::YELLOW,
    resistance5: Color::GREEN,
    resistance6: Color::BLUE,
    resistance7: Color::PURPLE,
    resistance8: Color::GRAY,
    resistance9: Color::WHITE,
    console_font_size: 10,
    console_char_spacing: 1,
    console_line_spacing: 2,
    console_padding_left: 15,
    console_padding_top: 5,
    console_padding_right: 5,
    console_padding_bottom: 5,
    title_padding_x: 6,
    title_padding_y: 3,
    button_icon_scale: ButtonIconSheetId::X16,
    toolpane_anchoring: ToolPaneAnchoring::LeftTop,
    toolpane_visibility: Visibility::Expanded,
    toolpane_padding_across: 3,
    toolpane_padding_along: 5,
    toolpane_group_expanded_gap: 16,
    toolpane_group_collapsed_gap: 16,
    toolpane_button_gap: 1,
};

static LIGHT_THEME: Theme = Theme {
    background: Color::WHITE,
    background1: Color::new(226, 227, 227, 255),
    background2: Color::new(188, 188, 188, 255),
    background3: Color::GRAY,
    foreground3: DEAD_CABLE,
    foreground2: Color::new(100, 100, 100, 255),
    foreground1: Color::new(75, 75, 75, 255),
    foreground: Color::new(40, 40, 40, 255),
    input: INPUT_LAVENDER,
    output: OUTPUT_APRICOT,
    available: Color::new(26, 115, 232, 255),
    interact: Color::new(231, 240, 253, 255),
    active: Color::BLUE,
    error: Color::MAGENTA,
    destructive: DESTRUCTIVE_RED,
    special: Color::new(135, 60, 190, 255),
    hyperref: Color::BLUE,
    dead_link: Color::BISQUE,
    caution: CAUTION_YELLOW,
    blueprints_background: Color::new(250, 250, 255, 255),
    resistance0: Color::BLACK,
    resistance1: Color::BROWN,
    resistance2: Color::RED,
    resistance3: Color::ORANGE,
    resistance4: Color::YELLOW,
    resistance5: Color::GREEN,
    resistance6: Color::BLUE,
    resistance7: Color::PURPLE,
    resistance8: Color::GRAY,
    resistance9: Color::WHITE,
    console_font_size: 10,
    console_char_spacing: 1,
    console_line_spacing: 2,
    console_padding_left: 15,
    console_padding_top: 5,
    console_padding_right: 5,
    console_padding_bottom: 5,
    title_padding_x: 6,
    title_padding_y: 3,
    button_icon_scale: ButtonIconSheetId::X16,
    toolpane_anchoring: ToolPaneAnchoring::LeftTop,
    toolpane_visibility: Visibility::Expanded,
    toolpane_padding_across: 3,
    toolpane_padding_along: 5,
    toolpane_group_expanded_gap: 16,
    toolpane_group_collapsed_gap: 16,
    toolpane_button_gap: 1,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Theme {
    #[serde(with = "color")]
    pub background: Color,
    #[serde(with = "color")]
    pub background1: Color,
    #[serde(with = "color")]
    pub background2: Color,
    #[serde(with = "color")]
    pub background3: Color,
    #[serde(with = "color")]
    pub foreground3: Color,
    #[serde(with = "color")]
    pub foreground2: Color,
    #[serde(with = "color")]
    pub foreground1: Color,
    #[serde(with = "color")]
    pub foreground: Color,
    #[serde(with = "color")]
    pub input: Color,
    #[serde(with = "color")]
    pub output: Color,
    #[serde(with = "color")]
    pub available: Color,
    #[serde(with = "color")]
    pub interact: Color,
    #[serde(with = "color")]
    pub active: Color,
    #[serde(with = "color")]
    pub error: Color,
    #[serde(with = "color")]
    pub destructive: Color,
    #[serde(with = "color")]
    pub special: Color,
    #[serde(with = "color")]
    pub hyperref: Color,
    #[serde(with = "color")]
    pub dead_link: Color,
    #[serde(with = "color")]
    pub caution: Color,
    #[serde(with = "color")]
    pub blueprints_background: Color,
    #[serde(with = "color")]
    pub resistance0: Color,
    #[serde(with = "color")]
    pub resistance1: Color,
    #[serde(with = "color")]
    pub resistance2: Color,
    #[serde(with = "color")]
    pub resistance3: Color,
    #[serde(with = "color")]
    pub resistance4: Color,
    #[serde(with = "color")]
    pub resistance5: Color,
    #[serde(with = "color")]
    pub resistance6: Color,
    #[serde(with = "color")]
    pub resistance7: Color,
    #[serde(with = "color")]
    pub resistance8: Color,
    #[serde(with = "color")]
    pub resistance9: Color,
    pub console_font_size: i32,
    pub console_char_spacing: i32,
    pub console_line_spacing: i32,
    pub console_padding_left: i32,
    pub console_padding_top: i32,
    pub console_padding_right: i32,
    pub console_padding_bottom: i32,
    pub title_padding_x: i32,
    pub title_padding_y: i32,
    pub button_icon_scale: ButtonIconSheetId,
    pub toolpane_anchoring: ToolPaneAnchoring,
    pub toolpane_visibility: Visibility,
    pub toolpane_padding_across: i32,
    pub toolpane_padding_along: i32,
    pub toolpane_group_expanded_gap: i32,
    pub toolpane_group_collapsed_gap: i32,
    pub toolpane_button_gap: i32,
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

impl Theme {
    pub const fn console_line_height(&self) -> i32 {
        self.console_font_size + self.console_line_spacing
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
            ColorId::Resistance0 => &self.resistance0,
            ColorId::Resistance1 => &self.resistance1,
            ColorId::Resistance2 => &self.resistance2,
            ColorId::Resistance3 => &self.resistance3,
            ColorId::Resistance4 => &self.resistance4,
            ColorId::Resistance5 => &self.resistance5,
            ColorId::Resistance6 => &self.resistance6,
            ColorId::Resistance7 => &self.resistance7,
            ColorId::Resistance8 => &self.resistance8,
            ColorId::Resistance9 => &self.resistance9,
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
            ColorId::Resistance0 => &mut self.resistance0,
            ColorId::Resistance1 => &mut self.resistance1,
            ColorId::Resistance2 => &mut self.resistance2,
            ColorId::Resistance3 => &mut self.resistance3,
            ColorId::Resistance4 => &mut self.resistance4,
            ColorId::Resistance5 => &mut self.resistance5,
            ColorId::Resistance6 => &mut self.resistance6,
            ColorId::Resistance7 => &mut self.resistance7,
            ColorId::Resistance8 => &mut self.resistance8,
            ColorId::Resistance9 => &mut self.resistance9,
        }
    }
}
