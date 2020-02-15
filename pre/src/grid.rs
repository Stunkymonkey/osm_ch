use super::*;

/// get min and max of lat and lng
fn get_min_max(nodes: &Vec<Node>) -> GridBounds {
    let lat_min = nodes
        .par_iter()
        .map(|node| node.latitude)
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();
    let lat_max = nodes
        .par_iter()
        .map(|node| node.latitude)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();
    let lng_min = nodes
        .par_iter()
        .map(|node| node.longitude)
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();
    let lng_max = nodes
        .par_iter()
        .map(|node| node.longitude)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap();
    return GridBounds {
        lat_min,
        lat_max,
        lng_min,
        lng_max,
    };
}

fn get_grid_lat(node: &Node, grid_bounds: &GridBounds) -> usize {
    let lat_percent =
        (node.latitude - grid_bounds.lat_min) / (grid_bounds.lat_max - grid_bounds.lat_min);
    return (lat_percent * (LAT_GRID_AMOUNT - 1) as f32) as usize;
}

fn get_grid_lng(node: &Node, grid_bounds: &GridBounds) -> usize {
    let lng_percent =
        (node.longitude - grid_bounds.lng_min) / (grid_bounds.lng_max - grid_bounds.lng_min);
    return (lng_percent * (LNG_GRID_AMOUNT - 1) as f32) as usize;
}

fn calculate_grid_id(lat_index: usize, lng_index: usize) -> GridId {
    let grid_id = lng_index * LAT_GRID_AMOUNT + lat_index;
    return grid_id;
}

fn get_grid_id(node: &Node, grid_bounds: &GridBounds) -> GridId {
    let lat_index = get_grid_lat(node, grid_bounds);
    let lng_index = get_grid_lng(node, grid_bounds);
    return calculate_grid_id(lat_index, lng_index);
}

pub fn generate_grid(
    grid: &mut Vec<GridId>,
    grid_offset: &mut Vec<usize>,
    nodes: &Vec<Node>,
) -> GridBounds {
    let grid_bounds: GridBounds = get_min_max(nodes);

    *grid_offset = vec![0; (LAT_GRID_AMOUNT * LNG_GRID_AMOUNT) + 1];

    // calculate how much nodes go into each cell
    let mut target_cells: Vec<usize> = vec![0; LAT_GRID_AMOUNT * LNG_GRID_AMOUNT];
    for node in nodes {
        target_cells[get_grid_id(node, &grid_bounds)] += 1;
    }

    // generate offset based on target_cells
    for i in 1..grid_offset.len() {
        grid_offset[i] = grid_offset[i - 1] + target_cells[i - 1];
    }

    *grid = vec![INVALID_NODE; nodes.len()];

    // fill offsets, where not already filled
    for (i, node) in nodes.iter().enumerate() {
        let grid_id = get_grid_id(node, &grid_bounds);
        let start_index = grid_offset[grid_id];
        let end_index = grid_offset[grid_id + 1];
        for j in start_index..end_index {
            if grid[j] == INVALID_NODE {
                grid[j] = i;
                break;
            }
        }
    }

    return grid_bounds;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_grid_bounds() {
        let mut nodes = Vec::new();
        nodes.push(Node {
            latitude: 10.0,
            longitude: 30.0,
            rank: 0,
        });
        nodes.push(Node {
            latitude: 20.0,
            longitude: 30.0,
            rank: 0,
        });
        nodes.push(Node {
            latitude: 10.0,
            longitude: 40.0,
            rank: 0,
        });
        nodes.push(Node {
            latitude: 20.0,
            longitude: 40.0,
            rank: 0,
        });

        let grid_bounds: GridBounds = get_min_max(&nodes);

        assert_eq!(grid_bounds.lat_min, 10.0);
        assert_eq!(grid_bounds.lat_max, 20.0);
        assert_eq!(grid_bounds.lng_min, 30.0);
        assert_eq!(grid_bounds.lng_max, 40.0);
    }

    #[test]
    fn grid_id() {
        let mut nodes = Vec::new();
        nodes.push(Node {
            latitude: 10.0,
            longitude: 10.0,
            rank: 0,
        });
        nodes.push(Node {
            latitude: 20.0,
            longitude: 10.0,
            rank: 0,
        });
        nodes.push(Node {
            latitude: 10.0,
            longitude: 20.0,
            rank: 0,
        });
        nodes.push(Node {
            latitude: 20.0,
            longitude: 20.0,
            rank: 0,
        });

        let grid_bounds: GridBounds = get_min_max(&nodes);

        let node_id = get_grid_id(
            &Node {
                latitude: 10.1,
                longitude: 10.1,
                rank: 0,
            },
            &grid_bounds,
        );
        assert_eq!(node_id, 1);

        let node_id = get_grid_id(
            &Node {
                latitude: 10.4,
                longitude: 10.4,
                rank: 0,
            },
            &grid_bounds,
        );
        assert_eq!(node_id, 413);

        let node_id = get_grid_id(
            &Node {
                latitude: 17.3,
                longitude: 12.7,
                rank: 0,
            },
            &grid_bounds,
        );
        assert_eq!(node_id, 3634);
    }
}
