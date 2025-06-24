// 导入所需的库和模块
use legogro16::{
    create_random_proof, generate_random_parameters, prepare_verifying_key, verify_commitment,
    verify_proof,
};

use ark_ec::{AffineCurve, PairingEngine, ProjectiveCurve};
use ark_ff::{Field, PrimeField, UniformRand, Zero};
use ark_relations::{
    lc,
    r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError},
};
use ark_std::{end_timer, rand::{rngs::StdRng, SeedableRng}};
use std::{ops::MulAssign, time::Instant};

/// A simple circuit that computes `a * b = c`.
///
/// This struct has been parameterized with `num_constraints` and `num_public_inputs`
/// to allow for dynamically adjusting the circuit's size and the number of public inputs
/// during testing.
struct MySillyCircuit<F: Field> {
    a: Option<F>,
    b: Option<F>,
    num_constraints: usize,   // Controls the total number of constraints in the circuit.
    num_public_inputs: usize, // Controls the number of public input variables.
}

// Implementation of the ConstraintSynthesizer trait for our circuit.
impl<F: Field> ConstraintSynthesizer<F> for MySillyCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        // 1. Declare witness variables `a` and `b`.
        // The `||` creates a closure that provides the value. `?` propagates any error.
        let a = cs.new_witness_variable(|| self.a.ok_or(SynthesisError::AssignmentMissing))?;
        let b = cs.new_witness_variable(|| self.b.ok_or(SynthesisError::AssignmentMissing))?;

        // 2. Calculate `c` from `a` and `b` to assign to the public input variable.
        let c_val = match (self.a, self.b) {
            (Some(val_a), Some(val_b)) => {
                let mut res = val_a;
                res.mul_assign(&val_b);
                res
            }
            // During parameter generation, no assignments are provided, so we use zero as a placeholder.
            _ => F::zero(),
        };

        // 3. Create the specified number of public input variables.
        let mut public_input_vars = Vec::with_capacity(self.num_public_inputs);
        
        // The first public input is the actual result `c = a * b`.
        let c_0 = cs.new_input_variable(|| Ok(c_val))?;
        public_input_vars.push(c_0);

        // Fill the remaining public inputs with zero to meet `num_public_inputs`.
        // These are extra public inputs that can be committed to by Pedersen commitments
        // but are not constrained in this circuit.
        for _ in 1..self.num_public_inputs {
            let dummy_input = cs.new_input_variable(|| Ok(F::zero()))?;
            public_input_vars.push(dummy_input);
        }

        // 4. The core constraint: `a * b = c_0`. This adds 1 constraint.
        cs.enforce_constraint(lc!() + a, lc!() + b, lc!() + public_input_vars[0])?;

        // 5. Add dummy constraints to reach the total `num_constraints`.
        // We already have one core constraint, so we need `num_constraints - 1` more.
        if self.num_constraints > 0 {
            let zero_var = cs.new_witness_variable(|| Ok(F::zero()))?;
            for _ in 0..self.num_constraints.saturating_sub(1) {
                // Add a simple dummy constraint, e.g., `a * 0 = 0`.
                cs.enforce_constraint(lc!() + a, lc!() + zero_var, lc!() + zero_var)?;
            }
        }

        Ok(())
    }
}

