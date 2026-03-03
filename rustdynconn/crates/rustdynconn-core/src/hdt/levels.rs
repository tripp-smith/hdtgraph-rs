use crate::ett::ETTForest;
use crate::hdt::replacement::ReplacementIndex;
use crate::util::edge::{canonical_edge, EdgeKey, EdgeRec};
use hashbrown::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct HDTGraph {
    ett: ETTForest,
    adj: HashMap<u32, HashSet<u32>>,
    edges: HashMap<EdgeKey, EdgeRec>,
    comp_id: HashMap<u32, usize>,
    comp_nodes: HashMap<usize, HashSet<u32>>,
    next_comp: usize,
    replacement: ReplacementIndex,
}

impl HDTGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: u32) {
        if self.adj.contains_key(&node) {
            return;
        }
        self.adj.insert(node, HashSet::new());
        self.ett.ensure_node(node);
        let cid = self.next_comp;
        self.next_comp += 1;
        self.comp_id.insert(node, cid);
        self.comp_nodes.insert(cid, HashSet::from([node]));
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
    pub fn node_count(&self) -> usize {
        self.adj.len()
    }

    pub fn has_edge(&self, u: u32, v: u32) -> bool {
        self.edges.contains_key(&canonical_edge(u, v))
    }

    pub fn add_edge(&mut self, u: u32, v: u32) -> bool {
        if u == v {
            return false;
        }
        self.add_node(u);
        self.add_node(v);
        let edge = canonical_edge(u, v);
        if self.edges.contains_key(&edge) {
            return false;
        }

        self.edges.insert(
            edge,
            EdgeRec {
                u: edge.0,
                v: edge.1,
            },
        );
        self.adj.get_mut(&u).expect("node exists").insert(v);
        self.adj.get_mut(&v).expect("node exists").insert(u);

        if self.connected(u, v) {
            self.replacement.insert_non_tree(edge);
        } else {
            let _ = self.ett.link(u, v);
            self.replacement.insert_tree(edge);
            self.merge_components(u, v);
        }
        true
    }

    pub fn remove_edge(&mut self, u: u32, v: u32) -> bool {
        let edge = canonical_edge(u, v);
        if self.edges.remove(&edge).is_none() {
            return false;
        }

        if let Some(neighbors) = self.adj.get_mut(&u) {
            neighbors.remove(&v);
        }
        if let Some(neighbors) = self.adj.get_mut(&v) {
            neighbors.remove(&u);
        }

        if self.replacement.tree_edges.contains(&edge) {
            self.ett.cut(u, v);
            self.replacement.remove(edge);
            self.recompute_components();
            self.rebuild_spanning_forest();
        } else {
            self.replacement.remove(edge);
        }
        true
    }

    pub fn connected(&self, u: u32, v: u32) -> bool {
        if u == v {
            return self.adj.contains_key(&u);
        }
        match (self.comp_id.get(&u), self.comp_id.get(&v)) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        }
    }

    pub fn component(&self, u: u32) -> Vec<u32> {
        let Some(cid) = self.comp_id.get(&u).copied() else {
            return Vec::new();
        };
        self.comp_nodes
            .get(&cid)
            .map(|nodes| nodes.iter().copied().collect())
            .unwrap_or_default()
    }

    pub fn components(&self) -> Vec<Vec<u32>> {
        self.comp_nodes
            .values()
            .map(|nodes| nodes.iter().copied().collect())
            .collect()
    }

    pub fn edges(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.edges.values().map(|e| (e.u, e.v))
    }

    pub fn nodes(&self) -> impl Iterator<Item = u32> + '_ {
        self.adj.keys().copied()
    }

    pub fn levels(&self) -> usize {
        let n = self.node_count().max(1);
        (usize::BITS as usize - n.leading_zeros() as usize) + 1
    }

    fn merge_components(&mut self, u: u32, v: u32) {
        let (Some(cu), Some(cv)) = (self.comp_id.get(&u).copied(), self.comp_id.get(&v).copied())
        else {
            return;
        };
        if cu == cv {
            return;
        }
        let (target, source) = if self.comp_nodes.get(&cu).map_or(0, |s| s.len())
            >= self.comp_nodes.get(&cv).map_or(0, |s| s.len())
        {
            (cu, cv)
        } else {
            (cv, cu)
        };
        let moved = self.comp_nodes.remove(&source).unwrap_or_default();
        let entry = self.comp_nodes.entry(target).or_default();
        for node in moved {
            self.comp_id.insert(node, target);
            entry.insert(node);
        }
    }

    fn recompute_components(&mut self) {
        self.comp_id.clear();
        self.comp_nodes.clear();
        self.next_comp = 0;

        let mut seen = HashSet::new();
        let nodes: Vec<u32> = self.adj.keys().copied().collect();
        for start in nodes {
            if !seen.insert(start) {
                continue;
            }
            let cid = self.next_comp;
            self.next_comp += 1;
            let mut stack = vec![start];
            let mut members = HashSet::new();
            while let Some(cur) = stack.pop() {
                members.insert(cur);
                if let Some(ns) = self.adj.get(&cur) {
                    for &next in ns {
                        if seen.insert(next) {
                            stack.push(next);
                        }
                    }
                }
            }
            for &node in &members {
                self.comp_id.insert(node, cid);
            }
            self.comp_nodes.insert(cid, members);
        }
    }

    fn rebuild_spanning_forest(&mut self) {
        self.ett = ETTForest::new();
        for &node in self.adj.keys() {
            self.ett.ensure_node(node);
        }
        self.replacement.tree_edges.clear();
        self.replacement.non_tree_edges.clear();

        let all_edges: Vec<EdgeKey> = self.edges.keys().copied().collect();
        for edge in all_edges {
            let (u, v) = (edge.0, edge.1);
            if self.ett.connected(u, v) {
                self.replacement.insert_non_tree(edge);
            } else {
                let _ = self.ett.link(u, v);
                self.replacement.insert_tree(edge);
            }
        }
    }
}
