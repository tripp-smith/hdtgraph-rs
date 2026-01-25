mod api;
mod mapping;

use api::DynamicGraphPy;
use pyo3::prelude::*;

#[pymodule]
fn _rustdynconn(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<DynamicGraphPy>()?;
    Ok(())
}
