use crate::{
    icon_sheets::{ButtonIconSheetId, ButtonIconSheets, NodeIconSheetSet, NodeIconSheetSets},
    ui::{Orientation, Padding, Visibility},
};
use raylib::prelude::*;
use serde::{Deserialize, Serialize, de::Visitor};
use serde_derive::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::LazyLock,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SerdeColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl SerdeColor {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl From<Color> for SerdeColor {
    fn from(Color { r, g, b, a }: Color) -> Self {
        Self { r, g, b, a }
    }
}

impl From<SerdeColor> for Color {
    fn from(SerdeColor { r, g, b, a }: SerdeColor) -> Self {
        Self { r, g, b, a }
    }
}

macro_rules! named_colors {
    ($($color:ident),* $(,)?) => {
        static COLOR_NAME: LazyLock<HashMap<SerdeColor, String>> =
            LazyLock::new(|| HashMap::from_iter([$((Color::$color.into(), stringify!($color).to_lowercase())),*]));

        static NAME_COLOR: LazyLock<HashMap<String, SerdeColor>> = LazyLock::new(|| {
            HashMap::from_iter([$((stringify!($color).to_lowercase(), Color::$color.into())),*])
        });
    };
}

named_colors![
    INDIANRED,
    LIGHTCORAL,
    SALMON,
    DARKSALMON,
    LIGHTSALMON,
    CRIMSON,
    RED,
    FIREBRICK,
    DARKRED,
    PINK,
    LIGHTPINK,
    HOTPINK,
    DEEPPINK,
    MEDIUMVIOLETRED,
    PALEVIOLETRED,
    CORAL,
    TOMATO,
    ORANGERED,
    DARKORANGE,
    ORANGE,
    GOLD,
    YELLOW,
    LIGHTYELLOW,
    LEMONCHIFFON,
    LIGHTGOLDENRODYELLOW,
    PAPAYAWHIP,
    MOCCASIN,
    PEACHPUFF,
    PALEGOLDENROD,
    KHAKI,
    DARKKHAKI,
    LAVENDER,
    THISTLE,
    PLUM,
    VIOLET,
    ORCHID,
    FUCHSIA,
    MAGENTA,
    MEDIUMORCHID,
    MEDIUMPURPLE,
    REBECCAPURPLE,
    BLUEVIOLET,
    DARKVIOLET,
    DARKORCHID,
    DARKMAGENTA,
    PURPLE,
    DARKPURPLE,
    INDIGO,
    SLATEBLUE,
    DARKSLATEBLUE,
    MEDIUMSLATEBLUE,
    GREENYELLOW,
    CHARTREUSE,
    LAWNGREEN,
    LIME,
    LIMEGREEN,
    PALEGREEN,
    LIGHTGREEN,
    MEDIUMSPRINGGREEN,
    SPRINGGREEN,
    MEDIUMSEAGREEN,
    SEAGREEN,
    FORESTGREEN,
    GREEN,
    DARKGREEN,
    YELLOWGREEN,
    OLIVEDRAB,
    OLIVE,
    DARKOLIVEGREEN,
    MEDIUMAQUAMARINE,
    DARKSEAGREEN,
    LIGHTSEAGREEN,
    DARKCYAN,
    TEAL,
    AQUA,
    CYAN,
    LIGHTCYAN,
    PALETURQUOISE,
    AQUAMARINE,
    TURQUOISE,
    MEDIUMTURQUOISE,
    DARKTURQUOISE,
    CADETBLUE,
    STEELBLUE,
    LIGHTSTEELBLUE,
    POWDERBLUE,
    LIGHTBLUE,
    SKYBLUE,
    LIGHTSKYBLUE,
    DEEPSKYBLUE,
    DODGERBLUE,
    CORNFLOWERBLUE,
    ROYALBLUE,
    BLUE,
    MEDIUMBLUE,
    DARKBLUE,
    NAVY,
    MIDNIGHTBLUE,
    CORNSILK,
    BLANCHEDALMOND,
    BISQUE,
    NAVAJOWHITE,
    WHEAT,
    BURLYWOOD,
    TAN,
    ROSYBROWN,
    SANDYBROWN,
    GOLDENROD,
    DARKGOLDENROD,
    PERU,
    CHOCOLATE,
    SADDLEBROWN,
    SIENNA,
    BROWN,
    DARKBROWN,
    MAROON,
    WHITE,
    SNOW,
    HONEYDEW,
    MINTCREAM,
    AZURE,
    ALICEBLUE,
    GHOSTWHITE,
    WHITESMOKE,
    SEASHELL,
    BEIGE,
    OLDLACE,
    FLORALWHITE,
    IVORY,
    ANTIQUEWHITE,
    LINEN,
    LAVENDERBLUSH,
    MISTYROSE,
    GAINSBORO,
    LIGHTGRAY,
    SILVER,
    DARKGRAY,
    GRAY,
    DIMGRAY,
    LIGHTSLATEGRAY,
    SLATEGRAY,
    DARKSLATEGRAY,
    BLACK,
    BLANK,
    RAYWHITE,
    SPACEGRAY,
    LIFELESSNEBULA,
    HAUNTINGWHITE,
    GLEEFULDUST,
    INTERFERENCEGRAY,
    REDSTONE,
    DESTRUCTIVERED,
    DEADCABLE,
    INPUTLAVENDER,
    OUTPUTAPRICOT,
    WIPBLUE,
    CAUTIONYELLOW,
];

struct ColorVisitor;

struct HexCode;

impl serde::de::Expected for HexCode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("6, or 8 digits of 0-F")
    }
}

