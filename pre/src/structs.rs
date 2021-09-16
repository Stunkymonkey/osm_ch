use crate::OPTIMIZE_BY;
use crate::TRAVEL_TYPE;
use serde::Serialize;
use std::cmp::Ordering;
use std::convert::From;

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
    #[allow(dead_code)]
    Time,
    #[allow(dead_code)]
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
    #[serde(skip_serializing)]
    pub id: Option<EdgeId>,
    pub contrated_previous: Option<EdgeId>,
    pub contrated_next: Option<EdgeId>,
}

impl PartialOrd for Way {
    fn partial_cmp(&self, other: &Way) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Way {
    fn cmp(&self, other: &Way) -> Ordering {
        self.source
            .cmp(&other.source)
            .then(self.target.cmp(&other.target))
            .then(self.weight.cmp(&other.weight))
            .then(self.contrated_previous.cmp(&other.contrated_previous))
            .then(self.contrated_next.cmp(&other.contrated_next))
    }
}

impl Way {
    /// general constructor
    pub fn new(from: NodeId, to: NodeId, weight: Weight) -> Self {
        Way {
            source: from,
            target: to,
            weight,
            id: None,
            contrated_previous: None,
            contrated_next: None,
        }
    }

    #[allow(dead_code)]
    pub fn test(from: NodeId, to: NodeId, weight: Weight, id: NodeId) -> Self {
        Way {
            source: from,
            target: to,
            weight,
            id: Some(id),
            contrated_previous: None,
            contrated_next: None,
        }
    }
    #[allow(dead_code)]
    pub fn shortcut(
        from: NodeId,
        to: NodeId,
        weight: Weight,
        previous: NodeId,
        next: NodeId,
        id: NodeId,
    ) -> Self {
        Way {
            source: from,
            target: to,
            weight,
            id: Some(id),
            contrated_previous: Some(previous),
            contrated_next: Some(next),
        }
    }
}

impl From<OsmWay> for Way {
    fn from(full_edge: OsmWay) -> Self {
        let mut speed: usize = match TRAVEL_TYPE {
            TravelType::Car => full_edge.speed,
            TravelType::CarBicycle => full_edge.speed,
            TravelType::Bicycle if full_edge.speed <= 20 => full_edge.speed,
            TravelType::Bicycle => 20,
            TravelType::BicyclePedestrian if full_edge.speed <= 20 => full_edge.speed,
            TravelType::BicyclePedestrian => 20,
            TravelType::Pedestrian => 7,
            TravelType::All => full_edge.speed,
            TravelType::Undefined => 1,
        };
        if speed == 0 {
            speed = 1;
        }
        let weight = match OPTIMIZE_BY {
            OptimizeBy::Distance => full_edge.distance,
            OptimizeBy::Time => full_edge.distance / speed,
        };
        Way::new(full_edge.source, full_edge.target, weight)
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Node {
    pub latitude: f32,
    pub longitude: f32,
    pub rank: Rank,
}

#[derive(Serialize)]
pub struct GridBounds {
    pub lat_min: f32,
    pub lat_max: f32,
    pub lng_min: f32,
    pub lng_max: f32,
}

#[derive(Serialize)]
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
