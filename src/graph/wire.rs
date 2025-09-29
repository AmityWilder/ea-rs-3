use super::{Graph, node::NodeId};
use crate::ivec::IVec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WireId(pub(super) u128);

impl std::fmt::Display for WireId {
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

#[derive(Debug, PartialEq, Eq)]
pub struct Wire {
    id: WireId,
    pub elbow_pos: IVec2,
    pub(super) src: NodeId,
    pub(super) dst: NodeId,
}

impl Wire {
    pub const fn new(id: WireId, elbow_pos: IVec2, src: NodeId, dst: NodeId) -> Self {
        Self {
            id,
            elbow_pos,
            src,
            dst,
        }
    }

    #[inline]
    pub const fn id(&self) -> &WireId {
        &self.id
    }

    pub const fn snap_to_elbow_pos(pos: IVec2, start_pos: IVec2, end_pos: IVec2) -> IVec2 {
        let x_delta = end_pos.x - start_pos.x;
        let y_delta = end_pos.y - start_pos.y;
        let x_dir = x_delta.signum();
        let y_dir = y_delta.signum();
        let x_dist = x_delta.abs();
        let y_dist = y_delta.abs();
        let IVec2 { x, y } = pos;
        let candidates = [
            // vertical
            {
                let pos = IVec2::new(start_pos.x, end_pos.y);
                (pos, (pos.x - x).pow(2) + (pos.y - y).pow(2))
            },
            // horizontal
            {
                let pos = IVec2::new(end_pos.x, start_pos.y);
                (pos, (pos.x - x).pow(2) + (pos.y - y).pow(2))
            },
            // diagonal start
            {
                let pos = if x_dist > y_dist {
                    IVec2::new(start_pos.x + x_dir * y_dist, start_pos.y + y_dir * y_dist)
                } else if x_dist < y_dist {
                    IVec2::new(start_pos.x + x_dir * x_dist, start_pos.y + y_dir * x_dist)
                } else {
                    start_pos
                };
                (pos, (pos.x - x).pow(2) + (pos.y - y).pow(2))
            },
            // diagonal end
            {
                let pos = if x_dist > y_dist {
                    IVec2::new(end_pos.x - x_dir * y_dist, end_pos.y - y_dir * y_dist)
                } else if x_dist < y_dist {
                    IVec2::new(end_pos.x - x_dir * x_dist, end_pos.y - y_dir * x_dist)
                } else {
                    start_pos
                };
                (pos, (pos.x - x).pow(2) + (pos.y - y).pow(2))
            },
        ];
        let mut i = 0;
        if candidates[1].1 < candidates[i].1 {
            i = 1
        }
        if candidates[2].1 < candidates[i].1 {
            i = 2
        }
        if candidates[3].1 < candidates[i].1 {
            i = 3
        }
        candidates[i].0
    }

    /// Returns an error if `graph` does not contain both `self.src` and `self.end`
    pub fn snap_elbow_pos(&mut self, graph: &Graph) -> Result<(), ()> {
        if let Some((src, dst)) = graph.get_wire_nodes(self) {
            self.elbow_pos = Self::snap_to_elbow_pos(self.elbow_pos, src.position, dst.position);
            Ok(())
        } else {
            Err(())
        }
    }
}
