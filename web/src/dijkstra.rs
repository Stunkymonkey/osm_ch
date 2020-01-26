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
}

impl Dijkstra {
    /// general constructor
    pub fn new(amount_nodes: usize) -> Self {
        let heap = BinaryHeap::new();
        let visited = VisitedList::new(amount_nodes);
        let dist = vec![(std::usize::MAX, None); amount_nodes];
        Dijkstra {
            dist: dist,
            visited: visited,
            heap: heap,
            start_node: INVALID_NODE,
        }
    }

    pub fn find_path(&mut self, start: NodeId, end: NodeId) {
        println!("{:?} {:?}", start, end);
    }
}
