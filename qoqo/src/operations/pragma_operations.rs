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

use crate::operations::convert_operation_to_pyobject;
use crate::{convert_into_circuit, CircuitWrapper};
use ndarray::{Array1, Array2};
use num_complex::Complex64;
use numpy::{PyArray1, PyArray2, PyReadonlyArray1, PyReadonlyArray2, ToPyArray};
use pyo3::exceptions::{PyRuntimeError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::PyByteArray;
use pyo3::types::PySet;
use qoqo_calculator::CalculatorFloat;
use qoqo_calculator_pyo3::{convert_into_calculator_float, CalculatorFloatWrapper};
use qoqo_macros::*;
use roqoqo::operations::*;
use roqoqo::Circuit;
#[cfg(feature = "json_schema")]
use roqoqo::ROQOQO_VERSION;
use std::collections::HashMap;

/// Wrap function automatically generates functions in these traits.
#[wrap(Operate, OperatePragma, JsonSchema)]
#[derive(Eq)]
/// This PRAGMA operation sets the number of measurements of the circuit.
///
/// This is used for backends that allow setting the number of tries. However, setting the number of
/// measurements does not allow access to the underlying wavefunction or density matrix.
///
/// Args:
///     number_measurements (uint): The number of measurements.
///     readout (string): The register for the readout.
struct PragmaSetNumberOfMeasurements {
    number_measurements: usize,
    readout: String,
}

#[wrap(Operate, OperatePragma, JsonSchema)]
/// This PRAGMA measurement operation returns the statevector of a quantum register.
///
/// Args:
///     repetitions (CalculatorFloat): The number of repetitions as a symbolic float. At evaluation the floor of any float value is taken
///     circuit (Circuit): The Circuit that is looped.
///
pub struct PragmaLoop {
    repetitions: CalculatorFloat,
    circuit: Circuit,
}

/// Module containing the PragmaSetStateVector class.
#[pymodule]
fn pragma_set_statevector(_py: Python, module: &Bound<PyModule>) -> PyResult<()> {
    module.add_class::<PragmaSetStateVectorWrapper>()?;
    Ok(())
}

#[pyclass(name = "PragmaSetStateVector", module = "qoqo.operations")]
#[derive(Clone, Debug, PartialEq)]
/// This PRAGMA operation sets the statevector of a quantum register.
///
/// The Circuit() module automatically initializes the qubits in the |0> state, so this PRAGMA
/// operation allows you to set the state of the qubits to a state of your choosing.
/// For instance, to initialize the psi-minus Bell state, we pass the following vector to
/// the PRAGMA:
///     vector = np.array([0, 1 / np.sqrt(2), -1 / np.sqrt(2), 0])
///
/// Args:
///     internal (PragmaSetStateVector): The statevector that is initialized.
pub struct PragmaSetStateVectorWrapper {
    /// PragmaStateVector to be wrapped and converted to Python.
    pub internal: PragmaSetStateVector,
}

insert_pyany_to_operation!(
    "PragmaSetStateVector" =>{
        let array = op.call_method0("statevector").map_err(|_| QoqoError::ConversionError)?;
        let statevec_casted: PyReadonlyArray1<Complex64> = array.extract().unwrap();
        let statevec_array: Array1<Complex64> = statevec_casted.as_array().to_owned();
        Ok(PragmaSetStateVector::new(statevec_array).into())
    }
);
insert_operation_to_pyobject!(
    Operation::PragmaSetStateVector(internal) => {
        {
            let pyref: Py<PragmaSetStateVectorWrapper> =
                Py::new(py, PragmaSetStateVectorWrapper { internal }).unwrap();
            pyref.into_pyobject(py).map(|bound| bound.as_any().to_owned()).map_err(|_| PyValueError::new_err("Unable to convert to Python object"))
        }
    }
);

#[pymethods]
impl PragmaSetStateVectorWrapper {
    /// Create a PragmaSetStateVector.
    ///
    /// Args:
    ///     statevector (List[complex]): The statevector representing the qubit register.
    ///
    /// Returns:
    ///     self: The new PragmaSetStateVector.
    #[new]
    fn new(statevector: &Bound<PyAny>) -> PyResult<Self> {
        let try_cast: PyResult<Array1<Complex64>> =
            if let Ok(extracted) = statevector.extract::<PyReadonlyArray1<Complex64>>() {
                let statevec: Array1<Complex64> = extracted.as_array().to_owned();
                Ok(statevec)
            } else if let Ok(extracted) = statevector.extract::<PyReadonlyArray1<f64>>() {
                let statevec: Array1<f64> = extracted.as_array().to_owned();
                let statevec: Array1<Complex64> = statevec
                    .into_iter()
                    .map(|f| Complex64::new(f, 0.0))
                    .collect();
                Ok(statevec)
            } else if let Ok(extracted) = statevector.extract::<PyReadonlyArray1<isize>>() {
                let statevec: Array1<isize> = extracted.as_array().to_owned();
                let statevec: Array1<Complex64> = statevec
                    .into_iter()
                    .map(|f| Complex64::new(f as f64, 0.0))
                    .collect();
                Ok(statevec)
            } else {
                Err(PyTypeError::new_err(
                    "Internal error: no successful PyReadonlyArray1 extraction.",
                ))
            };

        match try_cast {
            Ok(array) => Ok(Self {
                internal: PragmaSetStateVector::new(array),
            }),
            Err(_) => {
                let statevec_casted: Vec<Complex64> = Vec::extract_bound(statevector)?;
                let statevec_array: Array1<Complex64> = Array1::from(statevec_casted);
                Ok(Self {
                    internal: PragmaSetStateVector::new(statevec_array),
                })
            }
        }
    }

    /// Return the statevector.
    ///
    /// Returns:
    ///     np.ndarray: The statevector representing the qubit register.
    fn statevector(&self) -> Py<PyArray1<Complex64>> {
        Python::with_gil(|py| -> Py<PyArray1<Complex64>> {
            self.internal.statevector().to_pyarray(py).unbind()
        })
    }

    /// List all involved qubits (here, all).
    ///
    /// Returns:
    ///     Set[int]: The involved qubits of the PRAGMA operation.
    fn involved_qubits<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<'py, PySet>> {
        PySet::new(py, ["All"])?
            .into_pyobject(py)
            .map_err(|_| PyRuntimeError::new_err("Unable to convert to Python object"))
    }

    /// Return tags classifying the type of the operation.
    ///
    /// Used for the type based dispatch in ffi interfaces.
    ///
    /// Returns:
    ///     List[str]: The tags of the operation.
    fn tags(&self) -> Vec<String> {
        self.internal.tags().iter().map(|s| s.to_string()).collect()
    }

    /// Return hqslang name of the operation.
    ///
    /// Returns:
    ///     str: The hqslang name of the operation.
    fn hqslang(&self) -> &'static str {
        self.internal.hqslang()
    }

    /// Return true when the operation has symbolic parameters.
    ///
    /// Returns:
    ///     bool: True if the operation contains symbolic parameters, False if it does not.
    fn is_parametrized(&self) -> bool {
        self.internal.is_parametrized()
    }

    /// Substitute the symbolic parameters in a clone of the PRAGMA operation according to the substitution_parameters input.
    ///
    /// Args:
    ///     substitution_parameters (Dict[str, float]): The dictionary containing the substitutions to use in the PRAGMA operation.
    ///
    /// Returns:
    ///     self: The PRAGMA operation operation with the parameters substituted.
    ///
    /// Raises:
    ///     RuntimeError: The parameter substitution failed.
    fn substitute_parameters(
        &self,
        substitution_parameters: std::collections::HashMap<String, f64>,
    ) -> PyResult<Self> {
        let mut calculator = qoqo_calculator::Calculator::new();
        for (key, val) in substitution_parameters.iter() {
            calculator.set_variable(key, *val);
        }
        Ok(Self {
            internal: self
                .internal
                .substitute_parameters(&calculator)
                .map_err(|x| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Parameter Substitution failed: {x:?}"
                    ))
                })?,
        })
    }

    /// Remap qubits in a clone of the PRAGMA operation.
    ///
    /// Args:
    ///     mapping (Dict[int, int]): The dictionary containing the {qubit: qubit} mapping to use in the PRAGMA operation.
    ///
    /// Returns:
    ///     self: The PRAGMA operation with the qubits remapped.
    ///
    /// Raises:
    ///     RuntimeError: The qubit remapping failed.
    fn remap_qubits(&self, mapping: std::collections::HashMap<usize, usize>) -> PyResult<Self> {
        let new_internal = self
            .internal
            .remap_qubits(&mapping)
            .map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("Qubit remapping failed: "))?;
        Ok(Self {
            internal: new_internal,
        })
    }

    /// Return a copy of the PRAGMA operation (copy here produces a deepcopy).
    ///
    /// Returns:
    ///     PragmaSetStateVector: A deep copy of self.
    fn __copy__(&self) -> PragmaSetStateVectorWrapper {
        self.clone()
    }

    /// Return a deep copy of the PRAGMA operation.
    ///
    /// Returns:
    ///     PragmaSetStateVector: A deep copy of self.
    fn __deepcopy__(&self, _memodict: &Bound<PyAny>) -> PragmaSetStateVectorWrapper {
        self.clone()
    }

    /// Return a string containing a formatted (string) representation of the PRAGMA operation.
    ///
    /// Returns:
    ///     str: The string representation of the operation.
    fn __format__(&self, _format_spec: &str) -> PyResult<String> {
        Ok(format!("{:?}", self.internal))
    }

    /// Return a string containing a printable representation of the PRAGMA operation.
    ///
    /// Returns:
    ///     str: The printable string representation of the operation.
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.internal))
    }

    /// Return the __richcmp__ magic method to perform rich comparison operations on PragmaSetStateVector.
    ///
    /// Args:
    ///     self: The PragmaSetStateVector object.
    ///     other: The object to compare self to.
    ///     op: Type of comparison.
    ///
    /// Returns:
    ///     bool: Whether the two operations compared evaluated to True or False.
    fn __richcmp__(
        &self,
        other: &Bound<PyAny>,
        op: pyo3::class::basic::CompareOp,
    ) -> PyResult<bool> {
        let other = crate::operations::convert_pyany_to_operation(other).map_err(|_| {
            pyo3::exceptions::PyTypeError::new_err(
                "Right hand side cannot be converted to Operation",
            )
        })?;
        match op {
            pyo3::class::basic::CompareOp::Eq => {
                Ok(Operation::from(self.internal.clone()) == other)
            }
            pyo3::class::basic::CompareOp::Ne => {
                Ok(Operation::from(self.internal.clone()) != other)
            }
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Other comparison not implemented.",
            )),
        }
    }

    #[cfg(feature = "json_schema")]
    /// Return the JsonSchema for the json serialisation of the class.
    ///
    /// Returns:
    ///     str: The json schema serialized to json
    #[staticmethod]
    pub fn json_schema() -> String {
        let schema = schemars::schema_for!(PragmaSetStateVector);
        serde_json::to_string_pretty(&schema).expect("Unexpected failure to serialize schema")
    }

    #[cfg(feature = "json_schema")]
    /// Returns the current version of the qoqo library .
    ///
    /// Returns:
    ///     str: The current version of the library.
    #[staticmethod]
    pub fn current_version() -> String {
        ROQOQO_VERSION.to_string()
    }

    #[cfg(feature = "json_schema")]
    /// Return the minimum version of qoqo that supports this object.
    ///
    /// Returns:
    ///     str: The minimum version of the qoqo library to deserialize this object.
    pub fn min_supported_version(&self) -> String {
        let min_version: (u32, u32, u32) =
            PragmaSetStateVector::minimum_supported_roqoqo_version(&self.internal);
        format!("{}.{}.{}", min_version.0, min_version.1, min_version.2)
    }
}

