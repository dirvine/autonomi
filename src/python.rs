use pyo3::prelude::*;

#[pymodule]
fn self_encryption(_py: Python<'_>, _m: &PyModule) -> PyResult<()> {
    // Module initialization code here
    Ok(())
}
