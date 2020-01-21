use crate::constants::*;

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

    pub fn unvisit_all(&mut self) {
        if self.visited_flag == std::usize::MAX {
            self.nodes = vec![0; self.nodes.len()];
            self.visited_flag = 1;
        } else {
            self.visited_flag += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_visited() {
        let mut visited = VisitedList::new(42);
        assert!(!visited.is_visited(17));
        visited.set_visited(17);
        assert!(visited.is_visited(17));
        visited.unvisit_all();
        assert!(!visited.is_visited(17));
    }
}