use crate::{graph::node::Gate, tool::Tool};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ToolPaneAnchoring {
    /// ```not_code
    /// .-.---------.-.
    /// | '---------' |
    /// |             |
    /// |             |
    /// '-------------'
    /// ```
    Top { left: i32, right: i32 },
    /// ```not_code
    /// .-----------.-.
    /// |-----------' |
    /// |             |
    /// |             |
    /// '-------------'
    /// ```
    TopLeft { right: i32 },
    /// ```not_code
    /// .-.-----------.
    /// | '-----------|
    /// |             |
    /// |             |
    /// '-------------'
    /// ```
    TopRight { left: i32 },
    /// ```not_code
    /// .-------------.
    /// |-------------|
    /// |             |
    /// |             |
    /// '-------------'
    /// ```
    TopFull,
    /// ```not_code
    /// .-------------.
    /// |-.           |
    /// | |           |
    /// |-'           |
    /// '-------------'
    /// ```
    Left { top: i32, bottom: i32 },
    /// ```not_code
    /// .-.-----------.
    /// | |           |
    /// | |           |
    /// |-'           |
    /// '-------------'
    /// ```
    LeftTop { bottom: i32 },
    /// ```not_code
    /// .-------------.
    /// |-.           |
    /// | |           |
    /// | |           |
    /// '-'-----------'
    /// ```
    LeftBottom { top: i32 },
    /// ```not_code
    /// .-.-----------.
    /// | |           |
    /// | |           |
    /// | |           |
    /// '-'-----------'
    /// ```
    #[default]
    LeftFull,
}

#[derive(Debug, Clone)]
pub struct ToolPane {
    pub tool: Tool,
    pub gate: Gate,
    pub anchoring: ToolPaneAnchoring,
}

impl ToolPane {
    pub fn new(tool: Tool, gate: Gate, anchoring: ToolPaneAnchoring) -> Self {
        Self {
            tool,
            gate,
            anchoring,
        }
    }
}
