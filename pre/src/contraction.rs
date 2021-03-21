use super::*;
use std::cmp::Reverse;

/// return new generated shortcuts
#[allow(clippy::too_many_arguments)]
pub fn calc_shortcuts(
    node: NodeId,
    dijkstra: &mut dijkstra::Dijkstra,
    edges: &[Way],
    up_offset: &[EdgeId],
    down_offset: &[EdgeId],
    down_index: &[EdgeId],
    shortcut_id: &AtomicUsize,
    rank: usize,
) -> Vec<Way> {
    let mut shortcuts = Vec::<Way>::new();
    // get node neighbors
    let source_edges: Vec<EdgeId> =
        graph_helper::get_down_edge_ids(node, &down_offset, &down_index);
    let target_edges: Vec<EdgeId> = graph_helper::get_up_edge_ids(node, &up_offset);

    //get minimum costs of one pair
    let mut minimum_neighbor_distances: BTreeMap<(NodeId, NodeId), (Weight, (EdgeId, EdgeId))> =
        BTreeMap::new();
    for source_edge in &source_edges {
        let source_node = edges[*source_edge].source;
        for target_edge in &target_edges {
            let target_node = edges[*target_edge].target;
            let weight = edges[*source_edge].weight + edges[*target_edge].weight;
            if weight < edges[*source_edge].weight || weight < edges[*target_edge].weight {
                panic!("overflow in weights! reduce DIST_MULTIPLICATOR");
            }
            // skip loops (dijkstra should get rid of it anyway)
            if source_node == target_node || source_node == node || target_node == node {
                continue;
            }
            let pair = (source_node, target_node);
            let lower_edges = (*source_edge, *target_edge);
            if let Some(old_minima) = minimum_neighbor_distances.get_mut(&pair) {
                if (*old_minima).0 > weight {
                    *old_minima = (weight, lower_edges);
                }
            } else {
                minimum_neighbor_distances.insert(pair, (weight, lower_edges));
            }
        }
    }

    // iterate over pairs and generate shortcuts
    for neighbors in minimum_neighbor_distances {
        let source_node = (neighbors.0).0;
        let target_node = (neighbors.0).1;
        let weight = (neighbors.1).0;
        let source_edge = ((neighbors.1).1).0;
        let target_edge = ((neighbors.1).1).1;

        let shortest_path =
            dijkstra.find_path(source_node, target_node, up_offset, edges, false, rank);

        // create new shortcut where found path is shortest
        if let Some(shortest_path) = shortest_path {
            if shortest_path.1 >= weight {
                shortcuts.push(Way {
                    source: source_node,
                    target: target_node,
                    weight,
                    id: Some(shortcut_id.fetch_add(1, Ordering::SeqCst)),
                    // do not use edge.index, because it will change during contraction
                    contrated_previous: Some(edges[source_edge].id.unwrap()),
                    contrated_next: Some(edges[target_edge].id.unwrap()),
                });
            }
        }
    }
    shortcuts
}

fn remove_redundant_edges(
    mut edges: &mut Vec<Way>,
    mut up_offset: &mut Vec<EdgeId>,
    mut down_offset: &mut Vec<EdgeId>,
    down_index: &mut Vec<EdgeId>,
    amount_nodes: usize,
) {
    // collect removing indices
    let remove_edges: Vec<EdgeId> = edges
        .iter()
        .zip(edges.iter().skip(1))
        .enumerate()
        .filter_map(|(i, (&x, &y))| {
            if x.source == y.source
                && x.target == y.target
                && x.weight <= y.weight
                && y.contrated_previous.is_none()
            {
                Some(i + 1)
            } else {
                None
            }
        })
        .collect();

    // check if ids is used in any shortcut
    let mut contraction_ids = BTreeSet::new();
    for edge in edges.iter_mut() {
        contraction_ids.insert(edge.contrated_previous);
        contraction_ids.insert(edge.contrated_next);
    }
    contraction_ids.remove(&None);

    let unused_edges: Vec<&EdgeId> = remove_edges
        .par_iter()
        .filter(|&x| !contraction_ids.contains(&edges[*x].id))
        .collect();
    println!("remove unused edges: {:?}", unused_edges.len());

    // remove all of them
    for edge_id in unused_edges.iter().rev() {
        edges.swap_remove(**edge_id);
    }

    // update graph
    *down_index =
        offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);
}

