use super::*;
use std::sync::atomic::{AtomicIsize, Ordering};

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
) -> Vec<AtomicIsize> {
    return remaining_nodes
        .par_iter()
        .map_with(dijkstra.clone(), |mut d, x| {
            AtomicIsize::new(
                deleted_neighbors[*x] as isize
                    + edge_difference(*x, &edges, &up_offset, &down_offset, &down_index, &mut d),
            )
        })
        .collect();
}

/// update all direct neighbors
pub fn update_neighbor_heuristics(
    neighbors: Vec<NodeId>,
    heuristics: &mut Vec<AtomicIsize>,
    dijkstra: &mut dijkstra::Dijkstra,
    deleted_neighbors: &Vec<Weight>,
    edges: &Vec<Way>,
    up_offset: &Vec<EdgeId>,
    down_offset: &Vec<EdgeId>,
    down_index: &Vec<EdgeId>,
) {
    neighbors
        .par_iter()
        .for_each_with(dijkstra.clone(), |mut dijkstra, neighbor| {
            heuristics[*neighbor as usize].store(
                deleted_neighbors[*neighbor as usize] as isize
                    + edge_difference(
                        *neighbor,
                        &edges,
                        &up_offset,
                        &down_offset,
                        &down_index,
                        &mut dijkstra,
                    ),
                Ordering::Relaxed,
            )
        });
}

/// get independent set of graph using heuristic
pub fn get_independent_set(
    remaining_nodes: &BTreeSet<NodeId>,
    heuristics: &Vec<AtomicIsize>,
    minimas_bool: &mut VisitedList,
    edges: &Vec<Way>,
    up_offset: &Vec<EdgeId>,
    down_offset: &Vec<EdgeId>,
    down_index: &Vec<NodeId>,
) -> Vec<NodeId> {
    minimas_bool.unvisit_all();
    // mark all neighbors with greater equal value as invalid
    for node in remaining_nodes {
        for neighbor in
            graph_helper::get_all_neighbours(*node, &edges, &up_offset, &down_offset, &down_index)
        {
            if !minimas_bool.is_visited(neighbor)
                && neighbor != *node
                && heuristics[*node].load(Ordering::Relaxed)
                    >= heuristics[neighbor].load(Ordering::Relaxed)
            {
                minimas_bool.set_visited(*node);
            }
        }
    }

    // collect all indices of valid nodes
    let result: Vec<NodeId> = remaining_nodes
        .par_iter()
        .filter(|&node| !minimas_bool.is_visited(*node))
        .map(|node| node.clone())
        .collect();
    return result;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_set_test() {
        // note: in this test no edge gets removed
        // 0->1->2->3->4->5->6->7->8
        let amount_nodes = 9;

        let mut remaining_nodes = BTreeSet::new();
        for node_id in 0..amount_nodes {
            remaining_nodes.insert(node_id);
        }

        let mut edges = Vec::<Way>::new();
        edges.push(Way::new(0, 1, 1));
        edges.push(Way::new(1, 2, 1));
        edges.push(Way::new(2, 3, 1));
        edges.push(Way::new(3, 4, 1));
        edges.push(Way::new(4, 5, 1));
        edges.push(Way::new(5, 6, 1));
        edges.push(Way::new(6, 7, 1));
        edges.push(Way::new(7, 8, 1));

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        let down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);

        let heuristics = vec![
            AtomicIsize::new(0),
            AtomicIsize::new(1),
            AtomicIsize::new(-2),
            AtomicIsize::new(1),
            AtomicIsize::new(4),
            AtomicIsize::new(3),
            AtomicIsize::new(1),
            AtomicIsize::new(-1),
            AtomicIsize::new(5),
        ];

        let mut minimas_bool = VisitedList::new(amount_nodes);

        let minima = get_independent_set(
            &remaining_nodes,
            &heuristics,
            &mut minimas_bool,
            &edges,
            &up_offset,
            &down_offset,
            &down_index,
        );

        let mut expected_minima = Vec::<NodeId>::new();
        expected_minima.push(0);
        expected_minima.push(2);
        expected_minima.push(7);

        assert_eq!(minima, expected_minima);

        remaining_nodes.remove(&0);
        remaining_nodes.remove(&2);
        remaining_nodes.remove(&7);

        let heuristics = vec![
            AtomicIsize::new(99),
            AtomicIsize::new(1),
            AtomicIsize::new(99),
            AtomicIsize::new(1),
            AtomicIsize::new(4),
            AtomicIsize::new(3),
            AtomicIsize::new(1),
            AtomicIsize::new(99),
            AtomicIsize::new(5),
        ];
        let minima = get_independent_set(
            &remaining_nodes,
            &heuristics,
            &mut minimas_bool,
            &edges,
            &up_offset,
            &down_offset,
            &down_index,
        );

        let mut expected_minima = Vec::<NodeId>::new();
        expected_minima.push(1);
        expected_minima.push(3);
        expected_minima.push(6);
        expected_minima.push(8);

        assert_eq!(minima, expected_minima);
    }
}
