use super::*;

/// get all up edges from one node
#[allow(dead_code)]
pub fn get_edges_from_id(ids: Vec<EdgeId>, edges: &[Way]) -> Vec<Way> {
    return ids.iter().map(|x| edges[*x]).collect();
}

/// get all up edge-ids from one node
#[allow(dead_code)]
pub fn get_up_edge_ids(node: NodeId, up_offset: &[EdgeId]) -> Vec<EdgeId> {
    (up_offset[node]..up_offset[node + 1]).collect()
}

/// get all down edge-ids from one node
#[allow(dead_code)]
pub fn get_down_edge_ids(
    node: NodeId,
    down_offset: &[EdgeId],
    down_index: &[EdgeId],
) -> Vec<EdgeId> {
    let prev: Vec<EdgeId> = (down_offset[node]..down_offset[node + 1]).collect();
    return prev.iter().map(|x| down_index[*x]).collect();
}

/// get all down edge-ids from one node
#[allow(dead_code)]
pub fn get_edge_ids(
    node: NodeId,
    up_offset: &[EdgeId],
    down_offset: &[EdgeId],
    down_index: &[EdgeId],
) -> (Vec<EdgeId>, Vec<EdgeId>) {
    let outgoing: Vec<NodeId> = get_up_edge_ids(node, up_offset);
    let incomming: Vec<NodeId> = get_down_edge_ids(node, down_offset, down_index);
    (outgoing, incomming)
}

/// get all edge-ids from one node
#[allow(dead_code)]
pub fn get_all_edge_ids(
    node: NodeId,
    up_offset: &[EdgeId],
    down_offset: &[EdgeId],
    down_index: &[EdgeId],
) -> Vec<EdgeId> {
    let (outgoing, incomming) = get_edge_ids(node, up_offset, down_offset, down_index);
    let mut connected_edges = outgoing;
    connected_edges.extend(&incomming);
    connected_edges
}

/// get all up neighbors from one node
#[allow(dead_code)]
pub fn get_up_neighbors(node: NodeId, edges: &[Way], up_offset: &[EdgeId]) -> Vec<EdgeId> {
    let next = get_up_edge_ids(node, up_offset);
    let mut tmp: Vec<EdgeId> = next.iter().map(|x| edges[*x].target).collect();
    tmp.dedup();
    tmp
}

/// get all up neighbors from one node
#[allow(dead_code)]
pub fn get_down_neighbors(
    node: NodeId,
    edges: &[Way],
    down_offset: &[EdgeId],
    down_index: &[EdgeId],
) -> Vec<EdgeId> {
    let prev = get_down_edge_ids(node, down_offset, down_index);
    let mut tmp: Vec<EdgeId> = prev.iter().map(|x| edges[*x].source).collect();
    tmp.par_sort_unstable();
    tmp.dedup();
    tmp
}

/// returning all previous and next neighbors
#[allow(dead_code)]
pub fn get_neighbours(
    node: NodeId,
    edges: &[Way],
    up_offset: &[EdgeId],
    down_offset: &[EdgeId],
    down_index: &[EdgeId],
) -> (Vec<usize>, Vec<usize>) {
    let targets: Vec<NodeId> = get_up_neighbors(node, edges, up_offset);
    let sources: Vec<NodeId> = get_down_neighbors(node, edges, down_offset, down_index);
    (targets, sources)
}

