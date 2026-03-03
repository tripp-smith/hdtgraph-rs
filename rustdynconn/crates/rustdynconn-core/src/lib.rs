mod ett;
mod hdt;
mod util;

use hdt::HDTGraph;

#[derive(Debug, Default)]
pub struct DynamicGraph {
    backend: HDTGraph,
}

impl DynamicGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn node_count(&self) -> usize {
        self.backend.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.backend.edge_count()
    }

    pub fn add_node(&mut self, node: u32) {
        self.backend.add_node(node);
    }

    pub fn has_edge(&self, u: u32, v: u32) -> bool {
        self.backend.has_edge(u, v)
    }

    pub fn add_edge(&mut self, u: u32, v: u32) -> bool {
        self.backend.add_edge(u, v)
    }

    pub fn remove_edge(&mut self, u: u32, v: u32) -> bool {
        self.backend.remove_edge(u, v)
    }

    pub fn connected(&self, u: u32, v: u32) -> bool {
        self.backend.connected(u, v)
    }

    pub fn component(&self, u: u32) -> Vec<u32> {
        self.backend.component(u)
    }

    pub fn components(&self) -> Vec<Vec<u32>> {
        self.backend.components()
    }

    pub fn edges(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.backend.edges()
    }

    pub fn nodes(&self) -> impl Iterator<Item = u32> + '_ {
        self.backend.nodes()
    }

    pub fn levels(&self) -> usize {
        self.backend.levels()
    }
}
