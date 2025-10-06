use super::{Graph, node::NodeId};
use raylib::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WireId(pub(super) u128);

impl std::fmt::Display for WireId {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "w{:x}", self.0)
    }
}

impl std::str::FromStr for WireId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.strip_prefix('w')
            .ok_or(())
            .and_then(|x| u128::from_str_radix(x, 16).map_err(|_| ()))
            .map(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Elbow {
    Horizontal,
    DiagonalStart,
    Vertical,
    #[default]
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

    pub fn draw_immediate<D: RaylibDraw>(
        d: &mut D,
        start_pos: Vector2,
        end_pos: Vector2,
        elbow: Elbow,
        color: Color,
    ) {
        let elbow_pos = elbow.calculate(start_pos, end_pos);
        d.draw_line_strip(&[start_pos, elbow_pos, end_pos], color);
    }

    /// Returns [`None`] if wire is not valid for the graph
    #[must_use]
    pub fn draw<D: RaylibDraw>(
        &self,
        d: &mut D,
        graph: &Graph,
        offset: Vector2,
        color: Color,
    ) -> Option<()> {
        let (start, end) = graph.get_wire_nodes(self)?;
        Self::draw_immediate(
            d,
            start.position().as_vec2() + offset,
            end.position().as_vec2() + offset,
            self.elbow,
            color,
        );
        Some(())
    }
}
