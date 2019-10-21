use std::collections::{VecDeque, HashSet};

use Input;

pub fn dijkstra(start: u32, end: u32, input: &Input) -> Vec<u32> {
    // all distances are at max_value
    let mut dist: Vec<u32> = vec!(u32::max_value(); input.offset_table.len());
    // TODO: is a hack, if node id is u32::max value this is going to break
    let mut prev = vec!(u32::max_value(); input.offset_table.len());
    let mut evaluated: HashSet<u32> = HashSet::new();
    // add all nodes to be evaluated
    let mut to_evaluate: HashSet<u32> = HashSet::new();
    to_evaluate.insert(start);
    // dist of start node is 0
    *dist.get_mut(start as usize).unwrap() = 0;

    loop {
        let min_dist_node = find_min_dist_node(&dist, &to_evaluate);
        if min_dist_node == end { break; }
        let neighbors = get_remaining_neighbors(min_dist_node, &evaluated, &input);
        // TODO: this blows up due to nodes having 3 million+ targets
        for neighbor in &neighbors {
            let edge_weight = get_edge_weight(min_dist_node, *neighbor, &input);
            if edge_weight == u32::max_value() {
                continue;
            }
            let mut temp = dist[min_dist_node as usize] + edge_weight;
            if temp < dist[*neighbor as usize] {
                dist[*neighbor as usize] = temp;
                prev[*neighbor as usize] = min_dist_node;
                if !evaluated.contains(&neighbor) {
                    to_evaluate.insert(*neighbor);
                }
            }
        }
        &to_evaluate.remove(&min_dist_node);
        &evaluated.insert(min_dist_node);
    }
    return get_shortest_path(end, &prev);
}

pub fn get_point_id(lat: f32, long: f32, input: &Input) -> u32 {
    let mut min_distance: f32 = std::f32::MAX;
    let mut min_distance_id: u32 = 0;

    for i in 1..input.offset_table.len() {
        // TODO: should not be there, offset_table seems to be bigger than lat/lng table
        if i >= input.latitude.len() {
            break;
        }
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

/// returns index of node left in node_ids with smallest dist
fn find_min_dist_node(dists: &Vec<u32>, node_ids: &HashSet<u32>) -> u32 {
    let mut min_dist: u32 = u32::max_value();
    let mut min_dist_node: u32 = 0;
    for node_id in node_ids {
        if &dists[*node_id as usize] < &min_dist {
            min_dist = dists[*node_id as usize];
            min_dist_node = *node_id;
        }
    }
    return min_dist_node;
}

/// returns all neighbors that were not evaluated yet TODO: offset_table has weird values
fn get_remaining_neighbors(node_id: u32, evaluated_nodes: &HashSet<u32>, input: &Input) -> Vec<u32> {
    let first_edge = input.offset_table[node_id as usize];
    let last_edge = input.offset_table[(node_id + 1) as usize];
    let mut neighbors: Vec<u32> = Vec::new();
    for i in first_edge..last_edge {
        if !evaluated_nodes.contains(&input.target[i as usize]) {
            neighbors.push(input.target[i as usize]);
        }
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
    // TODO: u32::max_value hack again
    if !prev[target as usize] == u32::max_value() {
        while !current_node == u32::max_value() {
            shortest_path.push_front(current_node);
            current_node = prev[current_node as usize]
        }
    }
    return Vec::from(shortest_path);
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_find_min() {
        let vector = vec!(2, 1, 3, 4, 5);
        let mut valid_ids = HashSet::new();
        valid_ids.insert(3);
        valid_ids.insert(4);
        valid_ids.insert(2);

        //1 is not a valid node
        assert_eq!(find_min_dist_node(&vector, &valid_ids), 2);
    }
}