fn sort_edges_ranked(
    edges: &mut Vec<Way>,
    down_offset: &[EdgeId],
    down_index: &mut Vec<EdgeId>,
    nodes: &[Node],
) {
    //sort by source then rank
    edges.par_sort_by(|a, b| {
        a.source
            .cmp(&b.source)
            .then(nodes[a.target].rank.cmp(&nodes[b.target].rank).reverse())
    });

    *down_index = vec![INVALID_EDGE; edges.len()];
    // fill offsets, where not already filled
    for (i, edge) in edges.iter().enumerate() {
        let start_index = down_offset[edge.target];
        let end_index = down_offset[edge.target + 1];
        for j in down_index.iter_mut().take(end_index).skip(start_index) {
            if *j == INVALID_EDGE {
                *j = i;
                break;
            }
        }
    }

    // sort down_index subvectors
    for node in 0..nodes.len() {
        let subvector = &mut down_index[down_offset[node]..down_offset[node + 1]];
        subvector.sort_by_key(|edge_id| Reverse(nodes[edges[*edge_id].source].rank));
    }
}

fn revert_indices(edges: &mut Vec<Way>) {
    let maximum_id = edges
        .par_iter()
        .map(|edge| edge.id)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap()
        .unwrap();
    let mut indices = vec![INVALID_NODE; maximum_id + 1];

    for (i, edge) in edges.iter().enumerate() {
        indices[edge.id.unwrap()] = i;
    }

    edges.par_iter_mut().for_each(|edge| {
        if edge.contrated_previous.is_some() {
            edge.contrated_previous = Some(indices[edge.contrated_previous.unwrap()]);
            edge.contrated_next = Some(indices[edge.contrated_next.unwrap()]);
        }
    });
}

