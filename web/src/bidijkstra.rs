use super::*;
use min_heap::*;
use std::collections::BinaryHeap;
use visited_list::*;

#[derive(Clone)]
pub struct Dijkstra {
    dist_up: Vec<(NodeId, Option<Weight>)>,
    dist_down: Vec<(NodeId, Option<Weight>)>,
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

    pub fn find_path(
        &mut self,
        start: NodeId,
        end: NodeId,
        nodes: &Vec<Node>,
        edges: &Vec<Way>,
        up_offset: &Vec<EdgeId>,
        down_offset: &Vec<EdgeId>,
        down_index: &Vec<EdgeId>,
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
                if self.visited_up.is_visited(node) && weight > self.dist_up[node].0 {
                    continue;
                }
                if weight > best_weight {
                    break;
                }

                // iterate over neighbors
                for edge in graph_helper::get_up_edge_ids(node, &up_offset) {
                    let current_way: Way = edges[edge];
                    // skip nodes with lower rank
                    if nodes[current_way.target].rank <= nodes[node].rank {
                        continue;
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
                if self.visited_down.is_visited(node) && weight > self.dist_down[node].0 {
                    continue;
                }
                if weight > best_weight {
                    break;
                }

                for edge in graph_helper::get_down_edge_ids(node, &down_offset, &down_index) {
                    let current_way: Way = edges[edge];
                    // skip nodes with lower rank
                    if nodes[current_way.source].rank <= nodes[node].rank {
                        continue;
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
            return None;
        } else {
            return self.resolve_path(
                start,
                end,
                meeting_node,
                best_weight,
                nodes[meeting_node].rank,
                &edges,
            );
        }
    }

    /// backtrack the shortcuts to original edges
    fn resolve_path(
        &self,
        start: NodeId,
        end: NodeId,
        meeting_node: NodeId,
        weight: Weight,
        meeting_rank: Rank,
        edges: &Vec<Way>,
    ) -> Option<(Vec<NodeId>, f32)> {
        assert!(self.visited_up.is_visited(meeting_node));
        assert!(self.visited_down.is_visited(meeting_node));
        println!("meeting_node {:?}", edges[meeting_node]);

        let mut path: Vec<NodeId> = Vec::with_capacity(meeting_rank.pow(2));

        // TODO
        path.push(start);
        path.push(meeting_node);
        path.push(end);
        // let mut current_dist = dist[end];
        // path.push(end);
        // while let Some(prev) = current_dist.1 {
        //     path.push(prev);
        //     current_dist = dist[prev];
        // }
        // path.reverse();
        // while let Some(prev) = current_dist.1 {
        //     path.push(prev);
        //     current_dist = dist[prev];
        // }

        return Some((path, weight as f32 / DIST_MULTIPLICATOR as f32));
    }
}
