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
