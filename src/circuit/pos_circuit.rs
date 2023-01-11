use ark_ff::Field;
use ark_relations::{
    lc, ns,
    r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError, Variable},
};

pub const MIMC_ROUNDS: usize = 322;

// y = mimc_vde(key + x, m)
// yn = y
pub struct PosDemo<'a, F: Field> {
    pub key: Option<F>, // verify input 1
    pub x: &'a [Option<F>],
    pub m: Option<F>, // verify input 2
    pub constants: &'a [F],
    pub y_bits: &'a [Option<[Option<F>; 256]>],
    pub yn: &'a [Option<F>], // verify input 3
    pub difficulty: usize
}

impl<'a, F: Field> ConstraintSynthesizer<F> for PosDemo<'a, F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        assert_eq!(self.constants.len(), MIMC_ROUNDS);
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
            for j in 0..MIMC_ROUNDS {
                let tmp_val = xl_val.map(|mut e| {
                    e.add_assign(&self.constants[j]);
                    e.square()
                });
                let tmp = cs.new_witness_variable(|| tmp_val.ok_or(SynthesisError::AssignmentMissing))?;
                cs.enforce_constraint(
                    lc!() + xl + (self.constants[j], Variable::One),
                    lc!() + xl + (self.constants[j], Variable::One),
                    lc!() + tmp,
                )?;
    
                let new_xl_val = xl_val.map(|mut e| {
                    e.add_assign(&self.constants[j]);
                    e.mul_assign(&tmp_val.unwrap());
                    e.add_assign(&xr_val.unwrap());
                    e
                });
                let new_xl = cs.new_witness_variable(|| new_xl_val.ok_or(SynthesisError::AssignmentMissing))?;
                cs.enforce_constraint(
                    lc!() + tmp,
                    lc!() + xl + (self.constants[j], Variable::One),
                    lc!() + new_xl - xr,
                )?;
    
                // Start to create proof of yn = y[0..n]
                if j == MIMC_ROUNDS - 1 {
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
        
        // let mut x_val = S::from_str_vartime("0");
        // let mut x = cs.alloc(
        //     || "x",
        //     || x_val.ok_or(SynthesisError::AssignmentMissing),
        // )?;

        // for i in 0..self.x.len() {
        //     // x += self.x[i];
        //     let xi_val = S::from_str_vartime(&self.x[i].to_string());
        //     let xi = cs.alloc(
        //         || "xi",
        //         || xi_val.ok_or(SynthesisError::AssignmentMissing),
        //     )?;

        //     let newx_val = xi_val.map(|mut e| {
        //         e.add_assign(&x_val.unwrap());
        //         e
        //     });
        //     let newx = cs.alloc(
        //         || "newx",
        //         || newx_val.ok_or(SynthesisError::AssignmentMissing),
        //     )?;
        //     cs.enforce(
        //         || "newx = x + xi",
        //         |lc| lc + xi + x,
        //         |lc| lc + CS::one(),
        //         |lc| lc + newx,
        //     );

        //     x = newx;
        //     x_val = newx_val;

        //     // x_hash = mimc(x)
        //     if i == self.x.len() - 1 {
        //         let mut xl_val = x_val;
        //         let mut xl = cs.alloc(
        //             || "preimage xl",
        //             || xl_val.ok_or(SynthesisError::AssignmentMissing),
        //         )?;

        //         let mut xr_val = x_val;
        //         let mut xr = cs.alloc(
        //             || "preimage xr",
        //             || xr_val.ok_or(SynthesisError::AssignmentMissing),
        //         )?;

        //         for j in 0..MIMC_ROUNDS {
        //             // xL, xR := xR + (xL + Ci)^3, xL

        //             // tmp = (xL + Ci)^2
        //             let tmp_val = xl_val.map(|mut e| {
        //                 e.add_assign(&self.constants[j]);
        //                 e.square()
        //             });
        //             let tmp = cs.alloc(
        //                 || "tmp",
        //                 || tmp_val.ok_or(SynthesisError::AssignmentMissing),
        //             )?;
        //             cs.enforce(
        //                 || "tmp = (xL + Ci)^2",
        //                 |lc| lc + xl + (self.constants[j], CS::one()),
        //                 |lc| lc + xl + (self.constants[j], CS::one()),
        //                 |lc| lc + tmp,
        //             );

        //             // new_xL = xR + (xL + Ci)^3
        //             // new_xL = xR + tmp * (xL + Ci)
        //             // new_xL - xR = tmp * (xL + Ci)
        //             let new_xl_val = xl_val.map(|mut e| {
        //                 e.add_assign(&self.constants[j]);
        //                 e.mul_assign(&tmp_val.unwrap());
        //                 e.add_assign(&xr_val.unwrap());
        //                 e
        //             });
        //             let new_xl = if j == (MIMC_ROUNDS - 1) {
        //                 // This is the last round, xL is our image and so
        //                 // we allocate a public input.
        //                 cs.alloc_input(
        //                     || "image",
        //                     || new_xl_val.ok_or(SynthesisError::AssignmentMissing),
        //                 )?
        //             } else {
        //                 cs.alloc(
        //                     || "new_xl",
        //                     || new_xl_val.ok_or(SynthesisError::AssignmentMissing),
        //                 )?
        //             };

        //             cs.enforce(
        //                 || "new_xL = xR + (xL + Ci)^3",
        //                 |lc| lc + tmp,
        //                 |lc| lc + xl + (self.constants[j], CS::one()),
        //                 |lc| lc + new_xl - xr,
        //             );

        //             // xR = xL
        //             xr = xl;
        //             xr_val = xl_val;

        //             // xL = new_xL
        //             xl = new_xl;
        //             xl_val = new_xl_val;
        //         }
        //     }
        // }

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