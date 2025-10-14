use crate::{
    console::attempt::*,
    graph::{Graph, node::Gate, wire::Elbow},
    ivec::IVec2,
};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireBP {
    pub elbow: Elbow,
    /// Relative to [`Blueprint::nodes`]
    pub src: u32,
    /// Relative to [`Blueprint::nodes`]
    pub dst: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeBP {
    /// for tooltip
    pub name: Option<String>,
    pub state: bool,
    pub gate: Gate,
    pub position: (i32, i32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blueprint {
    pub nodes: Vec<NodeBP>,
    pub wires: Vec<WireBP>,
}

impl Graph {
    pub fn place_blueprint(&mut self, blueprint: &Blueprint) {
        let ids: Vec<_> = blueprint
            .nodes
            .iter()
            .map(|node_bp| {
                let (x, y) = node_bp.position;
                self.create_node(node_bp.gate, IVec2::new(x, y))
                    .warn("a blueprint node could not be created")
                    .ok()
                    .map(|node| *node.id())
            })
            .collect();

        for wire_bp in &blueprint.wires {
            if let (Some(src), Some(dst)) = (ids[wire_bp.src as usize], ids[wire_bp.dst as usize]) {
                self.create_wire(wire_bp.elbow, src, dst)
                    .issue_warning("a blueprint wire could not be created (one or both nodes it connects are missing)");
            }
        }
    }
}
