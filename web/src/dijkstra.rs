use Input;
use std::collections::VecDeque;

pub fn dijkstra(start: u32, end: u32, input: Input) -> Vec<u32> {
    // all distances are at max_value
    let mut dist = vec!(u32::max_value(); input.offset_table.len());
    // TODO: is a hack, if node id is u32::max value this is going to break
    let mut prev = vec!(u32::max_value(); input.offset_table.len());
    // add all nodes to be evaluated
    let mut to_evaluate: Vec<u32> = input.offset_table.clone();
    // dist of start node is 0
    *dist.get_mut(start as usize).unwrap() = 0;
    while !to_evaluate.is_empty() {
        let min_dist_node = index_of_min_dist(&dist, &to_evaluate);
        if min_dist_node == end { break; }
        to_evaluate.remove(min_dist_node as usize);
        for neighbor in get_remaining_neighbors(min_dist_node,&to_evaluate, &input) {
            let mut temp = dist[min_dist_node as usize] + get_edge_weight(start, neighbor, &input);
            if temp < dist[neighbor as usize] {
                dist[neighbor as usize] = temp;
                prev[neighbor as usize] = min_dist_node;
            }
        }
    }
    return get_shortest_path(end, &prev);
}

pub fn get_point_id(lat: f32, long: f32) -> u32 {
    // calculate distance to all points and get minimum
    return 0;
}

pub fn get_coordinates(id: u32) -> (f32, f32) {
    return (0.0, 0.0);
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
fn index_of_min_dist(dists: &Vec<u32>, node_ids: &Vec<u32>) -> u32 {
    let mut smallest: u32 = dists[node_ids[0] as usize];
    let mut smallest_index: u32 = node_ids[0];
    for node_id in node_ids {
        if &dists[*node_id as usize] < &smallest {
            smallest = dists[*node_id as usize];
            smallest_index = *node_id;
        }
    }
    return smallest_index;
}

/// returns all neighbors that are contained in remaining_nodes
fn get_remaining_neighbors(node_id: u32, remaining_nodes: &Vec<u32>, input: &Input) -> Vec<u32> {
    let first_edge = input.offset_table[node_id as usize];
    let last_edge = input.offset_table[(node_id + 1) as usize];
    let mut neighbors: Vec<u32> = Vec::new();
    for i in first_edge..last_edge {
        if remaining_nodes.contains(&input.target[i as usize]) {
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
        let valid_ids = vec!(0, 2, 3, 4);
        // 1 is not a valid node
        assert_eq!(index_of_min_dist(&vector, &valid_ids), 0);
    }
}