/// Module containing the PragmaSetDensityMatrix class.
#[pymodule]
fn pragma_set_density_matrix(_py: Python, module: &Bound<PyModule>) -> PyResult<()> {
    module.add_class::<PragmaSetDensityMatrixWrapper>()?;
    Ok(())
}

#[pyclass(name = "PragmaSetDensityMatrix", module = "qoqo.operations")]
#[derive(Clone, Debug, PartialEq)]
/// This PRAGMA operation sets the density matrix of a quantum register.
///
/// The Circuit() module automatically initializes the qubits in the |0> state, so this PRAGMA
/// operation allows you to set the state of the qubits by setting a density matrix of your choosing.
///
/// Args:
///     density_matrix (a 2d array of complex numbers): The density matrix that is initialized.
///
pub struct PragmaSetDensityMatrixWrapper {
    /// PragmaSetDensityMatrix to be wrapped and converted to Python.
    pub internal: PragmaSetDensityMatrix,
}

insert_pyany_to_operation!(
    "PragmaSetDensityMatrix" =>{
        let array = op.call_method0("density_matrix").map_err(|_| QoqoError::ConversionError)?;
        let density_matrix_op: PyReadonlyArray2<Complex64> = array.extract().unwrap();
        let density_matrix: Array2<Complex64> = density_matrix_op.as_array().to_owned();
        Ok(PragmaSetDensityMatrix::new(density_matrix).into())
    }
);
insert_operation_to_pyobject!(
    Operation::PragmaSetDensityMatrix(internal) => {
        {
            let pyref: Py<PragmaSetDensityMatrixWrapper> =
                Py::new(py, PragmaSetDensityMatrixWrapper { internal }).unwrap();
            pyref.into_pyobject(py).map(|bound| bound.as_any().to_owned()).map_err(|_| PyValueError::new_err("Unable to convert to Python object"))

        }
    }
);

#[pymethods]
impl PragmaSetDensityMatrixWrapper {
    /// Create a PragmaSetDensityMatrix.
    ///
    /// Args:
    ///     density_matrix (Array2[complex]): The density matrix representing the qubit register.
    ///
    /// Returns:
    ///     self: The new PragmaSetDensityMatrix.
    #[new]
    fn new(density_matrix: &Bound<PyAny>) -> PyResult<Self> {
        let try_cast: PyResult<Array2<Complex64>> =
            if let Ok(extracted) = density_matrix.extract::<PyReadonlyArray2<Complex64>>() {
                let matrix: Array2<Complex64> = extracted.as_array().to_owned();
                Ok(matrix)
            } else if let Ok(extracted) = density_matrix.extract::<PyReadonlyArray2<f64>>() {
                let matrix: Array2<f64> = extracted.as_array().to_owned();
                let matrix: Array2<Complex64> = matrix.map(|f| Complex64::new(*f, 0.0));
                Ok(matrix)
            } else if let Ok(extracted) = density_matrix.extract::<PyReadonlyArray2<isize>>() {
                let matrix: Array2<isize> = extracted.as_array().to_owned();
                let matrix: Array2<Complex64> = matrix.map(|f| Complex64::new((*f) as f64, 0.0));
                Ok(matrix)
            } else {
                Err(PyTypeError::new_err(
                    "Internal error: no successful PyReadonlyArray2 extraction.",
                ))
            };
        match try_cast {
            Ok(density_matrix) => Ok(Self {
                internal: PragmaSetDensityMatrix::new(density_matrix),
            }),
            Err(_) => {
                let density_matrix_casted: Vec<Vec<Complex64>> =
                    Vec::extract_bound(density_matrix)?;
                let ncol = density_matrix_casted.first().map_or(0, |row| row.len());
                let mut density_matrix_array2: Array2<Complex64> = Array2::zeros((0, ncol));
                for subvec in density_matrix_casted {
                    let int_array1: Array1<Complex64> = Array1::from(subvec);
                    density_matrix_array2
                        .push_row((&int_array1).into())
                        .unwrap();
                }
                Ok(Self {
                    internal: PragmaSetDensityMatrix::new(density_matrix_array2),
                })
            }
        }
    }

    /// Return the set density matrix.
    ///
    /// Returns:
    ///     np.ndarray: The density matrix (2d array) representing the qubit register.
    fn density_matrix(&self) -> Py<PyArray2<Complex64>> {
        Python::with_gil(|py| -> Py<PyArray2<Complex64>> {
            self.internal.density_matrix().to_pyarray(py).unbind()
        })
    }

