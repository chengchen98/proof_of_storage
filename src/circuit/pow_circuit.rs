use ark_ff::Field;

// We'll use these interfaces to construct our circuit.
use ark_relations::{
    lc, ns,
    r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError, Variable},
};

pub const INPUT_SIZE: usize = 20;

/// Prove the process of computing g\^x
/// 
/// x is bit
pub struct PowDemo<'a, F: Field> {
    pub g: Option<F>,
    pub x_bits: &'a [Option<u8>]
}

impl<'a, F: Field> ConstraintSynthesizer<F> for PowDemo<'a, F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        assert_eq!(self.x_bits.len(), INPUT_SIZE);

        let ns = ns!(cs, "round");
        let cs = ns.cs();

        let mut y_val = Some(F::one());
        let mut g_val = self.g;

        let mut g = cs.new_witness_variable(|| g_val.ok_or(SynthesisError::AssignmentMissing))?;

        let mut y = cs.new_witness_variable(|| y_val.ok_or(SynthesisError::AssignmentMissing))?;

        for i in 0..INPUT_SIZE {
            let bit = self.x_bits[i];
            let mut bit_val = None;
            if bit == Some(1) {
                bit_val = Some(F::one());
            }
            else if bit == Some(0) {
                bit_val = Some(F::zero());
            }

            // bit = xi
            let bit = cs.new_witness_variable(|| bit_val.ok_or(SynthesisError::AssignmentMissing))?;

            // xi = 0 or 1
            cs.enforce_constraint(
                lc!() + bit,
                lc!() + bit,
                lc!() + bit
            )?;

            // 1 - (1 - g^(2^i)) * xi = y
            // (1 - g^(2^i)) * xi = 1 - y
    
            // tmp1 = g^(2^i) * xi
            let tmp1_val = g_val.map(|mut e| {
                e.mul_assign(&bit_val.unwrap());
                e
            });

            let tmp1 = cs.new_witness_variable(|| tmp1_val.ok_or(SynthesisError::AssignmentMissing))?;

            cs.enforce_constraint(
                lc!() + g,
                lc!() + bit,
                lc!() + tmp1
            )?;

            // xi - tmp1 = 1 - y
            // y(tmp2) = tmp1 + 1 - xi
            // tmp2 + xi = tmp1 + 1
            let tmp2_val = tmp1_val.map(|mut e| {
                e.add_assign(&F::one());
                e.sub_assign(&bit_val.unwrap());
                e
            });

            let tmp2 = cs.new_witness_variable(|| tmp2_val.ok_or(SynthesisError::AssignmentMissing))?;
             
            cs.enforce_constraint(
                lc!() + tmp1 + (F::one(), Variable::One),
                lc!() + Variable::One,
                lc!() + tmp2 + bit
            )?;
            
            // newy = tmp2 * y
            let newy_val = tmp2_val.map(|mut e| {
                e.mul_assign(&y_val.unwrap());
                e
            });

            let newy = if i == INPUT_SIZE - 1 {
                cs.new_input_variable(|| newy_val.ok_or(SynthesisError::AssignmentMissing))?
            } else {
                cs.new_witness_variable(|| newy_val.ok_or(SynthesisError::AssignmentMissing))?
            };

            cs.enforce_constraint(
                lc!() + y,
                lc!() + tmp2,
                lc!() + newy
             )?;
 
            let newg_val = g_val.map(|e| {
                let g2 = e.square();
                g2
            });

            let newg = cs.new_witness_variable(|| newg_val.ok_or(SynthesisError::AssignmentMissing))?;

            cs.enforce_constraint(
                lc!() + g,
                lc!() + g,
                lc!() + newg
            )?;

            g = newg;
            g_val = newg_val;
            y = newy;
            y_val = newy_val;
        }
        Ok(())
    }
}

#[test]
fn test_pow() {
    use std::time::{Duration, Instant};
    use ark_ff::{BigInteger256, BigInteger};
    use ark_std::rand::Rng;
    use ark_std::test_rng;
    use ark_bls12_381::{Fr, Bls12_381};

    // We're going to use the Groth proving system.
    use ark_groth16::{
        create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
    };

    let mut rng = test_rng();

    println!("Creating parameters...");
    let mut x_bit = [None; INPUT_SIZE];

    let params = {
        let c = PowDemo::<Fr> {
            g: None,
            x_bits: &x_bit
        };

        generate_random_parameters::<Bls12_381, _, _>(c, &mut rng).unwrap()
    };

    let pvk = prepare_verifying_key(&params.vk);

    println!("Creating proofs...");

    const SAMPLES: u32 = 50;
    let mut total_proving = Duration::new(0, 0);
    let mut total_verifying = Duration::new(0, 0);

    // let mut proof_vec = vec![];

    for sample in 0..SAMPLES {
        
        println!("Test sample: {:?}", sample + 1);

        let g: Fr = rng.gen();
        let x = BigInteger256::from(2);
        let y = g.pow(x);

        let bits = x.to_bits_le();
        for i in 0..INPUT_SIZE {
            if bits[i] == true {
                x_bit[i] = Some(1);
            }
            else {
                x_bit[i] = Some(0);
            }
        }

        // proof_vec.truncate(0);

        let start = Instant::now();
        {
            let c = PowDemo {
                g: Some(g),
                x_bits: &x_bit
            };

            let proof = create_random_proof(c, &params, &mut rng).unwrap();
            assert!(verify_proof(&pvk, &proof, &[y]).is_ok());

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