/// returning all neighbors
#[allow(dead_code)]
pub fn get_all_neighbours(
    node: NodeId,
    edges: &[Way],
    up_offset: &[EdgeId],
    down_offset: &[EdgeId],
    down_index: &[EdgeId],
) -> Vec<usize> {
    let (targets, sources) = get_neighbours(node, edges, up_offset, down_offset, down_index);
    let mut neighbours = targets;
    neighbours.extend(&sources);
    neighbours.par_sort_unstable();
    neighbours.dedup();
    neighbours
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_index() {
        //  0->      ->3
        //     \   /
        //       1 -> 4
        //     /  \
        //  2->    ->5

        let amount_nodes = 6;

        let mut edges = Vec::<Way>::new();
        edges.push(Way::new(0, 1, 1));
        edges.push(Way::new(2, 1, 1));
        edges.push(Way::new(1, 3, 1));
        edges.push(Way::new(1, 5, 1));
        edges.push(Way::new(1, 4, 1));

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        let down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);

        let up = get_up_edge_ids(0, &up_offset);
        assert_eq!(up, vec![0]);

        let down = get_down_edge_ids(4, &down_offset, &down_index);
        assert_eq!(down, vec![2]);

        let up = get_up_edge_ids(1, &up_offset);
        assert_eq!(up, vec![1, 2, 3]);
        let down = get_down_edge_ids(1, &down_offset, &down_index);
        assert_eq!(down, vec![0, 4]);
    }
    #[test]
    fn edge_index_line() {
        //  0->1->2->3

        let amount_nodes = 4;

        let mut edges = Vec::<Way>::new();
        edges.push(Way::test(0, 1, 1, 0));
        edges.push(Way::test(1, 2, 1, 1));
        edges.push(Way::test(2, 3, 1, 2));

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        let down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);

        let up = get_up_edge_ids(0, &up_offset);
        assert_eq!(up, vec![0]);
        let down = get_down_edge_ids(0, &down_offset, &down_index);
        assert_eq!(down, vec![]);

        let up = get_up_edge_ids(1, &up_offset);
        assert_eq!(up, vec![1]);
        let down = get_down_edge_ids(1, &down_offset, &down_index);
        assert_eq!(down, vec![0]);

        let up = get_up_edge_ids(2, &up_offset);
        assert_eq!(up, vec![2]);
        let down = get_down_edge_ids(2, &down_offset, &down_index);
        assert_eq!(down, vec![1]);

        let up = get_up_edge_ids(3, &up_offset);
        assert_eq!(up, vec![]);
        let down = get_down_edge_ids(3, &down_offset, &down_index);
        assert_eq!(down, vec![2]);
    }

    #[test]
    fn edges() {
        //  0->      ->3
        //     \   /
        //       1 -> 4
        //     /  \
        //  2->    ->5

        let amount_nodes = 6;

        let mut edges = Vec::<Way>::new();
        edges.push(Way::new(0, 1, 1));
        edges.push(Way::new(2, 1, 1));
        edges.push(Way::new(1, 3, 1));
        edges.push(Way::new(1, 5, 1));
        edges.push(Way::new(1, 4, 1));

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        let down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);

        let up = get_up_edge_ids(1, &up_offset);
        let edge_ids = get_edges_from_id(up, &edges);
        assert_eq!(
            edge_ids,
            vec![Way::new(1, 3, 1), Way::new(1, 4, 1), Way::new(1, 5, 1)]
        );

        let down = get_down_edge_ids(1, &down_offset, &down_index);
        let edge_ids = get_edges_from_id(down, &edges);
        assert_eq!(edge_ids, vec![Way::new(0, 1, 1), Way::new(2, 1, 1),]);

        let up = get_up_edge_ids(2, &up_offset);
        let edge_ids = get_edges_from_id(up, &edges);
        assert_eq!(edge_ids, vec![Way::new(2, 1, 1)]);

        let down = get_down_edge_ids(2, &down_offset, &down_index);
        let edge_ids = get_edges_from_id(down, &edges);
        assert_eq!(edge_ids, vec![]);

        let up = get_up_edge_ids(4, &up_offset);
        let edge_ids = get_edges_from_id(up, &edges);
        assert_eq!(edge_ids, vec![]);

        let down = get_down_edge_ids(4, &down_offset, &down_index);
        let edge_ids = get_edges_from_id(down, &edges);
        assert_eq!(edge_ids, vec![Way::new(1, 4, 1)]);
    }

    #[test]
    fn neighbours() {
        //  0->      ->3
        //     \   /
        //       1 -> 4
        //     /  \
        //  2->    ->5

        let amount_nodes = 6;

        let mut edges = Vec::<Way>::new();
        edges.push(Way::new(0, 1, 1));
        edges.push(Way::new(2, 1, 1));
        edges.push(Way::new(1, 3, 1));
        edges.push(Way::new(1, 5, 1));
        edges.push(Way::new(1, 4, 1));

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        let down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);

        let (targets, sources) = get_neighbours(1, &edges, &up_offset, &down_offset, &down_index);

        assert_eq!(targets, [3, 4, 5]);
        assert_eq!(sources, [0, 2]);

        let neighbours = get_all_neighbours(1, &edges, &up_offset, &down_offset, &down_index);
        assert_eq!(neighbours, [0, 2, 3, 4, 5]);
    }
}
