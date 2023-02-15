// Bring in some tools for using pairing-friendly curves
// We're going to use the BLS12-381 pairing-friendly elliptic curve.
// For randomness (during paramgen and proof generation)
use ark_ff::Field;

// We'll use these interfaces to construct our circuit.
use ark_relations::{
    lc,
    r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError, Variable},
};

const MIMC5_DF_ROUNDS: usize = 322;

/// This is our demo circuit for proving knowledge of the
/// preimage of a MiMC hash invocation.
pub struct MiMC5DFDemo<'a, F: Field> {
    xl: Option<F>,
    xr: Option<F>,
    constants: &'a [F],
}

/// Our demo circuit implements this `Circuit` trait which
/// is used during paramgen and proving in order to
/// synthesize the constraint system.
impl<'a, F: Field> ConstraintSynthesizer<F> for MiMC5DFDemo<'a, F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        assert_eq!(self.constants.len(), MIMC5_DF_ROUNDS);

        // Allocate the first component of the preimage.
        let mut xl_value = self.xl;
        let mut xl = cs.new_witness_variable(|| xl_value.ok_or(SynthesisError::AssignmentMissing))?;

        // Allocate the second component of the preimage.
        let mut xr_value = self.xr;
        let mut xr = cs.new_witness_variable(|| xr_value.ok_or(SynthesisError::AssignmentMissing))?;

        for i in 0..MIMC5_DF_ROUNDS {
            // xL, xR := xR + (xL + Ci)^3, xL

            // tmp1 = (xL + Ci) ^ 2
            let tmp1_value = xl_value.map(|mut e| {
                e.add_assign(&self.constants[i]);
                e.square_in_place();
                e
            });
            let tmp1 = cs.new_witness_variable(|| tmp1_value.ok_or(SynthesisError::AssignmentMissing))?;

            cs.enforce_constraint(
                lc!() + xl + (self.constants[i], Variable::One),
                lc!() + xl + (self.constants[i], Variable::One),
                lc!() + tmp1,
            )?;

            // tmp2 = tmp1 ^ 2 = (xL + Ci) ^ 4;
            let tmp2_value = tmp1_value.map(|mut e| {
                e.square_in_place();
                e
            });
            let tmp2 = cs.new_witness_variable(|| tmp2_value.ok_or(SynthesisError::AssignmentMissing))?;

            cs.enforce_constraint(
                lc!() + tmp1,
                lc!() + tmp1,
                lc!() + tmp2,
            )?;

            // new_xL = xR + (xL + Ci)^5
            // new_xL = xR + tmp2 * (xL + Ci)
            // new_xL - xR = tmp2 * (xL + Ci)
            let new_xl_value = xl_value.map(|mut e| {
                e.add_assign(&self.constants[i]);
                e.mul_assign(&tmp2_value.unwrap());
                e.add_assign(&xr_value.unwrap());
                e
            });

            let new_xl = if i == (MIMC5_DF_ROUNDS - 1) {
                // This is the last round, xL is our image and so
                // we allocate a public input.
                cs.new_input_variable(|| new_xl_value.ok_or(SynthesisError::AssignmentMissing))?
            } else {
                cs.new_witness_variable(|| new_xl_value.ok_or(SynthesisError::AssignmentMissing))?
            };

            cs.enforce_constraint(
                lc!() + tmp2,
                lc!() + xl + (self.constants[i], Variable::One),
                lc!() + new_xl - xr,
            )?;

            // xR = xL
            xr = xl;
            xr_value = xl_value;

            // xL = new_xL
            xl = new_xl;
            xl_value = new_xl_value;
        }

        Ok(())
    }
}

#[test]
fn test_mimc5_df() {
    // For benchmarking
    use std::time::{Duration, Instant};
    use ark_std::rand::Rng;
    use ark_std::test_rng;
    use ark_bls12_381::{Fr, Bls12_381};
    use crate::common::mimc_df::mimc5_df;

    // We're going to use the Groth proving system.
    use ark_groth16::{
        create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
    };

    // This may not be cryptographically safe, use
    // `OsRng` (for example) in production software.
    let rng = &mut test_rng();

    // Generate the MiMC round constants
    let constants = (0..MIMC5_DF_ROUNDS).map(|_| rng.gen()).collect::<Vec<_>>();

    println!("Creating parameters...");

    // Create parameters for our circuit
    let params = {
        let c = MiMC5DFDemo::<Fr> {
            xl: None,
            xr: None,
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

        // Generate a random preimage and compute the image
        let xl = rng.gen();
        let xr = rng.gen();
        let image = mimc5_df(xl, xr, &constants);

        // proof_vec.truncate(0);

        let start = Instant::now();
        {
            // Create an instance of our circuit (with the
            // witness)
            let c = MiMC5DFDemo {
                xl: Some(xl),
                xr: Some(xr),
                constants: &constants,
            };

            // Create a groth16 proof with our parameters.
            let proof = create_random_proof(c, &params, rng).unwrap();
            assert!(verify_proof(&pvk, &proof, &[image]).unwrap());

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