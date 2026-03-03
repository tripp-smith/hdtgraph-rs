use crate::hdt::levels::LevelState;
use crate::util::edge::{canonical_edge, EdgeKey, EdgeRec};
use hashbrown::HashMap;

pub fn find_replacement(
    levels: &mut [LevelState],
    edges: &mut HashMap<EdgeKey, EdgeRec>,
    level: usize,
    u: u32,
    v: u32,
) -> Option<EdgeKey> {
    let smaller_anchor =
        if levels[level].ett.component_size(u) <= levels[level].ett.component_size(v) {
            u
        } else {
            v
        };
    let smaller: Vec<u32> = levels[level]
        .ett
        .iter_vertices_in_component(smaller_anchor)
        .collect();

    for i in (0..=level).rev() {
        for &x in &smaller {
            let neighbors: Vec<u32> = levels[i]
                .adj_non_tree
                .get(&x)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .collect();
            for y in neighbors {
                let e = canonical_edge(x, y);
                if let Some(rec) = edges.get_mut(&e) {
                    if levels[level].ett.connected(x, y) {
                        // Keep the edge at its current level for correctness in this
                        // connectivity-only stage. Eager promotion can hide future
                        // replacement candidates from level-0 cuts before full HDT
                        // invariants are implemented.
                        continue;
                    }
                    for l in 0..=level {
                        let _ = levels[l].ett.link(rec.u, rec.v);
                        levels[l].remove_non_tree_edge(rec.u, rec.v);
                    }
                    rec.is_tree = true;
                    return Some(e);
                }
            }
        }
    }
    None
}
