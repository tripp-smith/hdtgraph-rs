use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct Node {
    pub key: u32,
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub parent: Option<usize>,
    pub path_parent: Option<usize>,
    pub size: usize,
}

impl Node {
    fn new(key: u32) -> Self {
        Self {
            key,
            left: None,
            right: None,
            parent: None,
            path_parent: None,
            size: 1,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct SplayTree {
    nodes: Vec<Node>,
    root: Option<usize>,
}

impl SplayTree {
    pub fn insert(&mut self, key: u32) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(Node::new(key));
        match self.root {
            None => self.root = Some(idx),
            Some(mut cur) => loop {
                let ord = key.cmp(&self.nodes[cur].key);
                let slot = if ord != Ordering::Greater {
                    &mut self.nodes[cur].left
                } else {
                    &mut self.nodes[cur].right
                };
                if let Some(next) = *slot {
                    cur = next;
                } else {
                    *slot = Some(idx);
                    self.nodes[idx].parent = Some(cur);
                    self.fix_upwards(Some(cur));
                    break;
                }
            },
        }
        idx
    }

    fn node_size(&self, idx: Option<usize>) -> usize {
        idx.map_or(0, |i| self.nodes[i].size)
    }

    fn refresh(&mut self, idx: usize) {
        let l = self.nodes[idx].left;
        let r = self.nodes[idx].right;
        self.nodes[idx].size = 1 + self.node_size(l) + self.node_size(r);
    }

    fn fix_upwards(&mut self, mut at: Option<usize>) {
        while let Some(i) = at {
            self.refresh(i);
            at = self.nodes[i].parent;
        }
    }

    pub fn rotate_left(&mut self, x: usize) {
        let y = self.nodes[x]
            .right
            .expect("rotate_left requires right child");
        let b = self.nodes[y].left;
        self.nodes[x].right = b;
        if let Some(bi) = b {
            self.nodes[bi].parent = Some(x);
        }

        let p = self.nodes[x].parent;
        self.nodes[y].parent = p;
        if let Some(pi) = p {
            if self.nodes[pi].left == Some(x) {
                self.nodes[pi].left = Some(y);
            } else {
                self.nodes[pi].right = Some(y);
            }
        } else {
            self.root = Some(y);
        }

        self.nodes[y].left = Some(x);
        self.nodes[x].parent = Some(y);
        self.refresh(x);
        self.refresh(y);
    }

    pub fn rotate_right(&mut self, x: usize) {
        let y = self.nodes[x]
            .left
            .expect("rotate_right requires left child");
        let b = self.nodes[y].right;
        self.nodes[x].left = b;
        if let Some(bi) = b {
            self.nodes[bi].parent = Some(x);
        }

        let p = self.nodes[x].parent;
        self.nodes[y].parent = p;
        if let Some(pi) = p {
            if self.nodes[pi].left == Some(x) {
                self.nodes[pi].left = Some(y);
            } else {
                self.nodes[pi].right = Some(y);
            }
        } else {
            self.root = Some(y);
        }

        self.nodes[y].right = Some(x);
        self.nodes[x].parent = Some(y);
        self.refresh(x);
        self.refresh(y);
    }

    pub fn splay(&mut self, x: usize) {
        while let Some(p) = self.nodes[x].parent {
            if self.nodes[p].parent.is_none() {
                if self.nodes[p].left == Some(x) {
                    self.rotate_right(p);
                } else {
                    self.rotate_left(p);
                }
            } else {
                let g = self.nodes[p].parent.expect("has grandparent");
                let zigzig_left = self.nodes[g].left == Some(p) && self.nodes[p].left == Some(x);
                let zigzig_right = self.nodes[g].right == Some(p) && self.nodes[p].right == Some(x);
                if zigzig_left {
                    self.rotate_right(g);
                    self.rotate_right(p);
                } else if zigzig_right {
                    self.rotate_left(g);
                    self.rotate_left(p);
                } else if self.nodes[p].left == Some(x) {
                    self.rotate_right(p);
                    self.rotate_left(g);
                } else {
                    self.rotate_left(p);
                    self.rotate_right(g);
                }
            }
        }
        self.root = Some(x);
    }

    pub fn access(&mut self, x: usize) {
        self.splay(x);
    }

    pub fn split(self, key: u32) -> (Self, Self) {
        let keys = self.inorder_keys();
        let mut left = Self::default();
        let mut right = Self::default();
        for k in keys {
            if k <= key {
                left.insert(k);
            } else {
                right.insert(k);
            }
        }
        (left, right)
    }

    pub fn concat(left: Self, right: Self) -> Self {
        let mut t = Self::default();
        for k in left.inorder_keys() {
            t.insert(k);
        }
        for k in right.inorder_keys() {
            t.insert(k);
        }
        t
    }

    pub fn find(&self, key: u32) -> Option<usize> {
        let mut cur = self.root;
        while let Some(i) = cur {
            match key.cmp(&self.nodes[i].key) {
                Ordering::Less => cur = self.nodes[i].left,
                Ordering::Greater => cur = self.nodes[i].right,
                Ordering::Equal => return Some(i),
            }
        }
        None
    }

    fn recompute_all_sizes(&mut self) {
        fn dfs(t: &mut SplayTree, i: Option<usize>) -> usize {
            let Some(idx) = i else {
                return 0;
            };
            let l = dfs(t, t.nodes[idx].left);
            let r = dfs(t, t.nodes[idx].right);
            t.nodes[idx].size = 1 + l + r;
            t.nodes[idx].size
        }
        let _ = dfs(self, self.root);
    }

    pub fn inorder_keys(&self) -> Vec<u32> {
        fn walk(t: &SplayTree, i: Option<usize>, out: &mut Vec<u32>) {
            if let Some(idx) = i {
                walk(t, t.nodes[idx].left, out);
                out.push(t.nodes[idx].key);
                walk(t, t.nodes[idx].right, out);
            }
        }
        let mut out = Vec::new();
        walk(self, self.root, &mut out);
        out
    }

    pub fn check_invariants(&self) {
        fn dfs(
            t: &SplayTree,
            i: Option<usize>,
            seen: &mut std::collections::HashSet<usize>,
        ) -> (usize, Vec<u32>) {
            let Some(idx) = i else {
                return (0, Vec::new());
            };
            assert!(seen.insert(idx), "cycle detected");
            let (ls, mut lk) = dfs(t, t.nodes[idx].left, seen);
            let (rs, rk) = dfs(t, t.nodes[idx].right, seen);
            if let Some(l) = t.nodes[idx].left {
                assert_eq!(t.nodes[l].parent, Some(idx));
            }
            if let Some(r) = t.nodes[idx].right {
                assert_eq!(t.nodes[r].parent, Some(idx));
            }
            let mut keys = Vec::new();
            keys.append(&mut lk);
            keys.push(t.nodes[idx].key);
            keys.extend(rk);
            assert!(keys.windows(2).all(|w| w[0] <= w[1]), "BST order broken");
            let size = 1 + ls + rs;
            assert_eq!(t.nodes[idx].size, size, "size mismatch");
            (size, keys)
        }
        let mut seen = std::collections::HashSet::new();
        let _ = dfs(self, self.root, &mut seen);
    }
}

#[cfg(test)]
mod tests {
    use super::SplayTree;

    #[test]
    fn splay_middle_preserves_order() {
        let mut t = SplayTree::default();
        let mut mid = 0;
        for i in 0..10 {
            let idx = t.insert(i);
            if i == 5 {
                mid = idx;
            }
        }
        t.splay(mid);
        assert_eq!(t.inorder_keys(), (0..10).collect::<Vec<_>>());
        t.check_invariants();
    }

    #[test]
    fn rotations_keep_sizes() {
        let mut t = SplayTree::default();
        for i in 0..6 {
            t.insert(i);
        }
        let root = t.find(2).unwrap();
        t.splay(root);
        t.rotate_left(root);
        t.check_invariants();
    }

    #[test]
    fn zig_zag_accesses() {
        let mut t = SplayTree::default();
        for i in 0..20 {
            t.insert(i);
        }
        for &k in &[10, 2, 18, 3, 17, 6, 15] {
            let i = t.find(k).unwrap();
            t.access(i);
            t.check_invariants();
        }
    }

    #[test]
    fn sorted_inorder_after_many_splays() {
        let mut t = SplayTree::default();
        for i in 0..32 {
            t.insert(i);
        }
        for i in (0..32).rev() {
            t.splay(t.find(i).unwrap());
        }
        assert_eq!(t.inorder_keys(), (0..32).collect::<Vec<_>>());
        t.check_invariants();
    }

    #[test]
    fn path_parent_field_is_stable() {
        let mut t = SplayTree::default();
        for i in 0..5 {
            t.insert(i);
        }
        let n = t.find(3).unwrap();
        t.nodes[n].path_parent = Some(t.find(1).unwrap());
        t.splay(n);
        assert!(t.nodes[n].path_parent.is_some());
        t.check_invariants();
    }
}
