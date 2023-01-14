use ark_ff::Field;
use ark_relations::{
    lc, ns,
    r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError, Variable},
};

pub const MIMC_DF_ROUNDS: usize = 322;
pub const MIMC_HASH_ROUNDS: usize = 10;

// y = mimc_vde(key + x, m)
// yn = y
pub struct PosDemo<'a, F: Field> {
    pub key: Option<F>, // verify input 1
    pub x: &'a [Option<F>],
    pub m: Option<F>, // verify input 2
    pub df_constants: &'a [F],
    pub y_bits: &'a [Option<[Option<F>; 256]>],
    pub yn: &'a [Option<F>], // verify input 3
    pub difficulty: usize,
    pub hash_constants: &'a [F] // verify input 4
}

impl<'a, F: Field> ConstraintSynthesizer<F> for PosDemo<'a, F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        assert_eq!(self.df_constants.len(), MIMC_DF_ROUNDS);
        assert_eq!(self.x.len(), self.yn.len());

        let key_val = self.key;
        let key = cs.new_input_variable(|| key_val.ok_or(SynthesisError::AssignmentMissing))?;

        let m_val = self.m;
        cs.new_input_variable(|| m_val.ok_or(SynthesisError::AssignmentMissing))?;
    
        // Start to create all proofs: y = mimc(key + x, m)
        for i in 0..self.x.len() {
            let ns = ns!(cs, "sample");
            let cs = ns.cs();

            let x_val = self.x[i];
            let x = cs.new_witness_variable(|| x_val.ok_or(SynthesisError::AssignmentMissing))?;

            // xl = key + x
            let mut xl_val = key_val.map(|mut e| {
                e.add_assign(&x_val.unwrap());
                e
            });
            let mut xl = cs.new_witness_variable(|| xl_val.ok_or(SynthesisError::AssignmentMissing))?;
            cs.enforce_constraint(
                lc!() + key + x,
                lc!() + Variable::One,
                lc!() + xl,
            )?;

            // xr = m
            let mut xr_val = m_val;
            let mut xr = cs.new_witness_variable(|| xr_val.ok_or(SynthesisError::AssignmentMissing))?;

            // Start to create single mimc proof: yi = mimc(key + xi, m)
            for j in 0..MIMC_DF_ROUNDS {
                let tmp_val = xl_val.map(|mut e| {
                    e.add_assign(&self.df_constants[j]);
                    e.square()
                });
                let tmp = cs.new_witness_variable(|| tmp_val.ok_or(SynthesisError::AssignmentMissing))?;
                cs.enforce_constraint(
                    lc!() + xl + (self.df_constants[j], Variable::One),
                    lc!() + xl + (self.df_constants[j], Variable::One),
                    lc!() + tmp,
                )?;
    
                let new_xl_val = xl_val.map(|mut e| {
                    e.add_assign(&self.df_constants[j]);
                    e.mul_assign(&tmp_val.unwrap());
                    e.add_assign(&xr_val.unwrap());
                    e
                });
                let new_xl = cs.new_witness_variable(|| new_xl_val.ok_or(SynthesisError::AssignmentMissing))?;
                cs.enforce_constraint(
                    lc!() + tmp,
                    lc!() + xl + (self.df_constants[j], Variable::One),
                    lc!() + new_xl - xr,
                )?;
    
                // Start to create proof of yn = y[0..n]
                if j == MIMC_DF_ROUNDS - 1 {
                    let y_bits_val = self.y_bits[i];
                    {
                        let mut y_val = Some(F::zero());

                        let mut two_val = Some(F::one());
                        let mut two = cs.new_witness_variable(|| two_val.ok_or(SynthesisError::AssignmentMissing))?;

                        let mut y = cs.new_witness_variable(|| y_val.ok_or(SynthesisError::AssignmentMissing))?;

                        for k in 0..y_bits_val.unwrap().len() {
                            // bit = xi
                            let bit_val = y_bits_val.unwrap()[k];
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
                                e.add_assign(&y_val.unwrap());
                                e
                            });
                            let tmp2 = cs.new_witness_variable(|| tmp2_val.ok_or(SynthesisError::AssignmentMissing))?;
                            cs.enforce_constraint(
                                lc!() + tmp1 + y,
                                lc!() + Variable::One,
                                lc!() + tmp2
                            )?;

                            // yn = tmp2
                            if k == self.difficulty - 1 {
                                let yn_val = self.yn[i];
                                let yn = cs.new_input_variable(|| yn_val.ok_or(SynthesisError::AssignmentMissing))?;

                                cs.enforce_constraint(
                                    lc!() + tmp2,
                                    lc!() + Variable::One,
                                    lc!() + yn
                                )?;
                            }

                            // y = y0 * 2^0 + y1 * 2^1 + ..
                            if k == y_bits_val.unwrap().len() - 1 {
                                let y_val = new_xl_val;
                                let y = cs.new_witness_variable(|| y_val.ok_or(SynthesisError::AssignmentMissing))?;
                                cs.enforce_constraint(
                                    lc!() + tmp2,
                                    lc!() + Variable::One,
                                    lc!() + y
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
                            y = tmp2;
                            y_val = tmp2_val;
                        }
                    }
                }

                // xR = xL
                xr = xl;
                xr_val = xl_val;
    
                // xL = new_xL
                xl = new_xl;
                xl_val = new_xl_val;
            }
        }

        let mut res_val  = self.key;
        let mut res = cs.new_witness_variable(|| res_val.ok_or(SynthesisError::AssignmentMissing))?;

        // Start to compute mimc hash.
        for i in 0..self.x.len() {
            let x_in_val= self.x[i];
            let x_in = cs.new_witness_variable(|| x_in_val.ok_or(SynthesisError::AssignmentMissing))?;

            let key_val = res_val;
            let key = cs.new_witness_variable(|| key_val.ok_or(SynthesisError::AssignmentMissing))?;    
            
            let mut h_val = Some(F::zero());
            let mut h = cs.new_witness_variable(|| h_val.ok_or(SynthesisError::AssignmentMissing))?;

            // Create every single mimc hash.
            for j in 0..MIMC_HASH_ROUNDS {
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
                        e.add_assign(&self.hash_constants[j]);
                        e
                    });
                    t = cs.new_witness_variable(|| t_val.ok_or(SynthesisError::AssignmentMissing))?;
                    cs.enforce_constraint(
                        lc!() + h + key + (self.hash_constants[j], Variable::One),
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
            let new_res = if i == (self.x.len() - 1) {
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

#[cfg(test)]
mod test {
    use ark_ff::{BigInteger256, BigInteger};
    use ark_bls12_381::Fr;

    #[test]
    fn test_random() {
        let y = BigInteger256::from_bits_le(&[true]);
        println!("{:?}", y);
        let z = Fr::from(y);
        let a: BigInteger256 = z.into();
        println!("{:?}", a);
    }
}