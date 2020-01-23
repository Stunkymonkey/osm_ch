use super::*;

/// return new generated shortcuts
pub fn contract_node(
    node: NodeId,
    edges: &Vec<Way>,
    up_offset: &Vec<EdgeId>,
    down_offset: &Vec<EdgeId>,
    down_index: &Vec<EdgeId>,
    dijkstra: &mut dijkstra::Dijkstra,
) -> (Vec<Way>, Vec<EdgeId>) {
    dijkstra.avoid_node(node);
    let mut shortcuts = Vec::<Way>::new();
    let mut used_edges = Vec::<EdgeId>::new();
    // get node neighbors
    let source_indexes: Vec<EdgeId> = (down_offset[node]..down_offset[node + 1]).collect();
    let source_edges: Vec<EdgeId> = source_indexes.par_iter().map(|x| down_index[*x]).collect();
    let target_edges: Vec<EdgeId> = (up_offset[node]..up_offset[node + 1]).collect();

    for source_edge in source_edges {
        let source_node = edges[source_edge].source;
        for target_edge in &target_edges {
            let target_node = edges[*target_edge].target;
            let weight = edges[source_edge].weight + edges[*target_edge].weight;
            // debug
            // println!("s_e:{:?}, t_e:{:?}", source_edge, target_edge);
            // println!("s_n:{:?}, t_n:{:?}", source_node, target_node);
            // TODO check if this is neede, dijkstra should get rid of it anyway
            if source_node == target_node {
                continue;
            }
            // prevent dijkstra from running on whole graph
            dijkstra.set_max_weight(weight);
            let shortest_path = dijkstra.find_path(source_node, target_node, up_offset, edges);
            // create new shortcut where no other path is found
            if shortest_path.is_none() {
                shortcuts.push(Way {
                    source: source_node,
                    target: target_node,
                    weight: weight,
                    contrated_previous: Some(node),
                    contrated_next: None,
                });
                used_edges.push(source_edge);
                used_edges.push(*target_edge);
            }
        }
    }
    return (shortcuts, used_edges);
}

pub fn run_contraction(
    nodes: &mut Vec<Node>,
    edges: &mut Vec<Way>,
    up_offset: &mut Vec<EdgeId>,
    down_offset: &mut Vec<EdgeId>,
    down_index: &mut Vec<EdgeId>,) {
    let mut resulting_edges = Vec::<Way>::new();

    // ordering
    let tmp: Vec<NodeId> = (0..nodes.len()).collect();
    let mut remaining_nodes: HashSet<NodeId> = HashSet::from_iter(tmp.iter().cloned());
    let mut heuristic = vec![0; nodes.len()];

    let mut local_minima =
        ordering::get_local_minima(&heuristic, &up_offset, &down_offset, &down_index);

    // K_NEIGHBORS

    // mark all neighbors as invalid
    // partition = let (even, odd): (Vec<i32>, Vec<i32>) = a.par_iter().partition(|&n| n % 2 == 0);

    // let evens = numbers.drain_filter(|x| *x % 2 == 0).collect::<Vec<_>>();
    // let odds = numbers;

    let mut dijkstra: dijkstra::Dijkstra = dijkstra::Dijkstra::new(nodes.len());

    for node in 0..nodes.len() {
        let (shortcuts, used_edges) = contraction::contract_node(
            node,
            &edges,
            &up_offset,
            &down_offset,
            &down_index,
            &mut dijkstra,
        );
        // for edge_id in used_edges.iter().rev() {
        //     resulting_edges.push(edges[edge_id]);
        //     edges.remove(edge_id);
        // }
        // TODO remove node/edges and reduce edges from remaining graph
    }

    // while:
    // (re)calculate heuristic
    // get all minimas
    // calculate independent set via local minimas
    //      pick local minimum
    //      mark all neighbors as invalid
    //      ...
    // contract all valid nodes
    // collect shortcuts
    // rebuild graph with new shortcuts

    /*
    Update Priorities of all Nodes with Simulated Contractions
    while Remaining Graph not Empty do
        I ← Independent Node Set
        E ← Necessary Shortcuts
        Move I to their Level
        Insert E into Remaining graph
        Update Priority of Neighbors of I with Simulated Contractions
    end while
    */
}