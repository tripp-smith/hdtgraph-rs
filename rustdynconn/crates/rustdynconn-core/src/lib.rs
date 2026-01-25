mod util;

use hashbrown::{HashMap, HashSet};
use std::collections::VecDeque;
use util::edge::{canonical_edge, EdgeKey, EdgeRec};

#[derive(Debug, Default)]
pub struct DynamicGraph {
    adj: HashMap<u32, HashSet<u32>>,
    edges: HashMap<EdgeKey, EdgeRec>,
}

impl DynamicGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn node_count(&self) -> usize {
        self.adj.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn add_node(&mut self, node: u32) {
        self.adj.entry(node).or_default();
    }

    pub fn has_edge(&self, u: u32, v: u32) -> bool {
        let key = canonical_edge(u, v);
        self.edges.contains_key(&key)
    }

    pub fn add_edge(&mut self, u: u32, v: u32) -> bool {
        if u == v {
            return false;
        }
        self.add_node(u);
        self.add_node(v);
        let key = canonical_edge(u, v);
        if self.edges.contains_key(&key) {
            return false;
        }
        self.edges.insert(key, EdgeRec { u: key.0, v: key.1 });
        self.adj.get_mut(&u).expect("node exists").insert(v);
        self.adj.get_mut(&v).expect("node exists").insert(u);
        true
    }

    pub fn remove_edge(&mut self, u: u32, v: u32) -> bool {
        let key = canonical_edge(u, v);
        if self.edges.remove(&key).is_none() {
            return false;
        }
        self.adj.entry(u).or_default().remove(&v);
        self.adj.entry(v).or_default().remove(&u);
        true
    }

    pub fn connected(&self, u: u32, v: u32) -> bool {
        if u == v {
            return self.adj.contains_key(&u);
        }
        if !self.adj.contains_key(&u) || !self.adj.contains_key(&v) {
            return false;
        }
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        visited.insert(u);
        queue.push_back(u);
        while let Some(cur) = queue.pop_front() {
            if let Some(neighbors) = self.adj.get(&cur) {
                for &nbr in neighbors {
                    if nbr == v {
                        return true;
                    }
                    if visited.insert(nbr) {
                        queue.push_back(nbr);
                    }
                }
            }
        }
        false
    }

    pub fn component(&self, u: u32) -> Vec<u32> {
        if !self.adj.contains_key(&u) {
            return Vec::new();
        }
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut out = Vec::new();
        visited.insert(u);
        queue.push_back(u);
        while let Some(cur) = queue.pop_front() {
            out.push(cur);
            if let Some(neighbors) = self.adj.get(&cur) {
                for &nbr in neighbors {
                    if visited.insert(nbr) {
                        queue.push_back(nbr);
                    }
                }
            }
        }
        out
    }

    pub fn components(&self) -> Vec<Vec<u32>> {
        let mut visited = HashSet::new();
        let mut comps = Vec::new();
        for &node in self.adj.keys() {
            if visited.contains(&node) {
                continue;
            }
            let mut queue = VecDeque::new();
            let mut comp = Vec::new();
            visited.insert(node);
            queue.push_back(node);
            while let Some(cur) = queue.pop_front() {
                comp.push(cur);
                if let Some(neighbors) = self.adj.get(&cur) {
                    for &nbr in neighbors {
                        if visited.insert(nbr) {
                            queue.push_back(nbr);
                        }
                    }
                }
            }
            comps.push(comp);
        }
        comps
    }

    pub fn edges(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.edges.values().map(|rec| (rec.u, rec.v))
    }

    pub fn nodes(&self) -> impl Iterator<Item = u32> + '_ {
        self.adj.keys().copied()
    }

    pub fn levels(&self) -> usize {
        1
    }
}
