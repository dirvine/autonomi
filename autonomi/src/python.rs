use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList};
use std::path::PathBuf;
use std::collections::HashMap;
use xor_name::XorName;
use sn_evm::ProofOfPayment;

// First, define all error handling
#[derive(Debug)]
enum PyAutoError {
    Network(NetworkError),
    Data(GetError),
    Put(PutError),
    Cost(CostError),
    Vault(VaultError),
    Register(RegisterError),
    Address(DataError),
}

impl From<PyAutoError> for PyErr {
    fn from(err: PyAutoError) -> PyErr {
        match err {
            PyAutoError::Network(e) => 
                PyErr::new::<pyo3::exceptions::PyConnectionError, _>(format!("{}", e)),
            PyAutoError::Data(e) => 
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)),
            PyAutoError::Put(e) => 
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)),
            PyAutoError::Cost(e) => 
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)),
            PyAutoError::Vault(e) => 
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)),
            PyAutoError::Register(e) => 
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)),
            PyAutoError::Address(e) => 
                PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)),
        }
    }
}

// Define all Python classes that map to Rust types
#[pyclass]
struct Client {
    inner: crate::Client,
}

#[pyclass]
struct Wallet {
    inner: crate::Wallet,
}

#[pyclass]
struct Archive {
    inner: crate::client::archive::Archive,
}

#[pyclass]
struct PrivateArchive {
    inner: crate::client::archive_private::PrivateArchive,
}

#[pyclass]
struct Metadata {
    inner: crate::client::archive::Metadata,
}

#[pyclass]
struct RegisterSecretKey {
    inner: crate::client::registers::RegisterSecretKey,
}

#[pyclass]
struct VaultSecretKey {
    inner: crate::client::vault::VaultSecretKey,
}

#[pyclass]
struct UserData {
    inner: crate::client::vault::user_data::UserData,
}

#[pyclass]
struct Receipt {
    inner: HashMap<XorName, ProofOfPayment>,
}

#[pyclass]
enum PaymentOption {
    Wallet(Wallet),
    Receipt(Receipt),
}

// Runtime helper to avoid duplicating tokio runtime creation
struct Runtime(tokio::runtime::Runtime);

impl Runtime {
    fn new() -> PyResult<Self> {
        tokio::runtime::Runtime::new()
            .map(Runtime)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Failed to create runtime: {}", e)))
    }

    fn block_on<F: std::future::Future>(&self, future: F) -> F::Output {
        self.0.block_on(future)
    }
}

#[pymethods]
impl Client {
    #[new]
    fn connect(peers: Vec<String>) -> PyResult<Self> {
        let rt = Runtime::new()?;
        let peers = peers
            .into_iter()
            .map(|p| p.parse())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;

        let client = rt.block_on(crate::Client::connect(&peers))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyConnectionError, _>(format!("{}", e)))?;

