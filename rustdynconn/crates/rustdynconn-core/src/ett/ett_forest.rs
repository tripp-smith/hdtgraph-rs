use hashbrown::{HashMap, HashSet};

#[derive(Debug, Default, Clone)]
pub struct ETTForest {
    neighbors: HashMap<u32, HashSet<u32>>,
    comp_id: HashMap<u32, u32>,
    comp_nodes: HashMap<u32, HashSet<u32>>,
    next_comp: u32,
}

impl ETTForest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ensure_node(&mut self, node: u32) {
        if self.neighbors.contains_key(&node) {
            return;
        }
        self.neighbors.insert(node, HashSet::new());
        let cid = self.next_comp;
        self.next_comp += 1;
        self.comp_id.insert(node, cid);
        self.comp_nodes.insert(cid, HashSet::from([node]));
    }

    pub fn has_node(&self, node: u32) -> bool {
        self.neighbors.contains_key(&node)
    }

    pub fn link(&mut self, u: u32, v: u32) -> bool {
        if u == v {
            return false;
        }
        self.ensure_node(u);
        self.ensure_node(v);
        if self.connected(u, v) {
            return false;
        }
        self.neighbors.get_mut(&u).unwrap().insert(v);
        self.neighbors.get_mut(&v).unwrap().insert(u);
        self.merge_components(u, v);
        self.debug_invariants();
        true
    }

    pub fn cut(&mut self, u: u32, v: u32) -> bool {
        let mut removed = false;
        if let Some(ns) = self.neighbors.get_mut(&u) {
            removed |= ns.remove(&v);
        }
        if let Some(ns) = self.neighbors.get_mut(&v) {
            removed |= ns.remove(&u);
        }
        if removed {
            self.rebuild_components();
            self.debug_invariants();
        }
        removed
    }

    pub fn connected(&self, u: u32, v: u32) -> bool {
        match (self.comp_id.get(&u), self.comp_id.get(&v)) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        }
    }

    pub fn find_root(&self, u: u32) -> u32 {
        let Some(cid) = self.comp_id.get(&u).copied() else {
            return u;
        };
        self.comp_nodes
            .get(&cid)
            .and_then(|nodes| nodes.iter().min().copied())
            .unwrap_or(u)
    }

    pub fn component_size(&self, u: u32) -> usize {
        let Some(cid) = self.comp_id.get(&u).copied() else {
            return 0;
        };
        self.comp_nodes.get(&cid).map_or(0, |nodes| nodes.len())
    }

    pub fn iter_vertices_in_component(&self, u: u32) -> impl Iterator<Item = u32> + '_ {
        let cid = self.comp_id.get(&u).copied();
        let mut out: Vec<u32> = cid
            .and_then(|id| self.comp_nodes.get(&id).cloned())
            .unwrap_or_default()
            .into_iter()
            .collect();
        out.sort_unstable();
        out.into_iter()
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
        let tgt = self.comp_nodes.entry(target).or_default();
        for n in moved {
            self.comp_id.insert(n, target);
            tgt.insert(n);
        }
    }

    fn rebuild_components(&mut self) {
        self.comp_id.clear();
        self.comp_nodes.clear();
        self.next_comp = 0;
        let mut seen = HashSet::new();
        let nodes: Vec<u32> = self.neighbors.keys().copied().collect();
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
                if let Some(ns) = self.neighbors.get(&cur) {
                    for &n in ns {
                        if seen.insert(n) {
                            stack.push(n);
                        }
                    }
                }
            }
            for &n in &members {
                self.comp_id.insert(n, cid);
            }
            self.comp_nodes.insert(cid, members);
        }
    }

    pub fn debug_invariants(&self) {
        #[cfg(feature = "debug_invariants")]
        {
            for (&u, ns) in &self.neighbors {
                for &v in ns {
                    assert!(
                        self.neighbors.get(&v).is_some_and(|back| back.contains(&u)),
                        "missing back edge"
                    );
                }
            }
            for (&node, &cid) in &self.comp_id {
                assert!(self
                    .comp_nodes
                    .get(&cid)
                    .is_some_and(|nodes| nodes.contains(&node)));
            }
            for (&cid, nodes) in &self.comp_nodes {
                for &n in nodes {
                    assert_eq!(self.comp_id.get(&n).copied(), Some(cid));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ETTForest;
    use proptest::prelude::*;
    use std::collections::{HashMap, HashSet, VecDeque};

    fn baseline_connected(adj: &HashMap<u32, HashSet<u32>>, u: u32, v: u32) -> bool {
        if u == v {
            return adj.contains_key(&u);
        }
        let mut q = VecDeque::from([u]);
        let mut seen = HashSet::from([u]);
        while let Some(cur) = q.pop_front() {
            if let Some(ns) = adj.get(&cur) {
                for &n in ns {
                    if n == v {
                        return true;
                    }
                    if seen.insert(n) {
                        q.push_back(n);
                    }
                }
            }
        }
        false
    }

    #[test]
    fn link_and_connected() {
        let mut f = ETTForest::new();
        f.link(1, 2);
        assert!(f.connected(1, 2));
    }
    #[test]
    fn cut_disconnects() {
        let mut f = ETTForest::new();
        f.link(1, 2);
        f.cut(1, 2);
        assert!(!f.connected(1, 2));
    }
    #[test]
    fn size_updates() {
        let mut f = ETTForest::new();
        f.link(1, 2);
        f.link(2, 3);
        assert_eq!(f.component_size(1), 3);
    }
    #[test]
    fn root_is_stable_min() {
        let mut f = ETTForest::new();
        f.link(5, 2);
        f.link(2, 9);
        assert_eq!(f.find_root(9), 2);
    }
    #[test]
    fn iterator_unique() {
        let mut f = ETTForest::new();
        f.link(1, 2);
        f.link(2, 3);
        let v: Vec<_> = f.iter_vertices_in_component(2).collect();
        assert_eq!(v, vec![1, 2, 3]);
    }
    #[test]
    fn two_components_iter() {
        let mut f = ETTForest::new();
        f.link(1, 2);
        f.link(4, 5);
        assert_eq!(f.iter_vertices_in_component(1).count(), 2);
        assert_eq!(f.iter_vertices_in_component(4).count(), 2);
    }
    #[test]
    fn cut_in_middle() {
        let mut f = ETTForest::new();
        f.link(1, 2);
        f.link(2, 3);
        f.cut(2, 3);
        assert!(!f.connected(1, 3));
    }
    #[test]
    fn ensure_node_singleton() {
        let mut f = ETTForest::new();
        f.ensure_node(7);
        assert_eq!(f.component_size(7), 1);
    }

    proptest! {
        #[test]
        fn prop_random_ops(ops in prop::collection::vec((0u8..3,0u32..25,0u32..25), 1..300)) {
            let mut f = ETTForest::new();
            let mut adj: HashMap<u32, HashSet<u32>> = HashMap::new();
            for (op,u,v) in ops {
                adj.entry(u).or_default();
                adj.entry(v).or_default();
                f.ensure_node(u);
                f.ensure_node(v);
                match op {
                    0 => {
                        if !baseline_connected(&adj,u,v) && u != v {
                            f.link(u,v);
                            adj.get_mut(&u).unwrap().insert(v);
                            adj.get_mut(&v).unwrap().insert(u);
                        }
                    }
                    1 => {
                        if adj.get(&u).is_some_and(|ns| ns.contains(&v)) {
                            f.cut(u,v);
                            adj.get_mut(&u).unwrap().remove(&v);
                            adj.get_mut(&v).unwrap().remove(&u);
                        }
                    }
                    _ => {
                        prop_assert_eq!(f.connected(u,v), baseline_connected(&adj,u,v));
                    }
                }
            }
        }
    }
}
