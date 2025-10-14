use crate::{
    GRID_SIZE,
    console::{GateRef, NodeRef, PositionRef, WireRef, attempt::*},
    graph::{
        node::{Gate, Node, NodeId},
        wire::{Elbow, Flow, Wire, WireId},
    },
    ivec::IVec2,
    logln,
};
use rustc_hash::{FxHashMap, FxHashSet};
use serde_derive::Deserialize;
use std::{
    collections::{
        VecDeque,
        hash_map::{Keys, Values},
    },
    sync::{
        Arc,
        nonpoison::{Mutex, RwLock},
    },
};
use thiserror::Error;

pub mod blueprint;
pub mod eag;
pub mod node;
pub mod wire;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
#[error("out of IDs")]
pub struct OutOfIDsError;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
#[error("nodes should not be moved without updating their position in node_grid")]
pub struct NodeGridDesyncError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GraphId(u32);

static NEXT_GRAPH_ID: Mutex<GraphId> = Mutex::new(GraphId(0));

/// Defaults to [`Self::INVALID`]
impl Default for GraphId {
    fn default() -> Self {
        Self::INVALID
    }
}

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

impl GraphId {
    pub const INVALID: Self = Self(!0);

    /// Returns the current value and increments `self`.
    /// Returns [`None`] if [`Self::INVALID`] would have been returned.
    /// Does not increment if `self` is [`Self::INVALID`].
    #[inline]
    pub const fn step(&mut self) -> Result<Self, OutOfIDsError> {
        const INVALID: GraphId = GraphId::INVALID;
        match *self {
            INVALID => Err(OutOfIDsError),
            id => {
                self.0 += 1;
                Ok(id)
            }
        }
    }

    #[inline]
    pub fn next() -> Result<Self, OutOfIDsError> {
        NEXT_GRAPH_ID.lock().step()
    }
}

macro_rules! dbg_ord_prinln {
    ($($bindings:pat),*$(,)? => $($args:tt)*) => {{
        #[cfg(feature = "dbg_order_algorithm")] {
            |$($bindings),*| { println!($($args)*); }
        }
        #[cfg(not(feature = "dbg_order_algorithm"))] {
            #[allow(unused_variables)]
            |$($bindings),*| {}
        }
    }};

    ($($args:tt)*) => {{
        #[cfg(feature = "dbg_order_algorithm")]
        println!($($args)*);
    }};
}

#[derive(Debug, Deserialize)]
#[serde(from = "eag::GraphTemplate")]
pub struct Graph {
    id: GraphId,
    nodes: FxHashMap<NodeId, Node>,
    wires: FxHashMap<WireId, Wire>,
    node_grid: FxHashMap<IVec2, NodeId>,
    named_nodes: FxHashMap<NodeId, String>,
    eval_order: Vec<NodeId>,
    eval_order_dict: FxHashMap<NodeId, (usize, usize)>,
    is_eval_order_dirty: bool,
}

/// Intended only for when the id was given BY the graph, just now
impl std::ops::Index<&NodeId> for Graph {
    type Output = Node;

    #[inline]
    fn index(&self, id: &NodeId) -> &Self::Output {
        self.node(id).fatal(
            "node is not in graph; graph functions that return an ID should \
            always return an ID that is valid until the graph is mutated",
        )
    }
}

/// Intended only for when the id was given BY the graph, just now
impl std::ops::IndexMut<&NodeId> for Graph {
    #[inline]
    fn index_mut(&mut self, id: &NodeId) -> &mut Self::Output {
        self.node_mut(id).fatal(
            "node is not in graph; graph functions that return an ID should \
            always return an ID that is valid until the graph is mutated",
        )
    }
}

/// Intended only for when the id was given BY the graph, just now
impl std::ops::Index<&WireId> for Graph {
    type Output = Wire;

    #[inline]
    fn index(&self, id: &WireId) -> &Self::Output {
        let g = *self.id();
        self.wire(id).fatal(format_args!(
            "wire {id} is not in graph {g}; graph functions that return an ID should \
            always return an ID that is valid until the graph is mutated"
        ))
    }
}

