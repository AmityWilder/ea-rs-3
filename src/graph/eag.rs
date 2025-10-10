use crate::{
    graph::{
        Graph, GraphId, GraphList,
        node::{Node, NodeId},
        wire::{Wire, WireId},
    },
    ivec::IVec2,
};
use rustc_hash::{FxBuildHasher, FxHashMap};
use serde::{
    de::{Deserialize, Deserializer, Visitor},
    ser::{Serialize, SerializeSeq, SerializeStruct, Serializer},
};
use serde_derive::Deserialize;
use std::sync::{Arc, RwLock};

impl Serialize for Graph {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        struct Nodes<'a>(&'a FxHashMap<NodeId, Node>);

        impl Serialize for Nodes<'_> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
                for node in self.0.values() {
                    seq.serialize_element(&(
                        node.gate,
                        (node.position.x, node.position.y),
                        node.state,
                    ))?;
                }
                seq.end()
            }
        }

        struct Wires<'a>(&'a FxHashMap<WireId, Wire>, FxHashMap<NodeId, usize>);

        impl Serialize for Wires<'_> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
                for wire in self.0.values() {
                    seq.serialize_element(&(
                        wire.elbow,
                        self.1
                            .get(&wire.src)
                            .expect("wire src should always be valid"),
                        self.1
                            .get(&wire.dst)
                            .expect("wire dst should always be valid"),
                    ))?;
                }
                seq.end()
            }
        }

        let mut graph = serializer.serialize_struct("Graph", 2)?;
        graph.serialize_field("nodes", &Nodes(&self.nodes))?;
        graph.serialize_field(
            "wires",
            &Wires(
                &self.wires,
                self.nodes
                    .keys()
                    .enumerate()
                    .map(|(n, id)| (*id, n))
                    .collect(),
            ),
        )?;
        graph.end()
    }
}

#[derive(Debug)]
struct Nodes(FxHashMap<NodeId, Node>, NodeId);

impl<'de> Deserialize<'de> for Nodes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        struct NodesVisitor;

        impl<'de> Visitor<'de> for NodesVisitor {
            type Value = Nodes;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of struct Node")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: ::serde::de::SeqAccess<'de>,
            {
                let mut value = seq
                    .size_hint()
                    .map(|n| FxHashMap::with_capacity_and_hasher(n, FxBuildHasher))
                    .unwrap_or_default();

                let mut next_node_id = NodeId(0);
                while let Some((gate, (x, y), state)) = seq.next_element()? {
                    let id = next_node_id.step().unwrap();
                    value.insert(id, Node::new(id, gate, IVec2 { x, y }, state));
                }
                Ok(Nodes(value, next_node_id))
            }
        }

        deserializer.deserialize_seq(NodesVisitor)
    }
}

#[derive(Debug)]
struct Wires(FxHashMap<WireId, Wire>, WireId);

impl<'de> Deserialize<'de> for Wires {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        struct WiresVisitor;

        impl<'de> Visitor<'de> for WiresVisitor {
            type Value = Wires;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of struct Wire")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: ::serde::de::SeqAccess<'de>,
            {
                let mut value = seq
                    .size_hint()
                    .map(|n| FxHashMap::with_capacity_and_hasher(n, FxBuildHasher))
                    .unwrap_or_default();

                let mut next_wire_id = WireId(0);
                while let Some((elbow, src, dst)) = seq.next_element()? {
                    let id = next_wire_id.step().unwrap();
                    value.insert(id, Wire::new(id, elbow, NodeId(src), NodeId(dst)));
                }
                Ok(Wires(value, next_wire_id))
            }
        }

        deserializer.deserialize_seq(WiresVisitor)
    }
}

#[derive(Debug, Deserialize)]
pub struct GraphTemplate {
    nodes: Nodes,
    wires: Wires,
}

impl From<GraphTemplate> for Graph {
    fn from(
        GraphTemplate {
            nodes: Nodes(nodes, next_node_id),
            wires: Wires(wires, next_wire_id),
        }: GraphTemplate,
    ) -> Self {
        Self {
            next_node_id,
            next_wire_id,
            id: GraphId(0),
            node_grid: nodes
                .values()
                .map(|node| (node.position, *node.id()))
                .collect(),
            nodes,
            wires,
            eval_order: Vec::default(),
            is_eval_order_dirty: true,
        }
    }
}

impl Serialize for GraphList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.graphs.len()))?;
        for graph in &self.graphs {
            seq.serialize_element(&*graph.read().unwrap())?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for GraphList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct GraphListVisitor;

        impl<'de> Visitor<'de> for GraphListVisitor {
            type Value = GraphList;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("struct GraphList")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut graphs = seq.size_hint().map(Vec::with_capacity).unwrap_or_default();
                let mut next_graph_id = GraphId(0);
                while let Some(mut value) = seq.next_element::<Graph>()? {
                    value.id = next_graph_id.step().unwrap();
                    graphs.push(Arc::new(RwLock::new(value)));
                }
                Ok(GraphList {
                    graphs,
                    next_graph_id,
                })
            }
        }

        deserializer.deserialize_seq(GraphListVisitor)
    }
}
