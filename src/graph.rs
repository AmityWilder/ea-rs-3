use crate::ivec::IVec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Gate {
    #[default]
    Or,
    And,
    Nor,
    Xor,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Node {
    state: bool,
    pub gate: Gate,
    pub position: IVec2,
}

impl Node {
    pub const fn new(gate: Gate, position: IVec2) -> Self {
        Self {
            state: false,
            gate,
            position,
        }
    }

    pub const fn state(&self) -> bool {
        self.state
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Wire {
    src: usize,
    dst: usize,
}

impl Wire {
    pub const fn new(src: usize, dst: usize) -> Self {
        Self { src, dst }
    }
}

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
}
