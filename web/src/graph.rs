// based on https://rosettacode.org/wiki/Dijkstra%27s_algorithm#Rust
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::usize;

use Node;
use Way;

#[derive(Clone)]
pub struct Graph {
    nodes: Vec<Node>,
    ways: Vec<Way>,
    offset: Vec<usize>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct State {
    node: usize,
    cost: usize,
}
// Manually implement Ord so we get a min-heap instead of a max-heap
impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}
impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Graph {
    pub fn new(nodes: Vec<Node>, ways: Vec<Way>, offset: Vec<usize>) -> Self {
        Graph {
            nodes: nodes,
            ways: ways,
            offset: offset,
        }
    }

    /// returns closes point of given long & lat
    pub fn get_point_id(&self, lat: f32, long: f32) -> usize {
        let mut min_distance: f32 = std::f32::MAX;
        let mut min_distance_id: usize = 0;

        for i in 0..self.nodes.len() {
            let distance =
                calc_distance(lat, long, self.nodes[i].latitude, self.nodes[i].longitude);
            if distance < min_distance {
                min_distance = distance;
                min_distance_id = i;
            }
        }
        return min_distance_id;
    }

    /// converts node ids to node-coordinates
    pub fn get_coordinates(&self, path: Vec<usize>) -> Vec<Node> {
        println!("now getting coords");
        return path.iter().map(|x| self.nodes[*x]).collect::<Vec<Node>>();
    }

    /// returns the edge weight from source to target
    fn get_edge_weight(
        &self,
        source: usize,
        target: usize,
        weight: usize,
        use_distance: bool,
    ) -> usize {
        let distance = calc_distance(
            self.nodes[source].latitude,
            self.nodes[source].longitude,
            self.nodes[target].latitude,
            self.nodes[target].longitude,
        );
        if use_distance {
            return distance.ceil() as usize;
        } else {
            return (distance / weight as f32) as usize;
        }
    }

    /// executes dijkstra
    pub fn find_path(
        &self,
        start: usize,
        end: usize,
        is_car: bool,
        use_distance: bool,
    ) -> Option<(Vec<usize>, usize)> {
        let mut dist = vec![(usize::MAX, None); self.nodes.len()];

        let mut heap = BinaryHeap::new();
        dist[start] = (0, None);
        heap.push(State {
            node: start,
            cost: 0,
        });

        while let Some(State { node, cost }) = heap.pop() {
            if node == end {
                let mut path = Vec::with_capacity(dist.len() / 2);
                let mut current_dist = dist[end];
                path.push(end);
                while let Some(prev) = current_dist.1 {
                    path.push(prev);
                    current_dist = dist[prev];
                }
                path.reverse();
                return Some((path, cost));
            }

            if cost > dist[node].0 {
                continue;
            }
            for edge in self.offset[node]..self.offset[node + 1] {
                let current_way: Way = self.ways[edge];
                // TODO check if should skip due to is_car
                let next = State {
                    node: current_way.target,
                    cost: cost
                        + self.get_edge_weight(
                            current_way.source,
                            current_way.target,
                            current_way.weight,
                            use_distance,
                        ),
                };
                if next.cost < dist[next.node].0 {
                    dist[next.node] = (next.cost, Some(node));
                    heap.push(next);
                }
            }
        }
        None
    }
}

/// get distance on earth surface using haversine formula
fn calc_distance(lat_1: f32, long_1: f32, lat_2: f32, long_2: f32) -> f32 {
    let r: f32 = 6371.0; // constant used for meters
    let d_lat: f32 = (lat_2 - lat_1).to_radians();
    let d_lon: f32 = (long_2 - long_1).to_radians();
    let lat1: f32 = (lat_1).to_radians();
    let lat2: f32 = (lat_2).to_radians();

    let a: f32 = ((d_lat / 2.0).sin()) * ((d_lat / 2.0).sin())
        + ((d_lon / 2.0).sin()) * ((d_lon / 2.0).sin()) * (lat1.cos()) * (lat2.cos());
    let c: f32 = 2.0 * ((a.sqrt()).atan2((1.0 - a).sqrt()));
    return r * c;
}