impl<'de> serde::de::Visitor<'de> for ColorVisitor {
    type Value = SerdeColor;

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
                    SerdeColor::new(r, g, b, 255)
                }
                8 => {
                    let [r, g, b, a] = u32::from_str_radix(v, 16)
                        .map_err(|_| E::custom("invalid number"))?
                        .to_be_bytes();
                    SerdeColor::new(r, g, b, a)
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
            Ok(SerdeColor::new(r, g, b, 255))
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
            Ok(SerdeColor::new(
                r,
                g,
                b,
                (a * 255.0).clamp(0.0, 255.0) as u8,
            ))
        } else if let Some(color) = NAME_COLOR.get(v) {
            Ok(*color)
        } else {
            Err(E::custom("unknown color format"))
        }
    }
}

impl Serialize for SerdeColor {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(name) = COLOR_NAME.get(self) {
            serializer.serialize_str(name)
        } else {
            serializer.serialize_str(&if self.a != u8::MAX {
                format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
            } else {
                format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
            })
        }
    }
}

impl<'de> Deserialize<'de> for SerdeColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(ColorVisitor)
    }
}

pub trait CustomColors {
    const SPACEGRAY: Color = Color::new(28, 26, 41, 255);
    const LIFELESSNEBULA: Color = Color::new(75, 78, 94, 255);
    const HAUNTINGWHITE: Color = Color::new(148, 150, 166, 255);
    const GLEEFULDUST: Color = Color::new(116, 125, 237, 255);
    const INTERFERENCEGRAY: Color = Color::new(232, 234, 255, 255);
    const REDSTONE: Color = Color::new(212, 25, 25, 255);
    const DESTRUCTIVERED: Color = Color::new(219, 18, 18, 255);
    const DEADCABLE: Color = Color::new(122, 118, 118, 255);
    const INPUTLAVENDER: Color = Color::new(128, 106, 217, 255);
    const OUTPUTAPRICOT: Color = Color::new(207, 107, 35, 255);
    const WIPBLUE: Color = Color::new(26, 68, 161, 255);
    const CAUTIONYELLOW: Color = Color::new(250, 222, 37, 255);
}

impl CustomColors for Color {}

#[derive(Debug, Serialize)]
pub struct ThemeFont {
    pub path: Option<PathBuf>,
    pub font_size: f32,
    pub char_spacing: f32,
    pub line_spacing: f32,
    #[serde(skip)]
    pub font: OptionalFont,
}

impl Default for ThemeFont {
    fn default() -> Self {
        Self {
            path: None,
            font_size: 10.0,
            char_spacing: 1.0,
            line_spacing: 2.0,
            font: OptionalFont::Unloaded,
        }
    }
}

struct ThemeFontVisitor;

impl<'de> Visitor<'de> for ThemeFontVisitor {
    type Value = ThemeFont;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("ThemeFont")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        enum FieldIdent {
            Path,
            FontSize,
            CharSpacing,
            LineSpacing,
            #[serde(other)]
            Unknown,
        }

