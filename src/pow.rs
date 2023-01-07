use ff::PrimeField;
use bellman::{Circuit, ConstraintSystem, SynthesisError};

pub const INPUT_SIZE: usize = 20;

/// Prove the process of computing g\^x
/// 
/// x is bit
pub struct PowDemo<S: PrimeField>{
    pub g: Option<S>,
    pub x_bit: [Option<u8>; INPUT_SIZE]
}

pub fn pow<S: PrimeField>(g: S, x: &Vec<u8>) -> S {
    let mut y = S::one();
    let mut g_val = g;
    for i in 0..x.len() {
       if x[i] == 1 {
            y.mul_assign(&g_val);
       }
       g_val = g_val.square();
    }
    y
}

impl<S:PrimeField> Circuit<S> for PowDemo<S> {
    fn synthesize<CS: ConstraintSystem<S>>(self, cs: &mut CS) -> Result<(), SynthesisError>{
        let mut y_val = S::from_str_vartime("1");
        let mut g_val = self.g; 

        let mut g = cs.alloc(|| "g", || {
            g_val.ok_or(SynthesisError::AssignmentMissing)
        })?;

        let mut y = cs.alloc(|| "y", || {
            y_val.ok_or(SynthesisError::AssignmentMissing)
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
            
            // tmp1 = xi * g^(2^i)
            let tmp1_val = g_val.map(|mut e| {
                e.mul_assign(&bit_val.unwrap());
                e
            });

            let tmp1 = cs.alloc(|| "tmp1", || {
                tmp1_val.ok_or(SynthesisError::AssignmentMissing)
            })?;

            cs.enforce(
                || "xi * g^(2*i) = tmp1",
                |lc| lc + bit,
                |lc| lc + g,
                |lc| lc + tmp1
            );

            // tmp2 = tmp1 + 1 - xi
            let tmp2_val = tmp1_val.map(|mut e| {
                e.add_assign(&S::from_str_vartime("1").unwrap());
                e.sub_assign(&bit_val.unwrap());
                e
            });

            let tmp2 = cs.alloc(|| "tmp2", || {
                tmp2_val.ok_or(SynthesisError::AssignmentMissing)
            })?;
             
            cs.enforce(
                || "tmp1 + 1 = tmp2 + xi",
                |lc| lc + tmp1 + (S::from_str_vartime("1").unwrap(), CS::one()),
                |lc| lc + CS::one(),
                |lc| lc + tmp2 + bit
            );
            
            // newy = tmp2 * y
            let newy_val = tmp2_val.map(|mut e| {
                e.mul_assign(&y_val.unwrap());
                e
            });
        
            let newy = if i == INPUT_SIZE - 1 {
                cs.alloc_input(|| "image", || {
                    newy_val.ok_or(SynthesisError::AssignmentMissing)
                })?
            } else {
                cs.alloc(|| "newy", || {
                    newy_val.ok_or(SynthesisError::AssignmentMissing)
                })?
            };

            cs.enforce(
                || "y * tmp2 = newy",
                |lc| lc + y,
                |lc| lc + tmp2,
                |lc| lc + newy
             );
 
            let newg_val = g_val.map(|e| {
                let tt = e.square();
                tt
            });

            let newg = cs.alloc(|| "newg", || {
                newg_val.ok_or(SynthesisError::AssignmentMissing)
            })?;

            cs.enforce(
                || "g * g = newg",
                |lc| lc + g,
                |lc| lc + g,
                |lc| lc + newg
            );

            g = newg;
            g_val = newg_val;
            y = newy;
           y_val = newy_val;
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
    use crate::convert::s_to_bits;

    use bellman::groth16::{
        create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
        Proof,
    };

    use super::*;

    #[test]
    fn test_pow() {
        let mut rng = thread_rng();
    
        println!("Creating parameters...");
    
        let params = {
            let c = PowDemo {
                g: None,
                x_bit: [None; INPUT_SIZE],
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
            let g = Scalar::random(&mut rng);
            let y = pow(g, &x_bit);
            
            let mut new_x: [Option<u8>; INPUT_SIZE] = [None; INPUT_SIZE];
            for i in 0..INPUT_SIZE {
                new_x[i] = Some(x_bit[i]);
            }
    
            proof_vec.truncate(0);
    
            let start = Instant::now();
            {
                let c = PowDemo {
                    g: Some(g),
                    x_bit: new_x
                };
    
                let proof = create_random_proof(c, &params, &mut rng).unwrap();
    
                proof.write(&mut proof_vec).unwrap();
            }
    
            total_proving += start.elapsed();
    
            let start = Instant::now();
            let proof = Proof::read(&proof_vec[..]).unwrap();
            // Check the proof
            assert!(verify_proof(&pvk, &proof, &[y]).is_ok());
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