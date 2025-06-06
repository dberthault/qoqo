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

//! Prelude to bring the most common roqoqo traits into scope.
//!
//! # Example
//!
//! ```
//! use roqoqo::prelude::*;
//!```
//!

pub use crate::backends::EvaluatingBackend;
pub use crate::circuit::AsVec;
pub use crate::measurements::{Measure, MeasureExpectationValues};
#[cfg(feature = "unstable_analog_operations")]
pub use crate::operations::OperateSpinsAnalog;
pub use crate::operations::{
    Define, InvolveModes, InvolveQubits, InvolvedModes, InvolvedQubits, Operate,
    OperateConstantGate, OperateFourQubit, OperateFourQubitGate, OperateGate, OperateModeGate,
    OperateMultiQubit, OperateMultiQubitGate, OperatePragma, OperatePragmaNoise,
    OperatePragmaNoiseProba, OperateSingleMode, OperateSingleModeGate, OperateSingleQubit,
    OperateSingleQubitGate, OperateThreeQubit, OperateThreeQubitGate, OperateTwoMode,
    OperateTwoModeGate, OperateTwoQubit, OperateTwoQubitGate, Rotate, Substitute, SubstituteModes,
    SupportedVersion, AVAILABLE_GATES_HQSLANG,
};
pub use crate::{RoqoqoBackendError, RoqoqoError};
