use crate::hdt::levels::LevelState;
use crate::hdt::replacement::find_replacement;
use crate::util::edge::{canonical_edge, EdgeKey, EdgeRec};
use hashbrown::HashMap;

#[derive(Debug, Default)]
pub struct HdtGraph {
    edges: HashMap<EdgeKey, EdgeRec>,
    levels: Vec<LevelState>,
    max_node_seen: u32,
}

impl HdtGraph {
    pub fn add_node(&mut self, node: u32) {
        self.max_node_seen = self.max_node_seen.max(node + 1);
        self.ensure_levels();
        for level in &mut self.levels {
            level.ensure_node(node);
        }
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
    pub fn node_count(&self) -> usize {
        self.levels.first().map_or(0, |l| {
            l.adj_non_tree
                .len()
                .max(l.ett.iter_vertices_in_component(0).count())
        })
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
        let key = canonical_edge(u, v);
        if self.edges.contains_key(&key) {
            return false;
        }

        let is_tree = !self.levels[0].ett.connected(u, v);
        let rec = EdgeRec {
            u: key.0,
            v: key.1,
            level: 0,
            is_tree,
        };
        self.edges.insert(key, rec);
        if is_tree {
            for level in &mut self.levels {
                let _ = level.ett.link(u, v);
            }
        } else {
            self.levels[0].add_non_tree_edge(u, v);
        }
        true
    }

    pub fn remove_edge(&mut self, u: u32, v: u32) -> bool {
        let key = canonical_edge(u, v);
        let Some(rec) = self.edges.remove(&key) else {
            return false;
        };
        if !rec.is_tree {
            self.levels[rec.level].remove_non_tree_edge(u, v);
            return true;
        }

        for l in 0..=rec.level.min(self.levels.len() - 1) {
            self.levels[l].ett.cut(u, v);
        }
        let _ = find_replacement(&mut self.levels, &mut self.edges, rec.level, u, v);
        true
    }

    pub fn connected(&self, u: u32, v: u32) -> bool {
        self.levels.first().is_some_and(|l| l.ett.connected(u, v))
    }

    pub fn component(&self, u: u32) -> Vec<u32> {
        self.levels
            .first()
            .map_or_else(Vec::new, |l| l.ett.iter_vertices_in_component(u).collect())
    }

    pub fn components(&self) -> Vec<Vec<u32>> {
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        if let Some(level0) = self.levels.first() {
            for &node in level0.adj_non_tree.keys() {
                if seen.contains(&node) {
                    continue;
                }
                let comp: Vec<u32> = level0.ett.iter_vertices_in_component(node).collect();
                for n in &comp {
                    seen.insert(*n);
                }
                if !comp.is_empty() {
                    out.push(comp);
                }
            }
        }
        out
    }

    pub fn edges(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.edges.values().map(|e| (e.u, e.v))
    }
    pub fn nodes(&self) -> impl Iterator<Item = u32> + '_ {
        self.levels
            .first()
            .into_iter()
            .flat_map(|l| l.adj_non_tree.keys().copied())
    }

    pub fn levels(&self) -> usize {
        self.levels.len().max(1)
    }

    fn ensure_levels(&mut self) {
        if self.levels.is_empty() {
            self.levels.push(LevelState::default());
        }
        let target = (self.max_node_seen.max(1) as f64).log2().floor() as usize + 1;
        while self.levels.len() < target {
            let mut next = LevelState::default();
            for (&edge, rec) in &self.edges {
                next.ensure_node(edge.0);
                next.ensure_node(edge.1);
                if rec.is_tree {
                    let _ = next.ett.link(edge.0, edge.1);
                }
            }
            self.levels.push(next);
        }
    }
}