    /// List all involved qubits (here, all).
    ///
    /// Returns:
    ///     Set[int]: The involved qubits of the PRAGMA operation.
    fn involved_qubits<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<'py, PySet>> {
        PySet::new(py, ["All"])?
            .into_pyobject(py)
            .map_err(|_| PyRuntimeError::new_err("Unable to convert to Python object"))
    }

    /// Return tags classifying the type of the operation.
    ///
    /// Used for type based dispatch in ffi interfaces.
    ///
    /// Returns:
    ///     List[str]: The tags of the Operation.
    fn tags(&self) -> Vec<String> {
        self.internal.tags().iter().map(|s| s.to_string()).collect()
    }

    /// Return hqslang name of the operation.
    ///
    /// Returns:
    ///     str: The hqslang name of the operation.
    fn hqslang(&self) -> &'static str {
        self.internal.hqslang()
    }

    /// Return true when the operation has symbolic parameters.
    ///
    /// Returns:
    ///     bool: True if the operation contains symbolic parameters, False if it does not.
    fn is_parametrized(&self) -> bool {
        self.internal.is_parametrized()
    }

    /// Substitute the symbolic parameters in a clone of the PRAGMA operation according to the input.
    ///
    /// Args:
    ///     substitution_parameters (Dict[str, float]): The dictionary containing the substitutions to use in the PRAGMA operation.
    ///
    /// Returns:
    ///     self: The PRAGMA operation with the parameters substituted.
    ///
    /// Raises:
    ///     RuntimeError: The parameter substitution failed.
    fn substitute_parameters(
        &self,
        substitution_parameters: std::collections::HashMap<String, f64>,
    ) -> PyResult<Self> {
        let mut calculator = qoqo_calculator::Calculator::new();
        for (key, val) in substitution_parameters.iter() {
            calculator.set_variable(key, *val);
        }
        Ok(Self {
            internal: self
                .internal
                .substitute_parameters(&calculator)
                .map_err(|x| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Parameter Substitution failed: {x:?}"
                    ))
                })?,
        })
    }

    /// Remap qubits in a clone of the PRAGMA operation.
    ///
    /// Args:
    ///     mapping (Dict[int, int]): The dictionary containing the {qubit: qubit} mapping to use in the PRAGMA operation.
    ///
    /// Returns:
    ///     self: The PRAGMA operation with the qubits remapped.
    ///
    /// Raises:
    ///     RuntimeError: The qubit remapping failed.
    fn remap_qubits(&self, mapping: std::collections::HashMap<usize, usize>) -> PyResult<Self> {
        let new_internal = self
            .internal
            .remap_qubits(&mapping)
            .map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("Qubit remapping failed: "))?;
        Ok(Self {
            internal: new_internal,
        })
    }

    /// Return a copy of the PRAGMA operation (copy here produces a deepcopy).
    ///
    /// Returns:
    ///     PragmaSetDensityMatrix: A deep copy of self.
    fn __copy__(&self) -> PragmaSetDensityMatrixWrapper {
        self.clone()
    }

    /// Return a deep copy of the PRAGMA operation.
    ///
    /// Returns:
    ///     PragmaSetDensityMatrix: A deep copy of self.
    fn __deepcopy__(&self, _memodict: &Bound<PyAny>) -> PragmaSetDensityMatrixWrapper {
        self.clone()
    }

    /// Return a string containing a formatted (string) representation of the PRAGMA operation.
    ///
    /// Returns:
    ///     str: The string representation of the operation.
    fn __format__(&self, _format_spec: &str) -> PyResult<String> {
        Ok(format!("{:?}", self.internal))
    }

    /// Return a string containing a printable representation of the PRAGMA operation.
    ///
    /// Returns:
    ///     str: The printable string representation of the operation.
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.internal))
    }

    /// Return the __richcmp__ magic method to perform rich comparison operations on PragmaSetStateVector.
    ///
    /// Args:
    ///     self: The PragmaSetDensityMatrix object.
    ///     other: The object to compare self to.
    ///     op: Type of comparison.
    ///
    /// Returns:
    ///     bool: Whether the two operations compared evaluated to True or False.
    fn __richcmp__(
        &self,
        other: &Bound<PyAny>,
        op: pyo3::class::basic::CompareOp,
    ) -> PyResult<bool> {
        let other = crate::operations::convert_pyany_to_operation(other).map_err(|_| {
            pyo3::exceptions::PyTypeError::new_err(
                "Right hand side cannot be converted to Operation",
            )
        })?;
        match op {
            pyo3::class::basic::CompareOp::Eq => {
                Ok(Operation::from(self.internal.clone()) == other)
            }
            pyo3::class::basic::CompareOp::Ne => {
                Ok(Operation::from(self.internal.clone()) != other)
            }
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Other comparison not implemented.",
            )),
        }
    }

    #[cfg(feature = "json_schema")]
    /// Return the JsonSchema for the json serialisation of the class.
    ///
    /// Returns:
    ///     str: The json schema serialized to json
    #[staticmethod]
    pub fn json_schema() -> String {
        let schema = schemars::schema_for!(PragmaSetDensityMatrix);
        serde_json::to_string_pretty(&schema).expect("Unexpected failure to serialize schema")
    }

    #[cfg(feature = "json_schema")]
    /// Returns the current version of the qoqo library .
    ///
    /// Returns:
    ///     str: The current version of the library.
    #[staticmethod]
    pub fn current_version() -> String {
        ROQOQO_VERSION.to_string()
    }

    #[cfg(feature = "json_schema")]
    /// Return the minimum version of qoqo that supports this object.
    ///
    /// Returns:
    ///     str: The minimum version of the qoqo library to deserialize this object.
    pub fn min_supported_version(&self) -> String {
        let min_version: (u32, u32, u32) =
            PragmaSetDensityMatrix::minimum_supported_roqoqo_version(&self.internal);
        format!("{}.{}.{}", min_version.0, min_version.1, min_version.2)
    }
}

#[wrap(Operate, OperatePragma, JsonSchema)]
#[derive(Eq)]
/// The repeated gate PRAGMA operation.
///
/// This PRAGMA operation repeats the next gate in the circuit the given number of times
/// to increase the rate for error mitigation.
///
/// Args:
///     repetition_coefficient (int): The number of times the following gate is repeated.
struct PragmaRepeatGate {
    repetition_coefficient: usize,
}

#[wrap(Operate, OperatePragma, OperateMultiQubit, JsonSchema)]
/// The statistical overrotation PRAGMA operation.
///
/// This PRAGMA applies a statistical overrotation to the next rotation gate in the circuit, which
/// matches the hqslang name in the `gate` parameter of PragmaOverrotation and the involved qubits in `qubits`.
///
/// The applied overrotation corresponds to adding a random number to the rotation angle.
/// The random number is drawn from a normal distribution with mean `0`
/// and standard deviation `variance` and is multiplied by the `amplitude`.
///
/// Args:
///     gate (str): The unique hqslang name of the gate to overrotate.
///     qubits (List[int]): The qubits of the gate to overrotate.
///     amplitude (float): The amplitude the random number is multiplied by.
///     variance (float): The standard deviation of the normal distribution the random number is drawn from.
///
// #[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
struct PragmaOverrotation {
    gate_hqslang: String,
    qubits: Vec<usize>,
    amplitude: f64,
    variance: f64,
}

#[wrap(Operate, OperatePragma, JsonSchema)]
/// This PRAGMA operation boosts noise and overrotations in the circuit.
///
/// Args:
///     noise_coefficient (CalculatorFloat): The coefficient by which the noise is boosted.
struct PragmaBoostNoise {
    noise_coefficient: CalculatorFloat,
}

#[wrap(Operate, OperateMultiQubit, OperatePragma, JsonSchema)]
/// This PRAGMA operation signals the STOP of a parallel execution block.
///
/// Args:
///     qubits (List[int]): The qubits involved in parallel execution block.
///     execution_time (CalculatorFloat): The time for the execution of the block in seconds.
struct PragmaStopParallelBlock {
    qubits: Vec<usize>,
    execution_time: CalculatorFloat,
}

#[wrap(Operate, JsonSchema)]
/// The global phase PRAGMA operation.
///
/// This PRAGMA operation signals that the quantum register picks up a global phase,
/// i.e. it provides information that there is a global phase to be considered.
///
/// Args:
///     phase (CalculatorFloat): The picked up global phase.
struct PragmaGlobalPhase {
    phase: CalculatorFloat,
}

#[wrap(Operate, OperateMultiQubit, OperatePragma, JsonSchema)]
/// This PRAGMA operation makes the quantum hardware wait a given amount of time.
///
/// This PRAGMA operation is used for error mitigation reasons, for instance.
/// It can be used to boost the noise on the qubits since it gets worse with time.
///
/// Args:
///     qubits (List[int]): The qubits involved in the sleep block.
///     sleep_time (CalculatorFloat): The time for the execution of the block in seconds.
pub struct PragmaSleep {
    qubits: Vec<usize>,
    sleep_time: CalculatorFloat,
}

#[wrap(Operate, OperateSingleQubit, OperatePragma, JsonSchema)]
#[derive(Eq)]
/// This PRAGMA operation resets the chosen qubit to the zero state.
///
/// Args:
///     qubit (int): The qubit to be reset.
pub struct PragmaActiveReset {
    qubit: usize,
}

#[wrap(Operate, OperateMultiQubit, OperatePragma, JsonSchema)]
#[derive(Eq)]
/// This PRAGMA operation signals the START of a decomposition block.
///
/// Args:
///     qubits (List[int]): The qubits involved in the decomposition block.
///     reordering_dictionary (Dict[int, int]): The reordering dictionary of the block.
pub struct PragmaStartDecompositionBlock {
    qubits: Vec<usize>,
    reordering_dictionary: HashMap<usize, usize>,
}

#[wrap(Operate, OperateMultiQubit, OperatePragma, JsonSchema)]
#[derive(Eq)]
/// This PRAGMA operation signals the STOP of a decomposition block.
///
/// Args:
///     qubits (List[int]): The qubits involved in the decomposition block.
pub struct PragmaStopDecompositionBlock {
    qubits: Vec<usize>,
}

#[wrap(
    Operate,
    OperateSingleQubit,
    OperatePragma,
    OperatePragmaNoise,
    OperatePragmaNoiseProba,
    JsonSchema
)]
/// The damping PRAGMA noise operation.
///
/// This PRAGMA operation applies a pure damping error corresponding to zero temperature environments.
///
/// Note
///
/// Damping means going from state `|1>` to `|0>` and corresponds to zero-temperature in a physical
/// device where `|0>` is the ground state.
/// With respect to the definition of the Pauli operator `Z`, `|0>` is the excited state and damping leads to
/// an increase in energy.
///
/// Args:
///     qubit (int): The qubit on which to apply the damping.
///     gate_time (CalculatorFloat): The time (in seconds) the gate takes to be applied to the qubit on the (simulated) hardware
///     rate (CalculatorFloat): The error rate of the damping (in 1/second).
pub struct PragmaDamping {
    qubit: usize,
    gate_time: CalculatorFloat,
    rate: CalculatorFloat,
}

// #[pymethods]
// impl PragmaDampingWrapper {
//     /// Return the superoperator defining the evolution of the density matrix under the noise gate.
//     ///
//     /// Returns:
//     ///     np.ndarray: The superoperator representation of the PRAGMA operation.
//     pub fn superoperator(&self) -> PyResult<Py<PyArray2<f64>>> {
//         Ok(Python::with_gil(|py| -> Py<PyArray2<f64>> {
//             self.internal
//                 .superoperator()
//                 .unwrap()
//                 .to_pyarray(py)
//                 .to_owned()
//         }))
//     }
//     /// Return the probability of the noise gate affecting the qubit, based on its `gate_time` and `rate`.
//     ///
//     /// Returns:
//     ///     CalculatorFloat: The probability of the PRAGMA operation.
//     pub fn probability(&self) -> CalculatorFloatWrapper {
//         CalculatorFloatWrapper {
//             internal: self.internal.probability(),
//         }
//     }
//     /// Takes the power of the PRAGMA noise operation.
//     ///
//     /// Args:
//     ///     power (CalculatorFloat): The exponent in the power operation of the noise gate.
//     ///
//     /// Returns:
//     ///     self: The PRAGMA operation to the power of `power`.
//     pub fn powercf(&self, power: CalculatorFloatWrapper) -> Self {
//         Self {
//             internal: self.internal.powercf(power.internal),
//         }
//     }
// }

