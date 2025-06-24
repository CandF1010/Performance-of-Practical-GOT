
# README

This repository contains code related to performance testing and benchmarking of cryptographic operations within a specific context.

## Project Structure and Key Components

*   **Scalar Multiplication Test Code (BLS12-377)**
    The code for testing the scalar multiplication on the BLS12-377 elliptic curve is located in the [`ecc_exp_test`](https://github.com/CandF1010/Performance-of-Practical-GOT/tree/main/ecc_exp_test) directory. This section focuses on evaluating the efficiency and correctness of scalar multiplication operations for the BLS12-377 curve.

*   **legogro16 Benchmark Code**
    Benchmarking code for the `legogro16` system can be found at [`legogro16/tests/test_large_circuit2.rs`](https://github.com/CandF1010/Performance-of-Practical-GOT/blob/main/legogro16/tests/test_large_circuit2.rs). This benchmark specifically focuses on the performance of large arithmetic circuits.
    *   **Access Structure Input Length:**
        Within `test_large_circuit2.rs`, the length of the input for the access structure function `F` has been set to **1000**. This value represents the number of elements that need to be committed in the Pedersen vector commitment.
    *   **Arithmetic Circuit Multiplier Count:**
        The number of multiplications in the arithmetic circuit is tested across a range from **2^10 to 2^20**. This range allows for comprehensive evaluation of legogro16's performance scaling with increasing circuit complexity.
