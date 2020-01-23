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

pub fn get_local_minima(
    heuristic: &Vec<usize>,
    up_offset: &Vec<EdgeId>,
    down_offset: &Vec<EdgeId>,
    down_index: &Vec<NodeId>,
) -> Vec<NodeId> {
    //TODO
    return vec![0; 12];
}
