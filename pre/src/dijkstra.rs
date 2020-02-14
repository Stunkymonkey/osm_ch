// based on https://rosettacode.org/wiki/Dijkstra%27s_algorithm#Rust

use super::*;
use min_heap::*;
use std::collections::BinaryHeap;
use visited_list::*;

#[derive(Clone)]
pub struct Dijkstra {
    dist: Vec<(NodeId, Option<Weight>)>,
    visited: VisitedList,
    heap: BinaryHeap<MinHeapItem>,
    // if start node stays the same no recomputation/invalidation is needed
    start_node: NodeId,
    // to keep track if graph changes while contracting
    prev_edge_len: usize,
}

impl Dijkstra {
    /// general constructor
    pub fn new(amount_nodes: usize) -> Self {
        let heap = BinaryHeap::new();
        let visited = VisitedList::new(amount_nodes);
        let dist = vec![(WEIGHT_MAX, None); amount_nodes];
        Dijkstra {
            dist: dist,
            visited: visited,
            heap: heap,
            start_node: INVALID_NODE,
            prev_edge_len: WEIGHT_MAX,
        }
    }

    /// return path of edges(!) from source to target not path of nodes!
    pub fn find_path(
        &mut self,
        start: usize,
        end: usize,
        offset: &Vec<EdgeId>,
        edges: &Vec<Way>,
        with_path: bool,
    ) -> Option<(Vec<NodeId>, usize)> {
        if start == end {
            return Some((vec![], 0));
        }
        if start != self.start_node || self.prev_edge_len != edges.len() {
            self.prev_edge_len = edges.len();
            self.heap.clear();
            self.visited.unvisit_all();
            self.heap.push(MinHeapItem::new(start, 0));
        }
        if self.visited.is_visited(end) {
            return self.resolve_path(end, &edges, with_path);
        }
        self.dist[start] = (0, None);
        self.visited.set_visited(start);
        self.start_node = start;

        while let Some(MinHeapItem { node, weight }) = self.heap.pop() {
            // found end
            if node == end {
                return self.resolve_path(end, &edges, with_path);
            }

            // node has already been visited and can be skipped
            if self.visited.is_visited(node) && weight > self.dist[node].0 {
                continue;
            }

            // iterate over neighbors
            for edge in graph_helper::get_up_edge_ids(node, &offset) {
                let current_way: Way = edges[edge];
                // calculate new costs
                let next = MinHeapItem::new(current_way.target, weight + current_way.weight);
                // add way to heap
                if !self.visited.is_visited(next.node) || next.weight < self.dist[next.node].0 {
                    self.dist[next.node] = (next.weight, Some(edge));
                    self.visited.set_visited(next.node);
                    self.heap.push(next);
                }
            }
        }
        return None;
    }

    /// recreate path, of already visited
    fn resolve_path(
        &self,
        end: NodeId,
        edges: &Vec<Way>,
        with_path: bool,
    ) -> Option<(Vec<NodeId>, usize)> {
        let weight = self.dist[end].0;
        if !with_path {
            return Some((Vec::new(), weight));
        }
        let mut path = Vec::with_capacity(self.dist.len() / 2);
        let mut current_dist = self.dist[end];
        while let Some(prev) = current_dist.1 {
            path.push(prev);
            current_dist = self.dist[edges[prev].source];
        }
        path.reverse();
        return Some((path, weight));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dijkstra_no_path() {
        // Start: 1
        // Goal: 0
        //
        // 0->1->2

        let amount_nodes = 3;

        let mut edges = Vec::<Way>::new();
        edges.push(Way::new(0, 1, 1));
        edges.push(Way::new(1, 2, 1));

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);
        let mut d: Dijkstra = Dijkstra::new(amount_nodes);
        let result = d.find_path(1, 0, &up_offset, &edges, true);

        assert!(result.is_none());
    }

