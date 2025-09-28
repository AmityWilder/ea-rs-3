use crate::{graph::Gate, ivec::IVec2};
use raylib::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NodeIconSheetId {
    #[default]
    Basic,
    Background,
    Highlight,
    Ntd,
}

#[derive(Debug)]
pub struct NodeIconSheetSet {
    pub basic: Texture2D,
    pub background: Texture2D,
    pub highlight: Texture2D,
    pub ntd: Texture2D,
}

impl std::ops::Index<NodeIconSheetId> for NodeIconSheetSet {
    type Output = Texture2D;

    fn index(&self, index: NodeIconSheetId) -> &Self::Output {
        match index {
            NodeIconSheetId::Basic => &self.basic,
            NodeIconSheetId::Background => &self.background,
            NodeIconSheetId::Highlight => &self.highlight,
            NodeIconSheetId::Ntd => &self.ntd,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeIconSheetSetId {
    X8,
    X16,
    X32,
}

impl NodeIconSheetSetId {
    pub const fn icon_width(self) -> i32 {
        match self {
            Self::X8 => 8,
            Self::X16 => 16,
            Self::X32 => 32,
        }
    }
}

#[derive(Debug)]
pub struct NodeIconSheetSets {
    pub x8: NodeIconSheetSet,
    pub x16: NodeIconSheetSet,
    pub x32: NodeIconSheetSet,
}

impl std::ops::Index<NodeIconSheetSetId> for NodeIconSheetSets {
    type Output = NodeIconSheetSet;

    fn index(&self, index: NodeIconSheetSetId) -> &Self::Output {
        match index {
            NodeIconSheetSetId::X8 => &self.x8,
            NodeIconSheetSetId::X16 => &self.x16,
            NodeIconSheetSetId::X32 => &self.x32,
        }
    }
}

impl NodeIconSheetSets {
    pub fn load(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
    ) -> Result<Self, raylib::error::Error> {
        Ok(Self {
            x8: NodeIconSheetSet {
                basic: rl.load_texture_from_image(
                    thread,
                    &Image::load_image_from_mem(
                        ".png",
                        include_bytes!("../assets/nodeicons/nodeIconsBasic8x.png"),
                    )?,
                )?,
                background: rl.load_texture_from_image(
                    thread,
                    &Image::load_image_from_mem(
                        ".png",
                        include_bytes!("../assets/nodeicons/nodeIconsBackground8x.png"),
                    )?,
                )?,
                highlight: rl.load_texture_from_image(
                    thread,
                    &Image::load_image_from_mem(
                        ".png",
                        include_bytes!("../assets/nodeicons/nodeIconsHighlight8x.png"),
                    )?,
                )?,
                ntd: rl.load_texture_from_image(
                    thread,
                    &Image::load_image_from_mem(
                        ".png",
                        include_bytes!("../assets/nodeicons/nodeIconsNTD8x.png"),
                    )?,
                )?,
            },
            x16: NodeIconSheetSet {
                basic: rl.load_texture_from_image(
                    thread,
                    &Image::load_image_from_mem(
                        ".png",
                        include_bytes!("../assets/nodeicons/nodeIconsBasic16x.png"),
                    )?,
                )?,
                background: rl.load_texture_from_image(
                    thread,
                    &Image::load_image_from_mem(
                        ".png",
                        include_bytes!("../assets/nodeicons/nodeIconsBackground16x.png"),
                    )?,
                )?,
                highlight: rl.load_texture_from_image(
                    thread,
                    &Image::load_image_from_mem(
                        ".png",
                        include_bytes!("../assets/nodeicons/nodeIconsHighlight16x.png"),
                    )?,
                )?,
                ntd: rl.load_texture_from_image(
                    thread,
                    &Image::load_image_from_mem(
                        ".png",
                        include_bytes!("../assets/nodeicons/nodeIconsNTD16x.png"),
                    )?,
                )?,
            },
            x32: NodeIconSheetSet {
                basic: rl.load_texture_from_image(
                    thread,
                    &Image::load_image_from_mem(
                        ".png",
                        include_bytes!("../assets/nodeicons/nodeIconsBasic32x.png"),
                    )?,
                )?,
                background: rl.load_texture_from_image(
                    thread,
                    &Image::load_image_from_mem(
                        ".png",
                        include_bytes!("../assets/nodeicons/nodeIconsBackground32x.png"),
                    )?,
                )?,
                highlight: rl.load_texture_from_image(
                    thread,
                    &Image::load_image_from_mem(
                        ".png",
                        include_bytes!("../assets/nodeicons/nodeIconsHighlight32x.png"),
                    )?,
                )?,
                ntd: rl.load_texture_from_image(
                    thread,
                    &Image::load_image_from_mem(
                        ".png",
                        include_bytes!("../assets/nodeicons/nodeIconsNTD32x.png"),
                    )?,
                )?,
            },
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw<D: RaylibDraw>(
        &self,
        d: &mut D,
        zoom_exp: i32,
        sheet: NodeIconSheetId,
        dest_rec: Rectangle,
        gate: Gate,
        origin: Vector2,
        rotation: f32,
        tint: Color,
    ) {
        let scale = match zoom_exp {
            ..0 => {
                d.draw_rectangle_rec(dest_rec, tint);
                return;
            }
            0 => NodeIconSheetSetId::X8,
            1 => NodeIconSheetSetId::X16,
            2.. => NodeIconSheetSetId::X32,
        };
        let pos = match gate {
            Gate::Or => IVec2::new(0, 0),
            Gate::Nor => IVec2::new(1, 0),
            Gate::And => IVec2::new(2, 0),
            Gate::Xor => IVec2::new(3, 0),
            Gate::Resistor { .. } => IVec2::new(0, 1),
            Gate::Capacitor { .. } => IVec2::new(1, 1),
            Gate::Led { .. } => IVec2::new(2, 1),
            Gate::Delay { .. } => IVec2::new(3, 1),
            Gate::Battery => IVec2::new(0, 2),
        };
        let width = scale.icon_width();
        let source_rec = Rectangle {
            x: (pos.x * width) as f32,
            y: (pos.y * width) as f32,
            width: width as f32,
            height: width as f32,
        };
        d.draw_texture_pro(
            &self[scale][sheet],
            source_rec,
            dest_rec,
            origin,
            rotation,
            tint,
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonIconSheetId {
    X16,
    X32,
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonIconId {
    Pen,
    Erase,
    Edit,
    Interact,
    Or,
    And,
    Nor,
    Xor,
    Resistor,
    Capacitor,
    Led,
    Delay,
    Battery,
    BlueprintSelect,
    Clipboard,
    Settings,
}

impl ButtonIconSheetId {
    pub const fn icon_width(self) -> i32 {
        match self {
            Self::X16 => 16,
            Self::X32 => 32,
        }
    }
}

#[derive(Debug)]
pub struct ButtonIconSheets {
    pub x16: Texture2D,
    pub x32: Texture2D,
}

impl std::ops::Index<ButtonIconSheetId> for ButtonIconSheets {
    type Output = Texture2D;

    fn index(&self, index: ButtonIconSheetId) -> &Self::Output {
        match index {
            ButtonIconSheetId::X16 => &self.x16,
            ButtonIconSheetId::X32 => &self.x32,
        }
    }
}

impl ButtonIconSheets {
    pub fn load(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
    ) -> Result<Self, raylib::error::Error> {
        Ok(Self {
            x16: rl.load_texture_from_image(
                thread,
                &Image::load_image_from_mem(".png", include_bytes!("../assets/icons16x.png"))?,
            )?,
            x32: rl.load_texture_from_image(
                thread,
                &Image::load_image_from_mem(".png", include_bytes!("../assets/icons32x.png"))?,
            )?,
        })
    }

    pub fn draw<D: RaylibDraw>(
        &self,
        d: &mut D,
        scale: ButtonIconSheetId,
        dest_rec: Rectangle,
        icon: ButtonIconId,
        origin: Vector2,
        rotation: f32,
        tint: Color,
    ) {
        let pos = match icon {
            ButtonIconId::Pen => IVec2::new(2, 0),
            ButtonIconId::Erase => IVec2::new(2, 1),
            ButtonIconId::Edit => IVec2::new(3, 0),
            ButtonIconId::Interact => IVec2::new(3, 1),
            ButtonIconId::Or => IVec2::new(0, 0),
            ButtonIconId::And => IVec2::new(1, 0),
            ButtonIconId::Nor => IVec2::new(0, 1),
            ButtonIconId::Xor => IVec2::new(1, 1),
            ButtonIconId::Resistor => IVec2::new(0, 2),
            ButtonIconId::Capacitor => IVec2::new(1, 2),
            ButtonIconId::Led => IVec2::new(0, 3),
            ButtonIconId::Delay => IVec2::new(1, 3),
            ButtonIconId::Battery => IVec2::new(0, 4),
            ButtonIconId::BlueprintSelect => IVec2::new(2, 2),
            ButtonIconId::Clipboard => IVec2::new(3, 2),
            ButtonIconId::Settings => IVec2::new(2, 3),
        };
        let width = scale.icon_width();
        let source_rec = Rectangle {
            x: (pos.x * width) as f32,
            y: (pos.y * width) as f32,
            width: width as f32,
            height: width as f32,
        };
        d.draw_texture_pro(&self[scale], source_rec, dest_rec, origin, rotation, tint);
    }
}
