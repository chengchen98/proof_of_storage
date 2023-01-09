use ff::{PrimeField, PrimeFieldBits};
use bellman::{Circuit, ConstraintSystem, SynthesisError};

pub const MIMC_ROUNDS: usize = 322;

pub struct PosDemo<'a, S: PrimeField> {
    pub key: Option<S>, // verify input 1
    pub x: &'a [usize],
    pub m: Option<S>, // verify input 2
    pub constants: &'a [S],

    pub yn: &'a [Option<S>], // verify input 3
    pub difficulty: usize
}

impl<'a, S: PrimeField + PrimeFieldBits> Circuit<S> for PosDemo<'a, S> {
    fn synthesize<CS: ConstraintSystem<S>>(self, cs: &mut CS) -> Result<(), SynthesisError> {
        assert_eq!(self.x.len(), self.yn.len());
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
    
        // Create all proofs of response.
        for i in 0..self.x.len() {
            let cs = &mut cs.namespace(|| format!("samples {}", i));

            // xl = key + x
            let mut xl_val = key_val.map(|mut e| {
                e.add_assign(&S::from_str_vartime(&self.x[i].to_string()).unwrap());
                e
            });
            let mut xl = cs.alloc(
                || "preimage xl",
                || xl_val.ok_or(SynthesisError::AssignmentMissing),
            )?;
            cs.enforce(
                || "xl = key + x",
                |lc| lc + key + (S::from_str_vartime(&self.x[i].to_string()).unwrap(), CS::one()),
                |lc| lc + CS::one(),
                |lc| lc + xl,
            );

            // xr = m
            let mut xr_val = m_val;
            let mut xr = cs.alloc(
                || "preimage xr",
                || xr_val.ok_or(SynthesisError::AssignmentMissing),
            )?;

            // Create every proof of mimc hash.
            for j in 0..MIMC_ROUNDS {
                // xL, xR := xR + (xL + Ci)^3, xL
    
                // tmp = (xL + Ci)^2
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
    
                // new_xL = xR + (xL + Ci)^3
                // new_xL = xR + tmp * (xL + Ci)
                // new_xL - xR = tmp * (xL + Ci)
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
    
                if j == MIMC_ROUNDS - 1 {
                    let yn_val = self.yn[i];
                    cs.alloc_input(
                        || "yn",
                        || yn_val.ok_or(SynthesisError::AssignmentMissing),
                    )?;

                    let yn_bits = yn_val.unwrap().to_le_bits();

                    let y_val = new_xl_val;
                    let y_bits = y_val.unwrap().to_le_bits();

                    // Create proof of yn = y[0..n].
                    for k in 0..self.difficulty {
                        let comp_val= if yn_bits[k] == y_bits[k] {
                            Some(S::zero())
                        }
                        else {
                            Some(S::one())
                        };

                        let comp = cs.alloc(|| "comp", || {
                            comp_val.ok_or(SynthesisError::AssignmentMissing)
                        })?;

                        cs.enforce(
                            || "(comp + 1) * 1 = 1",
                            |lc| lc + comp + CS::one(),
                            |lc| lc + CS::one(),
                            |lc| lc + CS::one()
                        );
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
        
        let mut x_val = S::from_str_vartime("0");
        let mut x = cs.alloc(
            || "x",
            || x_val.ok_or(SynthesisError::AssignmentMissing),
        )?;

        for i in 0..self.x.len() {
            // x += self.x[i];
            let xi_val = S::from_str_vartime(&self.x[i].to_string());
            let xi = cs.alloc(
                || "xi",
                || xi_val.ok_or(SynthesisError::AssignmentMissing),
            )?;

            let newx_val = xi_val.map(|mut e| {
                e.add_assign(&x_val.unwrap());
                e
            });
            let newx = cs.alloc(
                || "newx",
                || newx_val.ok_or(SynthesisError::AssignmentMissing),
            )?;
            cs.enforce(
                || "newx = x + xi",
                |lc| lc + xi + x,
                |lc| lc + CS::one(),
                |lc| lc + newx,
            );

            x = newx;
            x_val = newx_val;

            // x_hash = mimc(x)
            if i == self.x.len() - 1 {
                let mut xl_val = x_val;
                let mut xl = cs.alloc(
                    || "preimage xl",
                    || xl_val.ok_or(SynthesisError::AssignmentMissing),
                )?;

                let mut xr_val = x_val;
                let mut xr = cs.alloc(
                    || "preimage xr",
                    || xr_val.ok_or(SynthesisError::AssignmentMissing),
                )?;

                for j in 0..MIMC_ROUNDS {
                    // xL, xR := xR + (xL + Ci)^3, xL

                    // tmp = (xL + Ci)^2
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

                    // new_xL = xR + (xL + Ci)^3
                    // new_xL = xR + tmp * (xL + Ci)
                    // new_xL - xR = tmp * (xL + Ci)
                    let new_xl_val = xl_val.map(|mut e| {
                        e.add_assign(&self.constants[j]);
                        e.mul_assign(&tmp_val.unwrap());
                        e.add_assign(&xr_val.unwrap());
                        e
                    });
                    let new_xl = if j == (MIMC_ROUNDS - 1) {
                        // This is the last round, xL is our image and so
                        // we allocate a public input.
                        cs.alloc_input(
                            || "image",
                            || new_xl_val.ok_or(SynthesisError::AssignmentMissing),
                        )?
                    } else {
                        cs.alloc(
                            || "new_xl",
                            || new_xl_val.ok_or(SynthesisError::AssignmentMissing),
                        )?
                    };

                    cs.enforce(
                        || "new_xL = xR + (xL + Ci)^3",
                        |lc| lc + tmp,
                        |lc| lc + xl + (self.constants[j], CS::one()),
                        |lc| lc + new_xl - xr,
                    );

                    // xR = xL
                    xr = xl;
                    xr_val = xl_val;

                    // xL = new_xL
                    xl = new_xl;
                    xl_val = new_xl_val;
                }
            }
        }

        Ok(())
    }
}