/// Intended only for when the id was given BY the graph, just now
impl std::ops::IndexMut<&WireId> for Graph {
    #[inline]
    fn index_mut(&mut self, id: &WireId) -> &mut Self::Output {
        let g = *self.id();
        self.wire_mut(id).fatal(format_args!(
            "wire {id} is not in graph {g}; graph functions that return an ID should \
            always return an ID that is valid until the graph is mutated"
        ))
    }
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
#[error(
    "the given ID is not an element of this graph; it may belong to another graph, or may have been removed"
)]
pub struct NotOfGraphError;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
#[error("the graph already contains a matching element")]
#[repr(transparent)]
pub struct AlreadyExistsError<T: ?Sized>(pub T);

impl<T> std::ops::Deref for AlreadyExistsError<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for AlreadyExistsError<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Graph {
    pub fn new(id: GraphId) -> Self {
        Self {
            id,
            nodes: FxHashMap::default(),
            wires: FxHashMap::default(),
            node_grid: FxHashMap::default(),
            named_nodes: FxHashMap::default(),
            eval_order: Vec::new(),
            eval_order_dict: FxHashMap::default(),
            is_eval_order_dirty: false,
        }
    }

    #[inline]
    pub const fn world_to_grid(world_pos: IVec2) -> IVec2 {
        IVec2::new(
            world_pos.x / GRID_SIZE as i32,
            world_pos.y / GRID_SIZE as i32,
        )
    }

    #[inline]
    pub const fn grid_to_world(grid_pos: IVec2) -> IVec2 {
        IVec2::new(grid_pos.x * GRID_SIZE as i32, grid_pos.y * GRID_SIZE as i32)
    }

    #[inline]
    pub const fn id(&self) -> &GraphId {
        &self.id
    }

    #[inline]
    pub fn node_at(&self, grid_pos: IVec2) -> Option<&Node> {
        self.node_grid.get(&grid_pos).map(|id| &self[id])
    }

    #[inline]
    pub fn node_mut_at(&mut self, grid_pos: IVec2) -> Option<&mut Node> {
        match self.node_grid.get(&grid_pos) {
            Some(&id) => Some(&mut self[&id]),
            None => None,
        }
    }

    #[inline]
    pub fn node(&self, id: &NodeId) -> Result<&Node, NotOfGraphError> {
        self.nodes.get(id).ok_or(NotOfGraphError)
    }

    #[inline]
    pub fn node_mut(&mut self, id: &NodeId) -> Result<&mut Node, NotOfGraphError> {
        self.nodes.get_mut(id).ok_or(NotOfGraphError)
    }

    #[inline]
    pub fn wire(&self, id: &WireId) -> Result<&Wire, NotOfGraphError> {
        self.wires.get(id).ok_or(NotOfGraphError)
    }

    #[inline]
    pub fn wire_mut(&mut self, id: &WireId) -> Result<&mut Wire, NotOfGraphError> {
        self.wires.get_mut(id).ok_or(NotOfGraphError)
    }

    /// Returns [`Err`] containing the existing node's ID if the position is already occupied.
    pub fn create_node(
        &mut self,
        gate: Gate,
        position: IVec2,
    ) -> Result<&mut Node, AlreadyExistsError<&mut Node>> {
        let grid_pos = Self::world_to_grid(position);
        if let Some(&existing) = self.node_grid.get(&grid_pos) {
            logln!(
                Info,
                "node at {} already exists: {}",
                PositionRef(position),
                NodeRef(existing),
            );
            Err(AlreadyExistsError(&mut self[&existing]))
        } else {
            let id = NodeId::next().fatal("out of IDs");
            self.node_grid.insert(grid_pos, id);
            let node = self
                .nodes
                .entry(id)
                .insert_entry(Node::new(id, gate, position, false))
                .into_mut();
            self.is_eval_order_dirty = true;

            logln!(
                Info,
                "create {} node {} at {}",
                GateRef(gate),
                NodeRef(*node.id()),
                PositionRef(position),
            );
            Ok(node)
        }
    }

