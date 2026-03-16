

use pyo3::prelude::*;
use sha2::{Sha256, Digest};
use std::fs::File;
use std::io::{self, Read};

#[pyfunction]
pub fn compute_sha256(py: Python<'_>, path: String) -> PyResult<String> {
    // Drop the GIL so the UI stays responsive
    py.allow_threads(|| {
        let mut file = File::open(&path).map_err(|e| {
            io::Error::new(io::ErrorKind::NotFound, format!("File not found: {}", e))
        })?;

        let mut hasher = Sha256::new();
        // 1MB buffer for optimal sequential read on SSDs
        let mut buffer = [0u8; 1024 * 1024]; 

        loop {
            let n = file.read(&mut buffer).map_err(|e| {
                io::Error::new(io::ErrorKind::Other, format!("Read error: {}", e))
            })?;
            
            if n == 0 { break; }
            hasher.update(&buffer[..n]);
        }

        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }).map_err(|e: io::Error| {
        pyo3::exceptions::PyIOError::new_err(e.to_string())
    })
}