use std::ops::{Mul, MulAssign, AddAssign, SubAssign, Add, Sub};

use ark_bls12_381::Fr;
use ark_ff::{Field, One};
use ark_relations::{
    lc,
    r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError, Variable},
};
pub const T: usize = 3;

// p = m - 1
// p1_bits 为 (p-1)/2 的二进制展开
// p2_bits 为 (p+1)/4 的二进制展开
pub struct SlothInvDemo {
    pub y: Option<Fr>,
    pub p: Option<Fr>,
}

pub fn fmod(x: Fr, m: Fr) -> Fr {
    x - x.mul(m.inverse().unwrap()).mul(m)
}

impl ConstraintSynthesizer<Fr> for SlothInvDemo {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        let mut res_val = self.y;
        let mut res = cs.new_input_variable(|| res_val.ok_or(SynthesisError::AssignmentMissing))?;

        for _ in 0..T {
            let mut x_val;
            if res_val == None {
                x_val = None;
            }
            else if fmod(res_val.unwrap(), Fr::one().add(Fr::one())) == Fr::one() {
                x_val = Some(fmod(res_val.unwrap().add(Fr::one()), self.p.unwrap()));
            }
            else {
                x_val = Some(fmod(res_val.unwrap().sub(Fr::one()), self.p.unwrap()));
            }
            let mut x = cs.new_witness_variable(|| x_val.ok_or(SynthesisError::AssignmentMissing))?;
    
            // if x_val == None {
            //     res_val = None;
            // }
            // else if fmod(x_val.unwrap(), Fr::one().add(Fr::one())) == Fr::one() {
            //     let xn_val = 
            // }
        }

        Ok(())
    }
}