#[wrap(
    Operate,
    OperateSingleQubit,
    OperatePragma,
    OperatePragmaNoise,
    OperatePragmaNoiseProba,
    JsonSchema
)]
/// The depolarising PRAGMA noise operation.
///
/// This PRAGMA operation applies a depolarising error corresponding to infinite temperature environments.
///
/// Args:
///     qubit (int): The qubit on which to apply the depolarising.
///     gate_time (CalculatorFloat): The time (in seconds) the gate takes to be applied to the qubit on the (simulated) hardware
///     rate (CalculatorFloat): The error rate of the depolarisation (in 1/second).
pub struct PragmaDepolarising {
    qubit: usize,
    gate_time: CalculatorFloat,
    rate: CalculatorFloat,
}

// #[pymethods]
// impl PragmaDepolarisingWrapper {
//     /// Return the superoperator defining the evolution of the density matrix under the noise gate.
//     ///
//     /// Returns:
//     ///     np.ndarray: The superoperator representation of the PRAGMA operation.
//     pub fn superoperator(&self) -> PyResult<Py<PyArray2<f64>>> {
//         Ok(Python::with_gil(|py| -> Py<PyArray2<f64>> {
//             self.internal
//                 .superoperator()
//                 .unwrap()
//                 .to_pyarray(py)
//                 .to_owned()
//         }))
//     }
//     /// Return the probability of the noise gate affecting the qubit, based on its `gate_time` and `rate`.
//     ///
//     /// Returns:
//     ///     CalculatorFloat: The probability of the PRAGMA operation.
//     pub fn probability(&self) -> CalculatorFloatWrapper {
//         CalculatorFloatWrapper {
//             internal: self.internal.probability(),
//         }
//     }
//     /// Take the power of the noise PRAGMA operation.
//     ///
//     /// Args:
//     ///     power (CalculatorFloat): The exponent in the power operation of the noise gate.
//     ///
//     /// Returns:
//     ///     self: The PRAGMA operation to the power of `power`.
//     pub fn powercf(&self, power: CalculatorFloatWrapper) -> Self {
//         Self {
//             internal: self.internal.powercf(power.internal),
//         }
//     }
// }

#[wrap(
    Operate,
    OperateSingleQubit,
    OperatePragma,
    OperatePragmaNoise,
    OperatePragmaNoiseProba,
    JsonSchema
)]
/// The dephasing PRAGMA noise operation.
///
/// This PRAGMA operation applies a pure dephasing error.
///
/// Args:
///     qubit (int): The qubit on which to apply the dephasing.
///     gate_time (CalculatorFloat): The time (in seconds) the gate takes to be applied to the qubit on the (simulated) hardware
///     rate (CalculatorFloat): The error rate of the dephasing (in 1/second).
pub struct PragmaDephasing {
    qubit: usize,
    gate_time: CalculatorFloat,
    rate: CalculatorFloat,
}

// #[pymethods]
// impl PragmaDephasingWrapper {
//     /// Return the superoperator defining the evolution of the density matrix under the noise gate.
//     ///
//     /// Returns:
//     ///     np.ndarray: The superoperator representation of the PRAGMA operation.
//     pub fn superoperator(&self) -> PyResult<Py<PyArray2<f64>>> {
//         Ok(Python::with_gil(|py| -> Py<PyArray2<f64>> {
//             self.internal
//                 .superoperator()
//                 .unwrap()
//                 .to_pyarray(py)
//                 .to_owned()
//         }))
//     }
//     /// Return the probability of the noise gate affecting the qubit, based on its `gate_time` and `rate`.
//     ///
//     /// Returns:
//     ///     CalculatorFloat: The probability of the PRAGMA operation.
//     pub fn probability(&self) -> CalculatorFloatWrapper {
//         CalculatorFloatWrapper {
//             internal: self.internal.probability(),
//         }
//     }
//     /// Take the power of the noise PRAGMA operation.
//     ///
//     /// Args:
//     ///     power (CalculatorFloat): The exponent in the power operation of the noise gate.
//     ///
//     /// Returns:
//     ///     self: The PRAGMA operation to the power of `power`.
//     pub fn powercf(&self, power: CalculatorFloatWrapper) -> Self {
//         Self {
//             internal: self.internal.powercf(power.internal),
//         }
//     }
// }

#[wrap(
    Operate,
    OperateSingleQubit,
    OperatePragma,
    OperatePragmaNoise,
    OperatePragmaNoiseProba,
    JsonSchema
)]
/// The random noise PRAGMA operation.
///
/// This PRAGMA operation applies a pure damping error corresponding to zero temperature environments.
///
/// Args:
///     qubit (int): The qubit on which to apply the damping.
///     gate_time (CalculatorFloat): The time (in seconds) the gate takes to be applied to the qubit on the (simulated) hardware
///     depolarising_rate (CalculatorFloat): The error rate of the depolarisation (in 1/second).
///     dephasing_rate (CalculatorFloat): The error rate of the dephasing (in 1/second).
pub struct PragmaRandomNoise {
    qubit: usize,
    gate_time: CalculatorFloat,
    depolarising_rate: CalculatorFloat,
    dephasing_rate: CalculatorFloat,
}

// #[pymethods]
// impl PragmaRandomNoiseWrapper {
//     /// Return the superoperator defining the evolution of the density matrix under the noise gate.
//     ///
//     /// Returns:
//     ///     np.ndarray: The superoperator representation of the PRAGMA operation.
//     pub fn superoperator(&self) -> PyResult<Py<PyArray2<f64>>> {
//         Ok(Python::with_gil(|py| -> Py<PyArray2<f64>> {
//             self.internal
//                 .superoperator()
//                 .unwrap()
//                 .to_pyarray(py)
//                 .to_owned()
//         }))
//     }
//     /// Return the probability of the noise gate affecting the qubit, based on its `gate_time` and `rate`.
//     ///
//     /// Returns:
//     ///     CalculatorFloat: The probability of the PRAGMA operation.
//     pub fn probability(&self) -> CalculatorFloatWrapper {
//         CalculatorFloatWrapper {
//             internal: self.internal.probability(),
//         }
//     }
//     /// Take the power of the noise PRAGMA operation.
//     ///
//     /// Args:
//     ///     power (CalculatorFloat): The exponent in the power operation of the noise gate.
//     ///
//     /// Returns:
//     ///     self: The PRAGMA operation to the power of `power`.
//     pub fn powercf(&self, power: CalculatorFloatWrapper) -> Self {
//         Self {
//             internal: self.internal.powercf(power.internal),
//         }
//     }
// }

/// Module containing the PragmaGeneralNoise class.
#[pymodule]
fn pragma_general_noise(_py: Python, module: &Bound<PyModule>) -> PyResult<()> {
    module.add_class::<PragmaGeneralNoiseWrapper>()?;
    Ok(())
}

#[pyclass(name = "PragmaGeneralNoise", module = "qoqo.operations")]
#[derive(Clone, Debug, PartialEq)]
/// The general noise PRAGMA operation.
///
/// This PRAGMA operation applies a noise term according to the given operators.
///
/// Args:
///     qubit (int): The qubit the PRAGMA operation is applied to.
///     gate_time (CalculatorFloat): The time (in seconds) the gate takes to be applied to the qubit on the (simulated) hardware
///     Rates: The rates representing the general noise matrix M (a 3x3 matrix as 2d array).
///
pub struct PragmaGeneralNoiseWrapper {
    /// PragmaGeneralNoise to be wrapped and converted to Python.
    pub internal: PragmaGeneralNoise,
}

insert_pyany_to_operation!(
    "PragmaGeneralNoise" =>{
        let qbt = op.call_method0("qubit")
                    .map_err(|_| QoqoError::ConversionError)?;
        let qubit: usize = qbt.extract()
                              .map_err(|_| QoqoError::ConversionError)?;

        let gatetm = &op.call_method0("gate_time")
                      .map_err(|_| QoqoError::ConversionError)?;
        let gate_time: CalculatorFloat = convert_into_calculator_float(gatetm).map_err(|_| {
            QoqoError::ConversionError
        })?;

        let array = op.call_method0("rates")
                      .map_err(|_| QoqoError::ConversionError)?;
        let rates_array: PyReadonlyArray2<f64> = array.extract().unwrap();
        let rates: Array2<f64> = rates_array.as_array().to_owned();

        Ok(PragmaGeneralNoise::new(qubit, gate_time, rates).into())
    }
);
insert_operation_to_pyobject!(
    Operation::PragmaGeneralNoise(internal) => {
        {
            let pyref: Py<PragmaGeneralNoiseWrapper> =
                Py::new(py, PragmaGeneralNoiseWrapper { internal }).unwrap();
            pyref.into_pyobject(py).map(|bound| bound.as_any().to_owned()).map_err(|_| PyValueError::new_err("Unable to convert to Python object"))
        }
    }
);