        Ok(Self { inner: client })
    }

    // Data Operations
    fn data_put(&self, data: &[u8], wallet: &Wallet) -> PyResult<String> {
        let rt = Runtime::new()?;
        let bytes = bytes::Bytes::copy_from_slice(data);
        
        let addr = rt.block_on(self.inner.data_put(bytes, wallet.inner.clone().into()))
            .map_err(PyAutoError::Put)?;

        Ok(crate::client::address::addr_to_str(addr))
    }

    fn data_get(&self, addr: &str) -> PyResult<PyObject> {
        let rt = Runtime::new()?;
        let addr = crate::client::address::str_to_addr(addr)
            .map_err(PyAutoError::Address)?;

        let data = rt.block_on(self.inner.data_get(addr))
            .map_err(PyAutoError::Data)?;

        Python::with_gil(|py| Ok(PyBytes::new(py, &data).into()))
    }

    fn data_cost(&self, data: &[u8]) -> PyResult<u64> {
        let rt = Runtime::new()?;
        let bytes = bytes::Bytes::copy_from_slice(data);
        let cost = rt.block_on(self.inner.data_cost(bytes))
            .map_err(PyAutoError::Cost)?;
        Ok(cost.as_atto())
    }

    // Private Data Operations
    fn private_data_put(&self, data: &[u8], wallet: &Wallet) -> PyResult<String> {
        let rt = Runtime::new()?;
        let bytes = bytes::Bytes::copy_from_slice(data);
        
        let access = rt.block_on(self.inner.private_data_put(bytes, wallet.inner.clone().into()))
            .map_err(PyAutoError::Put)?;

        Ok(access.to_hex())
    }

    fn private_data_get(&self, access: &str) -> PyResult<PyObject> {
        let rt = Runtime::new()?;
        let access = crate::client::data_private::PrivateDataAccess::from_hex(access)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;

        let data = rt.block_on(self.inner.private_data_get(access))
            .map_err(PyAutoError::Data)?;

        Python::with_gil(|py| Ok(PyBytes::new(py, &data).into()))
    }

    // File Operations
    fn file_download(&self, addr: &str, path: String) -> PyResult<()> {
        let rt = Runtime::new()?;
        let addr = crate::client::address::str_to_addr(addr)
            .map_err(PyAutoError::Address)?;

        rt.block_on(self.inner.file_download(addr, PathBuf::from(path)))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
        Ok(())
    }

    fn file_upload(&self, path: String, wallet: &Wallet) -> PyResult<String> {
        let rt = Runtime::new()?;
        let addr = rt.block_on(self.inner.dir_upload(PathBuf::from(path), &wallet.inner))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;
        Ok(crate::client::address::addr_to_str(addr))
    }

    // Register Operations
    fn register_create(&self, value: &[u8], name: &str, wallet: &Wallet) -> PyResult<String> {
        let rt = Runtime::new()?;
        let key = crate::Client::register_generate_key();
        let addr = rt.block_on(self.inner.register_create(
            bytes::Bytes::copy_from_slice(value),
            name,
            key,
            &wallet.inner,
        ))
        .map_err(PyAutoError::Register)?;
        Ok(addr.to_string())
    }

    fn register_get(&self, addr: &str) -> PyResult<Vec<PyObject>> {
        let rt = Runtime::new()?;
        let addr = addr.parse()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        
        let register = rt.block_on(self.inner.register_get(addr))
            .map_err(PyAutoError::Register)?;
        
        Python::with_gil(|py| {
            register.values()
                .into_iter()
                .map(|value| Ok(PyBytes::new(py, &value).into()))
                .collect()
        })
    }

    fn register_update(&self, addr: &str, new_value: &[u8], owner: &RegisterSecretKey) -> PyResult<()> {
        let rt = Runtime::new()?;
        let addr = addr.parse()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        
        rt.block_on(self.inner.register_update(
            addr,
            bytes::Bytes::copy_from_slice(new_value),
            owner.inner.clone(),
        ))
        .map_err(PyAutoError::Register)?;
        Ok(())
    }

    // Vault Operations
    fn vault_cost(&self, owner: &VaultSecretKey) -> PyResult<u64> {
        let rt = Runtime::new()?;
        let cost = rt.block_on(self.inner.vault_cost(&owner.inner))
            .map_err(PyAutoError::Cost)?;
        Ok(cost.as_atto())
    }

    fn get_user_data_from_vault(&self, secret_key: &VaultSecretKey) -> PyResult<UserData> {
        let rt = Runtime::new()?;
        let data = rt.block_on(self.inner.get_user_data_from_vault(&secret_key.inner))
            .map_err(PyAutoError::Vault)?;
        Ok(UserData { inner: data })
    }

    fn put_user_data_to_vault(
        &self,
        secret_key: &VaultSecretKey,
        wallet: &Wallet,
        user_data: &UserData,
    ) -> PyResult<u64> {
        let rt = Runtime::new()?;
        let cost = rt.block_on(self.inner.put_user_data_to_vault(
            &secret_key.inner,
            wallet.inner.clone().into(),
            user_data.inner.clone(),
        ))
        .map_err(PyAutoError::Vault)?;
        Ok(cost.as_atto())
    }

    // External Signer Operations
    fn get_quotes_for_content_addresses(&self, addrs: Vec<&str>) -> PyResult<(Vec<(String, u64)>, Vec<(String, u64, String)>, Vec<String>)> {
        let rt = Runtime::new()?;
        
        let content_addrs = addrs.into_iter()
            .map(|addr| crate::client::address::str_to_addr(addr))
            .collect::<Result<Vec<_>, _>>()
            .map_err(PyAutoError::Address)?;

        let (quotes, payments, free_chunks) = rt.block_on(
            self.inner.get_quotes_for_content_addresses(content_addrs.into_iter())
        )
        .map_err(PyAutoError::Cost)?;

        let quotes = quotes.into_iter()
            .map(|(addr, quote)| (
                crate::client::address::addr_to_str(addr),
                quote.cost.as_atto()
            ))
            .collect();

        let payments = payments.into_iter()
            .map(|(hash, rewards_addr, amount)| (
                hash.to_string(),
                amount,
                rewards_addr.to_string()
            ))
            .collect();

        let free_chunks = free_chunks.into_iter()
            .map(|addr| crate::client::address::addr_to_str(addr))
            .collect();

        Ok((quotes, payments, free_chunks))
    }
}

