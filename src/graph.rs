use crate::{
    graph::{
        node::{Gate, Node},
        wire::Wire,
    },
    ivec::IVec2,
};

pub mod node;
pub mod wire;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    nodes: Vec<Node>,
    wires: Vec<Wire>,
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}

impl Graph {
    pub const fn new() -> Self {
        Self {
            nodes: Vec::new(),
            wires: Vec::new(),
        }
    }

    pub fn find_node_at_pos(&self, pos: IVec2) -> Option<usize> {
        self.nodes.iter().position(|node| node.position == pos)
    }

    pub fn node(&self, idx: usize) -> Option<&Node> {
        self.nodes.get(idx)
    }

    pub fn node_mut(&mut self, idx: usize) -> Option<&mut Node> {
        self.nodes.get_mut(idx)
    }

    pub fn create_node(&mut self, gate: Gate, position: IVec2) -> (usize, &mut Node) {
        let idx = self.nodes.len();
        self.nodes.push(Node::new(gate, position));
        (idx, self.nodes.last_mut().expect("just pushed"))
    }

    pub fn create_wire(&mut self, src: usize, dst: usize) -> (usize, &mut Wire) {
        let idx = self.nodes.len();
        self.wires.push(Wire::new(src, dst));
        (idx, self.wires.last_mut().expect("just pushed"))
    }

    pub fn nodes_iter(
        &self,
    ) -> impl ExactSizeIterator<Item = &'_ Node> + DoubleEndedIterator + Clone {
        self.nodes.iter()
    }

    pub fn wires_iter(
        &self,
    ) -> impl ExactSizeIterator<Item = &'_ Wire> + DoubleEndedIterator + Clone {
        self.wires.iter()
    }

    /// Returns [`None`] if the start or end of the wire is not in the graph.
    pub fn get_wire_nodes<'a>(&'a self, wire: &Wire) -> Option<(&'a Node, &'a Node)> {
        self.nodes.get(wire.src).zip(self.nodes.get(wire.dst))
    }

    pub fn evaluate(&mut self) {
        for n in 0..self.nodes.len() {
            let mut inputs = self
                .wires
                .iter()
                .filter(|wire| wire.dst == n) // surely there's a better way than O(nk)
                .map(|wire| self.nodes[wire.src].state);
            self.nodes[n].state = match self.nodes[n].gate {
                Gate::Or => inputs.any(|x| x), // TODO
                Gate::And => false,            // TODO
                Gate::Nor => false,            // TODO
                Gate::Xor => false,            // TODO
                Gate::Resistor {} => false,    // TODO
                Gate::Capacitor {} => false,   // TODO
                Gate::Led {} => false,         // TODO
                Gate::Delay {} => false,       // TODO
                Gate::Battery => false,        // TODO
            };
        }
    }
}
