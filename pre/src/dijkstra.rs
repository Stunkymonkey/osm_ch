use constants::*;
use min_heap::*;
use std::collections::BinaryHeap;
use structs::*;
use visited_list::*;

pub struct Dijkstra {
    amount_nodes: NodeId,
    dist: Vec<(NodeId, Option<Weight>)>,
    visited: VisitedList,
    heap: BinaryHeap<MinHeapItem>,
    avoid_node: NodeId,
    max_weight: Weight,
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
            amount_nodes,
            dist: dist,
            visited: visited,
            heap: heap,
            avoid_node: INVALID_NODE,
            max_weight: WEIGHT_ZERO,
            start_node: INVALID_NODE,
        }
    }

    /// exclude node from dijkstra-path
    pub fn avoid_node(&mut self, node: NodeId) {
        self.avoid_node = node;
        self.start_node = INVALID_NODE;
    }

    /// set the maximum weight of dijkstra
    pub fn set_max_weight(&mut self, weight: Weight) {
        self.max_weight = weight;
    }

    /// dijkstra
    pub fn find_path(
        &mut self,
        start: usize,
        end: usize,
        offset: &Vec<EdgeId>,
        edges: &Vec<Way>,
    ) -> Option<(Vec<NodeId>, usize)> {
        self.dist[start] = (0, None);

        if start == end {
            panic!("start and end are the same: ({:?})", start);
        }
        if start != self.start_node {
            self.heap.clear();
            self.visited.invalidate_all();
            self.heap.push(MinHeapItem {
                weight: 0,
                node: start,
            });
        }
        if self.visited.is_visited(end) {
            return self.resolve_path(end, self.dist[end].0);
        }
        self.start_node = start;

        while let Some(MinHeapItem { node, weight }) = self.heap.pop() {
            // node has already bee visited and can be skipped
            if self.visited.is_visited(node) {
                continue;
            }
            // already better path to this node
            if weight > self.dist[node].0 {
                continue;
            }
            // iterate over neighbors
            for edge in offset[node]..offset[node + 1] {
                let current_way: Way = edges[edge];
                // skip the contracting node
                if current_way.target == self.avoid_node {
                    continue;
                }
                // calculate costs
                let next = MinHeapItem {
                    node: current_way.target,
                    weight: weight + current_way.weight,
                };
                // add way to heap
                if next.weight < self.dist[next.node].0 {
                    self.dist[next.node] = (next.weight, Some(node));
                    self.heap.push(next);
                }
            }
            self.visited.set_visited(node);
            // found end you are done
            if node == end {
                return self.resolve_path(end, weight);
            }
            if weight >= self.max_weight {
                break;
            }
        }
        return None;
    }

    /// recreate path, of already visited
    fn resolve_path(&self, end: NodeId, weight: Weight) -> Option<(Vec<NodeId>, usize)> {
        if !self.visited.is_visited(end) || self.dist[end].0 > self.max_weight {
            return None;
        }
        let mut path = Vec::with_capacity(self.dist.len() / 2);
        path.push(end);
        let mut current_dist = self.dist[end];
        while let Some(prev) = current_dist.1 {
            path.push(prev);
            current_dist = self.dist[prev];
        }
        path.reverse();
        return Some((path, weight));
    }
}
