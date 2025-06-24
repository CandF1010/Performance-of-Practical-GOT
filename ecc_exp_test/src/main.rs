use ark_bls12_377::{Fr, G1Projective};
use ark_ec::{CurveGroup, PrimeGroup}; // Provides traits for scalar multiplication, PrimeGroup provides generator()
use ark_ff::UniformRand;
use rand::thread_rng; // <-- Modified here, import directly from rand
use std::time::Instant;

fn main() {
    let mut rng = thread_rng(); // Initialize a thread-local random number generator

    println!("Starting benchmark for BN128 curve scalar multiplication performance...");
    println!("{:<10} | {:<15}", "Count", "Total Time (ms)");
    println!("----------------------------------");

    for count in (1000..=10000).step_by(1000) {
        let base_point = G1Projective::generator();

        let mut scalars = Vec::with_capacity(count);
        for _ in 0..count {
            scalars.push(Fr::rand(&mut rng));
        }

        let start_time = Instant::now();

        for i in 0..count {
            let _result = base_point * scalars[i];
        }

        let elapsed_time = start_time.elapsed();
        println!("{:<10} | {:<15.2}", count, elapsed_time.as_secs_f64() * 1000.0);
    }

    println!("\nBenchmark complete.");
}