    pub fn translate_node(
        &mut self,
        id: &NodeId,
        new_position: IVec2,
    ) -> Result<(), NotOfGraphError> {
        self.nodes
            .get_mut(id)
            .map(|node| {
                let old_grid_position = Self::world_to_grid(node.position);
                let new_grid_position = Self::world_to_grid(new_position);
                if old_grid_position != new_grid_position {
                    _ = self
                        .node_grid
                        .remove(&old_grid_position)
                        .filter(|x| x == id)
                        .ok_or(NodeGridDesyncError)
                        .error_unwrap();
                    self.node_grid.insert(new_grid_position, *id);

                    let old_position = std::mem::replace(&mut node.position, new_position);
                    logln!(
                        Info,
                        "move node {} from {} to {}",
                        NodeRef(*id),
                        PositionRef(old_position),
                        PositionRef(new_position),
                    );
                }
            })
            .ok_or(NotOfGraphError)
    }

    pub fn destroy_node(&mut self, id: &NodeId, soft: bool) -> Result<Node, NotOfGraphError> {
        self.nodes
            .remove(id)
            .inspect(|node| {
                _ = self
                    .node_grid
                    .remove(&Self::world_to_grid(node.position))
                    .filter(|x| x == id)
                    .ok_or(NodeGridDesyncError)
                    .error_unwrap();
                if soft {
                    logln!(Error, "not yet implemented");
                } else {
                    self.wires
                        .retain(|_, wire| &wire.src != id && &wire.dst != id);
                }
                self.is_eval_order_dirty = true;
            })
            .ok_or(NotOfGraphError)
            .ok_info(format_args!("destroy node {}", NodeRef(*id)))
    }

    /// # Errors
    /// Returns [`Err`] containing the existing wire's ID if there is already a wire from `src` to `dst`.
    ///
    /// # Panics
    /// This method may panic if `src == dst`
    pub fn create_wire(
        &mut self,
        elbow: Elbow,
        src: NodeId,
        dst: NodeId,
    ) -> Result<&mut Wire, AlreadyExistsError<&mut Wire>> {
        assert_ne!(src, dst, "cannot wire a node directly to itself");
        if let Some((&existing, _)) = self
            .wires
            .iter()
            .find(|(_, wire)| wire.src == src && wire.dst == dst)
        {
            Err(AlreadyExistsError(&mut self[&existing]))
        } else {
            let id = WireId::next().fatal("out of IDs");
            let wire = self
                .wires
                .entry(id)
                .insert_entry(Wire::new(id, elbow, src, dst))
                .into_mut();
            self.is_eval_order_dirty = true;
            logln!(
                Info,
                "create wire {} from {} to {}",
                WireRef(*wire.id()),
                NodeRef(src),
                NodeRef(dst),
            );
            Ok(wire)
        }
    }

    /// Returns [`None`] if `id` is not a wire in this graph.
    #[inline]
    pub fn destroy_wire(&mut self, id: &WireId) -> Result<Wire, NotOfGraphError> {
        self.wires
            .remove(id)
            .inspect(|_| {
                self.is_eval_order_dirty = true;
            })
            .ok_or(NotOfGraphError)
    }

