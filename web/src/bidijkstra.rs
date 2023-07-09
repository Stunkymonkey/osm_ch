use super::*;
use min_heap::*;
use std::collections::BinaryHeap;
use visited_list::*;

#[derive(Clone)]
pub struct Dijkstra {
    dist_up: Vec<(Weight, Option<EdgeId>)>,
    dist_down: Vec<(Weight, Option<EdgeId>)>,
    visited_up: VisitedList,
    visited_down: VisitedList,
    heap_up: BinaryHeap<MinHeapItem>,
    heap_down: BinaryHeap<MinHeapItem>,
}

impl Dijkstra {
    /// general constructor
    pub fn new(amount_nodes: usize) -> Self {
        Dijkstra {
            dist_up: vec![(std::usize::MAX, None); amount_nodes],
            dist_down: vec![(std::usize::MAX, None); amount_nodes],
            visited_up: VisitedList::new(amount_nodes),
            visited_down: VisitedList::new(amount_nodes),
            heap_up: BinaryHeap::new(),
            heap_down: BinaryHeap::new(),
        }
    }

    /// find path from start to end
    #[allow(clippy::too_many_arguments)]
    pub fn find_path(
        &mut self,
        start: NodeId,
        end: NodeId,
        nodes: &[Node],
        edges: &[Way],
        up_offset: &[EdgeId],
        down_offset: &[EdgeId],
        down_index: &[EdgeId],
    ) -> Option<(Vec<NodeId>, f32)> {
        self.heap_up.clear();
        self.heap_down.clear();
        self.visited_up.unvisit_all();
        self.visited_down.unvisit_all();

        if start == end {
            return Some((vec![], 0.0));
        }

        self.dist_up[start] = (0, None);
        self.dist_down[end] = (0, None);
        self.visited_up.set_visited(start);
        self.visited_down.set_visited(end);

        self.heap_up.push(MinHeapItem::new(start, 0));
        self.heap_down.push(MinHeapItem::new(end, 0));

        let mut best_weight = WEIGHT_MAX;
        let mut meeting_node = INVALID_NODE;

        // now loop over both-heaps
        while !self.heap_up.is_empty() || !self.heap_down.is_empty() {
            while let Some(MinHeapItem { node, weight }) = self.heap_up.pop() {
                // check if not already visited with cheaper costs
                if self.visited_up.is_visited(node) && weight > self.dist_up[node].0 {
                    continue;
                }
                if weight > best_weight {
                    break;
                }

                // stall on demand optimization
                if self.is_stallable_up(node, weight, nodes, edges, down_offset, down_index) {
                    continue;
                }

                // iterate over neighbors
                for edge in graph_helper::get_up_edge_ids(node, up_offset) {
                    let current_way: Way = edges[edge];
                    // skip nodes with lower rank
                    if nodes[current_way.target].rank <= nodes[node].rank {
                        break;
                    }
                    // calculate new costs
                    let next = MinHeapItem::new(current_way.target, weight + current_way.weight);
                    // add way to heap
                    if !self.visited_up.is_visited(next.node)
                        || next.weight < self.dist_up[next.node].0
                    {
                        self.dist_up[next.node] = (next.weight, Some(edge));
                        self.visited_up.set_visited(next.node);
                        self.heap_up.push(next);
                    }
                }

                if self.visited_down.is_visited(node)
                    && weight + self.dist_down[node].0 < best_weight
                {
                    best_weight = weight + self.dist_down[node].0;
                    meeting_node = node;
                }
                break;
            }
            while let Some(MinHeapItem { node, weight }) = self.heap_down.pop() {
                // check if not already visited with cheaper costs
                if self.visited_down.is_visited(node) && weight > self.dist_down[node].0 {
                    continue;
                }
                if weight > best_weight {
                    break;
                }

                // stall on demand optimization
                if self.is_stallable_down(node, weight, nodes, edges, up_offset) {
                    continue;
                }

                // iterate over neighbors
                for edge in graph_helper::get_down_edge_ids(node, down_offset, down_index) {
                    let current_way: Way = edges[edge];
                    // skip nodes with lower rank
                    if nodes[current_way.source].rank <= nodes[node].rank {
                        break;
                    }
                    // calculate new costs
                    let next = MinHeapItem::new(current_way.source, weight + current_way.weight);
                    // add way to heap
                    if !self.visited_down.is_visited(next.node)
                        || next.weight < self.dist_down[next.node].0
                    {
                        self.dist_down[next.node] = (next.weight, Some(edge));
                        self.visited_down.set_visited(next.node);
                        self.heap_down.push(next);
                    }
                }

                if self.visited_up.is_visited(node) && weight + self.dist_up[node].0 < best_weight {
                    best_weight = weight + self.dist_up[node].0;
                    meeting_node = node;
                }
                break;
            }
        }

        if meeting_node == INVALID_NODE {
            None
        } else {
            Some(self.resolve_path(meeting_node, best_weight, nodes[meeting_node].rank, edges))
        }
    }

