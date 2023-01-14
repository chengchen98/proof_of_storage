use ark_ff::Field;

// We'll use these interfaces to construct our circuit.
use ark_relations::{
    lc, ns,
    r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError, Variable},
};

pub const INPUT_SIZE: usize = 256;

/// Prove two equations: 
///
/// x2 = x_bit
/// 
/// x1 = x2\[0..n\]
pub struct EqualDemo<'a, F: Field>{
    pub x1: Option<F>,
    pub x2: Option<F>,
    pub x_bits: &'a [Option<u8>],
    pub difficulty: usize
}

impl<'a, F: Field> ConstraintSynthesizer<F> for EqualDemo<'a, F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError>{
        assert_eq!(self.x_bits.len(), INPUT_SIZE);
        
        let ns = ns!(cs, "round");
        let cs = ns.cs();

        let mut x_val = Some(F::zero());
        let mut two_val = Some(F::one());

        let mut two = cs.new_witness_variable(|| two_val.ok_or(SynthesisError::AssignmentMissing))?;

        let mut x = cs.new_witness_variable(|| x_val.ok_or(SynthesisError::AssignmentMissing))?;

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
            
            // tmp1 = xi * 2^i
            let tmp1_val = bit_val.map(|mut e| {
                e.mul_assign(&two_val.unwrap());
                e
            });
            let tmp1 = cs.new_witness_variable(|| tmp1_val.ok_or(SynthesisError::AssignmentMissing))?;
            cs.enforce_constraint(
                lc!() + bit,
                lc!() + two,
                lc!() + tmp1
            )?;
            
            // tmp2 = tmp1 + x
            let tmp2_val = tmp1_val.map(|mut e| {
                e.add_assign(&x_val.unwrap());
                e
            });
            let tmp2 = cs.new_witness_variable(|| tmp2_val.ok_or(SynthesisError::AssignmentMissing))?;
            cs.enforce_constraint(
                lc!() + tmp1 + x,
                lc!() + Variable::One,
                lc!() + tmp2
            )?;

            // tmp2 = x1
            if i == self.difficulty - 1 {
                let x1_val = self.x1;
                let x1 = cs.new_input_variable(|| x1_val.ok_or(SynthesisError::AssignmentMissing))?;
                cs.enforce_constraint(
                    lc!() + tmp2,
                    lc!() + Variable::One,
                    lc!() + x1
                )?;
            }

            // tmp2 = x2
            if i == INPUT_SIZE - 1 {
                let x2_val = self.x2;
                let x2 = cs.new_input_variable(|| x2_val.ok_or(SynthesisError::AssignmentMissing))?;
                cs.enforce_constraint(
                    lc!() + tmp2,
                    lc!() + Variable::One,
                    lc!() + x2
                )?;
            }
           
            let newtwo_val = two_val.map(|mut e| {
                 e.mul_assign(F::one().add(F::one()));
                 e
            });
            let newtwo = cs.new_witness_variable(|| newtwo_val.ok_or(SynthesisError::AssignmentMissing))?;
            cs.enforce_constraint(
                lc!() + two,
                lc!() + (F::one().add(F::one()), Variable::One),
                lc!() + newtwo
            )?;

            two = newtwo;
            two_val = newtwo_val;
            x = tmp2;
            x_val = tmp2_val;
        }

        Ok(())
    }
}

#[test]
fn test_equal() {
    // For benchmarking
    use std::time::{Duration, Instant};
    use ark_ff::{BigInteger256, BigInteger};
    use ark_std::rand::Rng;
    use ark_std::test_rng;
    use ark_bls12_381::{Fr, Bls12_381};

    // We're going to use the Groth proving system.
    use ark_groth16::{
        create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
    };

    let mut rng = &mut test_rng();
    const DIFFICULTY: usize = 8;

    println!("Creating parameters...");

    let mut x_bit = [None; INPUT_SIZE];

    let params = {
        let c = EqualDemo::<Fr> {
            x1: None,
            x2: None,
            x_bits: &x_bit,
            difficulty: DIFFICULTY
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

        let x2: Fr = rng.gen();
        let x2_bits: BigInteger256 = x2.into();
        let x2_bits = x2_bits.to_bits_le();

        let x1_bits = &x2_bits[.. DIFFICULTY];
        let x1 = BigInteger256::from_bits_le(&x1_bits);
        let x1 = Fr::from(x1);
        
        for i in 0..INPUT_SIZE {
            if x2_bits[i] == true {
                x_bit[i] = Some(1);
            }
            else {
                x_bit[i] = Some(0);
            }
        }

        // proof_vec.truncate(0);

        let start = Instant::now();
        {
            let c = EqualDemo {
                x1: Some(x1),
                x2: Some(x2),
                x_bits: &x_bit,
                difficulty: DIFFICULTY
            };

            let proof = create_random_proof(c, &params, &mut rng).unwrap();
            assert!(verify_proof(&pvk, &proof, &[x1, x2]).is_ok());
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