    #[inline]
    pub fn nodes_iter(&self) -> Values<'_, NodeId, Node> {
        self.nodes.values()
    }

    #[inline]
    pub fn wires_iter(&self) -> Values<'_, WireId, Wire> {
        self.wires.values()
    }

    #[inline]
    pub fn inputs_to<'a>(&'a self, node: &NodeId) -> impl Iterator<Item = &'a Wire> {
        self.wires.values().filter(move |wire| &wire.dst == node)
    }

    #[inline]
    pub fn outputs_from<'a>(&'a self, node: &NodeId) -> impl Iterator<Item = &'a Wire> {
        self.wires.values().filter(move |wire| &wire.src == node)
    }

    #[inline]
    pub fn wires_of<'a>(&'a self, node: &NodeId) -> impl Iterator<Item = (&'a Wire, Flow)> {
        self.wires
            .values()
            .filter_map(move |wire| match (&wire.src == node, &wire.dst == node) {
                (true, true) => Some((wire, Flow::Loop)),
                (true, false) => Some((wire, Flow::Output)),
                (false, true) => Some((wire, Flow::Input)),
                (false, false) => None,
            })
    }

    /// Not all nodes have names
    #[inline]
    pub fn name<'a>(&'a self, node: &NodeId) -> Option<&'a str> {
        self.named_nodes.get(node).map(String::as_str)
    }

    /// Returns [`None`] if the start or end of the wire is not in the graph.
    #[inline]
    pub fn get_wire_nodes<'a>(
        &'a self,
        wire: &Wire,
    ) -> Result<(&'a Node, &'a Node), NotOfGraphError> {
        self.nodes
            .get(&wire.src)
            .zip(self.nodes.get(&wire.dst))
            .ok_or(NotOfGraphError)
    }

    /// Returns [`None`] if the start or end of the wire is not in the graph.
    #[inline]
    pub fn get_wire_nodes_mut<'a: 'b, 'b>(
        &'a mut self,
        wire: &'b Wire,
    ) -> Result<(&'a mut Node, &'a mut Node), NotOfGraphError> {
        let [src, dst] = self.nodes.get_disjoint_mut([&wire.src, &wire.dst]);
        src.zip(dst).ok_or(NotOfGraphError)
    }

    #[inline]
    pub fn is_inputless(&self, node: &NodeId) -> bool {
        self.inputs_to(node).next().is_none()
    }

    #[inline]
    pub fn is_outputless(&self, node: &NodeId) -> bool {
        self.outputs_from(node).next().is_none()
    }

    #[inline]
    pub fn inputless_nodes(
        &self,
    ) -> std::iter::Filter<std::iter::Copied<Keys<'_, NodeId, Node>>, impl FnMut(&NodeId) -> bool>
    {
        let input_taking = FxHashSet::from_iter(self.wires.values().map(|wire| wire.dst));
        self.nodes
            .keys()
            .copied()
            .filter(move |node| !input_taking.contains(node))
    }

    #[inline]
    pub fn outputless_nodes(
        &self,
    ) -> std::iter::Filter<std::iter::Copied<Keys<'_, NodeId, Node>>, impl FnMut(&NodeId) -> bool>
    {
        let output_giving = FxHashSet::from_iter(self.wires.values().map(|wire| wire.src));
        self.nodes
            .keys()
            .copied()
            .filter(move |node| !output_giving.contains(node))
    }

    #[inline]
    pub fn adjacent(
        &self,
    ) -> (
        FxHashMap<NodeId, FxHashSet<NodeId>>,
        FxHashMap<NodeId, FxHashSet<NodeId>>,
    ) {
        let mut inputs = FxHashMap::<_, FxHashSet<_>>::default();
        let mut outputs = FxHashMap::<_, FxHashSet<_>>::default();
        for wire in self.wires.values() {
            inputs.entry(wire.dst).or_default().insert(wire.src);
            outputs.entry(wire.src).or_default().insert(wire.dst);
        }
        (inputs, outputs)
    }

    #[inline]
    pub fn adjacent_out(&self) -> FxHashMap<NodeId, FxHashSet<NodeId>> {
        let mut outputs = FxHashMap::<_, FxHashSet<_>>::default();
        for wire in self.wires.values() {
            outputs.entry(wire.src).or_default().insert(wire.dst);
        }
        outputs
    }

    #[inline]
    pub fn adjacent_in(&self) -> FxHashMap<NodeId, FxHashSet<NodeId>> {
        let mut inputs = FxHashMap::<_, FxHashSet<_>>::default();
        for wire in self.wires.values() {
            inputs.entry(wire.dst).or_default().insert(wire.src);
        }
        inputs
    }

    #[inline]
    pub const fn is_eval_order_dirty(&self) -> bool {
        self.is_eval_order_dirty
    }

    pub fn refresh_eval_order(&mut self) {
        if self.is_eval_order_dirty {
            dbg_ord_prinln!("refreshing...");
            self.eval_order_dict.clear();
            self.eval_order.clear();
            self.eval_order.reserve(self.nodes.len());

            let (adj_in, adj_out) = self.adjacent();
            dbg_ord_prinln!("  adj_in: {adj_in:?}");
            dbg_ord_prinln!("  adj_out: {adj_out:?}");
            let inputless: FxHashSet<_> = self.inputless_nodes().collect();
            dbg_ord_prinln!("  inputless: {inputless:?}");
            let mut queue: VecDeque<_> = self.outputless_nodes().map(|x| (0, x)).collect();
            dbg_ord_prinln!("  queue (outputless): {queue:?}");
            let mut discovered: FxHashSet<_> = queue.iter().map(|(_, x)| x).copied().collect();
            dbg_ord_prinln!("  discovered: {discovered:?}");
            let all_nodes: FxHashSet<_> = self.nodes.keys().copied().collect();

            loop {
                dbg_ord_prinln!("  loop");
                dbg_ord_prinln!("    bfs...");
                dbg_ord_prinln!("      queue: {queue:?}");
                // traverse with BFS starting at the end.
                while let Some((n, v)) = queue
                    .pop_front()
                    .inspect(dbg_ord_prinln!(v => "      v: {v:?}"))
                {
                    queue.extend(
                        adj_in
                            .get(&v)
                            .into_iter()
                            .flatten()
                            .copied()
                            .filter(|&w| discovered.insert(w))
                            .map(|w| (n + 1, w))
                            .inspect(dbg_ord_prinln!(w => "        w: {w:?}")),
                    );
                    dbg_ord_prinln!("      queue: {queue:?}");
                    self.eval_order_dict.insert(v, (n, self.eval_order.len()));
                    self.eval_order.push(v);
                }

                // some subgraphs may end in a cycle. find furthest nodes with DFS and use those as endpoints.
                dbg_ord_prinln!("    dfs...");
                let root_discovered = discovered.clone();
                for root in inputless.difference(&root_discovered).copied() {
                    let mut dfs_discovered = root_discovered.clone();
                    let mut stack = vec![root];
                    dbg_ord_prinln!("      stack (undiscovered inputless): {stack:?}");
                    if !stack.is_empty() {
                        while let Some(v) =
                            stack.pop().inspect(dbg_ord_prinln!(v => "      v: {v:?}"))
                        {
                            if dfs_discovered.insert(v) {
                                stack.extend(
                                    adj_out
                                        .get(&v)
                                        .into_iter()
                                        .flatten()
                                        .copied()
                                        .inspect(dbg_ord_prinln!(w => "        w: {w:?}")),
                                );
                                dbg_ord_prinln!("      stack: {stack:?}");
                                let all_discovered = adj_out.get(&v).is_some_and(|ws| {
                                    ws.difference(&dfs_discovered).next().is_none()
                                });
                                if all_discovered {
                                    dbg_ord_prinln!("      all w are already discovered; dead end");
                                    // end of path
                                    discovered.insert(v);
                                    queue.push_back((0, v));
                                    dbg_ord_prinln!("      queue: {queue:?}");
                                }
                            }
                        }
                    }
                }

                // some subgraphs both start and end in a cycle. choose an endpoint arbitrarily.
                if queue.is_empty() {
                    dbg_ord_prinln!("    arbitrary...");
                    if let Some(arbitrary) = all_nodes
                        .difference(&discovered)
                        .next()
                        .copied()
                        .inspect(dbg_ord_prinln!(v => "      v: {v:?}"))
                    {
                        discovered.insert(arbitrary);
                        queue.push_back((0, arbitrary));
                        dbg_ord_prinln!("      queue: {queue:?}");
                    } else {
                        dbg_ord_prinln!("  no nodes remain");
                        break;
                    }
                }
            }

            self.eval_order.reverse();
            self.is_eval_order_dirty = false;
            assert_eq!(
                self.eval_order.len(),
                self.nodes.len(),
                "every node should be visited by eval_order"
            );
        }
    }

    #[inline]
    pub const fn eval_order(&self) -> &[NodeId] {
        self.eval_order.as_slice()
    }

    #[inline]
    pub fn eval_order_of(&self, id: &NodeId) -> Result<(usize, usize), NotOfGraphError> {
        self.eval_order_dict.get(id).ok_or(NotOfGraphError).copied()
    }

    pub fn evaluate(&mut self) {
        assert!(
            !self.is_eval_order_dirty,
            "should not evaluate while evel order is dirty, remember to call refresh_eval_order"
        );
        assert_eq!(
            self.eval_order.len(),
            self.nodes.len(),
            "every node must be visited during eval; refresh_eval_order may need to be called"
        );
        let adj = self.adjacent_in();
        let mut input_buf = Vec::new();
        for id in &self.eval_order {
            input_buf.clear();
            input_buf.extend(adj.get(id).into_iter().flatten().map(|id| {
                self.nodes
                    .get(id)
                    .expect("all nodes in adj should be valid")
                    .state
            }));
            let node = self
                .nodes
                .get_mut(id)
                .expect("all nodes in eval_order should be valid");
            node.state = node.gate.evaluate(input_buf.iter().copied());
        }
    }
}

