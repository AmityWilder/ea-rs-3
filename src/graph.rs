use crate::{
    GRID_SIZE,
    console::{Console, GateRef, GraphRef, LogType, NodeRef, PositionRef},
    graph::{
        node::{Gate, Node, NodeId},
        wire::{Elbow, Flow, Wire, WireId},
    },
    ivec::IVec2,
    logln,
};
use rustc_hash::{FxHashMap, FxHashSet};
use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

pub mod node;
pub mod wire;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GraphId(u32);

impl std::fmt::Display for GraphId {
    #[inline]
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

    #[inline]
    fn world_to_grid(world_pos: IVec2) -> IVec2 {
        IVec2::new(
            world_pos.x / i32::from(GRID_SIZE),
            world_pos.y / i32::from(GRID_SIZE),
        )
    }

    #[inline]
    pub const fn id(&self) -> &GraphId {
        &self.id
    }

    #[inline]
    pub fn find_node_at(&self, pos: IVec2) -> Option<&NodeId> {
        self.node_grid.get(&Self::world_to_grid(pos))
    }

    #[inline]
    pub fn node(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    #[inline]
    pub fn node_mut(&mut self, id: &NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }

    #[inline]
    pub fn wire(&self, id: &WireId) -> Option<&Wire> {
        self.wires.get(id)
    }

    #[inline]
    pub fn wire_mut(&mut self, id: &WireId) -> Option<&mut Wire> {
        self.wires.get_mut(id)
    }

    /// Returns [`None`] if the position is already occupied
    #[must_use]
    pub fn create_node(
        &mut self,
        gate: Gate,
        position: IVec2,
        console: &mut Console,
    ) -> Option<&mut Node> {
        let id = self.next_node_id;
        self.next_node_id.0 += 1;
        let grid_pos = Self::world_to_grid(position);
        (!self.node_grid.contains_key(&grid_pos)).then(|| {
            self.node_grid.insert(grid_pos, id);
            let node = self
                .nodes
                .entry(id)
                .insert_entry(Node::new(id, gate, position))
                .into_mut();
            logln!(
                console,
                LogType::Info,
                "create {} node {} at {}",
                GateRef(gate.id()),
                NodeRef(self.id, *node.id()),
                PositionRef(position),
            );
            node
        })
    }

    pub fn translate_node(
        &mut self,
        id: &NodeId,
        new_position: IVec2,
        console: &mut Console,
    ) -> Option<()> {
        let node = self.nodes.get_mut(id)?;
        let old_grid_position = Self::world_to_grid(node.position);
        let new_grid_position = Self::world_to_grid(new_position);
        if old_grid_position != new_grid_position {
            let id = self
                .node_grid
                .remove(&old_grid_position)
                .filter(|x| x == id)
                .expect("nodes should not be moved without updating their position in node_grid");
            self.node_grid.insert(new_grid_position, id);

            let old_position = std::mem::replace(&mut node.position, new_position);
            logln!(
                console,
                LogType::Info,
                "move node {} from {} to {}",
                NodeRef(self.id, id),
                PositionRef(old_position),
                PositionRef(new_position),
            );
        }
        Some(())
    }

    pub fn destroy_node(&mut self, id: &NodeId, soft: bool, console: &mut Console) -> Option<Node> {
        self.nodes.remove(id).inspect(|node| {
            self.node_grid
                .remove(&Self::world_to_grid(node.position))
                .filter(|x| x == id)
                .expect("nodes should not be moved without updating their position in node_grid");
            if soft {
                todo!()
            } else {
                self.wires
                    .retain(|_, wire| &wire.src != id && &wire.dst != id);
            }
            logln!(
                console,
                LogType::Info,
                "destroy node {}",
                NodeRef(self.id, *id)
            );
        })
    }

    pub fn create_wire(
        &mut self,
        elbow: Elbow,
        src: NodeId,
        dst: NodeId,
        console: &mut Console,
    ) -> &mut Wire {
        assert_ne!(src, dst, "cannot wire a node directly to itself");
        if let Some(existing) = self
            .wires
            .iter()
            .find(|(_, wire)| wire.src == src && wire.dst == dst)
            .map(|(id, _)| *id)
        {
            let graph_ref = GraphRef(self.id);
            logln!(
                console,
                LogType::Info,
                "wire {} from {} to {} already exists",
                graph_ref.wire(existing),
                graph_ref.node(src),
                graph_ref.node(dst),
            );
            self.wire_mut(&existing).expect("just found")
        } else {
            let graph_ref = GraphRef(self.id);
            let id = self.next_wire_id;
            self.next_wire_id.0 += 1;
            let wire = self
                .wires
                .entry(id)
                .insert_entry(Wire::new(id, elbow, src, dst))
                .into_mut();

            logln!(
                console,
                LogType::Info,
                "create wire {} from {} to {}",
                graph_ref.wire(*wire.id()),
                graph_ref.node(src),
                graph_ref.node(dst),
            );
            wire
        }
    }

    #[inline]
    pub fn destroy_wire(&mut self, id: &WireId) -> Option<Wire> {
        self.wires.remove(id)
    }

    #[inline]
    pub fn nodes_iter(&self) -> std::collections::hash_map::Values<'_, NodeId, Node> {
        self.nodes.values()
    }