#[pymethods]
impl PragmaGeneralNoiseWrapper {
    /// Create a PragmaGeneralNoise.
    ///
    /// This PRAGMA operation applies a noise term according to the given operators.
    /// The operators are represented by a 3x3 matrix:
    ///
    /// .. math ::
    /// M = \begin{pmatrix}
    /// a & b & c \\
    /// d & e & f \\
    /// g & h & j \\
    /// \end{pmatrix}
    ///
    /// where the coefficients correspond to the following summands
    /// expanded from the first term of the non-coherent part of the Lindblad equation:
    ///
    ///     .. math::
    ///     \frac{d}{dt}\rho = \sum_{i,j=0}^{2} M_{i,j} L_{i} \rho L_{j}^{\dagger} - \frac{1}{2} \{ L_{j}^{\dagger} L_i, \rho \} \\
    ///         L_0 = \sigma^{+} \\
    ///         L_1 = \sigma^{-} \\
    ///         L_3 = \sigma^{z}
    ///
    /// Applying the Pragma with a given `gate_time` corresponds to applying the full time-evolution under the Lindblad equation for `gate_time` time.
    ///
    /// Args:
    ///     qubit (int): The qubit the PRAGMA operation is applied to.
    ///     gate_time (CalculatorFloat): The time (in seconds) the gate takes to be applied to the qubit on the (simulated) hardware
    ///     rates (Array2[float]): The rate matrix M.
    ///
    /// Returns:
    ///     self: The new PragmaGeneralNoise.
    #[new]
    fn new(qubit: usize, gate_time: &Bound<PyAny>, rates: &Bound<PyAny>) -> PyResult<Self> {
        let rates_array: Array2<f64> =
            if let Ok(rates_pyarray) = rates.extract::<PyReadonlyArray2<f64>>() {
                rates_pyarray.as_array().to_owned()
            } else {
                let rates_casted: Vec<Vec<f64>> = Vec::extract_bound(rates)?;
                let ncol = rates_casted.first().map_or(0, |row| row.len());
                let mut rates_array2: Array2<f64> = Array2::zeros((0, ncol));
                for subvec in rates_casted {
                    let int_array1: Array1<f64> = Array1::from(subvec);
                    rates_array2.push_row((&int_array1).into()).unwrap();
                }
                rates_array2
            };
        let gate_time_cf = convert_into_calculator_float(gate_time).map_err(|_| {
            pyo3::exceptions::PyTypeError::new_err(
                "Argument gate time cannot be converted to CalculatorFloat",
            )
        })?;

        Ok(Self {
            internal: PragmaGeneralNoise::new(qubit, gate_time_cf, rates_array),
        })
    }

    /// Return the qubit on which the PRAGMA operation is applied.
    ///
    /// Returns:
    ///     int: The qubit of the PRAGMA operation.
    fn qubit(&self) -> usize {
        *self.internal.qubit()
    }

    /// Return the `gate_time` of the PRAGMA operation.
    ///
    /// Returns:
    ///     CalculatorFloat: The gate time of the PRAGMA operation.
    fn gate_time(&self) -> CalculatorFloatWrapper {
        CalculatorFloatWrapper {
            internal: self.internal.gate_time().clone(),
        }
    }

    /// Return the rates of the PRAGMA operation.
    ///
    /// Returns:
    ///     np.ndarray: The rates of the PRAGMA operation.
    fn rates(&self) -> Py<PyArray2<f64>> {
        Python::with_gil(|py| -> Py<PyArray2<f64>> {
            self.internal.rates().to_pyarray(py).unbind()
        })
    }

    /// Return the superoperator of the PRAGMA operation.
    ///
    /// Returns:
    ///     np.ndarray: The matrix form of the superoperator of the PRAGMA operation.
    fn superoperator(&self) -> PyResult<Py<PyArray2<f64>>> {
        Python::with_gil(|py| -> PyResult<Py<PyArray2<f64>>> {
            match self.internal.superoperator() {
                Ok(x) => Ok(x.to_pyarray(py).unbind()),
                Err(err) => Err(PyRuntimeError::new_err(format!("{err:?}"))),
            }
        })
    }

    /// List all involved qubits.
    ///
    /// Returns:
    ///     Set[int]: The involved qubits of the PRAGMA operation.
    fn involved_qubits<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<'py, PySet>> {
        PySet::new(py, [*self.internal.qubit()])?
            .into_pyobject(py)
            .map_err(|_| PyRuntimeError::new_err("Unable to convert to Python object"))
    }

    /// Return tags classifying the type of the operation.
    ///
    /// Used for the type based dispatch in ffi interfaces.
    ///
    /// Returns:
    ///     List[str]: The tags of the Operation.
    fn tags(&self) -> Vec<String> {
        self.internal.tags().iter().map(|s| s.to_string()).collect()
    }

    /// Return hqslang name of the operation.
    ///
    /// Returns:
    ///     str: The hqslang name of the operation.
    fn hqslang(&self) -> &'static str {
        self.internal.hqslang()
    }

    /// Return true when the operation has symbolic parameters.
    ///
    /// Returns:
    ///     bool: True if the operation contains symbolic parameters, False if it does not.
    fn is_parametrized(&self) -> bool {
        self.internal.is_parametrized()
    }

    /// Substitute the symbolic parameters in a clone of the PRAGMA operation according to the input.
    ///
    /// Args:
    ///     substitution_parameters (Dict[str, float]): The dictionary containing the substitutions to use in the PRAGMA operation.
    ///
    /// Returns:
    ///     self: The PRAGMA operation with the parameters substituted.
    ///
    /// Raises:
    ///     RuntimeError: The parameter substitution failed.
    fn substitute_parameters(
        &self,
        substitution_parameters: std::collections::HashMap<String, f64>,
    ) -> PyResult<Self> {
        let mut calculator = qoqo_calculator::Calculator::new();
        for (key, val) in substitution_parameters.iter() {
            calculator.set_variable(key, *val);
        }
        Ok(Self {
            internal: self
                .internal
                .substitute_parameters(&calculator)
                .map_err(|x| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Parameter Substitution failed: {x:?}"
                    ))
                })?,
        })
    }

    /// Remap qubits in a clone of the PRAGMA operation.
    ///
    /// Args:
    ///     mapping (Dict[int, int]): The dictionary containing the {qubit: qubit} mapping to use in the PRAGMA operation.
    ///
    /// Returns:
    ///     self: The PRAGMA operation with the qubits remapped.
    ///
    /// Raises:
    ///     RuntimeError: The qubit remapping failed.
    fn remap_qubits(&self, mapping: std::collections::HashMap<usize, usize>) -> PyResult<Self> {
        let new_internal = self
            .internal
            .remap_qubits(&mapping)
            .map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("Qubit remapping failed: "))?;
        Ok(Self {
            internal: new_internal,
        })
    }

    /// Return a copy of the PRAGMA operation (copy here produces a deepcopy).
    ///
    /// Returns:
    ///     PragmaGeneralNoise: A deep copy of self.
    fn __copy__(&self) -> PragmaGeneralNoiseWrapper {
        self.clone()
    }

    /// Return a deep copy of the PRAGMA operation.
    ///
    /// Returns:
    ///     PragmaGeneralNoise: A deep copy of self.
    fn __deepcopy__(&self, _memodict: &Bound<PyAny>) -> PragmaGeneralNoiseWrapper {
        self.clone()
    }

    /// Return a string containing a formatted (string) representation of the PRAGMA operation.
    ///
    /// Returns:
    ///     str: The string representation of the operation.
    fn __format__(&self, _format_spec: &str) -> PyResult<String> {
        Ok(format!("{:?}", self.internal))
    }

    /// Return a string containing a printable representation of the PRAGMA operation.
    ///
    /// Returns:
    ///     str: The printable string representation of the operation.
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.internal))
    }

    /// Return the __richcmp__ magic method to perform rich comparison operations on PragmaSetStateVector.
    ///
    /// Args:
    ///     self: The PragmaGeneralNoise object.
    ///     other: The object to compare self to.
    ///     op: Type of comparison.
    ///
    /// Returns:
    ///     bool: Whether the two operations compared evaluated to True or False.
    fn __richcmp__(
        &self,
        other: &Bound<PyAny>,
        op: pyo3::class::basic::CompareOp,
    ) -> PyResult<bool> {
        let other = crate::operations::convert_pyany_to_operation(other).map_err(|_| {
            pyo3::exceptions::PyTypeError::new_err(
                "Right hand side cannot be converted to Operation",
            )
        })?;
        match op {
            pyo3::class::basic::CompareOp::Eq => {
                Ok(Operation::from(self.internal.clone()) == other)
            }
            pyo3::class::basic::CompareOp::Ne => {
                Ok(Operation::from(self.internal.clone()) != other)
            }
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Other comparison not implemented.",
            )),
        }
    }

    #[cfg(feature = "json_schema")]
    /// Return the JsonSchema for the json serialisation of the class.
    ///
    /// Returns:
    ///     str: The json schema serialized to json
    #[staticmethod]
    pub fn json_schema() -> String {
        let schema = schemars::schema_for!(PragmaGeneralNoise);
        serde_json::to_string_pretty(&schema).expect("Unexpected failure to serialize schema")
    }

    #[cfg(feature = "json_schema")]
    /// Returns the current version of the qoqo library .
    ///
    /// Returns:
    ///     str: The current version of the library.
    #[staticmethod]
    pub fn current_version() -> String {
        ROQOQO_VERSION.to_string()
    }

    #[cfg(feature = "json_schema")]
    /// Return the minimum version of qoqo that supports this object.
    ///
    /// Returns:
    ///     str: The minimum version of the qoqo library to deserialize this object.
    pub fn min_supported_version(&self) -> String {
        let min_version: (u32, u32, u32) =
            PragmaGeneralNoise::minimum_supported_roqoqo_version(&self.internal);
        format!("{}.{}.{}", min_version.0, min_version.1, min_version.2)
    }
}

