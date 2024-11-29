use pyo3::prelude::*;

#[pymodule]
fn _self_encryption(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // Define constants directly rather than using macros
    m.add("MIN_CHUNK_SIZE", 1)?; // From lib.rs
    m.add("MIN_ENCRYPTABLE_BYTES", 3)?; // 3 * MIN_CHUNK_SIZE
    m.add("MAX_CHUNK_SIZE", 1024 * 1024)?; // 1MiB default
    m.add("COMPRESSION_QUALITY", 6)?;

    // Expose functions needed by CLI
    m.add_function(wrap_pyfunction!(encrypt_from_file, m)?)?;
    m.add_function(wrap_pyfunction!(decrypt_from_storage, m)?)?;
    m.add_function(wrap_pyfunction!(streaming_decrypt_from_storage, m)?)?;

    Ok(())
}

#[pyfunction]
fn encrypt_from_file(file_path: &str, output_dir: &str) -> PyResult<(DataMap, Vec<XorName>)> {
    let path = std::path::Path::new(file_path);
    let out_path = std::path::Path::new(output_dir);
    Ok(crate::encrypt_from_file(path, out_path)?)
}

#[pyfunction]
fn decrypt_from_storage(data_map: &DataMap, output_file: &str, chunks_dir: &str) -> PyResult<()> {
    let out_path = std::path::Path::new(output_file);
    let chunks_path = std::path::Path::new(chunks_dir);
    Ok(crate::decrypt_from_storage(data_map, out_path, |hash| {
        let chunk_path = chunks_path.join(hex::encode(hash));
        std::fs::read(chunk_path).map_err(|e| Error::Generic(e.to_string()))
    })?)
}
