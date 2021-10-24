use super::*;

/// get node-ids by brute-force
#[allow(dead_code)]
pub fn get_closest_point_stupid(node: Node, nodes: &[Node]) -> usize {
    let mut tmp_minimum = std::f32::MAX;
    let mut tmp_closeset = INVALID_NODE;
    for (i, n) in nodes.iter().enumerate() {
        let dist = calc_distance(node, *n);
        if dist < tmp_minimum {
            tmp_minimum = dist;
            tmp_closeset = i;
        }
    }
    tmp_closeset
}

/// get node-ids using grid
pub fn get_closest_point(
    node: Node,
    nodes: &[Node],
    grid: &[NodeId],
    grid_offset: &[GridId],
    grid_bounds: &GridBounds,
) -> usize {
    let mut minimum = std::f32::MAX;
    let mut closeset = INVALID_NODE;

    let adjacent_nodes = get_adjacent_nodes(node, grid, grid_offset, grid_bounds);
    for node_id in adjacent_nodes {
        let tmp_node: Node = nodes[node_id];
        let dist = calc_distance(node, tmp_node);
        if dist < minimum {
            minimum = dist;
            closeset = node_id;
        }
    }
    closeset
}

/// get close node_ids
fn get_adjacent_nodes(
    node: Node,
    grid: &[NodeId],
    grid_offset: &[GridId],
    grid_bounds: &GridBounds,
) -> Vec<NodeId> {
    let grid_id_lat: isize = get_grid_lat(&node, grid_bounds) as isize;
    let grid_id_lng: isize = get_grid_lng(&node, grid_bounds) as isize;
    // println!("grid_id_lat {:?}", grid_id_lat);
    // println!("grid_id_lng {:?}", grid_id_lng);
    let mut grid_dist: isize = 1;

    loop {
        let mut cell_ids = Vec::<GridId>::new();
        // moving in circle around the target
        for i in -grid_dist..(grid_dist) {
            // first iteration add the middle
            if grid_dist == 1
                && i == 0
                && grid_id_lat >= 0
                && grid_id_lng >= 0
                && grid_id_lat < (LAT_GRID_AMOUNT as isize)
                && grid_id_lng < (LNG_GRID_AMOUNT as isize)
            {
                cell_ids.push(calculate_grid_id(
                    (grid_id_lat) as usize,
                    (grid_id_lng) as usize,
                ));
            }
            // north left to right
            if grid_id_lat + i >= 0
                && grid_id_lng + grid_dist >= 0
                && grid_id_lat + i < (LAT_GRID_AMOUNT as isize)
                && grid_id_lng + grid_dist < (LNG_GRID_AMOUNT as isize)
            {
                cell_ids.push(calculate_grid_id(
                    (grid_id_lat + i) as usize,
                    (grid_id_lng + grid_dist) as usize,
                ));
            }
            // east top to bottom
            if grid_id_lat + grid_dist >= 0
                && grid_id_lng - i >= 0
                && grid_id_lat + grid_dist < (LAT_GRID_AMOUNT as isize)
                && grid_id_lng - i < (LNG_GRID_AMOUNT as isize)
            {
                cell_ids.push(calculate_grid_id(
                    (grid_id_lat + grid_dist) as usize,
                    (grid_id_lng - i) as usize,
                ));
            }
            // south top to bottom
            if grid_id_lat - i >= 0
                && grid_id_lng - grid_dist >= 0
                && grid_id_lat - i < (LAT_GRID_AMOUNT as isize)
                && grid_id_lng - grid_dist < (LNG_GRID_AMOUNT as isize)
            {
                cell_ids.push(calculate_grid_id(
                    (grid_id_lat - i) as usize,
                    (grid_id_lng - grid_dist) as usize,
                ));
            }
            // west top to bottom
            if grid_id_lat - grid_dist >= 0
                && grid_id_lng + i >= 0
                && grid_id_lat - grid_dist < (LAT_GRID_AMOUNT as isize)
                && grid_id_lng + i < (LNG_GRID_AMOUNT as isize)
            {
                cell_ids.push(calculate_grid_id(
                    (grid_id_lat - grid_dist) as usize,
                    (grid_id_lng + i) as usize,
                ));
            }
        }

        // get all points from cells
        let adjacent_nodes = get_points_from_cells(&cell_ids, grid, grid_offset);

        if !adjacent_nodes.is_empty() {
            return adjacent_nodes;
        } else {
            // search in outer cells
            grid_dist += 1;
        }
    }
}

/// return node-ids from multiple cells
fn get_points_from_cells(
    grid_ids: &[GridId],
    grid: &[NodeId],
    grid_offset: &[GridId],
) -> Vec<NodeId> {
    let mut result = Vec::<NodeId>::new();
    // sequential is faster, then parallelizing
    for grid_id in grid_ids {
        for grid_index in grid
            .iter()
            .take(grid_offset[*grid_id + 1])
            .skip(grid_offset[*grid_id])
        {
            result.push(*grid_index);
        }
    }
    result
}

#[allow(clippy::suspicious_operation_groupings)]
fn get_grid_lat(node: &Node, grid_bounds: &GridBounds) -> usize {
    let lat_percent =
        (node.latitude - grid_bounds.lat_min) / (grid_bounds.lat_max - grid_bounds.lat_min);
    (lat_percent * (LAT_GRID_AMOUNT - 1) as f32) as usize
}

#[allow(clippy::suspicious_operation_groupings)]
fn get_grid_lng(node: &Node, grid_bounds: &GridBounds) -> usize {
    let lng_percent =
        (node.longitude - grid_bounds.lng_min) / (grid_bounds.lng_max - grid_bounds.lng_min);
    (lng_percent * (LNG_GRID_AMOUNT - 1) as f32) as usize
}

fn calculate_grid_id(lat_index: usize, lng_index: usize) -> GridId {
    lng_index * LAT_GRID_AMOUNT + lat_index
}

#[allow(dead_code)]
fn get_grid_id(node: &Node, grid_bounds: &GridBounds) -> GridId {
    let lat_index = get_grid_lat(node, grid_bounds);
    let lng_index = get_grid_lng(node, grid_bounds);
    calculate_grid_id(lat_index, lng_index)
}

/// get distance on earth surface using haversine formula
fn calc_distance(a: Node, b: Node) -> f32 {
    let lat_1 = a.latitude;
    let long_1 = a.longitude;
    let lat_2 = b.latitude;
    let long_2 = b.longitude;
    let r: f32 = 6371.0; // constant used for meters
    let d_lat: f32 = (lat_2 - lat_1).to_radians();
    let d_lon: f32 = (long_2 - long_1).to_radians();
    let lat1: f32 = (lat_1).to_radians();
    let lat2: f32 = (lat_2).to_radians();

    let a: f32 = ((d_lat / 2.0).sin()) * ((d_lat / 2.0).sin())
        + ((d_lon / 2.0).sin()) * ((d_lon / 2.0).sin()) * (lat1.cos()) * (lat2.cos());
    let c: f32 = 2.0 * ((a.sqrt()).atan2((1.0 - a).sqrt()));
    r * c
}

/// converts node ids to nodes
pub fn get_coordinates(path: Vec<NodeId>, nodes: &[Node]) -> Vec<Node> {
    return path.par_iter().map(|x| nodes[*x]).collect::<Vec<Node>>();
}
