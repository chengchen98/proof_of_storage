use ff::{PrimeField, PrimeFieldBits};
use bellman::{Circuit, ConstraintSystem, SynthesisError};

pub const MIMC_ROUNDS: usize = 322;

pub struct PosDemo<'a, S: PrimeField> {
    pub key: Option<S>, // verify input 1
    pub x: &'a [Option<S>],
    pub m: Option<S>, // verify input 2
    pub constants: &'a [S],

    pub yn: &'a [Option<S>], // verify input 3
    pub difficulty: usize
}

impl<'a, S: PrimeField + PrimeFieldBits> Circuit<S> for PosDemo<'a, S> {
    fn synthesize<CS: ConstraintSystem<S>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        assert_eq!(self.constants.len(), MIMC_ROUNDS);

        let key_val = self.key;
        let key = cs.alloc_input(
            || "key",
            || key_val.ok_or(SynthesisError::AssignmentMissing),
        )?;

        let m_val = self.m;
        cs.alloc_input(
            || "m",
            || m_val.ok_or(SynthesisError::AssignmentMissing),
        )?;
    
        // Start to create all proofs: y = mimc(key + x, m)
        for i in 0..self.x.len() {
            let cs = &mut cs.namespace(|| format!("samples {}", i));

            let x_val = self.x[i];
            let x = cs.alloc(
                || "x",
                || x_val.ok_or(SynthesisError::AssignmentMissing),
            )?;

            // xl = key + x
            let mut xl_val = key_val.map(|mut e| {
                e.add_assign(&x_val.unwrap());
                e
            });
            let mut xl = cs.alloc(
                || "preimage xl",
                || xl_val.ok_or(SynthesisError::AssignmentMissing),
            )?;
            cs.enforce(
                || "xl = key + x",
                |lc| lc + key + x,
                |lc| lc + CS::one(),
                |lc| lc + xl,
            );

            // xr = m
            let mut xr_val = m_val;
            let mut xr = cs.alloc(
                || "preimage xr",
                || xr_val.ok_or(SynthesisError::AssignmentMissing),
            )?;

            // Start to create single mimc proof: yi = mimc(key + xi, m)
            for j in 0..MIMC_ROUNDS {
                let tmp_val = xl_val.map(|mut e| {
                    e.add_assign(&self.constants[j]);
                    e.square()
                });
                let tmp = cs.alloc(
                    || "tmp",
                    || tmp_val.ok_or(SynthesisError::AssignmentMissing),
                )?;
                cs.enforce(
                    || "tmp = (xL + Ci)^2",
                    |lc| lc + xl + (self.constants[j], CS::one()),
                    |lc| lc + xl + (self.constants[j], CS::one()),
                    |lc| lc + tmp,
                );
    
                let new_xl_val = xl_val.map(|mut e| {
                    e.add_assign(&self.constants[j]);
                    e.mul_assign(&tmp_val.unwrap());
                    e.add_assign(&xr_val.unwrap());
                    e
                });
                let new_xl = cs.alloc(
                    || "new_xl",
                    || new_xl_val.ok_or(SynthesisError::AssignmentMissing),
                )?;
                cs.enforce(
                    || "new_xL = xR + (xL + Ci)^3",
                    |lc| lc + tmp,
                    |lc| lc + xl + (self.constants[j], CS::one()),
                    |lc| lc + new_xl - xr,
                );
    
                // Start to create proof of yn = y[0..n]
                if j == MIMC_ROUNDS - 1 {
                    // let yn_val = self.yn[i];
                    // let yn = cs.alloc_input(
                    //     || "yn",
                    //     || yn_val.ok_or(SynthesisError::AssignmentMissing),
                    // )?;

                    let x_bits = new_xl_val.unwrap().to_le_bits();
                    {
                        let mut x_val = S::from_str_vartime("0");

                        let mut two_val = S::from_str_vartime("1");
                        let mut two = cs.alloc(|| "g", || {
                            two_val.ok_or(SynthesisError::AssignmentMissing)
                        })?;

                        let mut y = cs.alloc(|| "y", || {
                            x_val.ok_or(SynthesisError::AssignmentMissing)
                        })?;

                        for k in 0..x_bits.len() {
                            // bit = xi
                            let bit_val;
                            if x_bits[k] == true {
                                bit_val = Some(S::one());
                            }
                            else {
                                bit_val = Some(S::zero());
                            }
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
                                || "tmp1 + y = tmp2",
                                |lc| lc + tmp1 + y,
                                |lc| lc + CS::one(),
                                |lc| lc + tmp2
                            );

                            // yn = tmp2
                            if k == self.difficulty - 1 {
                                let yn_val = self.yn[i];
                                let yn = cs.alloc_input(|| "yn", || {
                                    yn_val.ok_or(SynthesisError::AssignmentMissing)
                                })?;

                                cs.enforce(
                                    || "yn = tmp2",
                                    |lc| lc + tmp2,
                                    |lc| lc + CS::one(),
                                    |lc| lc + yn
                                );
                            }

                            // y = y0 * 2^0 + y1 * 2^1 + ..
                            if k == x_bits.len() - 1 {
                                let y_val = new_xl_val;
                                let y = cs.alloc(|| "y2", || {
                                    y_val.ok_or(SynthesisError::AssignmentMissing)
                                })?;

                                cs.enforce(
                                    || "new_xl = tmp2",
                                    |lc| lc + y,
                                    |lc| lc + CS::one(),
                                    |lc| lc + new_xl
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