use super::*;
/// amount of neighbors
pub fn node_degree(node: NodeId, up_offset: &[EdgeId], down_offset: &[EdgeId]) -> usize {
    up_offset[node + 1] - up_offset[node] + down_offset[node + 1] - down_offset[node]
}

/// calculating the edge-distance heuristic of single node
#[allow(clippy::too_many_arguments)]
fn edge_difference(
    node: NodeId,
    mut dijkstra: &mut dijkstra::Dijkstra,
    shortcut_id: &AtomicUsize,
    edges: &[Way],
    up_offset: &[EdgeId],
    down_offset: &[EdgeId],
    down_index: &[EdgeId],
    rank: usize,
) -> isize {
    let shortcuts = contraction::calc_shortcuts(
        node,
        &mut dijkstra,
        &edges,
        &up_offset,
        &down_offset,
        &down_index,
        &shortcut_id,
        rank,
    );
    let shortcut_len = shortcuts.len();
    shortcut_len as isize - node_degree(node, &up_offset, &down_offset) as isize
}

#[allow(clippy::too_many_arguments)]
pub fn calculate_single_heuristic(
    node: NodeId,
    mut dijkstra: &mut dijkstra::Dijkstra,
    deleted_neighbors: &[Weight],
    shortcut_id: &AtomicUsize,
    edges: &[Way],
    up_offset: &[EdgeId],
    down_offset: &[EdgeId],
    down_index: &[EdgeId],
    rank: usize,
) -> isize {
    deleted_neighbors[node] as isize
        + edge_difference(
            node,
            &mut dijkstra,
            &shortcut_id,
            &edges,
            &up_offset,
            &down_offset,
            &down_index,
            rank,
        )
}

/// calculate heuristic in parallel
#[allow(clippy::too_many_arguments)]
pub fn calculate_heuristics(
    deleted_neighbors: &[Weight],
    shortcut_id: &AtomicUsize,
    rank: usize,
    amount_nodes: usize,
    edges: &[Way],
    up_offset: &[EdgeId],
    down_offset: &[EdgeId],
    down_index: &[EdgeId],
) -> Vec<AtomicIsize> {
    let mut heuristics: Vec<AtomicIsize> = Vec::with_capacity(amount_nodes);
    for _i in 0..amount_nodes {
        heuristics.push(AtomicIsize::new(0));
    }

    let mut nodes: Vec<NodeId> = (0..amount_nodes).into_iter().collect();

    let thread_count = num_cpus::get();
    let chunk_size = (amount_nodes + thread_count - 1) / thread_count;

    if chunk_size > 0 {
        rayon::scope(|s| {
            for datachunk_items in nodes.chunks_mut(chunk_size) {
                s.spawn(|_| {
                    let mut dijkstra = dijkstra::Dijkstra::new(amount_nodes);
                    for node in datachunk_items {
                        let new_value = calculate_single_heuristic(
                            *node,
                            &mut dijkstra,
                            &deleted_neighbors,
                            &shortcut_id,
                            &edges,
                            &up_offset,
                            &down_offset,
                            &down_index,
                            rank,
                        );
                        heuristics[*node as usize].store(new_value, Ordering::Relaxed);
                    }
                });
            }
        });
    }
    heuristics
}

/// update all direct neighbors
#[allow(clippy::too_many_arguments)]
pub fn update_neighbor_heuristics(
    mut neighbors: Vec<NodeId>,
    heuristics: &mut Vec<AtomicIsize>,
    deleted_neighbors: &[Weight],
    shortcut_id: &AtomicUsize,
    rank: usize,
    amount_nodes: usize,
    edges: &[Way],
    up_offset: &[EdgeId],
    down_offset: &[EdgeId],
    down_index: &[EdgeId],
) {
    let thread_count = num_cpus::get();
    let chunk_size = (neighbors.len() + thread_count - 1) / thread_count;

    if chunk_size > 0 {
        rayon::scope(|s| {
            for datachunk_items in neighbors.chunks_mut(chunk_size) {
                s.spawn(|_| {
                    let mut dijkstra = dijkstra::Dijkstra::new(amount_nodes);
                    for neighbor in datachunk_items {
                        let new_value = calculate_single_heuristic(
                            *neighbor,
                            &mut dijkstra,
                            &deleted_neighbors,
                            &shortcut_id,
                            &edges,
                            &up_offset,
                            &down_offset,
                            &down_index,
                            rank,
                        );
                        heuristics[*neighbor as usize].store(new_value, Ordering::Relaxed);
                    }
                });
            }
        });
    }
}

/// get independent set of graph using heuristic
pub fn get_independent_set(
    remaining_nodes: &BTreeSet<NodeId>,
    heuristics: &[AtomicIsize],
    minimas_bool: &mut VisitedList,
    edges: &[Way],
    up_offset: &[EdgeId],
    down_offset: &[EdgeId],
    down_index: &[NodeId],
) -> Vec<NodeId> {
    let subset: Vec<NodeId>;
    let mut remaining_nodes_vector: Vec<NodeId> = remaining_nodes.iter().copied().collect();
    if remaining_nodes.len() > 10_000 {
        // sort remaining_nodes via heuristic
        remaining_nodes_vector.par_sort_by_key(|&node| heuristics[node].load(Ordering::Relaxed));
        // take lower 1/4
        subset = (&remaining_nodes_vector[0..remaining_nodes_vector.len() / 4]).to_vec();
    } else {
        subset = remaining_nodes_vector;
    }

    minimas_bool.unvisit_all();
    // mark all neighbors with greater equal value as invalid
    for node in &subset {
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
    let result: Vec<NodeId> = subset
        .par_iter()
        .filter(|&node| !minimas_bool.is_visited(*node))
        .map(|node| *node)
        .collect();
    result
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
