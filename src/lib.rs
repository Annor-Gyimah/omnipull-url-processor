
// lib.rs

use pyo3::prelude::*;
use std::time::Instant;

mod processor;
mod file_ops;
mod hash_ops;

#[pyclass]
struct UrlProcessResult {
    #[pyo3(get)]
    is_supported: bool,
    #[pyo3(get)]
    filename: Option<String>,
    #[pyo3(get)]
    size: Option<u64>,
    #[pyo3(get)]
    content_type: Option<String>,
    #[pyo3(get)]
    real_url: Option<String>, // Added for Hot-Swapping (TikTok etc)
    #[pyo3(get)]
    processing_time_ms: u128,
    #[pyo3(get)]
    last_error: Option<String>,
}

#[pyfunction]
#[pyo3(signature = (url, timeout, referer=None))]
fn process_url(url: &str, timeout: u64, referer: Option<String>) -> PyResult<UrlProcessResult> {
    let start = Instant::now();
    match processor::process(url, timeout, referer) {
        Ok(info) => Ok(UrlProcessResult {
            is_supported: info.is_supported,
            filename: info.filename,
            size: info.size,
            content_type: info.content_type,
            real_url: info.real_url,
            processing_time_ms: start.elapsed().as_millis(),
            last_error: info.error_msg,
        }),
        Err(e) => {
            let err_msg = format!("{:?}", e);
            Ok(UrlProcessResult {
                is_supported: false,
                filename: None,
                size: None,
                content_type: None,
                real_url: None,
                processing_time_ms: start.elapsed().as_millis(),
                last_error: Some(err_msg),
            })
        }
    }
}

#[pymodule]
fn omnipull_url_processor(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(process_url, m)?)?;
    m.add_class::<UrlProcessResult>()?;
    m.add_function(wrap_pyfunction!(file_ops::append_segment, m)?)?;
    m.add_function(wrap_pyfunction!(hash_ops::compute_sha256, m)?)?;
    Ok(())
}