    /// backtrack the shortcuts to original edges
    fn resolve_path(
        &self,
        meeting_node: NodeId,
        weight: Weight,
        meeting_rank: Rank,
        edges: &[Way],
    ) -> (Vec<NodeId>, f32) {
        assert!(self.visited_up.is_visited(meeting_node));
        assert!(self.visited_down.is_visited(meeting_node));

        let mut path: Vec<NodeId> = Vec::with_capacity(meeting_rank.pow(2));

        let up_edge = self.dist_up[meeting_node];
        let down_edge = self.dist_down[meeting_node];

        path.push(meeting_node);
        if up_edge.1.is_some() {
            self.walk_down(up_edge.1.unwrap(), true, &mut path, edges);
            path.reverse();
        }
        if down_edge.1.is_some() {
            self.walk_down(down_edge.1.unwrap(), false, &mut path, edges);
        }

        (path, weight as f32 / DIST_MULTIPLICATOR as f32)
    }

    // walk shortcuts from meeting point to end
    fn walk_down(&self, edge: EdgeId, is_upwards: bool, path: &mut Vec<NodeId>, edges: &[Way]) {
        resolve_edge(edge, path, is_upwards, edges);

        let current_edge = edges[edge];

        let prev = if is_upwards {
            self.dist_up[current_edge.source]
        } else {
            self.dist_down[current_edge.target]
        };
        if let Some(child) = prev.1 {
            self.walk_down(child, is_upwards, path, edges);
        }
    }

    fn is_stallable_up(
        &self,
        node: NodeId,
        weight: Weight,
        nodes: &[Node],
        edges: &[Way],
        down_offset: &[EdgeId],
        down_index: &[EdgeId],
    ) -> bool {
        for edge in graph_helper::get_down_edge_ids(node, down_offset, down_index) {
            let way: Way = edges[edge];
            if nodes[way.source].rank <= nodes[node].rank {
                break;
            }
            if self.visited_up.is_visited(way.source)
                && way.weight + self.dist_up[way.source].0 <= weight
            {
                return true;
            }
        }
        false
    }

    fn is_stallable_down(
        &self,
        node: NodeId,
        weight: Weight,
        nodes: &[Node],
        edges: &[Way],
        up_offset: &[EdgeId],
    ) -> bool {
        for edge in graph_helper::get_up_edge_ids(node, up_offset) {
            let way: Way = edges[edge];
            if nodes[way.target].rank <= nodes[node].rank {
                break;
            }
            if self.visited_down.is_visited(way.target)
                && way.weight + self.dist_down[way.target].0 <= weight
            {
                return true;
            }
        }
        false
    }
}

/// resolve shortcuts to original edges
fn resolve_edge(
    // &self,
    edge: EdgeId,
    path: &mut Vec<NodeId>,
    is_upwards: bool,
    edges: &[Way],
) {
    match (&edges[edge].contrated_previous, &edges[edge].contrated_next) {
        (Some(previous), Some(next)) => {
            if is_upwards {
                resolve_edge(*next, path, is_upwards, edges);
                resolve_edge(*previous, path, is_upwards, edges);
            } else {
                resolve_edge(*previous, path, is_upwards, edges);
                resolve_edge(*next, path, is_upwards, edges);
            }
        }
        _ => {
            if is_upwards {
                path.push(edges[edge].source)
            } else {
                path.push(edges[edge].target)
            }
        }
    }
}
