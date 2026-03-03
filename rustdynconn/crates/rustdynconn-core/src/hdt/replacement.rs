use crate::util::edge::EdgeKey;
use hashbrown::HashSet;

#[derive(Debug, Default)]
pub struct ReplacementIndex {
    pub tree_edges: HashSet<EdgeKey>,
    pub non_tree_edges: HashSet<EdgeKey>,
}

impl ReplacementIndex {
    pub fn insert_tree(&mut self, edge: EdgeKey) {
        self.tree_edges.insert(edge);
        self.non_tree_edges.remove(&edge);
    }

    pub fn insert_non_tree(&mut self, edge: EdgeKey) {
        if !self.tree_edges.contains(&edge) {
            self.non_tree_edges.insert(edge);
        }
    }

    pub fn remove(&mut self, edge: EdgeKey) {
        self.tree_edges.remove(&edge);
        self.non_tree_edges.remove(&edge);
    }
}