    #[inline]
    pub fn wires_iter(&self) -> std::collections::hash_map::Values<'_, WireId, Wire> {
        self.wires.values()
    }

    #[inline]
    pub fn wires_to<'a>(&'a self, node: &NodeId) -> impl Iterator<Item = (&'a WireId, &'a Wire)> {
        self.wires.iter().filter(move |(_, wire)| &wire.dst == node)
    }

    #[inline]
    pub fn wires_from<'a>(&'a self, node: &NodeId) -> impl Iterator<Item = (&'a WireId, &'a Wire)> {
        self.wires.iter().filter(move |(_, wire)| &wire.src == node)
    }

    #[inline]
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
    #[inline]
    pub fn get_wire_nodes<'a>(&'a self, wire: &Wire) -> Option<(&'a Node, &'a Node)> {
        self.nodes.get(&wire.src).zip(self.nodes.get(&wire.dst))
    }

    #[inline]
    pub fn is_inputless(&self, node: &NodeId) -> bool {
        self.wires_to(node).next().is_none()
    }

    #[inline]
    pub fn is_outputless(&self, node: &NodeId) -> bool {
        self.wires_from(node).next().is_none()
    }

    #[inline]
    pub fn inputless_nodes<T, F>(&self, f: F) -> T
    where
        F: FnOnce(
            std::collections::hash_set::Difference<'_, NodeId, rustc_hash::FxBuildHasher>,
        ) -> T,
    {
        let a = FxHashSet::from_iter(self.nodes.keys().copied());
        let b = FxHashSet::from_iter(self.wires.values().map(|wire| wire.dst));
        f(a.difference(&b))
    }

    #[inline]
    pub fn outputless_nodes<T, F>(&self, f: F) -> T
    where
        F: FnOnce(
            std::collections::hash_set::Difference<'_, NodeId, rustc_hash::FxBuildHasher>,
        ) -> T,
    {
        let a = FxHashSet::from_iter(self.nodes.keys().copied());
        let b = FxHashSet::from_iter(self.wires.values().map(|wire| wire.src));
        f(a.difference(&b))
    }

    #[inline]
    pub fn adjacent_out(&self) -> FxHashMap<NodeId, Vec<NodeId>> {
        let mut outputs = FxHashMap::<_, Vec<_>>::default();
        for wire in self.wires.values() {
            outputs.entry(wire.src).or_default().push(wire.dst);
        }
        outputs
    }

    #[inline]
    pub fn adjacent_in(&self) -> FxHashMap<NodeId, Vec<NodeId>> {
        let mut inputs = FxHashMap::<_, Vec<_>>::default();
        for wire in self.wires.values() {
            inputs.entry(wire.dst).or_default().push(wire.src);
        }
        inputs
    }

    fn eval_order(&self) -> impl Iterator<Item = NodeId> {
        let mut order = Vec::with_capacity(self.nodes.len());
        let mut visited = FxHashSet::default();
        let adj = self.adjacent_in();
        let mut queue = self.outputless_nodes(|it| VecDeque::from_iter(it.copied()));
        visited.extend(queue.iter().copied());
        // println!("adj: {adj:?}");
        // println!("visited: {visited:?}");
        // println!("queue: {queue:?}");
        while let Some(node) = queue.pop_front()
        // .inspect(|x| println!("queue.pop -> {x:?}\nqueue: {queue:?}"))
        {
            order.push(node);
            // println!(" order.push({:?})", node);
            queue.extend(adj.get(&node).into_iter().flatten().filter_map(|&id| {
                visited.insert(id).then_some(id)
                // .inspect(|x| println!("  queue.push({x:?})"))
            }));
        }
        // println!("order: {order:?}");
        order.into_iter().rev()
    }

    pub fn evaluate(&mut self) {
        // println!("tick");
        let adj = self.adjacent_in();
        let mut input_buf = Vec::new();
        for id in self.eval_order().collect::<Vec<_>>() {
            input_buf.clear();
            input_buf.extend(adj.get(&id).into_iter().flatten().map(|id| {
                self.nodes
                    .get(id)
                    .expect("all nodes in adj should be valid")
                    .state
            }));
            let node = self
                .nodes
                .get_mut(&id)
                .expect("all nodes in eval_order should be valid");
            // println!("  node {id} ({:?}) inputs: {input_buf:?}", &node.gate);
            node.state = node.gate.evaluate(input_buf.iter().copied());
        }
    }
}

