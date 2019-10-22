extern crate priority_queue;
use std::collections::{VecDeque, HashSet};
use Input;
use self::priority_queue::PriorityQueue;

pub fn dijkstra(start: u32, end: u32, input: &Input) -> Vec<u32> {
    if start == end { return Vec::new(); }
    let mut pq: PriorityQueue<u32, u32> = PriorityQueue::new();
    let mut explored = vec![false; input.offset_table.len()];
    pq.push(start, 0);
    // TODO: is a hack, if valid node id is u32::max value this is going to break
    let mut prev = vec!(u32::max_value(); input.offset_table.len());

    while !pq.is_empty() {
        let node_pq = pq.pop().unwrap();
        let node_id = node_pq.0;
        explored[node_id as usize] = true;
        let node_weight = node_pq.1;
        let mut neighbors = get_neighbors(node_id, &input);
        for neighbor in &neighbors {
            if neighbor == &end {
                prev[*neighbor as usize] = node_id;
                break;
            }
            let new_weight = node_weight + get_edge_weight(node_id, *neighbor, &input);
            match pq.get_priority(neighbor) {
                Some(old_weight) => {
                    if &new_weight < old_weight {
                        prev[*neighbor as usize] = node_id;
                        pq.change_priority(neighbor, new_weight);
                    }
                }
                None => {
                    if !explored[*neighbor as usize] {
                        prev[*neighbor as usize] = node_id;
                        pq.push(*neighbor, new_weight);
                    }
                }
            };
        }
    }
    return get_shortest_path(end, &prev);
}

pub fn get_point_id(lat: f32, long: f32, input: &Input) -> u32 {
    let mut min_distance: f32 = std::f32::MAX;
    let mut min_distance_id: u32 = 0;

    for i in 0..input.offset_table.len() - 1 {
        let distance = calc_distance(lat, long, input.latitude[i], input.longitude[i]);
        if distance < min_distance {
            min_distance = distance;
            min_distance_id = i as u32;
        }
    }
    return min_distance_id;
}

pub fn get_coordinates(id: u32, input: &Input) -> (f32, f32) {
    return (input.latitude[id as usize], input.longitude[id as usize]);
}

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

/// returns all neighbors
fn get_neighbors(node_id: u32, input: &Input) -> Vec<u32> {
    let first_edge = input.offset_table[node_id as usize];
    let last_edge = input.offset_table[(node_id + 1) as usize];
    let mut neighbors: Vec<u32> = Vec::new();
    for i in first_edge..last_edge {
            neighbors.push(input.target[i as usize]);
        }
    return neighbors;
}

/// returns the edge weight from source to target
fn get_edge_weight(source: u32, target: u32, input: &Input) -> u32 {
    let first_edge = input.offset_table[source as usize];
    let last_edge = input.offset_table[(source + 1) as usize];
    for i in first_edge..last_edge {
        if input.target[i as usize] == target {
            return input.weight[i as usize];
        }
    }
    return u32::max_value();
}

/// returns vector containing all node ids on the shortest path
fn get_shortest_path(target: u32, prev: &Vec<u32>) -> Vec<u32> {
    let mut shortest_path: VecDeque<u32> = VecDeque::new();
    let mut current_node = target;
    // TODO: breaks if node will have u32::max_value() as actual id
    if !prev[target as usize] == u32::max_value() {
        while !current_node == u32::max_value() {
            shortest_path.push_front(current_node);
            current_node = prev[current_node as usize]
        }
    }
    return Vec::from(shortest_path);
}