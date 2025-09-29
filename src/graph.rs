use crate::{
    graph::{
        node::{Gate, Node, NodeId},
        wire::{Wire, WireId},
    },
    ivec::IVec2,
};
use std::sync::{Arc, RwLock};

pub mod node;
pub mod wire;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GraphId(u32);

impl std::fmt::Display for GraphId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "g{:x}", self.0)
    }
}

impl std::str::FromStr for GraphId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.strip_prefix('g')
            .ok_or(())
            .and_then(|x| u32::from_str_radix(x, 16).map_err(|_| ()))
            .map(Self)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Graph {
    next_node_id: NodeId,
    next_wire_id: WireId,
    id: GraphId,
    nodes: Vec<Node>,
    wires: Vec<Wire>,
}

impl Graph {
    pub const fn new(id: GraphId) -> Self {
        Self {
            next_node_id: NodeId(0),
            next_wire_id: WireId(0),
            id,
            nodes: Vec::new(),
            wires: Vec::new(),
        }
    }

    pub const fn id(&self) -> GraphId {
        self.id
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
        self.nodes
            .push(Node::new(self.next_node_id, gate, position));
        self.next_node_id.0 += 1;
        (idx, self.nodes.last_mut().expect("just pushed"))
    }

    pub fn destroy_node(&mut self, idx: usize) -> Node {
        self.nodes.remove(idx)
    }

    pub fn create_wire(&mut self, src: usize, dst: usize) -> (usize, &mut Wire) {
        let idx = self.nodes.len();
        self.wires.push(Wire::new(self.next_wire_id, src, dst));
        self.next_wire_id.0 += 1;
        (idx, self.wires.last_mut().expect("just pushed"))
    }

    pub fn destroy_wire(&mut self, idx: usize) -> Wire {
        self.wires.remove(idx)
    }

    pub fn nodes_iter(
        &self,
    ) -> impl ExactSizeIterator<Item = &'_ Node> + DoubleEndedIterator + Clone {
        self.nodes.iter()
    }

    pub fn get_node_by_id(&self, id: NodeId) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id() == id)
    }

    pub fn get_mut_node_by_id(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|n| n.id() == id)
    }

    pub fn wires_iter(
        &self,
    ) -> impl ExactSizeIterator<Item = &'_ Wire> + DoubleEndedIterator + Clone {
        self.wires.iter()
    }

    pub fn get_wire_by_id(&self, id: WireId) -> Option<&Wire> {
        self.wires.iter().find(|n| n.id() == id)
    }

    pub fn get_mut_wire_by_id(&mut self, id: WireId) -> Option<&mut Wire> {
        self.wires.iter_mut().find(|n| n.id() == id)
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

pub struct GraphList {
    next_graph_id: GraphId,
    graphs: Vec<Arc<RwLock<Graph>>>,
}

impl std::ops::Deref for GraphList {
    type Target = Vec<Arc<RwLock<Graph>>>;

    fn deref(&self) -> &Self::Target {
        &self.graphs
    }
}

impl std::ops::DerefMut for GraphList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graphs
    }
}

impl GraphList {
    pub const fn new() -> Self {
        Self {
            next_graph_id: GraphId(0),
            graphs: Vec::new(),
        }
    }

    pub fn create_graph(&mut self) -> &mut Arc<RwLock<Graph>> {
        let id = self.next_graph_id;
        self.next_graph_id.0 += 1;
        self.graphs.push(Arc::new(RwLock::new(Graph::new(id))));
        self.graphs.last_mut().expect("just pushed")
    }

    pub fn get_by_id(&self, id: GraphId) -> Option<&Arc<RwLock<Graph>>> {
        self.graphs.iter().find(|g| g.read().unwrap().id == id)
    }

    pub fn get_mut_by_id(&mut self, id: GraphId) -> Option<&mut Arc<RwLock<Graph>>> {
        self.graphs.iter_mut().find(|g| g.read().unwrap().id == id)
    }
}
