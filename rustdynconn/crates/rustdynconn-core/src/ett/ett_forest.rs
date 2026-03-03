use hashbrown::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct ETTForest {
    neighbors: HashMap<u32, HashSet<u32>>,
}

impl ETTForest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ensure_node(&mut self, node: u32) {
        self.neighbors.entry(node).or_default();
    }

    pub fn has_node(&self, node: u32) -> bool {
        self.neighbors.contains_key(&node)
    }

    pub fn link(&mut self, u: u32, v: u32) -> bool {
        if u == v || self.connected(u, v) {
            return false;
        }
        self.ensure_node(u);
        self.ensure_node(v);
        self.neighbors.get_mut(&u).expect("node exists").insert(v);
        self.neighbors.get_mut(&v).expect("node exists").insert(u);
        true
    }

    pub fn cut(&mut self, u: u32, v: u32) -> bool {
        let mut removed = false;
        if let Some(n) = self.neighbors.get_mut(&u) {
            removed |= n.remove(&v);
        }
        if let Some(n) = self.neighbors.get_mut(&v) {
            removed |= n.remove(&u);
        }
        removed
    }

    pub fn connected(&self, u: u32, v: u32) -> bool {
        if u == v {
            return self.has_node(u);
        }
        let Some(_) = self.neighbors.get(&u) else {
            return false;
        };
        let Some(_) = self.neighbors.get(&v) else {
            return false;
        };

        let mut seen = HashSet::new();
        let mut stack = vec![u];
        seen.insert(u);
        while let Some(cur) = stack.pop() {
            if cur == v {
                return true;
            }
            if let Some(ns) = self.neighbors.get(&cur) {
                for &next in ns {
                    if seen.insert(next) {
                        stack.push(next);
                    }
                }
            }
        }
        false
    }
}
