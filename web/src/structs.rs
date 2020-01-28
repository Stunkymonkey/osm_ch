use serde::{Deserialize, Serialize};

use crate::constants::*;

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Way {
    pub source: NodeId,
    pub target: NodeId,
    pub weight: usize,
    pub contrated_previous: Option<EdgeId>,
    pub contrated_next: Option<EdgeId>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct Node {
    pub latitude: f32,
    pub longitude: f32,
    pub rank: Rank,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub enum OptimizeBy {
    Time,
    Distance,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GridBounds {
    pub lat_min: f32,
    pub lat_max: f32,
    pub lng_min: f32,
    pub lng_max: f32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FmiFile {
    pub nodes: Vec<Node>,
    pub up_offset: Vec<EdgeId>,
    pub down_offset: Vec<EdgeId>,
    pub down_index: Vec<EdgeId>,
    pub edges: Vec<Way>,
    pub grid_offset: Vec<GridId>,
    pub grid: Vec<NodeId>,
    pub grid_bounds: GridBounds,
    pub optimized_by: OptimizeBy,
}
