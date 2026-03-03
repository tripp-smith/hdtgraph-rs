use crate::ett::ETTForest;
use hashbrown::{HashMap, HashSet};

#[derive(Debug, Default, Clone)]
pub struct LevelState {
    pub ett: ETTForest,
    pub adj_non_tree: HashMap<u32, HashSet<u32>>,
}

impl LevelState {
    pub fn ensure_node(&mut self, node: u32) {
        self.ett.ensure_node(node);
        self.adj_non_tree.entry(node).or_default();
    }

    pub fn add_non_tree_edge(&mut self, u: u32, v: u32) {
        self.ensure_node(u);
        self.ensure_node(v);
        self.adj_non_tree.get_mut(&u).unwrap().insert(v);
        self.adj_non_tree.get_mut(&v).unwrap().insert(u);
    }

    pub fn remove_non_tree_edge(&mut self, u: u32, v: u32) {
        if let Some(ns) = self.adj_non_tree.get_mut(&u) {
            ns.remove(&v);
        }
        if let Some(ns) = self.adj_non_tree.get_mut(&v) {
            ns.remove(&u);
        }
    }
}
