mod util;

use hashbrown::{HashMap, HashSet};
use std::collections::VecDeque;
use util::edge::{canonical_edge, EdgeKey, EdgeRec};

#[derive(Debug, Default)]
pub struct DynamicGraph {
    adj: Vec<HashSet<u32>>,
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
        let len = self.adj.len();
        if (node as usize) >= len {
            self.adj.resize_with((node as usize) + 1, HashSet::new);
        }
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
        self.adj[u as usize].insert(v);
        self.adj[v as usize].insert(u);
        true
    }

    pub fn remove_edge(&mut self, u: u32, v: u32) -> bool {
        let key = canonical_edge(u, v);
        if self.edges.remove(&key).is_none() {
            return false;
        }
        if let Some(set) = self.adj.get_mut(u as usize) {
            set.remove(&v);
        }
        if let Some(set) = self.adj.get_mut(v as usize) {
            set.remove(&u);
        }
        true
    }

    pub fn connected(&self, u: u32, v: u32) -> bool {
        if u == v {
            return self.adj.get(u as usize).is_some();
        }
        if (u as usize) >= self.adj.len() || (v as usize) >= self.adj.len() {
            return false;
        }
        let mut visited = vec![false; self.adj.len()];
        let mut queue = VecDeque::new();
        visited[u as usize] = true;
        queue.push_back(u);
        while let Some(cur) = queue.pop_front() {
            for &nbr in &self.adj[cur as usize] {
                if nbr == v {
                    return true;
                }
                let idx = nbr as usize;
                if !visited[idx] {
                    visited[idx] = true;
                    queue.push_back(nbr);
                }
            }
        }
        false
    }

    pub fn component(&self, u: u32) -> Vec<u32> {
        if (u as usize) >= self.adj.len() {
            return Vec::new();
        }
        let mut visited = vec![false; self.adj.len()];
        let mut queue = VecDeque::new();
        let mut out = Vec::new();
        visited[u as usize] = true;
        queue.push_back(u);
        while let Some(cur) = queue.pop_front() {
            out.push(cur);
            for &nbr in &self.adj[cur as usize] {
                let idx = nbr as usize;
                if !visited[idx] {
                    visited[idx] = true;
                    queue.push_back(nbr);
                }
            }
        }
        out
    }

    pub fn components(&self) -> Vec<Vec<u32>> {
        let mut visited = vec![false; self.adj.len()];
        let mut comps = Vec::new();
        for node in 0..self.adj.len() {
            if visited[node] {
                continue;
            }
            let mut queue = VecDeque::new();
            let mut comp = Vec::new();
            visited[node] = true;
            queue.push_back(node as u32);
            while let Some(cur) = queue.pop_front() {
                comp.push(cur);
                for &nbr in &self.adj[cur as usize] {
                    let idx = nbr as usize;
                    if !visited[idx] {
                        visited[idx] = true;
                        queue.push_back(nbr);
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
        (0..self.adj.len()).map(|n| n as u32)
    }

    pub fn levels(&self) -> usize {
        1
    }
}
