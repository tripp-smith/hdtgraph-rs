use proptest::prelude::*;
use rustdynconn_core::DynamicGraph;
use std::collections::{HashMap, HashSet, VecDeque};

fn bfs_connected(adj: &HashMap<u32, HashSet<u32>>, u: u32, v: u32) -> bool {
    if u == v {
        return adj.contains_key(&u);
    }
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    visited.insert(u);
    queue.push_back(u);
    while let Some(cur) = queue.pop_front() {
        if let Some(neigh) = adj.get(&cur) {
            for &nbr in neigh {
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

#[test]
fn add_remove_edges() {
    let mut graph = DynamicGraph::new();
    assert!(!graph.connected(0, 1));
    assert!(graph.add_edge(0, 1));
    assert!(graph.connected(0, 1));
    assert!(!graph.add_edge(0, 1));
    assert!(graph.remove_edge(0, 1));
    assert!(!graph.connected(0, 1));
    assert!(!graph.remove_edge(0, 1));
}

#[test]
fn components_basic() {
    let mut graph = DynamicGraph::new();
    graph.add_edge(0, 1);
    graph.add_edge(2, 3);
    graph.add_edge(3, 4);
    let comps = graph.components();
    assert_eq!(comps.len(), 2);
}

proptest! {
    #[test]
    fn prop_sequence(ops in prop::collection::vec((0u8..3, 0u32..20, 0u32..20), 1..200)) {
        let mut graph = DynamicGraph::new();
        let mut adj: HashMap<u32, HashSet<u32>> = HashMap::new();
        for (op, u, v) in ops {
            match op {
                0 => {
                    if u != v {
                        graph.add_edge(u, v);
                        let (a, b) = if u <= v { (u, v) } else { (v, u) };
                        adj.entry(a).or_default().insert(b);
                        adj.entry(b).or_default().insert(a);
                    }
                }
                1 => {
                    graph.remove_edge(u, v);
                    adj.entry(u).or_default().remove(&v);
                    adj.entry(v).or_default().remove(&u);
                }
                _ => {
                    let expected = bfs_connected(&adj, u, v);
                    let actual = graph.connected(u, v);
                    prop_assert_eq!(expected, actual);
                }
            }
        }
    }
}
