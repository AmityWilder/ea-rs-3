use crate::{
    graph::node::GateId,
    ivec::{IRect, IVec2},
};
use raylib::prelude::*;
use serde_derive::{Deserialize, Serialize};

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

impl GateId {
    pub const fn icon_cell(self) -> IVec2 {
        match self {
            GateId::Or => IVec2::new(0, 0),
            GateId::Nor => IVec2::new(1, 0),
            GateId::And => IVec2::new(2, 0),
            GateId::Xor => IVec2::new(3, 0),
            GateId::Resistor => IVec2::new(0, 1),
            GateId::Capacitor => IVec2::new(1, 1),
            GateId::Led => IVec2::new(2, 1),
            GateId::Delay => IVec2::new(3, 1),
            GateId::Battery => IVec2::new(0, 2),
        }
    }

    pub const fn icon_cell_irec(self, icon_width: i32) -> IRect {
        let cell = self.icon_cell();
        IRect::new(
            cell.x * icon_width,
            cell.y * icon_width,
            icon_width,
            icon_width,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeIconSheetSetId {
    X8,
    X16,
    X32,
}

impl NodeIconSheetSetId {
    #[inline]
    pub const fn icon_width(self) -> i32 {
        match self {
            Self::X8 => 8,
            Self::X16 => 16,
            Self::X32 => 32,
        }
    }

    #[inline]
    pub const fn from_zoom_exp(zoom_exp: i32) -> Option<Self> {
        match zoom_exp {
            ..0 => None,
            0 => Some(NodeIconSheetSetId::X8),
            1 => Some(NodeIconSheetSetId::X16),
            2.. => Some(NodeIconSheetSetId::X32),
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

static DEFAULT_NODE_ICON_SHEETSETS_DATA: [[&[u8]; 4]; 3] = [
    [
        include_bytes!("../assets/nodeicons/nodeIconsBasic8x.png"),
        include_bytes!("../assets/nodeicons/nodeIconsBackground8x.png"),
        include_bytes!("../assets/nodeicons/nodeIconsHighlight8x.png"),
        include_bytes!("../assets/nodeicons/nodeIconsNTD8x.png"),
    ],
    [
        include_bytes!("../assets/nodeicons/nodeIconsBasic16x.png"),
        include_bytes!("../assets/nodeicons/nodeIconsBackground16x.png"),
        include_bytes!("../assets/nodeicons/nodeIconsHighlight16x.png"),
        include_bytes!("../assets/nodeicons/nodeIconsNTD16x.png"),
    ],
    [
        include_bytes!("../assets/nodeicons/nodeIconsBasic32x.png"),
        include_bytes!("../assets/nodeicons/nodeIconsBackground32x.png"),
        include_bytes!("../assets/nodeicons/nodeIconsHighlight32x.png"),
        include_bytes!("../assets/nodeicons/nodeIconsNTD32x.png"),
    ],
];

#[derive(
    Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Default, Serialize, Deserialize,
)]
pub enum ButtonIconSheetId {
    #[default]
    #[serde(rename = "16px")]
    X16,
    #[serde(rename = "32px")]
    X32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl ButtonIconId {
    pub const fn icon_cell(self) -> IVec2 {
        match self {
            Self::Pen => IVec2::new(2, 0),
            Self::Erase => IVec2::new(2, 1),
            Self::Edit => IVec2::new(3, 0),
            Self::Interact => IVec2::new(3, 1),
            Self::Or => IVec2::new(0, 0),
            Self::And => IVec2::new(1, 0),
            Self::Nor => IVec2::new(0, 1),
            Self::Xor => IVec2::new(1, 1),
            Self::Resistor => IVec2::new(0, 2),
            Self::Capacitor => IVec2::new(1, 2),
            Self::Led => IVec2::new(0, 3),
            Self::Delay => IVec2::new(1, 3),
            Self::Battery => IVec2::new(0, 4),
            Self::BlueprintSelect => IVec2::new(2, 2),
            Self::Clipboard => IVec2::new(3, 2),
            Self::Settings => IVec2::new(2, 3),
        }
    }

    pub const fn icon_cell_irec(self, icon_width: i32) -> IRect {
        let cell = self.icon_cell();
        IRect::new(
            cell.x * icon_width,
            cell.y * icon_width,
            icon_width,
            icon_width,
        )
    }
}

impl ButtonIconSheetId {
    #[inline]
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
}
