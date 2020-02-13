use super::*;

/// amount of neighbors
pub fn node_degree(node: NodeId, up_offset: &Vec<EdgeId>, down_offset: &Vec<EdgeId>) -> usize {
    return up_offset[node + 1] - up_offset[node] + down_offset[node + 1] - down_offset[node];
}

/// calculating the edge-distance heuristic of single node
fn edge_difference(
    node: NodeId,
    edges: &Vec<Way>,
    up_offset: &Vec<EdgeId>,
    down_offset: &Vec<EdgeId>,
    down_index: &Vec<EdgeId>,
    mut dijkstra: &mut dijkstra::Dijkstra,
) -> isize {
    let shortcuts = contraction::calc_shortcuts(
        node,
        &edges,
        &up_offset,
        &down_offset,
        &down_index,
        &mut dijkstra,
        &mut edges.len(),
    );
    // TODO save shortcuts
    return shortcuts.len() as isize - node_degree(node, &up_offset, &down_offset) as isize;
}

/// calculate heuristic in parallel
pub fn calculate_heuristics(
    remaining_nodes: &BTreeSet<NodeId>,
    dijkstra: &dijkstra::Dijkstra,
    deleted_neighbors: &Vec<Weight>,
    edges: &Vec<Way>,
    up_offset: &Vec<EdgeId>,
    down_offset: &Vec<EdgeId>,
    down_index: &Vec<EdgeId>,
) -> Vec<isize> {
    return remaining_nodes
        .par_iter()
        .map_with(dijkstra.clone(), |mut d, x| {
            deleted_neighbors[*x] as isize
                + edge_difference(*x, &edges, &up_offset, &down_offset, &down_index, &mut d)
        })
        .collect();
}

/// update all direct neighbors
pub fn update_neighbor_heuristics(
    neighbors: Vec<NodeId>,
    heuristics: &mut Vec<isize>,
    mut dijkstra: &mut dijkstra::Dijkstra,
    deleted_neighbors: &Vec<Weight>,
    edges: &Vec<Way>,
    up_offset: &Vec<EdgeId>,
    down_offset: &Vec<EdgeId>,
    down_index: &Vec<EdgeId>,
) {
    // TODO split vec in thread friendly amount and let run
    for neighbor in neighbors {
        heuristics[neighbor] = deleted_neighbors[neighbor] as isize
            + edge_difference(
                neighbor,
                &edges,
                &up_offset,
                &down_offset,
                &down_index,
                &mut dijkstra,
            );
    }
}

/// get index of local minima in heuristic
pub fn get_minima(heuristic: &Vec<isize>) -> NodeId {
    let index_of_min: Option<usize> = heuristic
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(index, _)| index);
    return index_of_min.unwrap();
}

/// get independent set of graph using heuristic
pub fn get_independent_set(
    heuristic: &Vec<isize>,
    edges: &Vec<Way>,
    up_offset: &Vec<EdgeId>,
    down_offset: &Vec<EdgeId>,
    down_index: &Vec<NodeId>,
) -> Vec<NodeId> {
    for (node, heuristic_value) in heuristic.iter().enumerate() {
        let neighbors =
            graph_helper::get_all_neighbours(node, &edges, &up_offset, &down_offset, &down_index);
    }

    //TODO
    //K_NEIGHBORS
    // mark all neighbors as invalid
    // partition = let (even, odd): (Vec<i32>, Vec<i32>) = a.par_iter().partition(|&n| n % 2 == 0);
    return vec![0; 12];
}
