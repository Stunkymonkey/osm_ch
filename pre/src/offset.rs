use super::*;

/// fill offset array
fn fill_offset(edges: Vec<NodeId>, offset: &mut Vec<usize>) {
    // TODO check two concated for loops is faster
    for edge in edges {
        offset[edge + 1] += 1;
    }
    for i in 1..offset.len() {
        offset[i] += offset[i - 1];
    }
}

pub fn generate_offsets(
    edges: &mut Vec<Way>,
    mut up_offset: &mut Vec<EdgeId>,
    mut down_offset: &mut Vec<EdgeId>,
    amount_nodes: usize,
) -> Vec<EdgeId> {
    up_offset.resize(amount_nodes + 1, 0);
    down_offset.resize(amount_nodes + 1, 0);

    // generate up edges
    edges.par_sort_unstable();
    let sources: Vec<EdgeId> = edges.par_iter().map(|x| x.source).rev().collect();
    fill_offset(sources, &mut up_offset);

    // generate down edges, but without sorting edges
    // first collect offsets
    let targets: Vec<EdgeId> = edges.par_iter().map(|x| x.target).rev().collect();
    fill_offset(targets, &mut down_offset);
    let mut down_index = vec![0; edges.len()];
    // fill offsets, where not already filled
    // TODO parallel?
    for (i, edge) in edges.iter().enumerate() {
        let start_index = down_offset[edge.target];
        let end_index = down_offset[edge.target + 1];
        for j in start_index..end_index {
            if down_index[j] == 0 {
                down_index[j] = i;
                break;
            }
        }
    }
    return down_index;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_offset_test() {
        let mut offset_test = vec![0; 8];
        let mut edges = vec![0, 0, 0, 2, 3, 4, 4, 4, 6];
        fill_offset(edges, &mut offset_test);

        println!("{:?}", offset_test);

        assert_eq!(offset_test[0], 0);
        assert_eq!(offset_test[1], 3);
        assert_eq!(offset_test[2], 3);
        assert_eq!(offset_test[3], 4);
        assert_eq!(offset_test[4], 5);
        assert_eq!(offset_test[5], 8);
        assert_eq!(offset_test[6], 8);
        assert_eq!(offset_test[7], 9);
    }

    // TODO make good test
    #[test]
    fn all_offsets() {
        let mut up_offset_test = vec![0; 8];
        let mut down_offset_test = vec![0; 8];
        let mut edges = Vec::<Way>::new();
        edges.push(Way {
            source: 0,
            target: 2,
            weight: 0,
            contrated_previous: None,
            contrated_next: None,
        });
        edges.push(Way {
            source: 1,
            target: 2,
            weight: 0,
            contrated_previous: None,
            contrated_next: None,
        });
        edges.push(Way {
            source: 1,
            target: 5,
            weight: 0,
            contrated_previous: None,
            contrated_next: None,
        });
        edges.push(Way {
            source: 4,
            target: 5,
            weight: 0,
            contrated_previous: None,
            contrated_next: None,
        });
        edges.push(Way {
            source: 0,
            target: 5,
            weight: 0,
            contrated_previous: None,
            contrated_next: None,
        });
        let down_index_test =
            generate_offsets(&mut edges, &mut up_offset_test, &mut down_offset_test, 8);

        println!("{:?}", up_offset_test);

        for i in 0..edges.len() {
            assert_eq!(
                down_index_test[down_offset_test[edges[up_offset_test[i]].target]],
                up_offset_test[i]
            );
        }
    }
}