/// A generic function to test the proving and verification process.
///
/// It accepts `num_constraints` and `num_pedersen_outputs` to allow testing
/// with different circuit sizes and numbers of Pedersen commitment outputs.
fn test_prove_and_verify<E: PairingEngine>(
    n_iters: usize,
    num_constraints: usize,
    num_pedersen_outputs: usize,
) where
    // This bound is required by `create_random_proof`
    E::Fr: PrimeField,
{
    let mut rng = StdRng::seed_from_u64(0u64);

    // Generate the bases for the Pedersen commitments, matching the number of outputs.
    let pedersen_bases = (0..(num_pedersen_outputs+2))
        .map(|_| E::G1Projective::rand(&mut rng).into_affine())
        .collect::<Vec<_>>();

    // Generate the Groth16 parameters. The number of public inputs for the circuit
    // must match the number of Pedersen bases.
    let params = generate_random_parameters::<E, _, _>(
        MySillyCircuit {
            a: None,
            b: None,
            num_constraints,
            num_public_inputs: num_pedersen_outputs, // Public inputs must match pedersen outputs.
        },
        &pedersen_bases,
        &mut rng,
    )
    .unwrap();

    // Prepare the verifying key.
    let pvk = prepare_verifying_key::<E>(&params.vk);

    for _ in 0..n_iters {
        // Generate random witness values.
        let a = E::Fr::rand(&mut rng);
        let b = E::Fr::rand(&mut rng);
        let mut c = a;
        c.mul_assign(&b);

        // Prepare the public output values for the Pedersen commitment.
        // The first value is the actual result `c`, and the rest are zero.
        let mut committed_values = vec![E::Fr::zero(); num_pedersen_outputs];
        if !committed_values.is_empty() {
            committed_values[0] = c;
        }

        // Generate randomness for the commitments.
        let v = E::Fr::rand(&mut rng);
        let link_v = E::Fr::rand(&mut rng);

        println!("Creating proofs...");

        let start_time = Instant::now();
        // Create the LegoGro16 proof.
        let proof = create_random_proof(
            MySillyCircuit {
                a: Some(a),
                b: Some(b),
                num_constraints,
                num_public_inputs: num_pedersen_outputs,
            },
            v,
            link_v,
            &params,
            &mut rng,
        )
        .unwrap();

        let elapsed = start_time.elapsed();
        println!("Proof created in: {:?}", elapsed);
        // Verify the Groth16 proof.
        assert!(verify_proof(&pvk, &proof).unwrap());
        
        // println!("public_inputs lenth: {}", committed_values.len());
        // println!("gamma_abc_g1 length: {}", pvk.vk.gamma_abc_g1.len());
        // println!("link_bases length: {}", pvk.vk.link_bases.len());

        // Verify the Pedersen commitment with the correct values and randomness.
        assert!(verify_commitment(&pvk, &proof, &committed_values, &v, &link_v).unwrap());

        // Test failure case: Use a wrong committed value.
        if !committed_values.is_empty() {
            let mut wrong_committed_values = committed_values.clone();
            wrong_committed_values[0] = a; // Use `a` instead of `c`.
            assert!(!verify_commitment(&pvk, &proof, &wrong_committed_values, &v, &link_v).unwrap());
        }

        // Test failure case: Use wrong randomness `v`.
        let wrong_v = E::Fr::rand(&mut rng);
        assert!(!verify_commitment(&pvk, &proof, &committed_values, &wrong_v, &link_v).unwrap());

        // Test failure case: Use wrong randomness `link_v`.
        let wrong_link_v = E::Fr::rand(&mut rng);
        assert!(!verify_commitment(&pvk, &proof, &committed_values, &v, &wrong_link_v).unwrap());
    }
}

mod bls12_377 {
    use super::*;
    use ark_bls12_377::Bls12_377;

    #[test]
    fn prove_and_verify() {
        const NUM_PEDERSEN_OUTPUTS: usize = 1000;

        // Iterate through desired circuit sizes: 2^10, 2^12, 2^14
        for i in (10..=20).step_by(1) {
            let num_constraints = 1 << i; // Calculate number of constraints (2^i)
            println!(
                "Testing Bls12_377 with num_constraints = {} (2^{}), num_pedersen_outputs = {}",
                num_constraints, i, NUM_PEDERSEN_OUTPUTS
            );
            // Run 1 iteration per configuration to keep test time reasonable.
            test_prove_and_verify::<Bls12_377>(1, num_constraints, NUM_PEDERSEN_OUTPUTS);
        }
    }
}

mod cp6_782 {
    use super::*;
    use ark_cp6_782::CP6_782;

    #[test]
    fn prove_and_verify() {
        const NUM_PEDERSEN_OUTPUTS: usize = 1000;

        // Iterate through desired circuit sizes: 2^10, 2^12, 2^14
        for i in (10..=14).step_by(2) {
            let num_constraints = 1 << i; // Calculate number of constraints (2^i)
            println!(
                "Testing CP6_782 with num_constraints = {} (2^{}), num_pedersen_outputs = {}",
                num_constraints, i, NUM_PEDERSEN_OUTPUTS
            );
            // Run 1 iteration per configuration to keep test time reasonable.
            test_prove_and_verify::<CP6_782>(1, num_constraints, NUM_PEDERSEN_OUTPUTS);
        }
    }
}