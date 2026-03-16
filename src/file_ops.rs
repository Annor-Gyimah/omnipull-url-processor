


use std::fs::OpenOptions;
use std::io;
use pyo3::prelude::*;

#[pyfunction]
// We add 'py: Python' as the first argument to access GIL management
pub fn append_segment(py: Python<'_>, target_path: String, source_path: String) -> PyResult<()> {
    
    // This closure drops the GIL, allowing the Python GUI to run 
    // while the disk I/O happens in the background.
    py.allow_threads(|| {
        let mut target = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&target_path)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut source = std::fs::File::open(&source_path)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Rust's io::copy is very fast because it uses internal 
        // optimizations like larger buffers or specialized syscalls.
        io::copy(&mut source, &mut target)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        
        // Note: sync_all() forces the hardware to finish writing. 
        // It's safe, but it IS slow. If you want max speed, 
        // you can comment this out and let the OS handle the flush.
        target.sync_all().map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(())
    }).map_err(|e: io::Error| PyErr::new::<pyo3::exceptions::PyIOError, _>(e.to_string()))
}