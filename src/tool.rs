use crate::ivec::IVec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ToolId {
    #[default]
    Create,
    Erase,
    Edit,
}

impl std::fmt::Display for ToolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolId::Create => "create",
            ToolId::Erase => "erase",
            ToolId::Edit => "edit",
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
            _ => Err(()),
        }
    }
}

impl ToolId {
    pub const fn init(self) -> Tool {
        match self {
            ToolId::Create => Tool::Create {},
            ToolId::Erase => Tool::Erase {},
            ToolId::Edit => Tool::Edit { target: None },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Tool {
    Create {},
    Erase {},
    Edit {
        /// (start_pos, idx)
        target: Option<(IVec2, usize)>,
    },
}

impl Default for Tool {
    fn default() -> Self {
        Self::Create {}
    }
}