    #[test]
    fn dijkstra_simple_path() {
        // Start: 0
        // Goal: 2
        //
        // 0-1->1-2->2
        //      |
        //      1
        //      |
        //      V
        //      3

        let amount_nodes = 4;

        let mut edges = Vec::<Way>::new();
        edges.push(Way::new(0, 1, 1));
        edges.push(Way::new(1, 2, 2));
        edges.push(Way::new(1, 3, 1));

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);
        let mut d: Dijkstra = Dijkstra::new(amount_nodes);
        let result = d.find_path(0, 2, &up_offset, &edges, true);

        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path.0, [0, 1]);
        assert_eq!(path.1, 3);
    }

    #[test]
    fn dijkstra_shortest_path() {
        // Start: 1
        // Goal: 3
        //
        // 0-9->1-9->2
        // |         A
        // 1         |
        // |         1
        // V         |
        // 3-1->4-1->5

        let amount_nodes = 6;

        let mut edges = Vec::<Way>::new();
        edges.push(Way::new(0, 1, 9));
        edges.push(Way::new(1, 2, 9));
        edges.push(Way::new(0, 3, 1));
        edges.push(Way::new(3, 4, 1));
        edges.push(Way::new(4, 5, 1));
        edges.push(Way::new(5, 2, 1));

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);
        let mut d: Dijkstra = Dijkstra::new(amount_nodes);
        let result = d.find_path(0, 2, &up_offset, &edges, true);

        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path.0, [1, 3, 4, 5]);
        assert_eq!(path.1, 4);
    }

    #[test]
    fn dijkstra_simple_line() {
        // 0->1->2->3
        let amount_nodes = 4;

        let mut edges = Vec::<Way>::new();
        edges.push(Way::new(0, 1, 1));
        edges.push(Way::new(1, 2, 1));
        edges.push(Way::new(2, 3, 1));

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);
        let mut d: Dijkstra = Dijkstra::new(amount_nodes);

        let result = d.find_path(3, 0, &up_offset, &edges, true);
        assert!(result.is_none());

        let result = d.find_path(0, 3, &up_offset, &edges, true);
        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path.0, [0, 1, 2]);
        assert_eq!(path.1, 3);
    }

    #[test]
    fn dijkstra_twice() {
        // 0->1->2
        let amount_nodes = 3;

        let mut edges = Vec::<Way>::new();
        edges.push(Way::new(0, 1, 1));
        edges.push(Way::new(1, 2, 1));

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);
        let mut d: Dijkstra = Dijkstra::new(amount_nodes);

        let result = d.find_path(0, 2, &up_offset, &edges, true);
        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path.0, [0, 1]);
        assert_eq!(path.1, 2);

        let result = d.find_path(0, 2, &up_offset, &edges, true);
        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path.0, [0, 1]);
        assert_eq!(path.1, 2);
    }

    #[test]
    fn dijkstra_change_edges() {
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

        let mut up_offset = Vec::<EdgeId>::new();
        let mut down_offset = Vec::<EdgeId>::new();
        offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);
        let mut d: Dijkstra = Dijkstra::new(amount_nodes);

        let result = d.find_path(0, 2, &up_offset, &edges, true);
        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path.0, [1]);
        assert_eq!(path.1, 1);

        let result = d.find_path(1, 2, &up_offset, &edges, true);
        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path.0, [3]);
        assert_eq!(path.1, 1);

        edges.remove(0);

        let result = d.find_path(0, 2, &up_offset, &edges, true);
        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path.0, [0]);
        assert_eq!(path.1, 1);

        let result = d.find_path(1, 2, &up_offset, &edges, true);
        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path.0, [2]);
        assert_eq!(path.1, 1);
    }

    #[test]
    fn dijkstra_multiple_paths() {
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
        offset::generate_offsets(&mut edges, &mut up_offset, &mut down_offset, amount_nodes);
        let mut d: Dijkstra = Dijkstra::new(amount_nodes);

        let result = d.find_path(4, 0, &up_offset, &edges, true);
        assert!(result.is_none());

        let result = d.find_path(4, 4, &up_offset, &edges, true);
        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path.0.len(), 0);
        assert_eq!(path.0, []);
        assert_eq!(path.1, 0);

        let result = d.find_path(6, 3, &up_offset, &edges, true);
        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path.0, [7]);
        assert_eq!(path.1, 20);

        let result = d.find_path(1, 4, &up_offset, &edges, true);
        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path.0, [2, 3, 4]);
        assert_eq!(path.1, 22);

        let result = d.find_path(0, 4, &up_offset, &edges, true);
        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path.0, [1, 6, 9, 10, 11]);
        assert_eq!(path.1, 13);
    }
}
