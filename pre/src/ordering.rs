use super::*;

/// amount of connected node
pub fn node_degree(node: NodeId, up_offset: &Vec<EdgeId>, down_offset: &Vec<EdgeId>) -> usize {
    return up_offset[node + 1] - up_offset[node] + down_offset[node + 1] - down_offset[node];
}

/// returning all previous and next neighbors
pub fn get_neighbours(
    node: NodeId,
    edges: &Vec<Way>,
    up_offset: &Vec<EdgeId>,
    down_offset: &Vec<EdgeId>,
    down_index: &Vec<EdgeId>,
) -> (Vec<usize>, Vec<usize>) {
    let prev = &up_offset[node..node + 1];
    let next = &down_offset[node..node + 1];
    let sources: Vec<NodeId> = prev.par_iter().map(|x| edges[*x].source).collect();
    let targets: Vec<NodeId> = next
        .par_iter()
        .map(|x| edges[down_index[*x]].target)
        .collect();
    return (sources, targets);
}

/// returning all neighbors
pub fn get_mixed_neighbours(
    node: NodeId,
    edges: &Vec<Way>,
    up_offset: &Vec<EdgeId>,
    down_offset: &Vec<EdgeId>,
    down_index: &Vec<EdgeId>,
) -> Vec<usize> {
    let (sources, targets) = get_neighbours(node, edges, up_offset, down_offset, down_index);
    let mut neighbours = sources;
    neighbours.extend(&targets);
    return neighbours;
}

fn edge_distance(
    node: NodeId,
    edges: &Vec<Way>,
    up_offset: &Vec<EdgeId>,
    down_offset: &Vec<EdgeId>,
    down_index: &Vec<EdgeId>,
    mut dijkstra: &mut dijkstra::Dijkstra,
) -> usize {
    let (shortcuts, _used_edges) = contraction::contract_node(
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
