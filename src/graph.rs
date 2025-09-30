use crate::{
    GRID_SIZE,
    graph::{
        node::{Gate, Node, NodeId},
        wire::{Elbow, Flow, Wire, WireId},
    },
    ivec::IVec2,
};
use rustc_hash::FxHashMap;
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

#[derive(Debug)]
pub struct Graph {
    next_node_id: NodeId,
    next_wire_id: WireId,
    id: GraphId,
    nodes: FxHashMap<NodeId, Node>,
    wires: FxHashMap<WireId, Wire>,
    node_grid: FxHashMap<IVec2, NodeId>,
}

impl Graph {
    pub fn new(id: GraphId) -> Self {
        Self {
            next_node_id: NodeId(0),
            next_wire_id: WireId(0),
            id,
            nodes: FxHashMap::default(),
            wires: FxHashMap::default(),
            node_grid: FxHashMap::default(),
        }
    }

    fn world_to_grid(world_pos: IVec2) -> IVec2 {
        IVec2::new(
            world_pos.x / i32::from(GRID_SIZE),
            world_pos.y / i32::from(GRID_SIZE),
        )
    }

    pub const fn id(&self) -> &GraphId {
        &self.id
    }

    pub fn find_node_at(&self, pos: IVec2) -> Option<&NodeId> {
        self.node_grid.get(&Self::world_to_grid(pos))
    }

    pub fn node(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    pub fn node_mut(&mut self, id: &NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }

    pub fn wire(&self, id: &WireId) -> Option<&Wire> {
        self.wires.get(id)
    }

    pub fn wire_mut(&mut self, id: &WireId) -> Option<&mut Wire> {
        self.wires.get_mut(id)
    }

    /// Returns [`None`] if the position is already occupied
    #[must_use]
    pub fn create_node(&mut self, gate: Gate, position: IVec2) -> Option<&mut Node> {
        let id = self.next_node_id;
        self.next_node_id.0 += 1;
        self.node_grid
            .insert(Self::world_to_grid(position), id)
            .is_none()
            .then(|| {
                self.nodes
                    .entry(id)
                    .insert_entry(Node::new(id, gate, position))
                    .into_mut()
            })
    }

    pub fn translate_node(&mut self, id: &NodeId, new_position: IVec2) -> Option<()> {
        let node = self.nodes.get_mut(id)?;
        let old_grid_position = Self::world_to_grid(node.position);
        node.position = new_position;
        let new_grid_position = Self::world_to_grid(node.position);
        if old_grid_position != new_grid_position {
            let id = self
                .node_grid
                .remove(&old_grid_position)
                .filter(|x| x == id)
                .expect("nodes should not be moved without updating their position in node_grid");
            self.node_grid.insert(new_grid_position, id);
        }
        Some(())
    }

    pub fn destroy_node(&mut self, id: &NodeId, soft: bool) -> Option<Node> {
        self.nodes.remove(id).inspect(|_| {
            if soft {
                todo!()
            } else {
                self.wires
                    .retain(|_, wire| &wire.src != id && &wire.dst != id);
            }
        })
    }

    pub fn create_wire(&mut self, elbow: Elbow, src: NodeId, dst: NodeId) -> &mut Wire {
        let id = self.next_wire_id;
        self.next_wire_id.0 += 1;
        self.wires
            .entry(id)
            .insert_entry(Wire::new(id, elbow, src, dst))
            .into_mut()
    }

    pub fn destroy_wire(&mut self, id: &WireId) -> Option<Wire> {
        self.wires.remove(id)
    }

    pub fn nodes_iter(&self) -> std::collections::hash_map::Values<'_, NodeId, Node> {
        self.nodes.values()
    }

    pub fn wires_iter(&self) -> std::collections::hash_map::Values<'_, WireId, Wire> {
        self.wires.values()
    }

    pub fn wires_to<'a>(&'a self, node: &NodeId) -> impl Iterator<Item = (&'a WireId, &'a Wire)> {
        self.wires.iter().filter(move |(_, wire)| &wire.dst == node)
    }

    pub fn wires_from<'a>(&'a self, node: &NodeId) -> impl Iterator<Item = (&'a WireId, &'a Wire)> {
        self.wires.iter().filter(move |(_, wire)| &wire.src == node)
    }

    pub fn wires_of<'a>(
        &'a self,
        node: &NodeId,
    ) -> impl Iterator<Item = (&'a WireId, &'a Wire, Flow)> {
        self.wires.iter().filter_map(move |(id, wire)| {
            match (&wire.src == node, &wire.dst == node) {
                (true, true) => Some((id, wire, Flow::Loop)),
                (true, false) => Some((id, wire, Flow::Output)),
                (false, true) => Some((id, wire, Flow::Input)),
                (false, false) => None,
            }
        })
    }

    /// Returns [`None`] if the start or end of the wire is not in the graph.
    pub fn get_wire_nodes<'a>(&'a self, wire: &Wire) -> Option<(&'a Node, &'a Node)> {
        self.nodes.get(&wire.src).zip(self.nodes.get(&wire.dst))
    }

    pub fn evaluate(&mut self) {
        let node_states: FxHashMap<NodeId, u8> = self
            .nodes
            .iter()
            .map(|(id, node)| (*id, node.state))
            .collect();
        for (id, node) in self.nodes.iter_mut() {
            let mut inputs = self
                .wires
                .values()
                .filter(|wire| &wire.dst == id) // surely there's a better way than O(nk)
                .map(|wire| {
                    node_states
                        .get(&wire.src)
                        .copied()
                        .expect("all wires should be valid")
                })
                .peekable();

            node.state = match node.gate {
                Gate::Or | Gate::Led { .. } => inputs.any(|x| x != 0) as u8,
                Gate::And => (inputs.peek().is_some() && inputs.all(|x| x != 0)) as u8,
                Gate::Nor => !inputs.any(|x| x != 0) as u8,
                Gate::Xor => (inputs.filter(|&x| x != 0).count() == 1) as u8,
                Gate::Resistor { resistance } => (inputs.sum::<u8>() > resistance) as u8,
                Gate::Capacitor { capacity } => {
                    let total = inputs.sum::<u8>().min(capacity);
                    if total > 0 {
                        total
                    } else {
                        node.state.saturating_sub(1)
                    }
                }
                Gate::Delay { ticks: _ } => 0, // TODO
                Gate::Battery => 1,
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

    pub fn try_get(&self, id: &GraphId) -> Option<&Arc<RwLock<Graph>>> {
        self.graphs
            .iter()
            .find(|g| g.try_read().unwrap().id() == id)
    }

    pub fn try_get_mut(&mut self, id: &GraphId) -> Option<&mut Arc<RwLock<Graph>>> {
        self.graphs
            .iter_mut()
            .find(|g| g.try_read().unwrap().id() == id)
    }

    pub fn get(&self, id: &GraphId) -> Option<&Arc<RwLock<Graph>>> {
        self.graphs.iter().find(|g| g.read().unwrap().id() == id)
    }

    pub fn get_mut(&mut self, id: &GraphId) -> Option<&mut Arc<RwLock<Graph>>> {
        self.graphs
            .iter_mut()
            .find(|g| g.read().unwrap().id() == id)
    }
}
