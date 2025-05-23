# List of Gate Operations

Operations are the atomic instructions in any quantum program that can be represented by qoqo/roqoqo. Gate operations are single-, two- three-, four- or multi-qubit unitary operations that apply a unitary transformation and can be executed on any universal quantum computer. Mathematically, a gate can be represented by a unitary matrix.

A list of the gate operations available in qoqo and roqoqo with their mathematical description is provided in this section. We differentiate between [single-qubit gates](single_qubit_gates.md), [two-qubit gates](two_qubit_gates.md), [three-qubit gates](three_qubit_gates.md), [four-qubit gates](four_qubit_gates.md) and [multi-qubit gates](multi_qubit_gates.md), depending on the number of qubits involved in the operation.

### Notation

* A rotation angle is usually annotated with \\( \theta \\) and its corresponding argument is `theta`.
* For the phase angle, the symbol \\( \varphi \\) is used.
* The rotation angle  \\( \phi \\)  in the x-y plane is addressed by the argument name `phi`.
* \\( \sigma_x \\), \\( \sigma_y \\), \\( \sigma_z \\) are the Pauli matrices X, Y, Z
\\[
    \sigma_x = \begin{pmatrix} 0 & 1 \\\\ 1 & 0 \end{pmatrix} := X, \quad \sigma_y = \begin{pmatrix} 0 & -i \\\\ i & 0 \end{pmatrix} := Y,  \quad \sigma_z = \begin{pmatrix} 1 & 0 \\\\ 0 & -1 \end{pmatrix} := Z
\\].

## [Single-qubit gates](single_qubit_gates.md)

| Gate | Short Description |
|---------|---------|
<!-- cmdrun cargo single run ../../build_gates_table.rs -- single_qubit_gate_operations.rs -->

## [Two-qubit gates](two_qubit_gates.md)

| Gate | Short Description |
|---------|---------|
<!-- cmdrun cargo single run ../../build_gates_table.rs -- two_qubit_gate_operations.rs -->

## [Three-qubit gates](three_qubit_gates.md)

| Gate | Short Description |
|---------|---------|
<!-- cmdrun cargo single run ../../build_gates_table.rs -- three_qubit_gate_operations.rs -->

## [Four-qubit gates](four_qubit_gates.md)

| Gate | Short Description |
|---------|---------|
<!-- cmdrun cargo single run ../../build_gates_table.rs -- four_qubit_gate_operations.rs -->

## [Multi-qubit gates](multi_qubit_gates.md)

| Gate | Short Description |
|---------|---------|
<!-- cmdrun cargo single run ../../build_gates_table.rs -- multi_qubit_gate_operations.rs -->

## [Bosonic operations](bosonic_operations.md)

| Gate | Short Description |
|---------|---------|
<!-- cmdrun cargo single run ../../build_gates_table.rs -- bosonic_operations.rs -->

## [Spin-Boson operations](soin_boson_operations.md)

| Gate | Short Description |
|---------|---------|
<!-- cmdrun cargo single run ../../build_gates_table.rs -- spin_boson_operations.rs -->