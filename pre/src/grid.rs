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
    return GridBounds{ lat_min, lat_max, lng_min, lng_max} ;
}

fn get_grid_id(node: &Node, grid_bounds: &GridBounds) -> usize {
    let lat_percent = (node.latitude - grid_bounds.lat_min) / (grid_bounds.lat_max - grid_bounds.lat_min);
    let lat_index = (lat_percent * (LAT_GRID_AMOUNT - 1) as f32) as usize;
    let lng_percent = (node.longitude - grid_bounds.lng_min) / (grid_bounds.lng_max - grid_bounds.lng_min);
    let lng_index = (lng_percent * (LNG_GRID_AMOUNT - 1) as f32) as usize;
    return lng_index * LAT_GRID_AMOUNT + lat_index;
}

pub fn generate_grid(grid: &mut Vec<GridId>, grid_offset: &mut Vec<usize>, nodes: &Vec<Node>) -> GridBounds{
    grid.resize(nodes.len(), 0);

    let grid_bounds: GridBounds = get_min_max(nodes);
    let mut tmp_grid = vec![vec![0; 0]; LAT_GRID_AMOUNT * LNG_GRID_AMOUNT];

    // TODO remove 2d array with only 1d array mapping
    // 2 times iterating: first calculates offsets second insert points compre to current
    // first without concurrent and then test if with gets the same result

    for (i, node) in nodes.iter().enumerate() {
        let grid_index = get_grid_id(node, &grid_bounds);
        tmp_grid[grid_index].push(i);
    }

    // convert tmp_grid to real grid
    grid_offset.resize((LAT_GRID_AMOUNT * LNG_GRID_AMOUNT) + 1, 0);
    let mut k = 0;
    for (i, cell) in tmp_grid.iter().enumerate() {
        grid.extend(cell.iter().cloned());
        grid_offset[i] = k;
        k += cell.len();
    }
    return grid_bounds;
}

// TODO write test
