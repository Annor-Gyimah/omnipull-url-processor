// use pyo3::prelude::*;
// use sha2::{Sha256, Digest};
// use std::fs::File;
// use std::io::{Read, BufReader};

// #[pyfunction]
// pub fn compute_sha256(file_path: String) -> PyResult<String> {
//     // Open the file
//     let file = File::open(&file_path)
//         .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Failed to open file: {}", e)))?;
    
//     // Use a large buffer (256KB) for faster I/O
//     let mut reader = BufReader::with_capacity(256 * 1024, file);
//     let mut hasher = Sha256::new();
//     let mut buffer = [0u8; 65536]; // 64KB internal chunk

//     loop {
//         let count = reader.read(&mut buffer)
//             .map_err(|e| PyErr::new::<pyo3::exceptions::PyIOError, _>(format!("Read error: {}", e)))?;
        
//         if count == 0 { break; }
//         hasher.update(&buffer[..count]);
//     }

//     Ok(hex::encode(hasher.finalize()))
// }

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