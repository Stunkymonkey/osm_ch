use constants::*;
use std::cmp::Ordering;

#[derive(Copy, Clone, Eq)]
pub struct MinHeapItem {
    pub weight: Weight,
    pub node: NodeId,
}

// Manually implement Ord so we get a min-heap instead of a max-heap
impl MinHeapItem {
    pub fn new(node: NodeId, weight: Weight) -> MinHeapItem {
        if weight != weight {
            panic!("weight can not be NaN");
        }
        MinHeapItem { node, weight }
    }
}

impl PartialOrd for MinHeapItem {
    fn partial_cmp(&self, other: &MinHeapItem) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MinHeapItem {
    fn cmp(&self, other: &MinHeapItem) -> Ordering {
        self.weight.cmp(&other.weight).reverse()
    }
}

impl PartialEq for MinHeapItem {
    fn eq(&self, other: &MinHeapItem) -> bool {
        self.weight == other.weight
    }
}
