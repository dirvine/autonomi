use pyo3::prelude::*;
use crate::{Client, Wallet};

#[pymodule]
fn _autonomi(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyClient>()?;
    m.add_class::<PyWallet>()?;
    Ok(())
}

#[pyclass]
struct PyClient {
    inner: Client,
}

#[pymethods]
impl PyClient {
    // Add Python methods here
}

#[pyclass]
struct PyWallet {
    inner: Wallet,
}

#[pymethods]
impl PyWallet {
    // Add Python methods here
} 