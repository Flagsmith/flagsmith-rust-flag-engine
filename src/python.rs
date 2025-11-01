use crate::engine::get_evaluation_result;
use crate::engine_eval::context::EngineEvaluationContext;
use crate::engine_eval::result::EvaluationResult;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pythonize::{depythonize, pythonize};

/// Evaluates feature flags based on the provided evaluation context.
///
/// This function takes a Python dictionary representing an EvaluationContext,
/// converts it to the Rust equivalent, performs the evaluation using the
/// high-performance Rust engine, and returns the result as a Python dictionary.
///
/// # Arguments
///
/// * `py` - Python GIL token
/// * `context` - Python dictionary containing the evaluation context with keys:
///   - environment: dict with 'key' and 'name'
///   - identity: optional dict with 'identifier', 'key', and 'traits'
///   - segments: optional dict of segment contexts
///   - features: optional dict of feature contexts
///
/// # Returns
///
/// A Python dictionary containing:
/// - flags: dict mapping feature names to flag results
/// - segments: list of matched segments
///
/// # Errors
///
/// Returns a PyErr if:
/// - The input context cannot be deserialized to EngineEvaluationContext
/// - The evaluation result cannot be converted back to Python
#[pyfunction]
fn get_evaluation_result_rust(py: Python, context: &Bound<'_, PyDict>) -> PyResult<PyObject> {
    // Convert Python dict to Rust EngineEvaluationContext
    let evaluation_context: EngineEvaluationContext = depythonize(context)?;

    // Call the Rust evaluation engine
    let result: EvaluationResult = get_evaluation_result(&evaluation_context);

    // Convert Rust result back to Python dict
    let py_result = pythonize(py, &result)?;

    Ok(py_result.into())
}

/// Python module for Flagsmith flag engine Rust bindings.
///
/// This module provides the Rust implementation of feature flag evaluation.
/// Note: Due to serialization overhead, the Python implementation is actually
/// faster for typical use cases. This is kept for compatibility and future optimizations.
#[pymodule]
fn flagsmith_flag_engine_rust(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_evaluation_result_rust, m)?)?;
    Ok(())
}