#[wrap(Operate, OperatePragma, JsonSchema)]
/// The conditional PRAGMA operation.
///
/// This PRAGMA executes a circuit when the condition bit/bool stored in a classical bit register is true.
///
/// Args:
///     condition_register (str): The name of the bit register containting the condition bool value.
///     condition_index (int): - The index in the bit register containting the condition bool value.
///     circuit (Circuit): - The circuit executed if the condition is met.
pub struct PragmaConditional {
    condition_register: String,
    condition_index: usize,
    circuit: Circuit,
}

#[wrap(Operate, OperatePragma, JsonSchema)]
/// A circuit controlled by a qubit.
///
/// The circuit is applied when the qubit is in state 1.
/// Note that this is a unitary operation (for example a CNOT(0,1)
/// is equvalent to a PragmaControlledCircuit(0, [PauliX(1)]) but it cannot be represented
/// by a unitary operation in qoqo for arbitraty circuits.
///
/// Args:
///     controlling_qubit (int): - The qubit controlling circuit application.
///     circuit (Circuit): - The circuit executed if the condition is met.
pub struct PragmaControlledCircuit {
    controlling_qubit: usize,
    circuit: Circuit,
}

#[pyclass(name = "PragmaChangeDevice", module = "qoqo.operations")]
#[derive(Clone, Debug, PartialEq, Eq)]
/// A wrapper around backend specific PRAGMA operations capable of changing a device.
///
/// This PRAGMA is a thin wrapper around device specific operations that can change
/// device properties.
pub struct PragmaChangeDeviceWrapper {
    /// PragmaGeneralNoise to be wrapped and converted to Python.
    pub internal: PragmaChangeDevice,
}

insert_pyany_to_operation!(
    "PragmaChangeDevice" =>{
        let wt = op.call_method0( "wrapped_tags").map_err(|_|QoqoError::ConversionError)?;
        let wrapped_tags: Vec<String> = wt.extract()
                                  .map_err(|_| QoqoError::ConversionError)?;
        let wh = op.call_method0( "wrapped_hqslang").map_err(|_|QoqoError::ConversionError)?;
        let wrapped_hqslang: String = wh.extract()
                                      .map_err(|_|QoqoError::ConversionError)?;
        let wo = op.call_method0( "wrapped_operation").map_err(|_|QoqoError::ConversionError)?;
        let wrapped_operation: Vec<u8> = wo.extract()
                                        .map_err(|_|QoqoError::ConversionError)?;
           Ok( PragmaChangeDevice{wrapped_tags, wrapped_hqslang, wrapped_operation}.into())
    }
);
insert_operation_to_pyobject!(
    Operation::PragmaChangeDevice(internal) => {
        {
            let pyref: Py<PragmaChangeDeviceWrapper> =
                Py::new(py, PragmaChangeDeviceWrapper { internal }).unwrap();
            pyref.into_pyobject(py).map(|bound| bound.as_any().to_owned()).map_err(|_| PyValueError::new_err("Unable to convert to Python object"))
        }
    }
);

#[pymethods]
impl PragmaChangeDeviceWrapper {
    /// A PragmaChangeDevice cannot be created directly.
    ///
    /// The intended mechanism for the creation of PragmaChangeDevice is to create a device specific Pragma
    /// and call the .to_pragma_change_device() function.
    #[new]
    fn new() -> PyResult<Self> {
        Err(PyTypeError::new_err("A PragmaChangeDevice wrapper Pragma cannot be created directly, use a .to_pragma_change_device() from the wrapped PRAGMA instead"))
    }

    /// Return the tags of the wrapped operations.
    ///
    /// Returns:
    ///     List[str]: The list of tags.
    fn wrapped_tags(&self) -> Vec<String> {
        self.internal
            .wrapped_tags
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Return the hqslang name of the wrapped operations.
    ///
    /// Returns:
    ///     str: The name of the wrapped operation.
    fn wrapped_hqslang(&self) -> String {
        self.internal.wrapped_hqslang.to_string()
    }

    /// Return the binary representation of the wrapped operations.
    ///
    /// Returns:
    ///     ByteArray: The the binary representation of the wrapped operation.
    fn wrapped_operation(&self) -> PyResult<Py<PyByteArray>> {
        let serialized: Vec<u8> = self.internal.wrapped_operation.clone();
        let b: Py<PyByteArray> = Python::with_gil(|py| -> Py<PyByteArray> {
            PyByteArray::new(py, &serialized[..]).into()
        });
        Ok(b)
    }

    /// List all involved qubits.
    ///
    /// Returns:
    ///     Set[int]: The involved qubits of the PRAGMA operation.
    fn involved_qubits<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<'py, PySet>> {
        PySet::new(py, ["All"])?
            .into_pyobject(py)
            .map_err(|_| PyRuntimeError::new_err("Unable to convert to Python object"))
    }

    /// Return tags classifying the type of the operation.
    ///
    /// Used for the type based dispatch in ffi interfaces.
    ///
    /// Returns:
    ///     List[str]: The tags of the Operation.
    fn tags(&self) -> Vec<String> {
        self.internal.tags().iter().map(|s| s.to_string()).collect()
    }

    /// Return hqslang name of the operation.
    ///
    /// Returns:
    ///     str: The hqslang name of the operation.
    fn hqslang(&self) -> &'static str {
        self.internal.hqslang()
    }

    /// Return true when the operation has symbolic parameters.
    ///
    /// Returns:
    ///     bool: True if the operation contains symbolic parameters, False if it does not.
    fn is_parametrized(&self) -> bool {
        self.internal.is_parametrized()
    }

    /// Substitute the symbolic parameters in a clone of the PRAGMA operation according to the input.
    ///
    /// Args:
    ///     substitution_parameters (Dict[str, float]): The dictionary containing the substitutions to use in the PRAGMA operation.
    ///
    /// Returns:
    ///     self: The PRAGMA operation with the parameters substituted.
    ///
    /// Raises:
    ///     RuntimeError: The parameter substitution failed.
    fn substitute_parameters(
        &self,
        substitution_parameters: std::collections::HashMap<String, f64>,
    ) -> PyResult<Self> {
        let mut calculator = qoqo_calculator::Calculator::new();
        for (key, val) in substitution_parameters.iter() {
            calculator.set_variable(key, *val);
        }
        Ok(Self {
            internal: self
                .internal
                .substitute_parameters(&calculator)
                .map_err(|x| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Parameter Substitution failed: {x:?}"
                    ))
                })?,
        })
    }

    /// Remap qubits in a clone of the PRAGMA operation.
    ///
    /// Args:
    ///     mapping (Dict[int, int]): The dictionary containing the {qubit: qubit} mapping to use in the PRAGMA operation.
    ///
    /// Returns:
    ///     self: The PRAGMA operation with the qubits remapped.
    ///
    /// Raises:
    ///     RuntimeError: The qubit remapping failed.
    fn remap_qubits(&self, mapping: std::collections::HashMap<usize, usize>) -> PyResult<Self> {
        let new_internal = self
            .internal
            .remap_qubits(&mapping)
            .map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("Qubit remapping failed: "))?;
        Ok(Self {
            internal: new_internal,
        })
    }

    /// Return a copy of the PRAGMA operation (copy here produces a deepcopy).
    ///
    /// Returns:
    ///     PragmaChangeDevice: A deep copy of self.
    fn __copy__(&self) -> PragmaChangeDeviceWrapper {
        self.clone()
    }

    /// Return a deep copy of the PRAGMA operation.
    ///
    /// Returns:
    ///     PragmaChangeDevice: A deep copy of self.
    fn __deepcopy__(&self, _memodict: &Bound<PyAny>) -> PragmaChangeDeviceWrapper {
        self.clone()
    }

    /// Return a string containing a formatted (string) representation of the PRAGMA operation.
    ///
    /// Returns:
    ///     str: The string representation of the operation.
    fn __format__(&self, _format_spec: &str) -> PyResult<String> {
        Ok(format!("{:?}", self.internal))
    }

    /// Return a string containing a printable representation of the PRAGMA operation.
    ///
    /// Returns:
    ///     str: The printable string representation of the operation.
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.internal))
    }

    /// Return the __richcmp__ magic method to perform rich comparison operations on PragmaSetStateVector.
    ///
    /// Args:
    ///     self: The PragmaGeneralNoise object.
    ///     other: The object to compare self to.
    ///     op: Type of comparison.
    ///
    /// Returns:
    ///     bool: Whether the two operations compared evaluated to True or False.
    fn __richcmp__(
        &self,
        other: &Bound<PyAny>,
        op: pyo3::class::basic::CompareOp,
    ) -> PyResult<bool> {
        let other = crate::operations::convert_pyany_to_operation(other).map_err(|_| {
            pyo3::exceptions::PyTypeError::new_err(
                "Right hand side cannot be converted to Operation",
            )
        })?;
        match op {
            pyo3::class::basic::CompareOp::Eq => {
                Ok(Operation::from(self.internal.clone()) == other)
            }
            pyo3::class::basic::CompareOp::Ne => {
                Ok(Operation::from(self.internal.clone()) != other)
            }
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Other comparison not implemented.",
            )),
        }
    }

    #[cfg(feature = "json_schema")]
    /// Return the JsonSchema for the json serialisation of the class.
    ///
    /// Returns:
    ///     str: The json schema serialized to json
    #[staticmethod]
    pub fn json_schema() -> String {
        let schema = schemars::schema_for!(PragmaChangeDevice);
        serde_json::to_string_pretty(&schema).expect("Unexpected failure to serialize schema")
    }

    #[cfg(feature = "json_schema")]
    /// Returns the current version of the qoqo library .
    ///
    /// Returns:
    ///     str: The current version of the library.
    #[staticmethod]
    pub fn current_version() -> String {
        ROQOQO_VERSION.to_string()
    }

    #[cfg(feature = "json_schema")]
    /// Return the minimum version of qoqo that supports this object.
    ///
    /// Returns:
    ///     str: The minimum version of the qoqo library to deserialize this object.
    pub fn min_supported_version(&self) -> String {
        let min_version: (u32, u32, u32) =
            PragmaChangeDevice::minimum_supported_roqoqo_version(&self.internal);
        format!("{}.{}.{}", min_version.0, min_version.1, min_version.2)
    }
}

