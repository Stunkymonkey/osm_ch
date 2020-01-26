use crate::constants::*;

#[derive(Clone)]
pub struct VisitedList {
    nodes: Vec<Weight>,
    visited_flag: usize,
}

impl VisitedList {
    pub fn new(num_nodes: usize) -> Self {
        VisitedList {
            nodes: vec![0; num_nodes],
            visited_flag: 1,
        }
    }

    pub fn is_visited(&self, node: NodeId) -> bool {
        return self.nodes[node] == self.visited_flag;
    }

    pub fn set_visited(&mut self, node: NodeId) {
        self.nodes[node] = self.visited_flag;
    }

    pub fn invalidate_all(&mut self) {
        if self.visited_flag == std::usize::MAX {
            self.nodes = vec![0; self.nodes.len()];
            self.visited_flag = 1;
        } else {
            self.visited_flag += 1;
        }
    }
}