#[pymethods]
impl Wallet {
    #[new]
    fn new(secret_key: Option<String>) -> PyResult<Self> {
        let wallet = if let Some(key) = secret_key {
            crate::Wallet::from_hex(&key)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?
        } else {
            crate::Wallet::random()
        };
        Ok(Self { inner: wallet })
    }

    /// Get the wallet's hex-encoded secret key
    fn to_hex(&self) -> String {
        self.inner.to_hex()
    }

    /// Get the wallet's address
    fn address(&self) -> String {
        self.inner.address().to_string()
    }

    /// Create a new random wallet
    #[staticmethod]
    fn random() -> Self {
        Self {
            inner: crate::Wallet::random()
        }
    }

    /// Create a wallet from a hex-encoded secret key
    #[staticmethod]
    fn from_hex(hex: &str) -> PyResult<Self> {
        let wallet = crate::Wallet::from_hex(hex)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        Ok(Self { inner: wallet })
    }

    /// Get the wallet's network (mainnet, testnet)
    fn network(&self) -> String {
        format!("{:?}", self.inner.network())
    }

    /// Clone the wallet
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone()
        }
    }
}

#[pymethods]
impl Archive {
    #[new]
    fn new() -> Self {
        Self {
            inner: crate::client::archive::Archive::new()
        }
    }

    /// Add a file to the archive
    fn add_file(&mut self, path: String, data_addr: &str, meta: Option<Metadata>) -> PyResult<()> {
        let addr = crate::client::address::str_to_addr(data_addr)
            .map_err(PyAutoError::Address)?;
        let meta = meta.unwrap_or_else(|| Metadata::new());
        self.inner.add_file(PathBuf::from(path), addr, meta.inner);
        Ok(())
    }

    /// Add a file with default metadata
    fn add_new_file(&mut self, path: String, data_addr: &str) -> PyResult<()> {
        let addr = crate::client::address::str_to_addr(data_addr)
            .map_err(PyAutoError::Address)?;
        self.inner.add_new_file(PathBuf::from(path), addr);
        Ok(())
    }

    /// List all files in the archive
    fn files(&self) -> Vec<(String, Metadata)> {
        self.inner.files()
            .into_iter()
            .map(|(path, meta)| (
                path.to_string_lossy().to_string(),
                Metadata { inner: meta }
            ))
            .collect()
    }

    /// List all data addresses in the archive
    fn addresses(&self) -> PyResult<Vec<String>> {
        Ok(self.inner.addresses()
            .into_iter()
            .map(|addr| crate::client::address::addr_to_str(addr))
            .collect())
    }

    /// Rename a file in the archive
    fn rename_file(&mut self, old_path: String, new_path: String) -> PyResult<()> {
        self.inner.rename_file(
            &PathBuf::from(old_path),
            &PathBuf::from(new_path)
        ).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))
    }
}

#[pymethods]
impl PrivateArchive {
    #[new]
    fn new() -> Self {
        Self {
            inner: crate::client::archive_private::PrivateArchive::new()
        }
    }

    /// Add a file to the private archive
    fn add_file(&mut self, path: String, access: &str, meta: Option<Metadata>) -> PyResult<()> {
        let access = crate::client::data_private::PrivateDataAccess::from_hex(access)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        let meta = meta.unwrap_or_else(|| Metadata::new());
        self.inner.add_file(PathBuf::from(path), access, meta.inner);
        Ok(())
    }

    /// Add a file with default metadata
    fn add_new_file(&mut self, path: String, access: &str) -> PyResult<()> {
        let access = crate::client::data_private::PrivateDataAccess::from_hex(access)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))?;
        self.inner.add_new_file(PathBuf::from(path), access);
        Ok(())
    }

    /// List all files in the archive
    fn files(&self) -> Vec<(String, Metadata)> {
        self.inner.files()
            .into_iter()
            .map(|(path, meta)| (
                path.to_string_lossy().to_string(),
                Metadata { inner: meta }
            ))
            .collect()
    }

    /// List all access keys in the archive
    fn access_keys(&self) -> Vec<String> {
        self.inner.addresses()
            .into_iter()
            .map(|access| access.to_hex())
            .collect()
    }

    /// Rename a file in the archive
    fn rename_file(&mut self, old_path: String, new_path: String) -> PyResult<()> {
        self.inner.rename_file(
            &PathBuf::from(old_path),
            &PathBuf::from(new_path)
        ).map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("{}", e)))
    }
}

