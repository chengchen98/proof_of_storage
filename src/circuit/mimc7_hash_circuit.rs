// Bring in some tools for using pairing-friendly curves
// We're going to use the BLS12-381 pairing-friendly elliptic curve.
// For randomness (during paramgen and proof generation)
use ark_ff::Field;

// We'll use these interfaces to construct our circuit.
use ark_relations::{
    lc,
    r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError, Variable},
};

const MIMC7_HASH_ROUNDS: usize = 91;

/// This is our demo circuit for proving knowledge of the
/// preimage of a MiMC hash invocation.
pub struct MiMC7HashDemo<'a, F: Field> {
    x_inputs: &'a [Option<F>],
    key: Option<F>,
    constants: &'a [F],
}

/// Our demo circuit implements this `Circuit` trait which
/// is used during paramgen and proving in order to
/// synthesize the constraint system.
impl<'a, F: Field> ConstraintSynthesizer<F> for MiMC7HashDemo<'a, F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        assert_eq!(self.constants.len(), MIMC7_HASH_ROUNDS);

        let mut res_val  = self.key;
        let mut res = cs.new_witness_variable(|| res_val.ok_or(SynthesisError::AssignmentMissing))?;

        // Start to compute mimc hash.
        for i in 0..self.x_inputs.len() {
            let x_in_val= self.x_inputs[i];
            let x_in = cs.new_witness_variable(|| x_in_val.ok_or(SynthesisError::AssignmentMissing))?;

            let key_val = res_val;
            let key = cs.new_witness_variable(|| key_val.ok_or(SynthesisError::AssignmentMissing))?;    
            
            let mut h_val = Some(F::zero());
            let mut h = cs.new_witness_variable(|| h_val.ok_or(SynthesisError::AssignmentMissing))?;

            // Create every single mimc hash.
            for j in 0..MIMC7_HASH_ROUNDS {
                let t_val;
                let t;
                if j == 0 {
                    // t = x[i] + key
                    t_val = x_in_val.map(|mut e| {
                        e.add_assign(&key_val.unwrap());
                        e
                    });
                    t = cs.new_witness_variable(|| t_val.ok_or(SynthesisError::AssignmentMissing))?;
                    cs.enforce_constraint(
                        lc!() + x_in + key,
                        lc!() + Variable::One,
                        lc!() + t
                    )?;
                }
                else {
                    // t = h + key + constants[j]
                    t_val = h_val.map(|mut e| {
                        e.add_assign(&key_val.unwrap());
                        e.add_assign(&self.constants[j]);
                        e
                    });
                    t = cs.new_witness_variable(|| t_val.ok_or(SynthesisError::AssignmentMissing))?;
                    cs.enforce_constraint(
                        lc!() + h + key + (self.constants[j], Variable::One),
                        lc!() + Variable::One,
                        lc!() + t
                    )?;
                }

                // t2 = t * t
                let t2_val = t_val.map(|mut e| {
                    e.square_in_place();
                    e
                });
                let t2 = cs.new_witness_variable(|| t2_val.ok_or(SynthesisError::AssignmentMissing))?;
                cs.enforce_constraint(
                    lc!() + t,
                    lc!() + t,
                    lc!() + t2
                )?;
                
                // t4 = t2 * t2
                let t4_val = t2_val.map(|mut e| {
                    e.square_in_place();
                    e
                });
                let t4 = cs.new_witness_variable(|| t4_val.ok_or(SynthesisError::AssignmentMissing))?;
                cs.enforce_constraint(
                    lc!() + t2,
                    lc!() + t2,
                    lc!() + t4
                )?;
                
                // t6 = t4 * t2
                let t6_val = t4_val.map(|mut e| {
                    e.mul_assign(&t2_val.unwrap());
                    e
                });
                let t6 = cs.new_witness_variable(|| t6_val.ok_or(SynthesisError::AssignmentMissing))?;
                cs.enforce_constraint(
                    lc!() + t4,
                    lc!() + t2,
                    lc!() + t6
                )?;
                
                // t7 = t6 * t
                let t7_val = t6_val.map(|mut e| {
                    e.mul_assign(&t_val.unwrap());
                    e
                });
                let t7 = cs.new_witness_variable(|| t7_val.ok_or(SynthesisError::AssignmentMissing))?;
                cs.enforce_constraint(
                    lc!() + t6,
                    lc!() + t,
                    lc!() + t7
                )?;

                h = t7;
                h_val = t7_val;
            }

            // new_h = h + key
            let new_h_val = h_val.map(|mut e| {
                e.add_assign(&key_val.unwrap());
                e
            });
            let new_h = cs.new_witness_variable(|| new_h_val.ok_or(SynthesisError::AssignmentMissing))?;
            cs.enforce_constraint(
                lc!() + h + key,
                lc!() + Variable::One,
                lc!() + new_h
            )?;

            // new_res = res + x[i]
            let new_res_val = res_val.map(|mut e| {
                e.add_assign(&x_in_val.unwrap());
                e.add_assign(&new_h_val.unwrap());
                e
            });
            let new_res = if i == (self.x_inputs.len() - 1) {
                cs.new_input_variable(|| new_res_val.ok_or(SynthesisError::AssignmentMissing))?
            }
            else {
                cs.new_witness_variable(|| new_res_val.ok_or(SynthesisError::AssignmentMissing))?
            };
            cs.enforce_constraint(
                lc!() + res + x_in + new_h,
                lc!() + Variable::One,
                lc!() + new_res
            )?;

            res = new_res;
            res_val = new_res_val;
        }

        Ok(())
    }
}

