use crate::mapping::NodeMapping;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyList, PyTuple};
use rustdynconn_core::DynamicGraph;

#[pyclass]
pub struct DynamicGraphPy {
    graph: DynamicGraph,
    mapping: NodeMapping,
}

#[pymethods]
impl DynamicGraphPy {
    #[new]
    pub fn new() -> Self {
        Self {
            graph: DynamicGraph::new(),
            mapping: NodeMapping::default(),
        }
    }

    pub fn add_node(&mut self, py: Python<'_>, node: &PyAny) -> PyResult<()> {
        let id = self.mapping.get_or_insert(py, node)?;
        self.graph.add_node(id);
        Ok(())
    }

    pub fn add_edge(&mut self, py: Python<'_>, u: &PyAny, v: &PyAny) -> PyResult<bool> {
        let uid = self.mapping.get_or_insert(py, u)?;
        let vid = self.mapping.get_or_insert(py, v)?;
        Ok(self.graph.add_edge(uid, vid))
    }

    pub fn remove_edge(&mut self, py: Python<'_>, u: &PyAny, v: &PyAny) -> PyResult<bool> {
        let Some(uid) = self.mapping.get_id(py, u)? else {
            return Ok(false);
        };
        let Some(vid) = self.mapping.get_id(py, v)? else {
            return Ok(false);
        };
        Ok(self.graph.remove_edge(uid, vid))
    }

    pub fn has_edge(&self, py: Python<'_>, u: &PyAny, v: &PyAny) -> PyResult<bool> {
        let Some(uid) = self.mapping.get_id(py, u)? else {
            return Ok(false);
        };
        let Some(vid) = self.mapping.get_id(py, v)? else {
            return Ok(false);
        };
        Ok(self.graph.has_edge(uid, vid))
    }

    pub fn connected(&self, py: Python<'_>, u: &PyAny, v: &PyAny) -> PyResult<bool> {
        let Some(uid) = self.mapping.get_id(py, u)? else {
            return Ok(false);
        };
        let Some(vid) = self.mapping.get_id(py, v)? else {
            return Ok(false);
        };
        Ok(self.graph.connected(uid, vid))
    }

    pub fn nodes(&self, py: Python<'_>) -> PyResult<PyObject> {
        let nodes = PyList::empty(py);
        for node_id in self.graph.nodes() {
            if let Some(obj) = self.mapping.get_obj(py, node_id)? {
                nodes.append(obj)?;
            }
        }
        Ok(nodes.into_py(py))
    }

    pub fn edges(&self, py: Python<'_>) -> PyResult<PyObject> {
        let edges = PyList::empty(py);
        for (u, v) in self.graph.edges() {
            let Some(u_obj) = self.mapping.get_obj(py, u)? else {
                continue;
            };
            let Some(v_obj) = self.mapping.get_obj(py, v)? else {
                continue;
            };
            edges.append(PyTuple::new(py, &[u_obj, v_obj]))?;
        }
        Ok(edges.into_py(py))
    }

    pub fn update(
        &mut self,
        py: Python<'_>,
        edges_add: Option<&PyAny>,
        edges_remove: Option<&PyAny>,
    ) -> PyResult<()> {
        if let Some(add_iter) = edges_add {
            for item in add_iter.iter()? {
                let tuple = item?.downcast::<PyTuple>()?;
                if tuple.len() != 2 {
                    continue;
                }
                let u = tuple.get_item(0);
                let v = tuple.get_item(1);
                let _ = self.add_edge(py, u, v)?;
            }
        }
        if let Some(rem_iter) = edges_remove {
            for item in rem_iter.iter()? {
                let tuple = item?.downcast::<PyTuple>()?;
                if tuple.len() != 2 {
                    continue;
                }
                let u = tuple.get_item(0);
                let v = tuple.get_item(1);
                let _ = self.remove_edge(py, u, v)?;
            }
        }
        Ok(())
    }

    pub fn to_networkx(&self, py: Python<'_>) -> PyResult<PyObject> {
        let nx = py.import("networkx")?;
        let graph = nx.getattr("Graph")?.call0()?;
        for node in self.nodes(py)?.downcast::<PyList>()? {
            graph.call_method1("add_node", (node,))?;
        }
        for edge in self.edges(py)?.downcast::<PyList>()? {
            let tuple = edge.downcast::<PyTuple>()?;
            graph.call_method1("add_edge", (tuple.get_item(0), tuple.get_item(1)))?;
        }
        Ok(graph.into_py(py))
    }

    #[getter]
    pub fn n(&self) -> usize {
        self.graph.node_count()
    }

    #[getter]
    pub fn m(&self) -> usize {
        self.graph.edge_count()
    }

    #[getter]
    pub fn levels(&self) -> usize {
        self.graph.levels()
    }
}
