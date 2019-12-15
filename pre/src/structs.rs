use serde::Serialize;

use constants::*;

#[derive(Serialize, Debug, Clone)]
pub struct Way {
    pub source: usize,
    pub target: usize,
    pub speed: usize,
    pub distance: usize,
    pub travel_type: usize,
}

#[derive(Serialize, Debug, Clone)]
pub struct Node {
    pub latitude: f32,
    pub longitude: f32,
}

#[derive(Eq, PartialEq, Debug)]
pub struct Shortcut {
    pub from: NodeId,
    pub to: NodeId,
    pub center_node: NodeId,
    pub weight: Weight,
}

#[derive(Serialize, Debug)]
pub struct FmiFile {
    pub nodes: Vec<Node>,
    pub edges: Vec<Way>,
    pub up_offset: Vec<EdgeId>,
    pub down_index: Vec<EdgeId>,
    pub down_offset: Vec<EdgeId>,
    pub grid_offset: Vec<usize>,
    pub grid: Vec<usize>,
}
