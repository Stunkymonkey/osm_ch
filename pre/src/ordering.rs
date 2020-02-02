use super::*;

/// amount of neighbors
pub fn node_degree(node: NodeId, up_offset: &Vec<EdgeId>, down_offset: &Vec<EdgeId>) -> usize {
    return up_offset[node + 1] - up_offset[node] + down_offset[node + 1] - down_offset[node];
}

/// calculating the edge-distance heuristic of single node
fn edge_distance(
    node: NodeId,
    edges: &Vec<Way>,
    up_offset: &Vec<EdgeId>,
    down_offset: &Vec<EdgeId>,
    down_index: &Vec<EdgeId>,
    mut dijkstra: &mut dijkstra::Dijkstra,
) -> isize {
    let (shortcuts, _used_edges) = contraction::calc_shortcuts(
        node,
        &edges,
        &up_offset,
        &down_offset,
        &down_index,
        &mut dijkstra,
    );
    return node_degree(node, &up_offset, &down_offset) - shortcuts.len();
}

pub fn calculate_heuristic(
    remaining_nodes: Vec<NodeId>,
    heuristic: &mut Vec<usize>,
    edges: &Vec<Way>,
    up_offset: &Vec<EdgeId>,
    down_offset: &Vec<EdgeId>,
    down_index: &Vec<EdgeId>,
    amount_nodes: usize,
) {
    // TODO in parallel
    for node in remaining_nodes {
        let mut dijkstra = dijkstra::Dijkstra::new(amount_nodes);
        heuristic[node] = edge_distance(
            node,
            &edges,
            &up_offset,
            &down_offset,
            &down_index,
            &mut dijkstra,
        );
    }
}

pub fn get_local_minima(
    heuristic: &Vec<usize>,
    up_offset: &Vec<EdgeId>,
    down_offset: &Vec<EdgeId>,
    down_index: &Vec<NodeId>,
) -> Vec<NodeId> {
    //TODO
    return vec![0; 12];
}