        let mut path = None;
        let mut font_size = None;
        let mut char_spacing = None;
        let mut line_spacing = None;
        while let Some(key) = map.next_key()? {
            match key {
                FieldIdent::Path => path = Some(map.next_value()?),
                FieldIdent::FontSize => font_size = Some(map.next_value()?),
                FieldIdent::CharSpacing => char_spacing = Some(map.next_value()?),
                FieldIdent::LineSpacing => line_spacing = Some(map.next_value()?),
                FieldIdent::Unknown => {}
            }
        }

        let font_size = font_size.unwrap_or(10.0);
        let char_spacing = char_spacing.unwrap_or(font_size * 0.1);
        let line_spacing = line_spacing.unwrap_or(font_size * 0.2);

        Ok(ThemeFont {
            path,
            font_size,
            char_spacing,
            line_spacing,
            font: OptionalFont::Unloaded,
        })
    }
}

impl<'de> Deserialize<'de> for ThemeFont {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(ThemeFontVisitor)
    }
}

/// NOTE: [`ThemeFont::clone`] assigns [`OptionalFont::Unloaded`] to the [`ThemeFont::font`] field,
/// because Raylib weak/strong fonts are not reference counted and may be used after free.
///
/// Remember to call [`ThemeFont::reload_font`] if the clone is going to be used.
impl Clone for ThemeFont {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            font_size: self.font_size,
            char_spacing: self.char_spacing,
            line_spacing: self.line_spacing,
            font: OptionalFont::Unloaded,
        }
    }
}

impl std::ops::Deref for ThemeFont {
    type Target = OptionalFont;

    fn deref(&self) -> &Self::Target {
        &self.font
    }
}

impl std::ops::DerefMut for ThemeFont {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.font
    }
}

impl AsRef<OptionalFont> for ThemeFont {
    fn as_ref(&self) -> &OptionalFont {
        self
    }
}

impl AsMut<OptionalFont> for ThemeFont {
    fn as_mut(&mut self) -> &mut OptionalFont {
        self
    }
}

impl AsRef<ffi::Font> for ThemeFont {
    fn as_ref(&self) -> &ffi::Font {
        self.font.as_ref()
    }
}

impl AsMut<ffi::Font> for ThemeFont {
    fn as_mut(&mut self) -> &mut ffi::Font {
        self.font.as_mut()
    }
}

impl RaylibFont for ThemeFont {}

impl ThemeFont {
    pub fn reload(&mut self, rl: &mut RaylibHandle, thread: &RaylibThread) {
        self.font = OptionalFont::load(rl, thread, self.path.as_ref());
    }

    pub fn line_height(&self) -> f32 {
        self.font_size + self.line_spacing
    }

    pub fn measure_text(&self, text: &str) -> Vector2 {
        self.font
            .measure_text(text, self.font_size, self.char_spacing)
    }

