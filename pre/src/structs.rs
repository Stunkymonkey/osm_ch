use serde::Serialize;
use std::cmp::Ordering;

use crate::constants::*;

#[derive(Debug, PartialEq)]
pub enum TravelType {
    Car,
    CarBicycle,
    Bicycle,
    BicyclePedestrian,
    Pedestrian,
    All,
    Undefined,
}

#[derive(Debug, PartialEq, Serialize)]
pub enum OptimizeBy {
    Time,
    Distance,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct OsmWay {
    pub source: NodeId,
    pub target: NodeId,
    pub speed: usize,
    pub distance: usize,
}

#[derive(Serialize, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Way {
    pub source: NodeId,
    pub target: NodeId,
    pub weight: usize,
    // the result must be EdgeIds, but during execution this will be NodeIds
    pub contrated_previous: Option<EdgeId>,
    pub contrated_next: Option<EdgeId>,
}

impl PartialOrd for Way {
    fn partial_cmp(&self, other: &Way) -> Option<Ordering> {
        return Some(self.cmp(other));
    }
}

impl Ord for Way {
    fn cmp(&self, other: &Way) -> Ordering {
        return self
            .source
            .cmp(&other.source)
            .then(self.target.cmp(&other.target))
            .then(self.contrated_previous.cmp(&other.contrated_previous))
            .then(self.contrated_next.cmp(&other.contrated_next));
    }
}

impl Way {
    /// general constructor
    pub fn new(from: NodeId, to: NodeId, weight: Weight) -> Self {
        Way {
            source: from,
            target: to,
            weight: weight,
            contrated_previous: None,
            contrated_next: None,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Node {
    pub latitude: f32,
    pub longitude: f32,
    pub rank: Rank,
}

#[derive(Serialize, Debug)]
pub struct GridBounds {
    pub lat_min: f32,
    pub lat_max: f32,
    pub lng_min: f32,
    pub lng_max: f32,
}

#[derive(Serialize, Debug)]
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
