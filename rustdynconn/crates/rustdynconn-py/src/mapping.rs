use pyo3::prelude::*;
use pyo3::types::PyAny;
use std::collections::HashMap;

#[derive(Default)]
pub struct NodeMapping {
    next_id: u32,
    buckets: HashMap<i64, Vec<(Py<PyAny>, u32)>>,
    id_to_obj: Vec<Py<PyAny>>,
}

impl NodeMapping {
    pub fn get_or_insert(&mut self, py: Python<'_>, obj: &PyAny) -> PyResult<u32> {
        let hash = obj.hash()?;
        if let Some(entries) = self.buckets.get(&hash) {
            for (stored, node_id) in entries {
                if stored
                    .as_ref(py)
                    .rich_compare(obj, pyo3::basic::CompareOp::Eq)?
                    .is_true()?
                {
                    return Ok(*node_id);
                }
            }
        }
        let id = self.next_id;
        self.next_id += 1;
        let owned: Py<PyAny> = obj.into_py(py);
        self.buckets
            .entry(hash)
            .or_default()
            .push((owned.clone_ref(py), id));
        if id as usize == self.id_to_obj.len() {
            self.id_to_obj.push(owned);
        } else {
            self.id_to_obj[id as usize] = owned;
        }
        Ok(id)
    }

    pub fn get_id(&self, py: Python<'_>, obj: &PyAny) -> PyResult<Option<u32>> {
        let hash = obj.hash()?;
        if let Some(entries) = self.buckets.get(&hash) {
            for (stored, node_id) in entries {
                if stored
                    .as_ref(py)
                    .rich_compare(obj, pyo3::basic::CompareOp::Eq)?
                    .is_true()?
                {
                    return Ok(Some(*node_id));
                }
            }
        }
        Ok(None)
    }

    pub fn get_obj(&self, py: Python<'_>, node_id: u32) -> PyResult<Option<PyObject>> {
        let idx = node_id as usize;
        if idx >= self.id_to_obj.len() {
            return Ok(None);
        }
        Ok(Some(self.id_to_obj[idx].clone_ref(py).into_py(py)))
    }

    pub fn len(&self) -> usize {
        self.id_to_obj.len()
    }
}
