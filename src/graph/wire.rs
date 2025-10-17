use super::{
    Graph, NotOfGraphError,
    id::{NodeId, WireId},
    node::Node,
};
use raylib::prelude::*;
use serde_derive::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub enum Elbow {
    #[serde(rename = "-")]
    Horizontal,
    #[serde(rename = "\\")]
    DiagonalStart,
    #[serde(rename = "|")]
    Vertical,
    #[default]
    #[serde(rename = "/")]
    DiagonalEnd,
}

impl Elbow {
    pub const fn calculate(self, start_pos: Vector2, end_pos: Vector2) -> Vector2 {
        let x_delta = end_pos.x - start_pos.x;
        let y_delta = end_pos.y - start_pos.y;
        let x_dir = x_delta.signum();
        let y_dir = y_delta.signum();
        let x_dist = x_delta.abs();
        let y_dist = y_delta.abs();
        match self {
            Elbow::Horizontal => Vector2::new(end_pos.x, start_pos.y),
            Elbow::Vertical => Vector2::new(start_pos.x, end_pos.y),
            Elbow::DiagonalStart if x_dist > y_dist => {
                Vector2::new(start_pos.x + x_dir * y_dist, start_pos.y + y_dir * y_dist)
            }
            Elbow::DiagonalStart if x_dist < y_dist => {
                Vector2::new(start_pos.x + x_dir * x_dist, start_pos.y + y_dir * x_dist)
            }
            Elbow::DiagonalEnd if x_dist > y_dist => {
                Vector2::new(end_pos.x - x_dir * y_dist, end_pos.y - y_dir * y_dist)
            }
            Elbow::DiagonalEnd if x_dist < y_dist => {
                Vector2::new(end_pos.x - x_dir * x_dist, end_pos.y - y_dir * x_dist)
            }
            Elbow::DiagonalStart => start_pos,
            Elbow::DiagonalEnd => end_pos,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Flow {
    Input = 0b01,
    Output = 0b10,
    Loop = 0b11,
}

impl Flow {
    #[inline]
    pub const fn is_input(self) -> bool {
        ((self as u8) & 1) != 0
    }

    #[inline]
    pub const fn is_output(self) -> bool {
        ((self as u8) & 2) != 0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Wire {
    id: WireId,
    pub elbow: Elbow,
    pub(super) src: NodeId,
    pub(super) dst: NodeId,
}

impl Wire {
    pub const fn new(id: WireId, elbow: Elbow, src: NodeId, dst: NodeId) -> Self {
        Self {
            id,
            elbow,
            src,
            dst,
        }
    }

    #[inline]
    pub const fn id(&self) -> &WireId {
        &self.id
    }

    #[inline]
    pub const fn src(&self) -> &NodeId {
        &self.src
    }

    #[inline]
    pub const fn dst(&self) -> &NodeId {
        &self.dst
    }

    #[inline]
    pub fn state(&self, graph: &Graph) -> Result<bool, NotOfGraphError> {
        if graph.wires.contains_key(&self.id) {
            Ok(graph
                .node(&self.src)
                .expect("invariant broken: no wire should ever have an invalid/out-of-date input or output node")
                .state)
        } else {
            Err(NotOfGraphError)
        }
    }

    #[inline]
    pub fn nodes<'a>(&self, graph: &'a Graph) -> Result<(&'a Node, &'a Node), NotOfGraphError> {
        graph
            .nodes
            .get(&self.src)
            .zip(graph.nodes.get(&self.dst))
            .ok_or(NotOfGraphError)
    }

    #[inline]
    pub fn nodes_mut<'a: 'b, 'b>(
        &'b mut self,
        graph: &'a mut Graph,
    ) -> Result<(&'a mut Node, &'a mut Node), NotOfGraphError> {
        let [src, dst] = graph.nodes.get_disjoint_mut([&self.src, &self.dst]);
        src.zip(dst).ok_or(NotOfGraphError)
    }

    pub fn draw_immediate<D: RaylibDraw>(
        d: &mut D,
        start_pos: Vector2,
        end_pos: Vector2,
        elbow: Elbow,
        color: Color,
    ) {
        let elbow_pos = elbow.calculate(start_pos, end_pos);
        d.draw_line_strip(&[start_pos, elbow_pos, end_pos], color);
        // let [arrow_pos1, arrow_pos2] = {
        //     let dir = (elbow_pos - start_pos).normalized();
        //     [
        //         elbow_pos + dir.rotated(3.5 * std::f32::consts::FRAC_PI_4) * f32::from(crate::GRID_SIZE),
        //         elbow_pos + dir.rotated(3.5 * -std::f32::consts::FRAC_PI_4) * f32::from(crate::GRID_SIZE),
        //     ]
        // };
        // d.draw_line_strip(&[arrow_pos1, elbow_pos, arrow_pos2], color);
    }

    /// Returns [`None`] if wire is not valid for the graph
    pub fn draw<D: RaylibDraw>(
        &self,
        d: &mut D,
        graph: &Graph,
        offset: Vector2,
        color: Color,
    ) -> Result<(), NotOfGraphError> {
        let (start, end) = graph.get_wire_nodes(self)?;
        Self::draw_immediate(
            d,
            start.position().as_vec2() + offset,
            end.position().as_vec2() + offset,
            self.elbow,
            color,
        );
        Ok(())
    }
}
