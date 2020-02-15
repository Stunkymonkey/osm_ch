use super::*;

/// fill offset array
fn fill_offset(edges: Vec<NodeId>, offset: &mut Vec<usize>) {
    for edge in edges {
        offset[edge + 1] += 1;
    }
    for i in 1..offset.len() {
        offset[i] += offset[i - 1];
    }
}

/// make sure edges are already sorted!!
pub fn generate_offsets_unstable(
    edges: &mut Vec<Way>,
    mut up_offset: &mut Vec<EdgeId>,
    mut down_offset: &mut Vec<EdgeId>,
    amount_nodes: usize,
) -> Vec<EdgeId> {
    up_offset.clear();
    up_offset.resize(amount_nodes + 1, 0);
    down_offset.clear();
    down_offset.resize(amount_nodes + 1, 0);

    // generate up edges
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

pub fn generate_offsets(
    edges: &mut Vec<Way>,
    up_offset: &mut Vec<EdgeId>,
    down_offset: &mut Vec<EdgeId>,
    amount_nodes: usize,
) -> Vec<EdgeId> {
    edges.par_sort_unstable();
    return generate_offsets_unstable(edges, up_offset, down_offset, amount_nodes);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_offset_test() {
        let mut offset = vec![0; 8];
        let edges = vec![0, 0, 0, 2, 3, 4, 4, 4, 6];
        fill_offset(edges, &mut offset);

        assert_eq!(offset[0], 0);
        assert_eq!(offset[1], 3);
        assert_eq!(offset[2], 3);
        assert_eq!(offset[3], 4);
        assert_eq!(offset[4], 5);
        assert_eq!(offset[5], 8);
        assert_eq!(offset[6], 8);
        assert_eq!(offset[7], 9);
    }

    #[test]
    fn all_offsets() {
        //      7 -> 8 -> 9
        //      |         |
        // 0 -> 5 -> 6 -  |
        // |         |  \ |
        // 1 -> 2 -> 3 -> 4

        let amount_nodes = 10;

        let mut edges = Vec::<Way>::new();
        edges.push(Way::new(0, 1, 1));
        edges.push(Way::new(1, 2, 1));
        edges.push(Way::new(2, 3, 1));
        edges.push(Way::new(3, 4, 20));
        edges.push(Way::new(0, 5, 5));
        edges.push(Way::new(5, 6, 1));
        edges.push(Way::new(6, 4, 20));
        edges.push(Way::new(6, 3, 20));
        edges.push(Way::new(5, 7, 5));
        edges.push(Way::new(7, 8, 1));
        edges.push(Way::new(8, 9, 1));
        edges.push(Way::new(9, 4, 1));

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        edges.par_sort_unstable();
        let down_index =
            offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);

        println!("{:?}", up_offset);

        assert_eq!(up_offset[0], 0);
        assert_eq!(up_offset[1], 2);
        assert_eq!(up_offset[2], 3);
        assert_eq!(up_offset[3], 4);
        assert_eq!(up_offset[4], 5);
        assert_eq!(up_offset[5], 5);
        assert_eq!(up_offset[6], 7);
        assert_eq!(up_offset[7], 9);
        assert_eq!(up_offset[8], 10);
        assert_eq!(up_offset[9], 11);
        assert_eq!(up_offset[10], 12);

        assert_eq!(down_offset[0], 0);
        assert_eq!(down_offset[1], 0);
        assert_eq!(down_offset[2], 1);
        assert_eq!(down_offset[3], 2);
        assert_eq!(down_offset[4], 4);
        assert_eq!(down_offset[5], 7);
        assert_eq!(down_offset[6], 8);
        assert_eq!(down_offset[7], 9);
        assert_eq!(down_offset[8], 10);
        assert_eq!(down_offset[9], 11);
        assert_eq!(down_offset[10], 12);

        assert_eq!(down_index[0], 0);
        assert_eq!(down_index[1], 2);
        assert_eq!(down_index[2], 3);
        assert_eq!(down_index[3], 7);
        assert_eq!(down_index[4], 4);
        assert_eq!(down_index[5], 8);
        assert_eq!(down_index[6], 11);
        assert_eq!(down_index[7], 1);
        assert_eq!(down_index[8], 5);
        assert_eq!(down_index[9], 6);
        assert_eq!(down_index[10], 9);
        assert_eq!(down_index[11], 10);
    }
}
