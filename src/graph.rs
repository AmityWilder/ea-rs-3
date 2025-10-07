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
    pub const fn step(&mut self) -> Option<Self> {
        const INVALID: GraphId = GraphId::INVALID;
        match *self {
            INVALID => None,
            id => {
                self.0 += 1;
                Some(id)
            }
        }
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
    eval_order: Vec<NodeId>,
    is_eval_order_dirty: bool,
}

type EvalOrder = std::iter::Rev<std::vec::IntoIter<NodeId>>;
type IOLessNodeIter<'a, F> =
    std::iter::Filter<std::iter::Copied<std::collections::hash_map::Keys<'a, NodeId, Node>>, F>;
type NodesIter<'a> = std::collections::hash_map::Values<'a, NodeId, Node>;
type WiresIter<'a> = std::collections::hash_map::Values<'a, WireId, Wire>;

impl Graph {
    pub fn new(id: GraphId) -> Self {
        Self {
            next_node_id: NodeId(0),
            next_wire_id: WireId(0),
            id,
            nodes: FxHashMap::default(),
            wires: FxHashMap::default(),
            node_grid: FxHashMap::default(),
            eval_order: Vec::new(),
            is_eval_order_dirty: false,
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

    /// Returns [`Err`] containing the existing node's ID if the position is already occupied.
    pub fn create_node(
        &mut self,
        gate: Gate,
        position: IVec2,
        console: &mut Console,
    ) -> Result<&mut Node, NodeId> {
        let id = self.next_node_id.step().expect("out of IDs");
        let grid_pos = Self::world_to_grid(position);
        if let Some(&existing) = self.node_grid.get(&grid_pos) {
            logln!(
                console,
                LogType::Info,
                "node at {} already exists: {}",
                PositionRef(position),
                NodeRef(self.id, existing),
            );
            Err(existing)
        } else {
            self.node_grid.insert(grid_pos, id);
            let node = self
                .nodes
                .entry(id)
                .insert_entry(Node::new(id, gate, position))
                .into_mut();
            self.is_eval_order_dirty = true;

            logln!(
                console,
                LogType::Info,
                "create {} node {} at {}",
                GateRef(gate.id()),
                NodeRef(self.id, *node.id()),
                PositionRef(position),
            );
            Ok(node)
        }
    }

    /// Returns [`None`] if `id` is not a node in this graph.
    pub fn translate_node(
        &mut self,
        id: &NodeId,
        new_position: IVec2,
        console: &mut Console,
    ) -> Option<()> {
        self.nodes.get_mut(id).map(|node| {
            let old_grid_position = Self::world_to_grid(node.position);
            let new_grid_position = Self::world_to_grid(new_position);
            if old_grid_position != new_grid_position {
                let id = self
                    .node_grid
                    .remove(&old_grid_position)
                    .filter(|x| x == id)
                    .expect(
                        "nodes should not be moved without updating their position in node_grid",
                    );
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
        })
    }

    /// Returns [`None`] if `id` is not a node in this graph.
    #[must_use]
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
            self.is_eval_order_dirty = true;
            logln!(
                console,
                LogType::Info,
                "destroy node {}",
                NodeRef(self.id, *id)
            );
        })
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
        console: &mut Console,
    ) -> Result<&mut Wire, WireId> {
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
                "wire from {} to {} already exists: wire {}",
                graph_ref.node(src),
                graph_ref.node(dst),
                graph_ref.wire(existing),
            );
            Err(existing)
        } else {
            let graph_ref = GraphRef(self.id);
            let id = self.next_wire_id.step().expect("out of IDs");
            let wire = self
                .wires
                .entry(id)
                .insert_entry(Wire::new(id, elbow, src, dst))
                .into_mut();
            self.is_eval_order_dirty = true;
            logln!(
                console,
                LogType::Info,
                "create wire {} from {} to {}",
                graph_ref.wire(*wire.id()),
                graph_ref.node(src),
                graph_ref.node(dst),
            );
            Ok(wire)
        }
    }

    /// Returns [`None`] if `id` is not a wire in this graph.
    #[must_use]
    #[inline]
    pub fn destroy_wire(&mut self, id: &WireId) -> Option<Wire> {
        self.wires.remove(id).inspect(|_| {
            self.is_eval_order_dirty = true;
        })
    }

    #[inline]
    pub fn nodes_iter(&self) -> NodesIter<'_> {
        self.nodes.values()
    }

    #[inline]
    pub fn wires_iter(&self) -> WiresIter<'_> {
        self.wires.values()
    }

    #[inline]
    pub fn wires_to<'a: 'b, 'b>(
        &'a self,
        node: &'b NodeId,
    ) -> impl Iterator<Item = (&'a WireId, &'a Wire)> {
        self.wires.iter().filter(move |(_, wire)| &wire.dst == node)
    }

    #[inline]
    pub fn wires_from<'a: 'b, 'b>(
        &'a self,
        node: &'b NodeId,
    ) -> impl Iterator<Item = (&'a WireId, &'a Wire)> {
        self.wires.iter().filter(move |(_, wire)| &wire.src == node)
    }

    #[inline]
    pub fn wires_of<'a: 'b, 'b>(
        &'a self,
        node: &'b NodeId,
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
    pub fn inputless_nodes(&self) -> IOLessNodeIter<'_, impl FnMut(&NodeId) -> bool> {
        let input_taking = FxHashSet::from_iter(self.wires.values().map(|wire| wire.dst));
        self.nodes
            .keys()
            .copied()
            .filter(move |node| !input_taking.contains(node))
    }

    #[inline]
    pub fn outputless_nodes(&self) -> IOLessNodeIter<'_, impl FnMut(&NodeId) -> bool> {
        let output_giving = FxHashSet::from_iter(self.wires.values().map(|wire| wire.src));
        self.nodes
            .keys()
            .copied()
            .filter(move |node| !output_giving.contains(node))
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

    #[inline]
    pub const fn is_eval_order_dirty(&self) -> bool {
        self.is_eval_order_dirty
    }

    pub fn refresh_eval_order(&mut self) {
        self.eval_order.clear();
        self.eval_order.reserve(self.nodes.len());
        let adj = self.adjacent_in();
        let mut queue: VecDeque<_> = self.outputless_nodes().collect();
        let mut discovered = FxHashSet::from_iter(queue.iter().copied());
        loop {
            // traverse with BFS starting at the end, inserting in reverse.
            while let Some(v) = queue.pop_front() {
                queue.extend(
                    adj.get(&v)
                        .into_iter()
                        .flatten()
                        .copied()
                        .filter(|&w| discovered.insert(w)),
                );
                self.eval_order.push(v);
            }
            // some subgraphs may end in a cycle. find furthest nodes with DFS and use those as endpoints.
            let mut stack: Vec<_> = self
                .inputless_nodes()
                .filter(|v| !discovered.contains(v))
                .collect();
            if !stack.is_empty() {
                while let Some(v) = stack.pop() {
                    if discovered.insert(v) {
                        stack.extend(adj.get(&v).into_iter().flatten().copied());
                        queue.push_back(v);
                    }
                }
            }
            // some subgraphs both start and end in a cycle. choose an endpoint arbitrarily.
            else if let Some(arbitrary) =
                self.nodes.keys().find(|v| !discovered.contains(v)).copied()
            {
                discovered.insert(arbitrary);
                queue.push_back(arbitrary);
            }
            // no nodes remain
            else {
                break;
            }
        }
        assert_eq!(
            self.eval_order.len(),
            self.nodes.len(),
            "every node should be visited by eval_order"
        );
        self.eval_order.reverse();
        self.is_eval_order_dirty = false;
    }

    #[inline]
    pub const fn eval_order(&self) -> &[NodeId] {
        self.eval_order.as_slice()
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
        self.graphs.push(Arc::new(RwLock::new(Graph::new(
            self.next_graph_id.step().expect("out of IDs"),
        ))));
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
                (id, Node::new(id, gate, IVec2::default()))
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
            next_node_id,
            next_wire_id,
            eval_order: Vec::new(),
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

    impl<T> ExactOrder<T> {
        pub fn len(&self) -> usize {
            self.0.iter().map(|series| series.len()).sum::<usize>()
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
            $($args:tt)*
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
                    $($args)*
                );
                g
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
            "{d} relies on {c} which relies on {b} which relies on {a}, forcing a strict order"
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
            "{e} and {f} rely on {d}, forcing {d} to come before both. \
            {d} relies on both {b} and {c}, forcing both to come before it. \
            {b} relies on {a}, forcing it to come before it. \
            {b} and {c} do not rely on each other and {e} and {f} do not rely on each other. \
            Each pair can theoretically be evaluated in parallel, so their order is free to be \
            rearranged by the implementation so long as they are ordered after all their inputs \
            are met and before any outputs need them"
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
            "{c} relies on {b} and {b} relies on {a}, but {a} also relies on {c}; the starting/ending \
            nodes don't matter, so long {b} does not come after {c} without {a} between them"
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
            "{b} relies on {a}, forcing {a} to come first. since {b}, {c}, and {d} form a cycle, their \
            order is fixed but their endpoint is not. "
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
        };
    }
}