#[derive(Debug)]
pub struct GraphList {
    next_graph_id: GraphId,
    graphs: Vec<Arc<RwLock<Graph>>>,
}

impl std::ops::Deref for GraphList {
    type Target = Vec<Arc<RwLock<Graph>>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.graphs
    }
}

impl std::ops::DerefMut for GraphList {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.graphs
    }
}

impl Default for GraphList {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl GraphList {
    pub const fn new() -> Self {
        Self {
            next_graph_id: GraphId(0),
            graphs: Vec::new(),
        }
    }

    #[inline]
    pub fn create_graph(&mut self) -> &mut Arc<RwLock<Graph>> {
        let id = self.next_graph_id;
        self.next_graph_id.0 += 1;
        self.graphs.push(Arc::new(RwLock::new(Graph::new(id))));
        self.graphs.last_mut().expect("just pushed")
    }

    #[inline]
    pub fn try_get(&self, id: &GraphId) -> Option<&Arc<RwLock<Graph>>> {
        self.graphs
            .iter()
            .find(|g| g.try_read().unwrap().id() == id)
    }

    #[inline]
    pub fn try_get_mut(&mut self, id: &GraphId) -> Option<&mut Arc<RwLock<Graph>>> {
        self.graphs
            .iter_mut()
            .find(|g| g.try_read().unwrap().id() == id)
    }

    #[inline]
    pub fn get(&self, id: &GraphId) -> Option<&Arc<RwLock<Graph>>> {
        self.graphs.iter().find(|g| g.read().unwrap().id() == id)
    }