/// Module containing the PragmaAnnotatedOp class.
#[pymodule]
fn pragma_annotated_op(_py: Python, module: &Bound<PyModule>) -> PyResult<()> {
    module.add_class::<PragmaAnnotatedOpWrapper>()?;
    Ok(())
}

#[pyclass(name = "PragmaAnnotatedOp", module = "qoqo.operations")]
#[derive(Clone, Debug, PartialEq)]
/// An annotated Operation.
///
/// Args:
///     operation (Operation): - The Operation to be annotated.
///     annotation (str): - The annotation.
pub struct PragmaAnnotatedOpWrapper {
    /// PragmaAnnotatedOp to be wrapped and converted to Python.
    pub internal: PragmaAnnotatedOp,
}

insert_pyany_to_operation!(
    "PragmaAnnotatedOp" =>{
        let annot_op = &op.call_method0( "operation").map_err(|_|QoqoError::ConversionError)?;
        let operation: Operation = convert_pyany_to_operation(annot_op)
                                  .map_err(|_| QoqoError::ConversionError)?;
        let annot = op.call_method0( "annotation").map_err(|_|QoqoError::ConversionError)?;
        let annotation: String = annot.extract()
                                      .map_err(|_|QoqoError::ConversionError)?;
           Ok( PragmaAnnotatedOp{ operation: Box::new(operation), annotation }.into())
    }
);

insert_operation_to_pyobject!(
    Operation::PragmaAnnotatedOp(internal) => {
        {
            let pyref: Py<PragmaAnnotatedOpWrapper> =
                Py::new(py, PragmaAnnotatedOpWrapper { internal }).unwrap();
            pyref.into_pyobject(py).map(|bound| bound.as_any().to_owned()).map_err(|_| PyValueError::new_err("Unable to convert to Python object"))
        }
    }
);

#[pymethods]
impl PragmaAnnotatedOpWrapper {
    /// Create a PragmaAnnotatedOp instance.
    ///
    /// Args:
    ///     operation (Operation): - The Operation to be annotated.
    ///     annotation (str): - The annotation.
    #[new]
    fn new(operation: &Bound<PyAny>, annotation: String) -> PyResult<Self> {
        let op = crate::operations::convert_pyany_to_operation(operation).map_err(|_| {
            pyo3::exceptions::PyTypeError::new_err(
                "Input operation cannot be converted to Operation",
            )
        })?;
        Ok(Self {
            internal: PragmaAnnotatedOp::new(op, annotation),
        })
    }

    /// Return the internal Operation.
    ///
    /// Returns:
    ///     Operation: The annotated Operation.
    fn operation<'py>(&'py self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let op = self.internal.operation.clone();
        convert_operation_to_pyobject(*op, py)
    }

    /// Return the annotation.
    ///
    /// Returns:
    ///     str: The annotation.
    fn annotation(&self) -> String {
        self.internal.annotation.clone()
    }

    /// List all involved qubits.
    ///
    /// Returns:
    ///     Set[int]: The involved qubits of the PRAGMA operation.
    fn involved_qubits<'py>(&'py self, py: Python<'py>) -> Bound<'py, PySet> {
        let involved = self.internal.involved_qubits();
        match involved {
            InvolvedQubits::All => PySet::new(py, ["All"]).expect("Couldn not create PySet"),
            InvolvedQubits::None => PySet::empty(py).expect("Couldn not create PySet"),
            InvolvedQubits::Set(x) => {
                let mut vector: Vec<usize> = Vec::new();
                for qubit in x {
                    vector.push(qubit)
                }
                PySet::new(py, &vector[..]).expect("Couldn not create PySet")
            }
        }
    }

    /// Return tags classifying the type of the operation.
    ///
    /// Used for the type based dispatch in ffi interfaces.
    ///
    /// Returns:
    ///     List[str]: The tags of the Operation.
    fn tags(&self) -> Vec<String> {
        self.internal.tags().iter().map(|s| s.to_string()).collect()
    }

    /// Return hqslang name of the operation.
    ///
    /// Returns:
    ///     str: The hqslang name of the operation.
    fn hqslang(&self) -> &'static str {
        self.internal.hqslang()
    }

    /// Return true when the operation has symbolic parameters.
    ///
    /// Returns:
    ///     bool: True if the operation contains symbolic parameters, False if it does not.
    fn is_parametrized(&self) -> bool {
        self.internal.is_parametrized()
    }

    /// Substitute the symbolic parameters in a clone of the PRAGMA operation according to the input.
    ///
    /// Args:
    ///     substitution_parameters (Dict[str, float]): The dictionary containing the substitutions to use in the PRAGMA operation.
    ///
    /// Returns:
    ///     self: The PRAGMA operation with the parameters substituted.
    ///
    /// Raises:
    ///     RuntimeError: The parameter substitution failed.
    fn substitute_parameters(
        &self,
        substitution_parameters: std::collections::HashMap<String, f64>,
    ) -> PyResult<Self> {
        let mut calculator = qoqo_calculator::Calculator::new();
        for (key, val) in substitution_parameters.iter() {
            calculator.set_variable(key, *val);
        }
        Ok(Self {
            internal: self
                .internal
                .substitute_parameters(&calculator)
                .map_err(|x| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!(
                        "Parameter Substitution failed: {x:?}"
                    ))
                })?,
        })
    }

    /// Remap qubits in a clone of the PRAGMA operation.
    ///
    /// Args:
    ///     mapping (Dict[int, int]): The dictionary containing the {qubit: qubit} mapping to use in the PRAGMA operation.
    ///
    /// Returns:
    ///     self: The PRAGMA operation with the qubits remapped.
    ///
    /// Raises:
    ///     RuntimeError: The qubit remapping failed.
    fn remap_qubits(&self, mapping: std::collections::HashMap<usize, usize>) -> PyResult<Self> {
        let new_internal = self
            .internal
            .remap_qubits(&mapping)
            .map_err(|_| pyo3::exceptions::PyRuntimeError::new_err("Qubit remapping failed: "))?;
        Ok(Self {
            internal: new_internal,
        })
    }

    /// Return a copy of the PRAGMA operation (copy here produces a deepcopy).
    ///
    /// Returns:
    ///     PragmaAnnotatedOp: A deep copy of self.
    fn __copy__(&self) -> PragmaAnnotatedOpWrapper {
        self.clone()
    }

    /// Return a deep copy of the PRAGMA operation.
    ///
    /// Returns:
    ///     PragmaAnnotatedOp: A deep copy of self.
    fn __deepcopy__(&self, _memodict: &Bound<PyAny>) -> PragmaAnnotatedOpWrapper {
        self.clone()
    }

    /// Return a string containing a formatted (string) representation of the PRAGMA operation.
    ///
    /// Returns:
    ///     str: The string representation of the operation.
    fn __format__(&self, _format_spec: &str) -> PyResult<String> {
        Ok(format!("{:?}", self.internal))
    }

    /// Return a string containing a printable representation of the PRAGMA operation.
    ///
    /// Returns:
    ///     str: The printable string representation of the operation.
    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{:?}", self.internal))
    }

    /// Return the __richcmp__ magic method to perform rich comparison operations on PragmaAnnotatedOp.
    ///
    /// Args:
    ///     self: The PragmaGeneralNoise object.
    ///     other: The object to compare self to.
    ///     op: Type of comparison.
    ///
    /// Returns:
    ///     bool: Whether the two operations compared evaluated to True or False.
    fn __richcmp__(
        &self,
        other: &Bound<PyAny>,
        op: pyo3::class::basic::CompareOp,
    ) -> PyResult<bool> {
        let other = crate::operations::convert_pyany_to_operation(other).map_err(|_| {
            pyo3::exceptions::PyTypeError::new_err(
                "Right hand side cannot be converted to Operation",
            )
        })?;
        match op {
            pyo3::class::basic::CompareOp::Eq => {
                Ok(Operation::from(self.internal.clone()) == other)
            }
            pyo3::class::basic::CompareOp::Ne => {
                Ok(Operation::from(self.internal.clone()) != other)
            }
            _ => Err(pyo3::exceptions::PyNotImplementedError::new_err(
                "Other comparison not implemented.",
            )),
        }
    }

    #[cfg(feature = "json_schema")]
    /// Return the JsonSchema for the json serialisation of the class.
    ///
    /// Returns:
    ///     str: The json schema serialized to json
    #[staticmethod]
    pub fn json_schema() -> String {
        let schema = schemars::schema_for!(PragmaAnnotatedOp);
        serde_json::to_string_pretty(&schema).expect("Unexpected failure to serialize schema")
    }

    #[cfg(feature = "json_schema")]
    /// Returns the current version of the qoqo library .
    ///
    /// Returns:
    ///     str: The current version of the library.
    #[staticmethod]
    pub fn current_version() -> String {
        ROQOQO_VERSION.to_string()
    }

    #[cfg(feature = "json_schema")]
    /// Return the minimum version of qoqo that supports this object.
    ///
    /// Returns:
    ///     str: The minimum version of the qoqo library to deserialize this object.
    pub fn min_supported_version(&self) -> String {
        let min_version: (u32, u32, u32) =
            PragmaAnnotatedOp::minimum_supported_roqoqo_version(&self.internal);
        format!("{}.{}.{}", min_version.0, min_version.1, min_version.2)
    }
}