#[pymethods]
impl Metadata {
    #[new]
    fn new() -> Self {
        Self {
            inner: crate::client::archive::Metadata::new()
        }
    }

    #[getter]
    fn uploaded(&self) -> u64 {
        self.inner.uploaded
    }

    #[getter]
    fn created(&self) -> u64 {
        self.inner.created
    }

    #[getter]
    fn modified(&self) -> u64 {
        self.inner.modified
    }
}

#[pymethods]
impl UserData {
    #[new]
    fn new() -> Self {
        Self {
            inner: crate::client::vault::user_data::UserData::new()
        }
    }

    /// Get the register secret key if set
    fn register_sk(&self) -> Option<String> {
        self.inner.register_sk.clone()
    }

    /// Get all register addresses and their names
    fn registers(&self) -> HashMap<String, String> {
        self.inner.registers.iter()
            .map(|(addr, name)| (addr.to_string(), name.clone()))
            .collect()
    }

    /// Get all file archives and their names
    fn file_archives(&self) -> HashMap<String, String> {
        self.inner.file_archives.iter()
            .map(|(addr, name)| (
                crate::client::address::addr_to_str(*addr),
                name.clone()
            ))
            .collect()
    }

    /// Get all private file archives and their names
    fn private_file_archives(&self) -> HashMap<String, String> {
        self.inner.private_file_archives.iter()
            .map(|(access, name)| (
                access.to_hex(),
                name.clone()
            ))
            .collect()
    }

    /// Add a file archive
    fn add_file_archive(&mut self, archive: &str) -> Option<String> {
        let addr = crate::client::address::str_to_addr(archive).ok()?;
        self.inner.add_file_archive(addr)
    }

    /// Add a file archive with a name
    fn add_file_archive_with_name(&mut self, archive: &str, name: String) -> Option<String> {
        let addr = crate::client::address::str_to_addr(archive).ok()?;
        self.inner.add_file_archive_with_name(addr, name)
    }

    /// Add a private file archive
    fn add_private_file_archive(&mut self, archive: &str) -> Option<String> {
        let access = crate::client::data_private::PrivateDataAccess::from_hex(archive).ok()?;
        self.inner.add_private_file_archive(access)
    }

    /// Add a private file archive with a name
    fn add_private_file_archive_with_name(&mut self, archive: &str, name: String) -> Option<String> {
        let access = crate::client::data_private::PrivateDataAccess::from_hex(archive).ok()?;
        self.inner.add_private_file_archive_with_name(access, name)
    }

    /// Remove a file archive
    fn remove_file_archive(&mut self, archive: &str) -> Option<String> {
        let addr = crate::client::address::str_to_addr(archive).ok()?;
        self.inner.remove_file_archive(addr)
    }

    /// Remove a private file archive
    fn remove_private_file_archive(&mut self, archive: &str) -> Option<String> {
        let access = crate::client::data_private::PrivateDataAccess::from_hex(archive).ok()?;
        self.inner.remove_private_file_archive(access)
    }
}

// Finally, add the module function to register everything
#[pymodule]
fn autonomi(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // Add constants
    m.add("CHUNK_UPLOAD_BATCH_SIZE", *crate::client::data::CHUNK_UPLOAD_BATCH_SIZE)?;
    m.add("CHUNK_DOWNLOAD_BATCH_SIZE", *crate::client::data::CHUNK_DOWNLOAD_BATCH_SIZE)?;
    m.add("FILE_UPLOAD_BATCH_SIZE", *crate::client::fs::FILE_UPLOAD_BATCH_SIZE)?;

    // Add utility function for vault content type
    #[pyfn(m)]
    fn app_name_to_vault_content_type(name: &str) -> u64 {
        crate::client::vault::app_name_to_vault_content_type(name)
    }

    // Register all classes
    m.add_class::<Client>()?;
    m.add_class::<Wallet>()?;
    m.add_class::<Archive>()?;
    m.add_class::<PrivateArchive>()?;
    m.add_class::<Metadata>()?;
    m.add_class::<RegisterSecretKey>()?;
    m.add_class::<VaultSecretKey>()?;
    m.add_class::<UserData>()?;
    m.add_class::<Receipt>()?;
    m.add_class::<PaymentOption>()?;

    Ok(())
} 