#[derive(Debug)]
pub struct GraphList {
    graphs: FxHashMap<GraphId, Arc<RwLock<Graph>>>,
}

impl std::ops::Deref for GraphList {
    type Target = FxHashMap<GraphId, Arc<RwLock<Graph>>>;

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
    pub fn new() -> Self {
        Self {
            graphs: FxHashMap::default(),
        }
    }

    #[inline]
    pub fn create_graph(&mut self) -> &mut Arc<RwLock<Graph>> {
        let id = GraphId::next().fatal_unwrap();
        self.graphs
            .insert(id, Arc::new(RwLock::new(Graph::new(id))));
        self.graphs.get_mut(&id).expect("just inserted")
    }

    #[inline]
    pub fn get(&self, id: &GraphId) -> Option<&Arc<RwLock<Graph>>> {
        self.graphs.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::node::GateInstance;

    fn gen_graph(
        id: GraphId,
        nodes: impl IntoIterator<Item = (NodeId, Gate)>,
        wires: impl IntoIterator<Item = (WireId, (NodeId, NodeId))>,
    ) -> Graph {
        let mut next_node_id = NodeId(0);
        let mut next_wire_id = WireId(0);
        let nodes = nodes
            .into_iter()
            .map(|(id, gate)| {
                next_node_id.0 = id.0.max(next_node_id.0);
                (id, Node::new(id, gate, IVec2::default(), false))
            })
            .collect();
        let wires = wires
            .into_iter()
            .map(|(id, (src, dst))| {
                next_wire_id.0 = id.0.max(next_wire_id.0);
                (id, Wire::new(id, Elbow::default(), src, dst))
            })
            .collect();
        _ = next_node_id.step();
        _ = next_wire_id.step();
        Graph {
            id,
            nodes,
            wires,
            node_grid: FxHashMap::default(),
            named_nodes: FxHashMap::default(),
            eval_order: Vec::new(),
            eval_order_dict: FxHashMap::default(),
            is_eval_order_dirty: true,
        }
    }

    /// must contain every node, but order does not matter
    struct Unordered<T>(FxHashSet<T>);

    impl<T: std::fmt::Debug> std::fmt::Debug for Unordered<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_set().entries(&self.0).finish()
        }
    }

    impl<T> FromIterator<T> for Unordered<T>
    where
        FxHashSet<T>: FromIterator<T>,
    {
        fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
            Self(FxHashSet::from_iter(iter))
        }
    }

    impl<T> PartialEq<[T]> for Unordered<T>
    where
        T: Copy,
        FxHashSet<T>: FromIterator<T> + PartialEq,
    {
        fn eq(&self, other: &[T]) -> bool {
            self.0 == other.iter().copied().collect::<FxHashSet<T>>()
        }
    }

    impl<T> Unordered<T> {
        #[inline]
        pub fn len(&self) -> usize {
            self.0.len()
        }
    }

    /// must be in order, but can start with any subset
    struct RingOrder<T>(VecDeque<Unordered<T>>);

    impl<T: std::fmt::Debug> std::fmt::Debug for RingOrder<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("(")?;
            let mut it = self.0.iter();
            if let Some(set) = it.next() {
                set.fmt(f)?;
                for set in it {
                    f.write_str(", ")?;
                    set.fmt(f)?;
                }
            }
            f.write_str(")")
        }
    }

    impl<T> FromIterator<Unordered<T>> for RingOrder<T>
    where
        VecDeque<Unordered<T>>: FromIterator<Unordered<T>>,
    {
        #[inline]
        fn from_iter<I: IntoIterator<Item = Unordered<T>>>(iter: I) -> Self {
            Self(VecDeque::from_iter(iter))
        }
    }

    impl<T> PartialEq<[T]> for RingOrder<T>
    where
        Unordered<T>: PartialEq<[T]>,
    {
        fn eq(&self, other: &[T]) -> bool {
            (0..self.0.len()).any(|i| {
                let mut slice = other;
                self.0
                    .iter()
                    .cycle()
                    .skip(i)
                    .take(self.0.len())
                    .all(|set| set == slice.split_off(..set.len()).unwrap())
            })
        }
    }

    impl<T> RingOrder<T> {
        #[inline]
        pub fn len(&self) -> usize {
            self.0.iter().map(|set| set.len()).sum::<usize>()
        }
    }

    /// Must be in order with exact start and end
    struct ExactOrder<T>(Vec<RingOrder<T>>);

    impl<T: std::fmt::Debug> std::fmt::Debug for ExactOrder<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_list().entries(&self.0).finish()
        }
    }

    impl<T> FromIterator<RingOrder<T>> for ExactOrder<T>
    where
        Vec<RingOrder<T>>: FromIterator<RingOrder<T>>,
    {
        fn from_iter<I: IntoIterator<Item = RingOrder<T>>>(iter: I) -> Self {
            Self(Vec::from_iter(iter))
        }
    }

    impl<T> PartialEq<[T]> for ExactOrder<T>
    where
        RingOrder<T>: PartialEq<[T]>,
    {
        fn eq(&self, mut other: &[T]) -> bool {
            self.0.iter().all(|series| {
                other
                    .split_off(..series.len())
                    .is_some_and(|slice| series == slice)
            })
        }
    }

    macro_rules! test_graph {
        (
            // nodes
            $({$gate:expr} $id:ident;)*
            // wires
            $($src:ident -> $dst:ident;)*
            // expected eval order
            [$(($({$($ord:ident),*}),*)),*];
            // optional message
            $(($($ord_args:tt)*))?
            $(
                $(|$graph:ident|)?
                $changes:block
                $(
                    // expected state
                    -> {$($eval_id:ident: $value:expr),*$(,)?}
                    // optional message
                    $(($($eval_args:tt)*))?
                )?
            )*
        ) => {
            {
                use Gate::*;
                let mut next_node_id = NodeId(0);
                let mut next_wire_id = WireId(0);
                let [$($id),*] = std::array::from_fn(|_| next_node_id.step().unwrap());
                let mut g = gen_graph(
                    GraphId(0),
                    [$(($id, $gate)),*],
                    [$(($src, $dst)),*].map(|x| (next_wire_id.step().unwrap(), x)),
                );
                // order
                g.refresh_eval_order();
                assert_eq!(
                    &ExactOrder::from_iter([$(
                        RingOrder::from_iter([$(
                            Unordered::from_iter([$(
                                $ord
                            ),*])
                        ),*])
                    ),*]),
                    g.eval_order.as_slice(),
                    $($($ord_args)*)?
                );
                // state
                $(
                    $(let $graph = &mut g;)?
                    $changes
                    g.evaluate();
                    $(assert_eq!(
                        FxHashMap::from_iter([$(($eval_id, $value)),*]),
                        g.nodes
                            .iter()
                            .map(|(id, node)| (*id, node.state))
                            .collect::<FxHashMap<_, _>>(),
                        $($($eval_args)*)?
                    );)?
                )*
                // return
                (g, [$($id),*])
            }
        };
    }

    #[test]
    fn test_one_to_one() {
        test_graph! {
            {Or} a;
            {Or} b;
            {Or} c;
            {Or} d;
            a -> b;
            b -> c;
            c -> d;
            [({a}), ({b}), ({c}), ({d})];
            ("{d} relies on {c} which relies on {b} which relies on {a}, forcing a strict order.")
        };
    }

    #[test]
    fn test_many_to_many() {
        test_graph! {
            {Nor} a;
            {Or} b;
            {Or} c;
            {Or} d;
            {Or} e;
            {Or} f;
            a -> b;
            b -> d;
            c -> d;
            d -> e;
            d -> f;
            [({a}), ({b, c}), ({d}), ({e, f})];
            ("{e} and {f} rely on {d}, forcing {d} to come before both. \
            {d} relies on both {b} and {c}, forcing both to come before it. \
            {b} relies on {a}, forcing it to come before it. \
            {b} and {c} do not rely on each other and {e} and {f} do not rely on each other. \
            Each pair can theoretically be evaluated in parallel, so their order is free to be \
            rearranged by the implementation so long as they are ordered after all their inputs \
            are met and before any outputs need them.")
        };
    }

    #[test]
    fn test_cyclic() {
        test_graph! {
            {Or} a;
            {Or} b;
            {Or} c;
            a -> b;
            b -> c;
            c -> a;
            [({a}, {b}, {c})];
            ("{c} relies on {b} and {b} relies on {a}, but {a} also relies on {c}; the starting/ending \
            nodes don't matter, so long {b} does not come after {c} without {a} between them.")
        };
    }

    #[test]
    fn test_cyclic_with_input() {
        test_graph! {
            {Or} a;
            {Or} b;
            {Or} c;
            {Or} d;
            a -> b;
            b -> c;
            c -> d;
            d -> b;
            [({a}), ({b}), ({c}), ({d})];
            ("{b} relies on {a}, forcing {a} to come first. since {b}, {c}, and {d} form a cycle, their \
            order is fixed but their endpoint is not. however, {b} requiring {a} to be up to date forces \
            all other nodes relying on {b} to yield until {b} is updated by {a}, making {b} the \
            entrypoint of the cycle.")
        };
    }

    #[test]
    fn test_cyclic_with_output() {
        test_graph! {
            {Or} a;
            {Or} b;
            {Or} c;
            {Or} d;
            b -> c;
            c -> d;
            d -> b;
            d -> a;
            [({b}), ({c}), ({d}), ({a})];
            ("{a} relies on {d}, forcing {d} to come before it. since {b}, {c}, and {d} form a cycle, their \
            order is fixed but their endpoint is not. however, {a} requiring {d} to be up to date forces {d} \
            to act as an endpoint for the cycle, giving it a strict order.")
        };
    }

    #[test]
    fn test_rs_nor_latch() {
        test_graph! {
            {Or} r;
            {Or} s;
            {Nor} q;
            {Nor} q_;
            r -> q;
            s -> q_;
            q -> q_;
            q_ -> q;
            [({r, s}), ({q, q_})];
            ("{q} and {q_} form a cycle, so their start/end is arbitrary. they each rely on \
            {r} and {s} respectively however, so {r} and {s} must come before them.")

            {}

            |g| {
                g.node_mut(&s).unwrap().gate = GateInstance::Nor;
            }
            {} -> {
                r: false,
                s: true,
                q: true,
                q_: false,
            }
            ("1: setting {s} should set {q} and unset {q_}, possibly taking an extra tick depending on the cycle order")

            |g| {
                g.node_mut(&s).unwrap().gate = GateInstance::Or;
            }
            {} -> {
                r: false,
                s: false,
                q: true,
                q_: false,
            }
            ("1: should remain latched after inputs are turned back off")

            |g| {
                g.node_mut(&r).unwrap().gate = GateInstance::Nor;
            }
            {} -> {
                r: true,
                s: false,
                q: false,
                q_: true,
            }
            ("2: setting {r} should set {q_} and unset {q}, possibly taking an extra tick depending on the cycle order")

            |g| {
                g.node_mut(&r).unwrap().gate = GateInstance::Or;
            }
            {} -> {
                r: false,
                s: false,
                q: false,
                q_: true,
            }
            ("2: should remain latched after inputs are turned back off")
        };
    }
}