#[cfg(feature = "unstable_simulation_repetitions")]
/// Wrap function automatically generates functions in these traits.
#[wrap(Operate, OperatePragma, JsonSchema)]
#[derive(Eq)]
/// This PRAGMA sets the number of repetitions for stochastic simulations of the quantum circuit.
///
/// This is different from the number of measurements, which is set either with
/// PragmaSetNumberOfMeasurements of with PragmaRepeatedMeasurement. PragmaSimulationRepetitions
/// only applies to stochastic simulations, i.e. simulations of quantum circuits that involve either
/// multiple subsequent measurements on the same qubits, or operations on qubits that have already
/// been measured, and sets the number of times that the whole circuit is simulated in order to obtain
/// sufficient statistics.
///
/// Args:
///     repetitions (int): Number of simulation repetitions.
struct PragmaSimulationRepetitions {
    repetitions: usize,
}

#[cfg(test)]
mod tests {
    use crate::operations::*;
    use bincode::serialize;
    use roqoqo::operations::*;
    use std::collections::HashSet;

    /// Test involved_qubits function for Pragmas with All
    #[test]
    fn test_pyo3_involved_qubits_all_change_device() {
        let wrapped: Operation = PragmaActiveReset::new(0).into();
        let input_definition: Operation = PragmaChangeDevice::new(&wrapped).unwrap().into();

        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let operation = convert_operation_to_pyobject(input_definition, py).unwrap();
            let to_involved = operation.call_method0("involved_qubits").unwrap();
            let involved_op: HashSet<String> = HashSet::extract_bound(&to_involved).unwrap();
            let mut involved_param: HashSet<String> = HashSet::new();
            involved_param.insert("All".to_owned());
            assert_eq!(involved_op, involved_param);

            assert!(PragmaChangeDeviceWrapper::new().is_err());
        })
    }

    #[test]
    fn test_pyo3_format_repr_change_device() {
        let wrapped: Operation = PragmaActiveReset::new(0).into();
        let input_measurement: Operation = PragmaChangeDevice::new(&wrapped).unwrap().into();
        let format_repr = format!("PragmaChangeDevice {{ wrapped_tags: {:?}, wrapped_hqslang: {:?}, wrapped_operation: {:?} }}", wrapped.tags(), wrapped.hqslang(), serialize(&wrapped).unwrap());

        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let operation = convert_operation_to_pyobject(input_measurement, py).unwrap();
            let to_format = operation.call_method1("__format__", ("",)).unwrap();
            let format_op: String = String::extract_bound(&to_format).unwrap();
            let to_repr = operation.call_method0("__repr__").unwrap();
            let repr_op: String = String::extract_bound(&to_repr).unwrap();
            assert_eq!(format_op, format_repr);
            assert_eq!(repr_op, format_repr);
        })
    }

    #[test]
    fn test_pyo3_copy_deepcopy_change_device() {
        let wrapped: Operation = PragmaActiveReset::new(0).into();
        let input_measurement: Operation = PragmaChangeDevice::new(&wrapped).unwrap().into();

        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let operation = convert_operation_to_pyobject(input_measurement, py).unwrap();
            let copy_op = operation.call_method0("__copy__").unwrap();
            let deepcopy_op = operation.call_method1("__deepcopy__", ("",)).unwrap();
            let copy_deepcopy_param = operation;

            let comparison_copy = bool::extract_bound(
                &copy_op
                    .call_method1("__eq__", (copy_deepcopy_param.clone(),))
                    .unwrap(),
            )
            .unwrap();
            assert!(comparison_copy);
            let comparison_deepcopy = bool::extract_bound(
                &deepcopy_op
                    .call_method1("__eq__", (copy_deepcopy_param,))
                    .unwrap(),
            )
            .unwrap();
            assert!(comparison_deepcopy);
        })
    }

    #[test]
    fn test_pyo3_tags_simple_change_device() {
        let wrapped: Operation = PragmaActiveReset::new(0).into();
        let input_measurement: Operation = PragmaChangeDevice::new(&wrapped).unwrap().into();

        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let operation = convert_operation_to_pyobject(input_measurement, py).unwrap();
            let to_tag = operation.call_method0("tags").unwrap();
            let tags_op: &Vec<String> = &Vec::extract_bound(&to_tag).unwrap();
            let tags_param: &[&str] = &["Operation", "PragmaOperation", "PragmaChangeDevice"];
            assert_eq!(tags_op, tags_param);
        })
    }

    #[test]
    fn test_pyo3_hqslang_change_device() {
        let wrapped: Operation = PragmaActiveReset::new(0).into();
        let input_measurement: Operation = PragmaChangeDevice::new(&wrapped).unwrap().into();

        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let operation = convert_operation_to_pyobject(input_measurement, py).unwrap();
            let hqslang_op: String =
                String::extract_bound(&operation.call_method0("hqslang").unwrap()).unwrap();
            assert_eq!(hqslang_op, "PragmaChangeDevice".to_string());
        })
    }

    #[test]
    fn test_pyo3_is_parametrized_change_device() {
        let wrapped: Operation = PragmaActiveReset::new(0).into();
        let input_measurement: Operation = PragmaChangeDevice::new(&wrapped).unwrap().into();

        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let operation = convert_operation_to_pyobject(input_measurement, py).unwrap();
            assert!(
                !bool::extract_bound(&operation.call_method0("is_parametrized").unwrap()).unwrap()
            );
        })
    }

    #[test]
    fn test_pyo3_substitute_parameters() {
        let wrapped: Operation = PragmaActiveReset::new(0).into();
        let first_op: Operation = PragmaChangeDevice::new(&wrapped).unwrap().into();
        let second_op: Operation = PragmaChangeDevice::new(&wrapped).unwrap().into();

        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let operation = convert_operation_to_pyobject(first_op, py).unwrap();
            let mut substitution_dict: HashMap<String, f64> = HashMap::new();
            substitution_dict.insert("test".to_owned(), 1.0);
            let substitute_op = operation
                .call_method1("substitute_parameters", (substitution_dict,))
                .unwrap();
            let substitute_param = convert_operation_to_pyobject(second_op, py).unwrap();

            let comparison = bool::extract_bound(
                &substitute_op
                    .call_method1("__eq__", (substitute_param,))
                    .unwrap(),
            )
            .unwrap();
            assert!(comparison);
        })
    }

    #[test]
    fn test_pyo3_remap_qubits() {
        let wrapped: Operation = PragmaActiveReset::new(0).into();
        let first_op: Operation = PragmaChangeDevice::new(&wrapped).unwrap().into();
        let second_op: Operation = PragmaChangeDevice::new(&wrapped).unwrap().into();

        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let operation = convert_operation_to_pyobject(first_op, py).unwrap();

            let mut qubit_mapping: HashMap<usize, usize> = HashMap::new();
            qubit_mapping.insert(0, 0);
            let remapped_op = operation
                .call_method1("remap_qubits", (qubit_mapping,))
                .unwrap();
            let comparison_op = convert_operation_to_pyobject(second_op, py).unwrap();

            let comparison = bool::extract_bound(
                &remapped_op
                    .call_method1("__eq__", (comparison_op,))
                    .unwrap(),
            )
            .unwrap();
            assert!(comparison);
        })
    }

    #[test]
    fn test_pyo3_remap_qubits_error() {
        let wrapped: Operation = PragmaActiveReset::new(0).into();
        let first_op: Operation = PragmaChangeDevice::new(&wrapped).unwrap().into();

        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let operation = convert_operation_to_pyobject(first_op, py).unwrap();

            let mut qubit_mapping: HashMap<usize, usize> = HashMap::new();
            qubit_mapping.insert(0, 2);
            let remapped_op = operation.call_method1("remap_qubits", (qubit_mapping,));
            assert!(remapped_op.is_err());
        })
    }

    #[test]
    fn test_pyo3_richcmp_change_device() {
        let wrapped_1: Operation = PragmaActiveReset::new(0).into();
        let definition_1: Operation = PragmaChangeDevice::new(&wrapped_1).unwrap().into();
        let wrapped_2: Operation = PragmaActiveReset::new(1).into();
        let definition_2: Operation = PragmaChangeDevice::new(&wrapped_2).unwrap().into();

        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let operation_one = convert_operation_to_pyobject(definition_1, py).unwrap();
            let operation_two = convert_operation_to_pyobject(definition_2, py).unwrap();

            let comparison = bool::extract_bound(
                &operation_one
                    .call_method1("__eq__", (operation_two.clone(),))
                    .unwrap(),
            )
            .unwrap();
            assert!(!comparison);

            let comparison = bool::extract_bound(
                &operation_one
                    .call_method1("__ne__", (operation_two.clone(),))
                    .unwrap(),
            )
            .unwrap();
            assert!(comparison);

            let comparison = operation_one.call_method1("__eq__", (vec!["fails"],));
            assert!(comparison.is_err());

            let comparison = operation_one.call_method1("__ge__", (operation_two,));
            assert!(comparison.is_err());
        })
    }
}