    #[inline]
    pub fn get_mut(&mut self, id: &GraphId) -> Option<&mut Arc<RwLock<Graph>>> {
        self.graphs
            .iter_mut()
            .find(|g| g.read().unwrap().id() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::node::GateNtd;

    fn gen_graph(
        id: GraphId,
        nodes: impl IntoIterator<Item = (NodeId, Gate)>,
        wires: impl IntoIterator<Item = (WireId, (NodeId, NodeId))>,
    ) -> Graph {
        let mut max_node_id = 0;
        let mut max_wire_id = 0;
        Graph {
            id,
            nodes: nodes
                .into_iter()
                .map(|(id, gate)| {
                    max_node_id = id.0.max(max_node_id);
                    (id, Node::new(id, gate, IVec2::default()))
                })
                .collect(),
            wires: wires
                .into_iter()
                .map(|(id, (src, dst))| {
                    max_wire_id = id.0.max(max_wire_id);
                    (id, Wire::new(id, Elbow::default(), src, dst))
                })
                .collect(),
            node_grid: FxHashMap::default(),
            next_node_id: NodeId(max_node_id + 1),
            next_wire_id: WireId(max_wire_id + 1),
        }
    }

    #[test]
    fn test0() {
        let mut g = gen_graph(
            GraphId(0),
            [
                (NodeId(0), Gate::Or),
                (NodeId(1), Gate::Or),
                (NodeId(2), Gate::Or),
                (NodeId(3), Gate::Or),
            ],
            [
                (WireId(0), (NodeId(0), NodeId(1))),
                (WireId(1), (NodeId(1), NodeId(2))),
                (WireId(2), (NodeId(2), NodeId(3))),
            ],
        );
        assert_eq!(
            g.adjacent_in(),
            FxHashMap::from_iter([
                (NodeId(1), vec![NodeId(0)]),
                (NodeId(2), vec![NodeId(1)]),
                (NodeId(3), vec![NodeId(2)]),
            ]),
            "adjacent_in"
        );
        assert_eq!(
            g.adjacent_out(),
            FxHashMap::from_iter([
                (NodeId(0), vec![NodeId(1)]),
                (NodeId(1), vec![NodeId(2)]),
                (NodeId(2), vec![NodeId(3)]),
            ]),
            "adjacent_out"
        );
        assert_eq!(
            g.inputless_nodes(|it| Vec::from_iter(it.copied()))
                .as_slice(),
            &[NodeId(0)],
            "inputless_nodes"
        );
        assert_eq!(
            g.outputless_nodes(|it| Vec::from_iter(it.copied()))
                .as_slice(),
            &[NodeId(3)],
            "outputless_nodes"
        );
        assert_eq!(
            g.eval_order().collect::<Vec<_>>().as_slice(),
            &[NodeId(0), NodeId(1), NodeId(2), NodeId(3),],
            "eval_order"
        );
        g.evaluate();
        assert_eq!(
            FxHashMap::from_iter(g.nodes.iter().map(|(id, node)| (*id, node.state))),
            FxHashMap::from_iter([
                (NodeId(0), false),
                (NodeId(1), false),
                (NodeId(2), false),
                (NodeId(3), false),
            ])
        );
        g.node_mut(&NodeId(0)).unwrap().gate = GateNtd::Nor;
        g.evaluate();
        assert_eq!(
            FxHashMap::from_iter(g.nodes.iter().map(|(id, node)| (*id, node.state))),
            FxHashMap::from_iter([
                (NodeId(0), true),
                (NodeId(1), true),
                (NodeId(2), true),
                (NodeId(3), true),
            ])
        );
        g.node_mut(&NodeId(0)).unwrap().gate = GateNtd::Or;
        g.evaluate();
        assert_eq!(
            FxHashMap::from_iter(g.nodes.iter().map(|(id, node)| (*id, node.state))),
            FxHashMap::from_iter([
                (NodeId(0), false),
                (NodeId(1), false),
                (NodeId(2), false),
                (NodeId(3), false),
            ])
        );
    }

    #[test]
    fn test1() {
        let g = gen_graph(
            GraphId(0),
            [
                (NodeId(0), Gate::Nor),
                (NodeId(1), Gate::Or),
                (NodeId(2), Gate::Or),
                (NodeId(3), Gate::Or),
                (NodeId(4), Gate::Or),
                (NodeId(5), Gate::Or),
            ],
            [
                (WireId(0), (NodeId(0), NodeId(1))),
                (WireId(1), (NodeId(1), NodeId(3))),
                (WireId(2), (NodeId(2), NodeId(3))),
                (WireId(3), (NodeId(3), NodeId(4))),
                (WireId(4), (NodeId(3), NodeId(5))),
            ],
        );
        assert_eq!(
            g.adjacent_in(),
            FxHashMap::from_iter([
                (NodeId(1), vec![NodeId(0)]),
                (NodeId(3), vec![NodeId(2), NodeId(1)]),
                (NodeId(4), vec![NodeId(3)]),
                (NodeId(5), vec![NodeId(3)]),
            ]),
            "adjacent_in"
        );
        assert_eq!(
            g.adjacent_out(),
            FxHashMap::from_iter([
                (NodeId(0), vec![NodeId(1)]),
                (NodeId(1), vec![NodeId(3)]),
                (NodeId(2), vec![NodeId(3)]),
                (NodeId(3), vec![NodeId(5), NodeId(4)]),
            ]),
            "adjacent_out"
        );
        assert_eq!(
            g.inputless_nodes(|it| Vec::from_iter(it.copied()))
                .as_slice(),
            &[NodeId(0), NodeId(2)],
            "inputless_nodes"
        );
        assert_eq!(
            g.outputless_nodes(|it| Vec::from_iter(it.copied()))
                .as_slice(),
            &[NodeId(4), NodeId(5)],
            "outputless_nodes"
        );
        assert_eq!(
            g.eval_order().collect::<Vec<_>>().as_slice(),
            &[
                // ---
                NodeId(0),
                // ---
                NodeId(1),
                NodeId(2),
                // ---
                NodeId(3),
                // ---
                NodeId(5),
                NodeId(4),
            ],
            "eval_order"
        );
    }
}
