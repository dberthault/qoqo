// Copyright © 2021-2024 HQS Quantum Simulations GmbH. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the
// License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied. See the License for the specific language governing permissions and
// limitations under the License.

#![deny(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(missing_debug_implementations)]
// #![allow(deprecated)]

//! Qoqo quantum computing toolkit
//!
//! Quantum Operation Quantum Operation
//! Yes we use [reduplication](https://en.wikipedia.org/wiki/Reduplication)

use pyo3::prelude::*;

use pyo3::types::PyDict;

use pyo3::{wrap_pyfunction, wrap_pymodule};

pub mod operations;

pub mod measurements;

pub mod devices;

mod circuit;
pub use circuit::{convert_into_circuit, CircuitWrapper, OperationIteratorWrapper};

mod quantum_program;
pub use quantum_program::{convert_into_quantum_program, QuantumProgramWrapper};

pub mod noise_models;

#[cfg(feature = "circuitdag")]
mod circuitdag;
#[cfg(feature = "circuitdag")]
pub use circuitdag::{convert_into_circuitdag, CircuitDagWrapper};

/// qoqo version information, used for qoqo import/export checks
pub const QOQO_VERSION: &str = env!("CARGO_PKG_VERSION");

use roqoqo::{operations::AVAILABLE_GATES_HQSLANG, RoqoqoBackendError, RoqoqoError};
use thiserror::Error;

/// Errors that can occur in qoqo.
#[derive(Error, Debug, PartialEq)]
pub enum QoqoError {
    /// Error an Operation cannot be extracted from PyAny object passed from python.
    #[error("Converting PyAny to Operation not possible")]
    ConversionError,
    /// Error a Circuit cannot be extracted from PyAny object passed from python.
    #[error("Cannot extract roqoqo object from python object")]
    CannotExtractObject,
    /// Error for version mismatch between separately compiled packages.
    ///
    /// Error when trying to extract a roqoqo object from a PyAny python object that has been created
    /// from a python package that has been compiled separately.
    /// To avoid unexpected behaviour this is only allowed when qoqo and roqoqo in both packages are the same version.
    #[error("Package versions of qoqo and roqoqo do not match versions of qoqo object passed from python")]
    VersionMismatch,
    /// Transparent forwarding of roqoqo errors.
    #[error(transparent)]
    RoqoqoError(#[from] RoqoqoError),
}

/// Errors that can occur in qoqo backends.
#[derive(Error, Debug, PartialEq)]
pub enum QoqoBackendError {
    /// Error a Circuit cannot be extracted from PyAny object passed from python.
    #[error("Cannot extract rust object from python object")]
    CannotExtractObject,
    /// Error for version mismatch between separately compiled packages.
    ///
    /// Error when trying to extract a Backend or Device from a PyAny python object that has been created
    /// from a python package that has been compiled separately.
    /// To avoid unexpected behaviour this is only allowed when qoqo and roqoqo in both packages are the same version.
    #[error("Package versions of qoqo backend and roqoqo backend do not match versions of qoqo object passed from python")]
    VersionMismatch,
    /// Transparent forwarding of roqoqo errors.
    #[error(transparent)]
    RoqoqoBackendError(#[from] RoqoqoBackendError),
}

/// List of hqslang of all available gates
#[pyfunction]
pub fn available_gates_hqslang() -> Vec<String> {
    AVAILABLE_GATES_HQSLANG
        .iter()
        .map(|&s| String::from(s))
        .collect::<Vec<String>>()
}

use pyo3::sync::GILOnceCell;
pub(crate) struct StruqtureVersionCell {
    pub(crate) version: String,
    cell: GILOnceCell<Py<PyAny>>,
}
impl StruqtureVersionCell {
    pub const fn new() -> Self {
        Self {
            cell: GILOnceCell::new(),
            version: String::new(),
        }
    }

    #[inline]
    pub fn get_version(&mut self, py: Python) -> String {
        if self.version == String::new() {
            let _binding = self.cell.get_or_init(py, || {
                let binding = py
                    .import("importlib.metadata")
                    .expect("Could not import importlib.metadata module for function")
                    .getattr("version")
                    .expect("Could not get version function of importlib.metadata")
                    .call1(("struqture_py",))
                    .expect("Could not get version attribute of struqture_py")
                    .unbind();
                let version: String = binding
                    .extract(py)
                    .expect("Could not extract version string");
                self.version = version;
                binding
            });
        }
        self.version.clone()
    }

    #[inline]
    pub fn get_operator(&self, py: Python, function_name: &str) -> &Py<PyAny> {
        self.cell.get_or_init(py, || {
            py.import("struqture_py.spins")
                .unwrap_or_else(|_| {
                    panic!("Could not import struqture_py.spins module for {function_name}")
                })
                .getattr("PlusMinusLindbladNoiseOperator")
                .expect("Could not get PlusMinusLindbladOperator class")
                .unbind()
        })
    }
}
pub(crate) static mut STRUQTURE_VERSION: StruqtureVersionCell = StruqtureVersionCell::new();

/// Quantum Operation Quantum Operation (qoqo)
///
/// Yes, we use reduplication.
///
/// qoqo is the HQS python package to represent quantum circuits.
///
/// .. autosummary::
///     :toctree: generated/
///
///     Circuit
///     CircuitDag
///     QuantumProgram
///     operations
///     measurements
///     devices
///     noise_models
///     available_gates_hqslang
///

#[pymodule]
fn qoqo(_py: Python, module: &Bound<PyModule>) -> PyResult<()> {
    module.add_class::<CircuitWrapper>()?;
    module.add_class::<QuantumProgramWrapper>()?;
    #[cfg(feature = "circuitdag")]
    module.add_class::<CircuitDagWrapper>()?;
    module.add_function(wrap_pyfunction!(available_gates_hqslang, module)?)?;
    let wrapper = wrap_pymodule!(operations::operations);
    module.add_wrapped(wrapper)?;
    let wrapper2 = wrap_pymodule!(measurements::measurements);
    module.add_wrapped(wrapper2)?;
    let wrapper3 = wrap_pymodule!(devices::devices);
    module.add_wrapped(wrapper3)?;
    let wrapper4 = wrap_pymodule!(noise_models::noise_models);
    module.add_wrapped(wrapper4)?;
    // Adding nice imports corresponding to maturin example
    let system = PyModule::import(_py, "sys")?;
    let binding = system.getattr("modules")?;
    let system_modules: &Bound<PyDict> = binding.downcast()?;
    system_modules.set_item("qoqo.operations", module.getattr("operations")?)?;
    system_modules.set_item("qoqo.measurements", module.getattr("measurements")?)?;
    system_modules.set_item("qoqo.devices", module.getattr("devices")?)?;
    system_modules.set_item("qoqo.noise_models", module.getattr("noise_models")?)?;
    Ok(())
}