    pub fn draw_text<D: RaylibDraw>(&self, d: &mut D, text: &str, position: Vector2, tint: Color) {
        d.draw_text_ex(
            self,
            text,
            position,
            self.font_size,
            self.char_spacing,
            tint,
        );
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeButtonIcons {
    #[serde(rename = "x16")]
    pub x16_path: Option<PathBuf>,
    #[serde(rename = "x32")]
    pub x32_path: Option<PathBuf>,
    #[serde(skip)]
    pub sheets: Option<ButtonIconSheets>,
}

impl std::ops::Deref for ThemeButtonIcons {
    type Target = ButtonIconSheets;

    fn deref(&self) -> &Self::Target {
        self.sheets
            .as_ref()
            .expect("icons must be reloaded before accessing")
    }
}

impl std::ops::DerefMut for ThemeButtonIcons {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.sheets
            .as_mut()
            .expect("icons must be reloaded before accessing")
    }
}

impl AsRef<ButtonIconSheets> for ThemeButtonIcons {
    fn as_ref(&self) -> &ButtonIconSheets {
        self
    }
}

impl AsMut<ButtonIconSheets> for ThemeButtonIcons {
    fn as_mut(&mut self) -> &mut ButtonIconSheets {
        self
    }
}

/// NOTE: [`ThemeButtonIcons::clone`] assigns [`None`] to the [`ThemeButtonIcons::sheets`] field.
///
/// Remember to call [`ThemeButtonIcons::reload`] if the clone is going to be used.
impl Clone for ThemeButtonIcons {
    fn clone(&self) -> Self {
        Self {
            x16_path: self.x16_path.clone(),
            x32_path: self.x32_path.clone(),
            sheets: None,
        }
    }
}

impl ThemeButtonIcons {
    pub fn reload(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
    ) -> Result<(), raylib::error::Error> {
        let mut load = |path: Option<&PathBuf>,
                        default: &[u8]|
         -> Result<Texture2D, raylib::error::Error> {
            match path {
                // SAFETY: ffi::LoadTexture uses the raw OS string anyway, load_texture using a &str just gets in our way
                Some(path) => rl.load_texture(thread, unsafe {
                    str::from_utf8_unchecked(path.as_os_str().as_encoded_bytes())
                }),
                None => rl
                    .load_texture_from_image(thread, &Image::load_image_from_mem(".png", default)?),
            }
        };
        self.sheets = Some(ButtonIconSheets {
            x16: load(
                self.x16_path.as_ref(),
                include_bytes!("../assets/icons16x.png"),
            )?,
            x32: load(
                self.x16_path.as_ref(),
                include_bytes!("../assets/icons32x.png"),
            )?,
        });
        Ok(())
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeNodeIcons {
    #[serde(rename = "basic8x")]
    pub basic8x_path: Option<PathBuf>,
    #[serde(rename = "background8x")]
    pub background8x_path: Option<PathBuf>,
    #[serde(rename = "highlight8x")]
    pub highlight8x_path: Option<PathBuf>,
    #[serde(rename = "ntd8x")]
    pub ntd8x_path: Option<PathBuf>,
    #[serde(rename = "basic16x")]
    pub basic16x_path: Option<PathBuf>,
    #[serde(rename = "background16x")]
    pub background16x_path: Option<PathBuf>,
    #[serde(rename = "highlight16x")]
    pub highlight16x_path: Option<PathBuf>,
    #[serde(rename = "ntd16x")]
    pub ntd16x_path: Option<PathBuf>,
    #[serde(rename = "basic32x")]
    pub basic32x_path: Option<PathBuf>,
    #[serde(rename = "background32x")]
    pub background32x_path: Option<PathBuf>,
    #[serde(rename = "highlight32x")]
    pub highlight32x_path: Option<PathBuf>,
    #[serde(rename = "ntd32x")]
    pub ntd32x_path: Option<PathBuf>,
    #[serde(skip)]
    pub sheetsets: Option<NodeIconSheetSets>,
}

impl std::ops::Deref for ThemeNodeIcons {
    type Target = NodeIconSheetSets;

    fn deref(&self) -> &Self::Target {
        self.sheetsets
            .as_ref()
            .expect("icons must be reloaded before accessing")
    }
}

impl std::ops::DerefMut for ThemeNodeIcons {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.sheetsets
            .as_mut()
            .expect("icons must be reloaded before accessing")
    }
}

impl AsRef<NodeIconSheetSets> for ThemeNodeIcons {
    fn as_ref(&self) -> &NodeIconSheetSets {
        self
    }
}

impl AsMut<NodeIconSheetSets> for ThemeNodeIcons {
    fn as_mut(&mut self) -> &mut NodeIconSheetSets {
        self
    }
}

/// NOTE: [`ThemeNodeIcons::clone`] assigns [`None`] to the [`ThemeNodeIcons::sheetsets`] field.
///
/// Remember to call [`ThemeNodeIcons::reload`] if the clone is going to be used.
impl Clone for ThemeNodeIcons {
    fn clone(&self) -> Self {
        Self {
            basic8x_path: self.basic8x_path.clone(),
            background8x_path: self.background8x_path.clone(),
            highlight8x_path: self.highlight8x_path.clone(),
            ntd8x_path: self.ntd8x_path.clone(),
            basic16x_path: self.basic16x_path.clone(),
            background16x_path: self.background16x_path.clone(),
            highlight16x_path: self.highlight16x_path.clone(),
            ntd16x_path: self.ntd16x_path.clone(),
            basic32x_path: self.basic32x_path.clone(),
            background32x_path: self.background32x_path.clone(),
            highlight32x_path: self.highlight32x_path.clone(),
            ntd32x_path: self.ntd32x_path.clone(),
            sheetsets: None,
        }
    }
}

impl ThemeNodeIcons {
    pub fn reload(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
    ) -> Result<(), raylib::error::Error> {
        let mut load = |path: &Option<PathBuf>,
                        default: &[u8]|
         -> Result<Texture2D, raylib::error::Error> {
            match path.as_ref() {
                // SAFETY: ffi::LoadTexture uses the raw OS string anyway, load_texture using a &str just gets in our way
                Some(path) => rl.load_texture(thread, unsafe {
                    str::from_utf8_unchecked(path.as_os_str().as_encoded_bytes())
                }),
                None => rl
                    .load_texture_from_image(thread, &Image::load_image_from_mem(".png", default)?),
            }
        };

        self.sheetsets = Some(NodeIconSheetSets {
            x8: NodeIconSheetSet {
                basic: load(
                    &self.basic8x_path,
                    include_bytes!("../assets/nodeicons/nodeIconsBasic8x.png"),
                )?,
                background: load(
                    &self.background8x_path,
                    include_bytes!("../assets/nodeicons/nodeIconsBackground8x.png"),
                )?,
                highlight: load(
                    &self.highlight8x_path,
                    include_bytes!("../assets/nodeicons/nodeIconsHighlight8x.png"),
                )?,
                ntd: load(
                    &self.ntd8x_path,
                    include_bytes!("../assets/nodeicons/nodeIconsNTD8x.png"),
                )?,
            },
            x16: NodeIconSheetSet {
                basic: load(
                    &self.basic16x_path,
                    include_bytes!("../assets/nodeicons/nodeIconsBasic16x.png"),
                )?,
                background: load(
                    &self.background16x_path,
                    include_bytes!("../assets/nodeicons/nodeIconsBackground16x.png"),
                )?,
                highlight: load(
                    &self.highlight16x_path,
                    include_bytes!("../assets/nodeicons/nodeIconsHighlight16x.png"),
                )?,
                ntd: load(
                    &self.ntd16x_path,
                    include_bytes!("../assets/nodeicons/nodeIconsNTD16x.png"),
                )?,
            },
            x32: NodeIconSheetSet {
                basic: load(
                    &self.basic32x_path,
                    include_bytes!("../assets/nodeicons/nodeIconsBasic32x.png"),
                )?,
                background: load(
                    &self.background32x_path,
                    include_bytes!("../assets/nodeicons/nodeIconsBackground32x.png"),
                )?,
                highlight: load(
                    &self.highlight32x_path,
                    include_bytes!("../assets/nodeicons/nodeIconsHighlight32x.png"),
                )?,
                ntd: load(
                    &self.ntd32x_path,
                    include_bytes!("../assets/nodeicons/nodeIconsNTD32x.png"),
                )?,
            },
        });
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum BaseTheme {
    #[default]
    Dark,
    Light,
}

impl BaseTheme {
    fn theme(self) -> Theme {
        match self {
            Self::Dark => Theme::dark_theme(),
            Self::Light => Theme::light_theme(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct ThemeLoader {
    pub base: Option<BaseTheme>,
    pub background: Option<SerdeColor>,
    pub background1: Option<SerdeColor>,
    pub background2: Option<SerdeColor>,
    pub background3: Option<SerdeColor>,
    pub foreground3: Option<SerdeColor>,
    pub foreground2: Option<SerdeColor>,
    pub foreground1: Option<SerdeColor>,
    pub foreground: Option<SerdeColor>,
    pub input: Option<SerdeColor>,
    pub output: Option<SerdeColor>,
    pub available: Option<SerdeColor>,
    pub interact: Option<SerdeColor>,
    pub active: Option<SerdeColor>,
    pub error: Option<SerdeColor>,
    pub destructive: Option<SerdeColor>,
    pub special: Option<SerdeColor>,
    pub hyperref: Option<SerdeColor>,
    pub dead_link: Option<SerdeColor>,
    pub caution: Option<SerdeColor>,
    pub blueprints_background: Option<SerdeColor>,
    pub resistance0: Option<SerdeColor>,
    pub resistance1: Option<SerdeColor>,
    pub resistance2: Option<SerdeColor>,
    pub resistance3: Option<SerdeColor>,
    pub resistance4: Option<SerdeColor>,
    pub resistance5: Option<SerdeColor>,
    pub resistance6: Option<SerdeColor>,
    pub resistance7: Option<SerdeColor>,
    pub resistance8: Option<SerdeColor>,
    pub resistance9: Option<SerdeColor>,
    pub general_font: Option<ThemeFont>,
    pub title_font: Option<ThemeFont>,
    pub properties_header_font: Option<ThemeFont>,
    pub console_font: Option<ThemeFont>,
    pub console_padding: Option<Padding>,
    pub title_padding: Option<Padding>,
    pub button_icon_scale: Option<ButtonIconSheetId>,
    pub toolpane_orientation: Option<Orientation>,
    pub toolpane_visibility: Option<Visibility>,
    pub toolpane_padding: Option<Padding>,
    pub toolpane_group_expanded_gap: Option<f32>,
    pub toolpane_group_collapsed_gap: Option<f32>,
    pub toolpane_button_gap: Option<f32>,
    pub properties_padding: Option<Padding>,
    pub properties_section_gap: Option<f32>,
    pub button_icons: Option<ThemeButtonIcons>,
    pub node_icons: Option<ThemeNodeIcons>,
}

impl From<ThemeLoader> for Theme {
    fn from(value: ThemeLoader) -> Self {
        let base = value.base.unwrap_or_default().theme();
        Self {
            background: value.background.map_or(base.background, Into::into),
            background1: value.background1.map_or(base.background1, Into::into),
            background2: value.background2.map_or(base.background2, Into::into),
            background3: value.background3.map_or(base.background3, Into::into),
            foreground3: value.foreground3.map_or(base.foreground3, Into::into),
            foreground2: value.foreground2.map_or(base.foreground2, Into::into),
            foreground1: value.foreground1.map_or(base.foreground1, Into::into),
            foreground: value.foreground.map_or(base.foreground, Into::into),
            input: value.input.map_or(base.input, Into::into),
            output: value.output.map_or(base.output, Into::into),
            available: value.available.map_or(base.available, Into::into),
            interact: value.interact.map_or(base.interact, Into::into),
            active: value.active.map_or(base.active, Into::into),
            error: value.error.map_or(base.error, Into::into),
            destructive: value.destructive.map_or(base.destructive, Into::into),
            special: value.special.map_or(base.special, Into::into),
            hyperref: value.hyperref.map_or(base.hyperref, Into::into),
            dead_link: value.dead_link.map_or(base.dead_link, Into::into),
            caution: value.caution.map_or(base.caution, Into::into),
            blueprints_background: value
                .blueprints_background
                .map_or(base.blueprints_background, Into::into),
            resistance: [
                value.resistance0.map_or(base.resistance[0], Into::into),
                value.resistance1.map_or(base.resistance[1], Into::into),
                value.resistance2.map_or(base.resistance[2], Into::into),
                value.resistance3.map_or(base.resistance[3], Into::into),
                value.resistance4.map_or(base.resistance[4], Into::into),
                value.resistance5.map_or(base.resistance[5], Into::into),
                value.resistance6.map_or(base.resistance[6], Into::into),
                value.resistance7.map_or(base.resistance[7], Into::into),
                value.resistance8.map_or(base.resistance[8], Into::into),
                value.resistance9.map_or(base.resistance[9], Into::into),
            ],
            general_font: value.general_font.unwrap_or(base.general_font),
            title_font: value.title_font.unwrap_or(base.title_font),
            properties_header_font: value
                .properties_header_font
                .unwrap_or(base.properties_header_font),
            console_font: value.console_font.unwrap_or(base.console_font),
            console_padding: value.console_padding.unwrap_or(base.console_padding),
            title_padding: value.title_padding.unwrap_or(base.title_padding),
            button_icon_scale: value.button_icon_scale.unwrap_or(base.button_icon_scale),
            toolpane_orientation: value
                .toolpane_orientation
                .unwrap_or(base.toolpane_orientation),
            toolpane_visibility: value
                .toolpane_visibility
                .unwrap_or(base.toolpane_visibility),
            toolpane_padding: value.toolpane_padding.unwrap_or(base.toolpane_padding),
            toolpane_group_expanded_gap: value
                .toolpane_group_expanded_gap
                .unwrap_or(base.toolpane_group_expanded_gap),
            toolpane_group_collapsed_gap: value
                .toolpane_group_collapsed_gap
                .unwrap_or(base.toolpane_group_collapsed_gap),
            toolpane_button_gap: value
                .toolpane_button_gap
                .unwrap_or(base.toolpane_button_gap),
            properties_padding: value.properties_padding.unwrap_or(base.properties_padding),
            properties_section_gap: value
                .properties_section_gap
                .unwrap_or(base.properties_section_gap),
            node_icons: value.node_icons.unwrap_or(base.node_icons),
            button_icons: value.button_icons.unwrap_or(base.button_icons),
        }
    }
}

impl From<Theme> for ThemeLoader {
    fn from(value: Theme) -> Self {
        Self {
            base: None,
            background: Some(value.background.into()),
            background1: Some(value.background1.into()),
            background2: Some(value.background2.into()),
            background3: Some(value.background3.into()),
            foreground3: Some(value.foreground3.into()),
            foreground2: Some(value.foreground2.into()),
            foreground1: Some(value.foreground1.into()),
            foreground: Some(value.foreground.into()),
            input: Some(value.input.into()),
            output: Some(value.output.into()),
            available: Some(value.available.into()),
            interact: Some(value.interact.into()),
            active: Some(value.active.into()),
            error: Some(value.error.into()),
            destructive: Some(value.destructive.into()),
            special: Some(value.special.into()),
            hyperref: Some(value.hyperref.into()),
            dead_link: Some(value.dead_link.into()),
            caution: Some(value.caution.into()),
            blueprints_background: Some(value.blueprints_background.into()),
            resistance0: Some(value.resistance[0].into()),
            resistance1: Some(value.resistance[1].into()),
            resistance2: Some(value.resistance[2].into()),
            resistance3: Some(value.resistance[3].into()),
            resistance4: Some(value.resistance[4].into()),
            resistance5: Some(value.resistance[5].into()),
            resistance6: Some(value.resistance[6].into()),
            resistance7: Some(value.resistance[7].into()),
            resistance8: Some(value.resistance[8].into()),
            resistance9: Some(value.resistance[9].into()),
            general_font: Some(value.general_font),
            title_font: Some(value.title_font),
            properties_header_font: Some(value.properties_header_font),
            console_font: Some(value.console_font),
            console_padding: Some(value.console_padding),
            title_padding: Some(value.title_padding),
            button_icon_scale: Some(value.button_icon_scale),
            toolpane_orientation: Some(value.toolpane_orientation),
            toolpane_visibility: Some(value.toolpane_visibility),
            toolpane_padding: Some(value.toolpane_padding),
            toolpane_group_expanded_gap: Some(value.toolpane_group_expanded_gap),
            toolpane_group_collapsed_gap: Some(value.toolpane_group_collapsed_gap),
            toolpane_button_gap: Some(value.toolpane_button_gap),
            properties_padding: Some(value.properties_padding),
            properties_section_gap: Some(value.properties_section_gap),
            node_icons: Some(value.node_icons),
            button_icons: Some(value.button_icons),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "ThemeLoader", into = "ThemeLoader")]
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
    pub general_font: ThemeFont,
    pub title_font: ThemeFont,
    pub properties_header_font: ThemeFont,
    pub console_font: ThemeFont,
    pub console_padding: Padding,
    pub title_padding: Padding,
    pub button_icon_scale: ButtonIconSheetId,
    pub toolpane_orientation: Orientation,
    pub toolpane_visibility: Visibility,
    /// Relative to toolpane orientation
    pub toolpane_padding: Padding,
    pub toolpane_group_expanded_gap: f32,
    pub toolpane_group_collapsed_gap: f32,
    pub toolpane_button_gap: f32,
    pub properties_padding: Padding,
    pub properties_section_gap: f32,
    pub button_icons: ThemeButtonIcons,
    pub node_icons: ThemeNodeIcons,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark_theme()
    }
}

impl Theme {
    pub fn reload_assets(
        &mut self,
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
    ) -> Result<(), raylib::error::Error> {
        for font_item in [
            &mut self.general_font,
            &mut self.title_font,
            &mut self.properties_header_font,
            &mut self.console_font,
        ] {
            font_item.reload(rl, thread);
        }
        self.node_icons.reload(rl, thread)?;
        self.button_icons.reload(rl, thread)?;
        Ok(())
    }

    pub fn dark_theme() -> Self {
        Self {
            background: Color::BLACK,
            background1: Color::SPACEGRAY,
            background2: Color::LIFELESSNEBULA,
            background3: Color::GLEEFULDUST,
            foreground3: Color::DEADCABLE,
            foreground2: Color::HAUNTINGWHITE,
            foreground1: Color::INTERFERENCEGRAY,
            foreground: Color::WHITE,
            input: Color::INPUTLAVENDER,
            output: Color::OUTPUTAPRICOT,
            available: Color::WIPBLUE,
            interact: Color::YELLOW,
            active: Color::REDSTONE,
            error: Color::MAGENTA,
            destructive: Color::DESTRUCTIVERED,
            special: Color::VIOLET,
            hyperref: Color::GLEEFULDUST,
            dead_link: Color::HAUNTINGWHITE,
            caution: Color::CAUTIONYELLOW,
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
            general_font: ThemeFont::default(),
            title_font: ThemeFont::default(),
            properties_header_font: ThemeFont {
                font_size: 20.0,
                char_spacing: 2.0,
                line_spacing: 2.0,
                ..Default::default()
            },
            console_font: ThemeFont::default(),
            console_padding: Padding {
                left: 15.0,
                top: 5.0,
                right: 5.0,
                bottom: 5.0,
            },
            title_padding: Padding::block(6.0, 3.0),
            button_icon_scale: ButtonIconSheetId::X16,
            toolpane_orientation: Orientation::Vertical,
            toolpane_visibility: Visibility::Expanded,
            toolpane_padding: Padding::block(3.0, 5.0),
            toolpane_group_expanded_gap: 16.0,
            toolpane_group_collapsed_gap: 16.0,
            toolpane_button_gap: 1.0,
            properties_padding: Padding {
                left: 5.0,
                top: 5.0,
                right: 5.0,
                bottom: 5.0,
            },
            properties_section_gap: 20.0,
            button_icons: ThemeButtonIcons::default(),
            node_icons: ThemeNodeIcons::default(),
        }
    }

    pub fn light_theme() -> Self {
        Self {
            background: Color::WHITE,
            background1: Color::new(226, 227, 227, 255),
            background2: Color::new(188, 188, 188, 255),
            background3: Color::GRAY,
            foreground3: Color::DEADCABLE,
            foreground2: Color::new(100, 100, 100, 255),
            foreground1: Color::new(75, 75, 75, 255),
            foreground: Color::new(40, 40, 40, 255),
            input: Color::INPUTLAVENDER,
            output: Color::OUTPUTAPRICOT,
            available: Color::new(26, 115, 232, 255),
            interact: Color::new(231, 240, 253, 255),
            active: Color::BLUE,
            error: Color::MAGENTA,
            destructive: Color::DESTRUCTIVERED,
            special: Color::new(135, 60, 190, 255),
            hyperref: Color::BLUE,
            dead_link: Color::BISQUE,
            caution: Color::CAUTIONYELLOW,
            blueprints_background: Color::new(250, 250, 255, 255),
            ..Default::default()
        }
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

#[derive(Debug, Default)]
pub enum OptionalFont {
    #[default]
    Unloaded,
    Strong(Font),
    Weak(WeakFont),
}

impl OptionalFont {
    /// Uses default if error occurs
    pub fn load<P>(rl: &mut RaylibHandle, _: &RaylibThread, path: Option<P>) -> Self
    where
        P: AsRef<Path>,
    {
        if let Some(path) = path
            && let Ok(filename) =
                std::ffi::CString::new(path.as_ref().as_os_str().as_encoded_bytes())
        {
            // SAFETY: LoadFont just opens the file under the hood, which uses the OS encoding
            let f = unsafe { ffi::LoadFont(filename.as_ptr()) };
            if !(f.glyphs.is_null() || f.texture.id == 0) {
                // SAFETY: guaranteed not to have duplicates of what we just created and didnt copy
                return Self::Strong(unsafe { Font::from_raw(f) });
            }
        }
        Self::Weak(rl.get_font_default())
    }
}

impl AsRef<ffi::Font> for OptionalFont {
    fn as_ref(&self) -> &ffi::Font {
        match self {
            Self::Unloaded => panic!("font must be loaded before using"),
            Self::Strong(font) => font.as_ref(),
            Self::Weak(font) => font.as_ref(),
        }
    }
}

impl AsMut<ffi::Font> for OptionalFont {
    fn as_mut(&mut self) -> &mut ffi::Font {
        match self {
            Self::Unloaded => panic!("font must be loaded before using"),
            Self::Strong(font) => font.as_mut(),
            Self::Weak(font) => font.as_mut(),
        }
    }
}

impl RaylibFont for OptionalFont {}