/// run full contraction
pub fn run_contraction(
    nodes: &mut Vec<Node>,
    mut edges: &mut Vec<Way>,
    mut up_offset: &mut Vec<EdgeId>,
    mut down_offset: &mut Vec<EdgeId>,
    mut down_index: &mut Vec<EdgeId>,
) {
    let amount_nodes: usize = nodes.len();
    // for keeping track of new created edge_ids
    let shortcut_id = AtomicUsize::new(edges.len());

    // make edges have indices
    edges
        .par_iter_mut()
        .enumerate()
        .for_each(|(i, x)| x.id = Some(i));

    let mut resulting_edges = Vec::<Way>::with_capacity(edges.len() * 2);

    let mut remaining_nodes = BTreeSet::new();
    for node_id in 0..amount_nodes {
        remaining_nodes.insert(node_id);
    }

    let mut rank: Rank = 0;
    let mut minimas_bool = VisitedList::new(amount_nodes);

    // update priorities of all nodes with simulated contractions
    let mut deleted_neighbors = vec![0; amount_nodes];
    let mut heuristics = ordering::calculate_heuristics(
        &deleted_neighbors,
        &shortcut_id,
        rank,
        amount_nodes,
        &edges,
        &up_offset,
        &down_offset,
        &down_index,
    );

    let thread_count = num_cpus::get();

    while !remaining_nodes.is_empty() {
        let get_independent_set_time = Instant::now();
        // I ← independent node set
        let mut minimas = ordering::get_independent_set(
            &remaining_nodes,
            &heuristics,
            &mut minimas_bool,
            &edges,
            &up_offset,
            &down_offset,
            &down_index,
        );
        if remaining_nodes.len() > 100_000 {
            println!(
                "get_independent_set time in: {:?}",
                get_independent_set_time.elapsed()
            );
        }

        let other_time = Instant::now();
        // E ← necessary shortcuts
        let parallel_shortcuts: RwLock<Vec<Way>> =
            RwLock::new(Vec::with_capacity(2 * minimas.len()));

        let chunk_size = (minimas.len() + thread_count - 1) / thread_count;
        if chunk_size > 0 {
            rayon::scope(|s| {
                for datachunk_items in minimas.chunks_mut(chunk_size) {
                    s.spawn(|_| {
                        let mut dijkstra = dijkstra::Dijkstra::new(amount_nodes);
                        for node in datachunk_items {
                            let node_shortcuts = calc_shortcuts(
                                *node,
                                &mut dijkstra,
                                &edges,
                                &up_offset,
                                &down_offset,
                                &down_index,
                                &shortcut_id,
                                rank,
                            );
                            let mut tmp = parallel_shortcuts.write().unwrap();
                            tmp.extend(node_shortcuts);
                        }
                    });
                }
            });
        }
        let mut shortcuts = parallel_shortcuts.into_inner().unwrap();

        // collecting all edges to be removed
        let mut connected_edges: Vec<EdgeId> = minimas
            .par_iter()
            .map(|node| {
                graph_helper::get_all_edge_ids(*node, &up_offset, &down_offset, &down_index)
            })
            .flatten()
            .collect();

        // dedup shortcuts with same start, end but have to keep with best weight preventing shortcuts in diamond-shapes
        shortcuts.par_sort_unstable();
        shortcuts.dedup_by(|a, b| a.source == b.source && a.target == b.target);

        // update heuristic of neighbors of I with simulated contractions
        let mut neighbors: Vec<NodeId> = minimas
            .par_iter()
            .map(|node| {
                graph_helper::get_all_neighbours(
                    *node,
                    &edges,
                    &up_offset,
                    &down_offset,
                    &down_index,
                )
            })
            .flatten()
            .collect();
        for neighbor in &neighbors {
            deleted_neighbors[*neighbor] += 1;
        }
        neighbors.par_sort_unstable();
        neighbors.dedup();
        ordering::update_neighbor_heuristics(
            neighbors,
            &mut heuristics,
            &deleted_neighbors,
            &shortcut_id,
            rank,
            amount_nodes,
            &edges,
            &up_offset,
            &down_offset,
            &down_index,
        );

        // sort in reverse order for removing from bottom up
        connected_edges.par_sort_by_key(|&edge| Reverse(edge));
        // insert E into remaining graph
        for edge_id in connected_edges.iter() {
            resulting_edges.push(edges.swap_remove(*edge_id));
        }

        // add new shortcuts to edges
        edges.par_extend(&shortcuts);

        // recalc edge-indices
        *down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);

        // move I to their Level
        for node in &minimas {
            nodes[*node].rank = rank;
            remaining_nodes.remove(&node);
        }
        rank += 1;
        if remaining_nodes.len() > 100_000 {
            println!("rest time in: {:?}", other_time.elapsed());
        }

        println!(
            "remaining_nodes {:?} \tindependent_set.len {:?} \tedges.len {:?} \tshortcuts.len {:?} \tremoving_edges.len {:?} \tresulting_edges.len {:?}",
            remaining_nodes.len(),
            minimas.len(),
            edges.len(),
            shortcuts.len(),
            connected_edges.len(),
            resulting_edges.len()
        );
    }
    println!("max_rank: {:?}", rank);

    // remove never used edges
    remove_redundant_edges(
        &mut resulting_edges,
        &mut up_offset,
        &mut down_offset,
        &mut down_index,
        amount_nodes,
    );

    // testing uniqueness of ids
    let unique_set: BTreeSet<usize> = edges.iter().cloned().map(|e| e.id.unwrap()).collect();
    assert_eq!(unique_set.len(), edges.len());

    *edges = resulting_edges;
    // and calculate the offsets
    *down_index =
        offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);

    // sort edges from top to down ranks for bidijkstra
    sort_edges_ranked(&mut edges, &down_offset, &mut down_index, &nodes);

    // revert the ids back to usual ids
    revert_indices(&mut edges);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// return new generated shortcuts
    pub fn contract_single_node(
        node: NodeId,
        mut edges: &mut Vec<Way>,
        mut up_offset: &mut Vec<EdgeId>,
        mut down_offset: &mut Vec<EdgeId>,
        mut down_index: &mut Vec<EdgeId>,
        mut dijkstra: &mut dijkstra::Dijkstra,
        resulting_edges: &mut Vec<Way>,
        amount_nodes: usize,
        shortcut_id: &AtomicUsize,
        rank: usize,
    ) {
        let shortcuts = calc_shortcuts(
            node,
            &mut dijkstra,
            &mut edges,
            &mut up_offset,
            &mut down_offset,
            &mut down_index,
            &shortcut_id,
            rank,
        );

        // get all connected edges of one node
        let mut connected_edges =
            graph_helper::get_all_edge_ids(node, &up_offset, &down_offset, &down_index);

        // sort reverse for iterating from bottom up
        connected_edges.sort_by_key(|&edge| Reverse(edge));
        // all connected nodes are moved to remaining_nodes
        for edge_id in connected_edges.iter() {
            resulting_edges.push(edges.swap_remove(*edge_id));
        }
        // add new shortcuts
        edges.par_extend(&shortcuts);
        // recalc edge-indices
        *down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);
    }

    #[test]
    fn calc_shortcuts_no_witness() {
        // 0 -> 2 -> 3
        // 1 ->/ \-> 4
        let amount_nodes = 5;

        let mut edges = Vec::<Way>::new();
        edges.push(Way::test(0, 2, 1, 0));
        edges.push(Way::test(1, 2, 2, 1));
        edges.push(Way::test(2, 3, 3, 2));
        edges.push(Way::test(2, 4, 1, 3));

        let shortcut_id = AtomicUsize::new(edges.len());

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        let down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);
        let mut dijkstra: dijkstra::Dijkstra = dijkstra::Dijkstra::new(amount_nodes);
        let shortcuts = calc_shortcuts(
            2,
            &mut dijkstra,
            &edges,
            &up_offset,
            &down_offset,
            &down_index,
            &shortcut_id,
            0,
        );

        let expected_shortcuts = vec![
            Way::shortcut(0, 3, 4, 0, 2, 4),
            Way::shortcut(0, 4, 2, 0, 3, 5),
            Way::shortcut(1, 3, 5, 1, 2, 6),
            Way::shortcut(1, 4, 3, 1, 3, 7),
        ];
        assert_eq!(expected_shortcuts, shortcuts);
    }

    #[test]
    fn calc_shortcuts_witness() {
        // 0 -> 1 -> 2
        //  \-> 3 ->/
        let amount_nodes = 4;

        let mut edges = Vec::<Way>::new();
        edges.push(Way::test(0, 1, 1, 0));
        edges.push(Way::test(1, 2, 1, 2));
        edges.push(Way::test(0, 3, 1, 1));
        edges.push(Way::test(3, 2, 1, 3));

        let shortcut_id = AtomicUsize::new(edges.len());

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        let down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);
        let mut dijkstra: dijkstra::Dijkstra = dijkstra::Dijkstra::new(amount_nodes);
        let shortcuts = calc_shortcuts(
            1,
            &mut dijkstra,
            &edges,
            &up_offset,
            &down_offset,
            &down_index,
            &shortcut_id,
            0,
        );

        let expected_shortcuts = vec![Way::shortcut(0, 2, 2, 0, 2, 4)];
        assert_eq!(expected_shortcuts, shortcuts);
    }

    #[test]
    fn calc_shortcuts_witness_via_center() {
        // 0 -> 1 -> 2
        // |  /
        // 3 -
        let amount_nodes = 4;

        let mut edges = Vec::<Way>::new();
        edges.push(Way::test(0, 1, 10, 0));
        edges.push(Way::test(0, 3, 1, 1));
        edges.push(Way::test(1, 2, 1, 2));
        edges.push(Way::test(3, 1, 1, 3));

        let shortcut_id = AtomicUsize::new(edges.len());

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        let down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);
        let mut dijkstra: dijkstra::Dijkstra = dijkstra::Dijkstra::new(amount_nodes);
        let shortcuts = calc_shortcuts(
            1,
            &mut dijkstra,
            &edges,
            &up_offset,
            &down_offset,
            &down_index,
            &shortcut_id,
            0,
        );

        // no need for a shortcut 0->1->2, because there is already the shortcut 3->1->2
        let expected_shortcuts = vec![Way::shortcut(3, 2, 2, 3, 2, 4)];
        assert_eq!(expected_shortcuts, shortcuts);
    }

    #[test]
    fn contract_simple_node() {
        // 0 -> 1 -> 2
        // |  /   \  |
        // 3 --->--- 4
        let amount_nodes = 5;

        let mut edges = Vec::<Way>::new();
        edges.push(Way::test(0, 1, 1, 0));
        edges.push(Way::test(1, 2, 1, 2));
        edges.push(Way::test(0, 3, 1, 1));
        edges.push(Way::test(3, 1, 5, 4));
        edges.push(Way::test(1, 4, 4, 3));
        edges.push(Way::test(3, 4, 3, 5));
        edges.push(Way::test(4, 2, 1, 6));

        let shortcut_id = AtomicUsize::new(edges.len());

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        let down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);
        let mut dijkstra: dijkstra::Dijkstra = dijkstra::Dijkstra::new(amount_nodes);
        let shortcuts = calc_shortcuts(
            1,
            &mut dijkstra,
            &edges,
            &up_offset,
            &down_offset,
            &down_index,
            &shortcut_id,
            0,
        );

        // there should be a shortcut 0->2, but no shortcuts 0->4, 3->2
        let expected_shortcuts = vec![Way::shortcut(0, 2, 2, 0, 2, 7)];
        assert_eq!(expected_shortcuts, shortcuts);
    }

    #[test]
    fn contract_triangle() {
        //   1
        //  / \
        // 0---2
        let amount_nodes = 3;

        let mut edges = Vec::<Way>::new();
        edges.push(Way::new(0, 1, 1));
        edges.push(Way::new(0, 2, 1));
        edges.push(Way::new(1, 0, 1));
        edges.push(Way::new(1, 2, 1));
        edges.push(Way::new(2, 0, 1));
        edges.push(Way::new(2, 1, 1));

        let shortcut_id = AtomicUsize::new(edges.len());

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        let down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);
        let mut dijkstra: dijkstra::Dijkstra = dijkstra::Dijkstra::new(amount_nodes);
        let shortcuts = calc_shortcuts(
            1,
            &mut dijkstra,
            &edges,
            &up_offset,
            &down_offset,
            &down_index,
            &shortcut_id,
            0,
        );

        let expected_shortcuts: Vec<Way> = vec![];
        assert_eq!(expected_shortcuts, shortcuts);
    }

    #[test]
    fn contract_order() {
        // 0 -> 1 -> 2
        // |  /   \  |
        // 3 --->--- 4
        let amount_nodes = 5;

        let mut edges = Vec::<Way>::new();
        edges.push(Way::test(0, 1, 1, 0));
        edges.push(Way::test(1, 2, 1, 2));
        edges.push(Way::test(0, 3, 1, 1));
        edges.push(Way::test(3, 1, 5, 4));
        edges.push(Way::test(1, 4, 4, 3));
        edges.push(Way::test(3, 4, 3, 5));
        edges.push(Way::test(4, 2, 1, 6));

        let shortcut_id = AtomicUsize::new(edges.len());

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        let down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);
        let mut dijkstra: dijkstra::Dijkstra = dijkstra::Dijkstra::new(amount_nodes);
        let shortcuts = calc_shortcuts(
            1,
            &mut dijkstra,
            &edges,
            &up_offset,
            &down_offset,
            &down_index,
            &shortcut_id,
            0,
        );

        let expected_shortcuts = vec![Way::shortcut(0, 2, 2, 0, 2, 7)];
        assert_eq!(expected_shortcuts, shortcuts);

        let shortcuts = calc_shortcuts(
            1,
            &mut dijkstra,
            &edges,
            &up_offset,
            &down_offset,
            &down_index,
            &shortcut_id,
            1,
        );

        let expected_shortcuts = vec![Way::shortcut(0, 2, 2, 0, 2, 8)];
        assert_eq!(expected_shortcuts, shortcuts);

        let shortcuts = calc_shortcuts(
            3,
            &mut dijkstra,
            &edges,
            &up_offset,
            &down_offset,
            &down_index,
            &shortcut_id,
            1,
        );

        let expected_shortcuts = vec![Way::shortcut(0, 4, 4, 1, 5, 9)];
        assert_eq!(expected_shortcuts, shortcuts);

        let shortcuts = calc_shortcuts(
            1,
            &mut dijkstra,
            &edges,
            &up_offset,
            &down_offset,
            &down_index,
            &shortcut_id,
            2,
        );

        let expected_shortcuts = vec![Way::shortcut(0, 2, 2, 0, 2, 10)];
        assert_eq!(expected_shortcuts, shortcuts);
    }

    #[test]
    fn contract_disconnect_small() {
        // --->4---3
        // |   |   |
        // 2   |   |
        // |   |   |
        // --->0---1

        let amount_nodes = 6;

        let mut edges = Vec::<Way>::new();
        edges.push(Way::test(0, 1, 4, 0));
        edges.push(Way::test(0, 4, 1, 1));
        edges.push(Way::test(1, 0, 1, 2));
        edges.push(Way::test(1, 3, 1, 3));
        edges.push(Way::test(2, 0, 1, 4));
        edges.push(Way::test(2, 4, 3, 5));
        edges.push(Way::test(3, 1, 1, 6));
        edges.push(Way::test(3, 4, 4, 7));
        edges.push(Way::test(4, 0, 1, 8));
        edges.push(Way::test(4, 3, 1, 9));

        let amount_edges = edges.len();
        let shortcut_id = AtomicUsize::new(amount_edges);

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        let mut down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);
        let mut dijkstra: dijkstra::Dijkstra = dijkstra::Dijkstra::new(amount_nodes);

        let mut resulting_edges = Vec::<Way>::new();
        contract_single_node(
            0,
            &mut edges,
            &mut up_offset,
            &mut down_offset,
            &mut down_index,
            &mut dijkstra,
            &mut resulting_edges,
            amount_edges,
            &shortcut_id,
            0,
        );
        let mut expected_edges = Vec::<Way>::new();

        expected_edges.push(Way::test(1, 3, 1, 3));
        expected_edges.push(Way::shortcut(1, 4, 2, 2, 1, 10));
        expected_edges.push(Way::shortcut(2, 4, 2, 4, 1, 11));
        expected_edges.push(Way::test(2, 4, 3, 5));
        expected_edges.push(Way::test(3, 1, 1, 6));
        expected_edges.push(Way::test(3, 4, 4, 7));
        expected_edges.push(Way::test(4, 3, 1, 9));

        let mut expected_resulting_edges = Vec::<Way>::new();
        expected_resulting_edges.push(Way::test(4, 0, 1, 8));
        expected_resulting_edges.push(Way::test(2, 0, 1, 4));
        expected_resulting_edges.push(Way::test(1, 0, 1, 2));
        expected_resulting_edges.push(Way::test(0, 4, 1, 1));
        expected_resulting_edges.push(Way::test(0, 1, 4, 0));

        assert_eq!(edges, expected_edges);
        assert_eq!(resulting_edges, expected_resulting_edges);

        let max_id = edges
            .par_iter()
            .map(|node| node.id.unwrap())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();
        assert_eq!(
            expected_resulting_edges.len() + expected_edges.len(),
            max_id + 1
        );
    }

    #[test]
    fn contract_disconnect_full() {
        //      7 -> 8 -> 9
        //      |         |
        // 0 -> 5 -> 6 -  |
        // |         |  \ |
        // 1 -> 2 -> 3 -> 4

        let amount_nodes = 10;

        let mut edges = Vec::<Way>::new();
        edges.push(Way::test(0, 1, 1, 4));
        edges.push(Way::test(1, 2, 1, 3));
        edges.push(Way::test(2, 3, 1, 2));
        edges.push(Way::test(3, 4, 20, 1));
        edges.push(Way::test(0, 5, 5, 0));
        edges.push(Way::test(5, 6, 1, 9));
        edges.push(Way::test(6, 4, 20, 8));
        edges.push(Way::test(6, 3, 20, 7));
        edges.push(Way::test(5, 7, 5, 6));
        edges.push(Way::test(7, 8, 1, 5));
        edges.push(Way::test(8, 9, 1, 11));
        edges.push(Way::test(9, 4, 1, 10));

        let amount_edges = edges.len();
        let shortcut_id = AtomicUsize::new(amount_edges);

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        let mut down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);
        let mut dijkstra: dijkstra::Dijkstra = dijkstra::Dijkstra::new(amount_nodes);

        let mut resulting_edges = Vec::<Way>::new();

        let contraction_order = vec![6, 5, 3, 8];
        for node in contraction_order {
            contract_single_node(
                node,
                &mut edges,
                &mut up_offset,
                &mut down_offset,
                &mut down_index,
                &mut dijkstra,
                &mut resulting_edges,
                amount_edges,
                &shortcut_id,
                0,
            );
        }
        let mut expected_edges = Vec::<Way>::new();
        expected_edges.push(Way::test(0, 1, 1, 4));
        expected_edges.push(Way::shortcut(0, 7, 10, 0, 6, 13));
        expected_edges.push(Way::test(1, 2, 1, 3));
        expected_edges.push(Way::shortcut(2, 4, 21, 2, 1, 14));
        expected_edges.push(Way::shortcut(7, 9, 2, 5, 11, 15));
        expected_edges.push(Way::test(9, 4, 1, 10));

        let mut expected_resulting_edges = Vec::<Way>::new();
        expected_resulting_edges.push(Way::test(6, 4, 20, 8));
        expected_resulting_edges.push(Way::test(6, 3, 20, 7));
        expected_resulting_edges.push(Way::test(5, 6, 1, 9));
        expected_resulting_edges.push(Way::test(5, 7, 5, 6));
        expected_resulting_edges.push(Way::shortcut(5, 3, 21, 9, 7, 12));
        expected_resulting_edges.push(Way::test(0, 5, 5, 0));
        expected_resulting_edges.push(Way::test(3, 4, 20, 1));
        expected_resulting_edges.push(Way::test(2, 3, 1, 2));
        expected_resulting_edges.push(Way::test(8, 9, 1, 11));
        expected_resulting_edges.push(Way::test(7, 8, 1, 5));

        assert_eq!(edges, expected_edges);
        assert_eq!(resulting_edges, expected_resulting_edges);

        let new_contraction_order = vec![1, 0, 9, 4, 7, 2];
        for node in new_contraction_order {
            contract_single_node(
                node,
                &mut edges,
                &mut up_offset,
                &mut down_offset,
                &mut down_index,
                &mut dijkstra,
                &mut resulting_edges,
                amount_edges,
                &shortcut_id,
                1,
            );
        }

        expected_resulting_edges.push(Way::test(1, 2, 1, 3));
        expected_resulting_edges.push(Way::test(0, 1, 1, 4));
        expected_resulting_edges.push(Way::shortcut(0, 7, 10, 0, 6, 13));
        expected_resulting_edges.push(Way::shortcut(0, 2, 2, 4, 3, 16));
        expected_resulting_edges.push(Way::test(9, 4, 1, 10));
        expected_resulting_edges.push(Way::shortcut(7, 9, 2, 5, 11, 15));
        expected_resulting_edges.push(Way::shortcut(7, 4, 3, 15, 10, 17));
        expected_resulting_edges.push(Way::shortcut(2, 4, 21, 2, 1, 14));

        assert_eq!(edges, vec![]);
        assert_eq!(resulting_edges, expected_resulting_edges);

        let max_id = resulting_edges
            .par_iter()
            .map(|node| node.id.unwrap())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();
        assert_eq!(expected_resulting_edges.len(), max_id + 1);
    }

    #[test]
    fn remove_redundant_test() {
        //   1
        //  / \
        // 0---2

        let amount_nodes = 3;
        let mut edges = Vec::<Way>::new();
        edges.push(Way::test(0, 1, 13, 0));
        edges.push(Way::shortcut(0, 2, 25, 0, 1, 2));
        edges.push(Way::test(0, 2, 26, 3));
        edges.push(Way::test(1, 2, 12, 1));

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        let mut down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);

        let mut expected_edges = Vec::<Way>::new();
        expected_edges.push(Way::test(0, 1, 13, 0));
        expected_edges.push(Way::shortcut(0, 2, 25, 0, 1, 2));
        expected_edges.push(Way::test(1, 2, 12, 1));

        remove_redundant_edges(
            &mut edges,
            &mut up_offset,
            &mut down_offset,
            &mut down_index,
            amount_nodes,
        );

        for (i, way) in edges.iter().enumerate() {
            println!("{:?} {:?}", i, way);
        }
        for (i, way) in expected_edges.iter().enumerate() {
            println!("{:?} {:?}", i, way);
        }

        assert_eq!(edges, expected_edges);
    }

    #[test]
    fn sort_edges_ranked_test() {
        //      7 -> 8 -> 9
        //      |         |
        // 0 -> 5 -> 6 -  |
        // |         |  \ |
        // 1 -> 2 -> 3 -> 4

        let mut nodes = Vec::new();
        nodes.push(Node {
            latitude: 0.0,
            longitude: 0.0,
            rank: 1,
        });
        nodes.push(Node {
            latitude: 0.0,
            longitude: 0.0,
            rank: 0,
        });
        nodes.push(Node {
            latitude: 0.0,
            longitude: 0.0,
            rank: 2,
        });
        nodes.push(Node {
            latitude: 0.0,
            longitude: 0.0,
            rank: 4,
        });
        nodes.push(Node {
            latitude: 0.0,
            longitude: 0.0,
            rank: 5,
        });
        nodes.push(Node {
            latitude: 0.0,
            longitude: 0.0,
            rank: 3,
        });
        nodes.push(Node {
            latitude: 0.0,
            longitude: 0.0,
            rank: 9,
        });
        nodes.push(Node {
            latitude: 0.0,
            longitude: 0.0,
            rank: 6,
        });
        nodes.push(Node {
            latitude: 0.0,
            longitude: 0.0,
            rank: 8,
        });
        nodes.push(Node {
            latitude: 0.0,
            longitude: 0.0,
            rank: 7,
        });

        let amount_nodes = nodes.len();
        let mut edges = Vec::<Way>::new();
        edges.push(Way::test(6, 4, 20, 8));
        edges.push(Way::test(6, 3, 20, 7));
        edges.push(Way::test(5, 6, 1, 9));
        edges.push(Way::test(5, 7, 5, 6));
        edges.push(Way::shortcut(5, 3, 21, 9, 7, 12));
        edges.push(Way::test(0, 5, 5, 0));
        edges.push(Way::test(3, 4, 20, 1));
        edges.push(Way::test(2, 3, 1, 2));
        edges.push(Way::test(8, 9, 1, 11));
        edges.push(Way::test(7, 8, 1, 5));
        edges.push(Way::test(1, 2, 1, 3));
        edges.push(Way::test(0, 1, 1, 4));
        edges.push(Way::shortcut(0, 7, 10, 0, 6, 13));
        edges.push(Way::shortcut(0, 2, 2, 4, 3, 16));
        edges.push(Way::test(9, 4, 1, 10));
        edges.push(Way::shortcut(7, 9, 2, 5, 11, 15));
        edges.push(Way::shortcut(7, 4, 3, 15, 10, 17));
        edges.push(Way::shortcut(2, 4, 21, 2, 1, 14));

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        let mut down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);

        //check edges and down_index

        let mut expected_edges = Vec::<Way>::new();

        expected_edges.push(Way::shortcut(0, 7, 10, 0, 6, 13));
        expected_edges.push(Way::test(0, 5, 5, 0));
        expected_edges.push(Way::shortcut(0, 2, 2, 4, 3, 16));
        expected_edges.push(Way::test(0, 1, 1, 4));
        expected_edges.push(Way::test(1, 2, 1, 3));
        expected_edges.push(Way::shortcut(2, 4, 21, 2, 1, 14));
        expected_edges.push(Way::test(2, 3, 1, 2));
        expected_edges.push(Way::test(3, 4, 20, 1));
        expected_edges.push(Way::test(5, 6, 1, 9));
        expected_edges.push(Way::test(5, 7, 5, 6));
        expected_edges.push(Way::shortcut(5, 3, 21, 9, 7, 12));
        expected_edges.push(Way::test(6, 4, 20, 8));
        expected_edges.push(Way::test(6, 3, 20, 7));
        expected_edges.push(Way::test(7, 8, 1, 5));
        expected_edges.push(Way::shortcut(7, 9, 2, 5, 11, 15));
        expected_edges.push(Way::shortcut(7, 4, 3, 15, 10, 17));
        expected_edges.push(Way::test(8, 9, 1, 11));
        expected_edges.push(Way::test(9, 4, 1, 10));

        let expected_down_index =
            vec![3, 2, 4, 12, 10, 6, 11, 17, 15, 7, 5, 1, 8, 9, 0, 13, 16, 14];

        sort_edges_ranked(&mut edges, &down_offset, &mut down_index, &nodes);

        assert_eq!(edges, expected_edges);
        assert_eq!(down_index, expected_down_index);
    }

    #[test]
    fn revert_indices_test() {
        //      7 -> 8 -> 9
        //      |         |
        // 0 -> 5 -> 6 -  |
        // |         |  \ |
        // 1 -> 2 -> 3 -> 4

        let amount_nodes = 10;
        let mut edges = Vec::<Way>::new();
        edges.push(Way::test(6, 4, 20, 8));
        edges.push(Way::test(6, 3, 20, 7));
        edges.push(Way::test(5, 6, 1, 9));
        edges.push(Way::test(5, 7, 5, 6));
        edges.push(Way::shortcut(5, 3, 21, 9, 7, 12));
        edges.push(Way::test(0, 5, 5, 0));
        edges.push(Way::test(3, 4, 20, 1));
        edges.push(Way::test(2, 3, 1, 2));
        edges.push(Way::test(8, 9, 1, 11));
        edges.push(Way::test(7, 8, 1, 5));
        edges.push(Way::test(1, 2, 1, 3));
        edges.push(Way::test(0, 1, 1, 4));
        edges.push(Way::shortcut(0, 7, 10, 0, 6, 13));
        edges.push(Way::shortcut(0, 2, 2, 4, 3, 16));
        edges.push(Way::test(9, 4, 1, 10));
        edges.push(Way::shortcut(7, 9, 2, 5, 11, 15));
        edges.push(Way::shortcut(7, 4, 3, 15, 10, 17));
        edges.push(Way::shortcut(2, 4, 21, 2, 1, 14));

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        let _down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);

        let mut expected_edges = Vec::<Way>::new();
        expected_edges.push(Way::test(0, 1, 1, 4));
        expected_edges.push(Way::shortcut(0, 2, 2, 0, 4, 16));
        expected_edges.push(Way::test(0, 5, 5, 0));
        expected_edges.push(Way::shortcut(0, 7, 10, 2, 10, 13));
        expected_edges.push(Way::test(1, 2, 1, 3));
        expected_edges.push(Way::test(2, 3, 1, 2));
        expected_edges.push(Way::shortcut(2, 4, 21, 5, 7, 14));
        expected_edges.push(Way::test(3, 4, 20, 1));
        expected_edges.push(Way::shortcut(5, 3, 21, 9, 11, 12));
        expected_edges.push(Way::test(5, 6, 1, 9));
        expected_edges.push(Way::test(5, 7, 5, 6));
        expected_edges.push(Way::test(6, 3, 20, 7));
        expected_edges.push(Way::test(6, 4, 20, 8));
        expected_edges.push(Way::shortcut(7, 4, 3, 15, 17, 17));
        expected_edges.push(Way::test(7, 8, 1, 5));
        expected_edges.push(Way::shortcut(7, 9, 2, 14, 16, 15));
        expected_edges.push(Way::test(8, 9, 1, 11));
        expected_edges.push(Way::test(9, 4, 1, 10));

        revert_indices(&mut edges);

        assert_eq!(edges, expected_edges);
    }
}