#[test]
fn test_mimc7_hash() {
    // For benchmarking
    use std::time::{Duration, Instant};
    use ark_std::rand::Rng;
    use ark_std::test_rng;
    use ark_bls12_381::{Fr, Bls12_381};
    use crate::common::mimc_hash::multi_mimc7_hash;

    // We're going to use the Groth proving system.
    use ark_groth16::{
        create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
    };

    // This may not be cryptographically safe, use
    // `OsRng` (for example) in production software.
    let rng = &mut test_rng();

    let x_inputs = (0..3).map(|_| rng.gen()).collect::<Vec<_>>();
    let new_x_inputs = x_inputs.clone().into_iter().map(|x| Some(x)).collect::<Vec<_>>();
    let key = rng.gen();
    // Generate the MiMC round constants
    let constants = (0..MIMC7_HASH_ROUNDS).map(|_| rng.gen()).collect::<Vec<_>>();

    // Generate a random preimage and compute the image
    let x_hash = multi_mimc7_hash(&x_inputs, key, &constants);

    println!("Creating parameters...");

    // Create parameters for our circuit
    let params = {
        let c = MiMC7HashDemo::<Fr> {
            x_inputs: &new_x_inputs,
            key: Some(key),
            constants: &constants,
        };

        generate_random_parameters::<Bls12_381, _, _>(c, rng).unwrap()
    };

    // Prepare the verification key (for proof verification)
    let pvk = prepare_verifying_key(&params.vk);

    println!("Creating proofs...");

    // Let's benchmark stuff!
    const SAMPLES: u32 = 50;
    let mut total_proving = Duration::new(0, 0);
    let mut total_verifying = Duration::new(0, 0);

    // Just a place to put the proof data, so we can
    // benchmark deserialization.
    // let mut proof_vec = vec![];

    for i in 0..SAMPLES {
        println!("Sample: {:?}", i);

        // proof_vec.truncate(0);

        let start = Instant::now();
        {
            // Create an instance of our circuit (with the
            // witness)
            let c = MiMC7HashDemo {
                x_inputs: &new_x_inputs,
                key: Some(key),
                constants: &constants,
            };

            // Create a groth16 proof with our parameters.
            let proof = create_random_proof(c, &params, rng).unwrap();
            assert!(verify_proof(&pvk, &proof, &[x_hash]).unwrap());

            // proof.write(&mut proof_vec).unwrap();
        }

        total_proving += start.elapsed();

        let start = Instant::now();
        // let proof = Proof::read(&proof_vec[..]).unwrap();
        // Check the proof

        total_verifying += start.elapsed();
    }
    let proving_avg = total_proving / SAMPLES;
    let proving_avg =
        proving_avg.subsec_nanos() as f64 / 1_000_000_000f64 + (proving_avg.as_secs() as f64);

    let verifying_avg = total_verifying / SAMPLES;
    let verifying_avg =
        verifying_avg.subsec_nanos() as f64 / 1_000_000_000f64 + (verifying_avg.as_secs() as f64);

    println!("Average proving time: {:?} seconds", proving_avg);
    println!("Average verifying time: {:?} seconds", verifying_avg);
}