use pyo3::prelude::*;

#[pymodule]
fn self_encryption(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // Define constants directly rather than using macros
    m.add("MIN_CHUNK_SIZE", 1)?;  // From lib.rs
    m.add("MIN_ENCRYPTABLE_BYTES", 3)?;  // 3 * MIN_CHUNK_SIZE
    m.add("MAX_CHUNK_SIZE", 1024 * 1024)?;  // 1MiB default
    m.add("COMPRESSION_QUALITY", 6)?;

    Ok(())
}
