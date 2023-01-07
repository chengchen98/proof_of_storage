use ff::PrimeField;
use bellman::{Circuit, ConstraintSystem, SynthesisError};

pub const INPUT_SIZE: usize = 256;

/// Prove two equations
/// 
/// x2 = x_bit
/// 
/// x1 = x2\[0..n\]
pub struct EqualDemo<S: PrimeField>{
    pub x1: Option<S>,
    pub x2: Option<S>,
    pub x_bit: [Option<u8>; INPUT_SIZE],
    pub difficulty: usize
}

impl<S:PrimeField> Circuit<S> for EqualDemo<S> {
    fn synthesize<CS: ConstraintSystem<S>>(self, cs: &mut CS) -> Result<(), SynthesisError>{
        let mut x_val = S::from_str_vartime("0");
        let mut two_val = S::from_str_vartime("1");  

        let mut two = cs.alloc(|| "g", || {
            two_val.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let mut y = cs.alloc(|| "y", || {
            x_val.ok_or(SynthesisError::AssignmentMissing)
        })?;

        for i in 0..INPUT_SIZE {
            let bit = self.x_bit[i];
            let mut bit_val = None;
            if bit == Some(1) {
                bit_val = Some(S::one());
            }
            else if bit == Some(0) {
                bit_val = Some(S::zero());
            }

            // bit = xi
            let bit = cs.alloc(|| "xi", || {
                bit_val.ok_or(SynthesisError::AssignmentMissing)
            })?;

            // xi = 0 or 1
            cs.enforce(
                || "xi * xi = xi",
                |lc| lc + bit,
                |lc| lc + bit,
                |lc| lc + bit
            );
            
            // tmp1 = xi * 2^i
            let tmp1_val = bit_val.map(|mut e| {
                e.mul_assign(&two_val.unwrap());
                e
            });

            let tmp1 = cs.alloc(|| "tmp1", || {
                tmp1_val.ok_or(SynthesisError::AssignmentMissing)
            })?;
            
            cs.enforce(
                || "xi * 2^i = tmp1",
                |lc| lc + bit,
                |lc| lc + two,
                |lc| lc + tmp1
            );
            
            // tmp2 = tmp1 + x
            let tmp2_val = tmp1_val.map(|mut e| {
                e.add_assign(&x_val.unwrap());
                e
            });

            let tmp2 = cs.alloc(|| "tmp2", || {
                tmp2_val.ok_or(SynthesisError::AssignmentMissing)
            })?;
             
            cs.enforce(
                || "tmp1 + 1 = tmp2 + x_i",
                |lc| lc + tmp1 + y,
                |lc| lc + CS::one(),
                |lc| lc + tmp2
            );

            if i == self.difficulty - 1 {
                let y1_val = self.x1;
                let y1 = cs.alloc_input(|| "y1", || {
                    y1_val.ok_or(SynthesisError::AssignmentMissing)
                })?;

                cs.enforce(
                    || "y1 = tmp2",
                    |lc| lc + tmp2,
                    |lc| lc + CS::one(),
                    |lc| lc + y1
                );
            } 

            if i == INPUT_SIZE - 1 {
                let y2_val = self.x2;
                let y2 = cs.alloc_input(|| "y2", || {
                    y2_val.ok_or(SynthesisError::AssignmentMissing)
                })?;

                cs.enforce(
                    || "y2 = tmp2",
                    |lc| lc + tmp2,
                    |lc| lc + CS::one(),
                    |lc| lc + y2
                );
            } 
           
            let newtwo_val = two_val.map(|mut e| {
                 e.mul_assign(S::from_str_vartime("2").unwrap());
                 e
            });

            let newtwo = cs.alloc(|| "new_g", || {
                newtwo_val.ok_or(SynthesisError::AssignmentMissing)
            })?;

            cs.enforce(
                || "new_two = two * 2",
                |lc| lc + two,
                |lc| lc + (S::from_str_vartime("2").unwrap(), CS::one()),
                |lc| lc + newtwo
            );

            two = newtwo;
            two_val = newtwo_val;
            y = tmp2;
            x_val = tmp2_val;
        }

        Ok(())
    }
}


#[cfg(test)]
mod test {
    use ff::Field;
    use rand::thread_rng;
    use std::time::{Duration, Instant};
    use bls12_381::{Bls12, Scalar};
    
    use crate::convert::{s_to_bits, bits_to_s};

    use bellman::groth16::{
        create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
        Proof,
    };

    use super::*;

    #[test]
    fn test_equal() {
        let mut rng = thread_rng();
        const DIFFICULTY: usize = 8;
    
        println!("Creating parameters...");
    
        let params = {
            let c = EqualDemo {
                x1: None,
                x2: None,
                x_bit: [None; INPUT_SIZE],
                difficulty: DIFFICULTY
            };

            generate_random_parameters::<Bls12, _, _>(c, &mut rng).unwrap()
        };
    
        let pvk = prepare_verifying_key(&params.vk);
    
        println!("Creating proofs...");
    
        const SAMPLES: u32 = 50;
        let mut total_proving = Duration::new(0, 0);
        let mut total_verifying = Duration::new(0, 0);
    
        let mut proof_vec = vec![];
    
        for sample in 0..SAMPLES {
            
            println!("Test sample: {:?}", sample + 1);

            let x = Scalar::random(&mut rng);
            let x_bit = s_to_bits(x, INPUT_SIZE);

            let x1 = bits_to_s(&x_bit, DIFFICULTY);
            let x2 = bits_to_s(&x_bit, INPUT_SIZE);
            
            let mut new_x_bit: [Option<u8>; INPUT_SIZE] = [None; INPUT_SIZE];
            for i in 0..INPUT_SIZE {
                new_x_bit[i] = Some(x_bit[i]);
            }
    
            proof_vec.truncate(0);
    
            let start = Instant::now();
            {
                let c = EqualDemo {
                    x1: Some(x1),
                    x2: Some(x2),
                    x_bit: new_x_bit,
                    difficulty: DIFFICULTY
                };
    
                let proof = create_random_proof(c, &params, &mut rng).unwrap();
    
                proof.write(&mut proof_vec).unwrap();
            }
    
            total_proving += start.elapsed();
    
            let start = Instant::now();
            let proof = Proof::read(&proof_vec[..]).unwrap();
            
            // Check the proof
            assert!(verify_proof(&pvk, &proof, &[x1, x2]).is_ok());
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
}