use crate::{graph::node::NodeId, ivec::IVec2};
use raylib::prelude::*;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolId {
    #[default]
    Create,
    Erase,
    Edit,
    Interact,
}

impl std::fmt::Display for ToolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolId::Create => "create",
            ToolId::Erase => "erase",
            ToolId::Edit => "edit",
            ToolId::Interact => "ineteract",
        }
        .fmt(f)
    }
}

impl std::str::FromStr for ToolId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "create" => Ok(ToolId::Create),
            "erase" => Ok(ToolId::Erase),
            "edit" => Ok(ToolId::Edit),
            "ineteract" => Ok(ToolId::Interact),
            _ => Err(()),
        }
    }
}

impl ToolId {
    pub const fn init(self) -> Tool {
        match self {
            ToolId::Create => Tool::Create { current_node: None },
            ToolId::Erase => Tool::Erase {},
            ToolId::Edit => Tool::Edit { target: None },
            ToolId::Interact => Tool::Interact {},
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EditDragging {
    pub start_pos: IVec2,
    pub temp_pos: Vector2,
    pub id: NodeId,
}

#[derive(Debug, Clone)]
pub enum Tool {
    Create { current_node: Option<NodeId> },
    Erase {},
    Edit { target: Option<EditDragging> },
    Interact {},
}

impl Default for Tool {
    fn default() -> Self {
        Self::Create { current_node: None }
    }
}

impl Tool {
    pub const fn id(&self) -> ToolId {
        match self {
            Tool::Create { .. } => ToolId::Create,
            Tool::Erase { .. } => ToolId::Erase,
            Tool::Edit { .. } => ToolId::Edit,
            Tool::Interact { .. } => ToolId::Interact,
        }
    }
}
