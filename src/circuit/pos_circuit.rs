use std::ops::{AddAssign, MulAssign, Add};

use ark_bls12_381::Fr;
use ark_ff::{Field, Zero, One, BigInteger256, BigInteger};
use ark_relations::{
    lc,
    r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError, Variable},
};

const MIMC5_DF_ROUNDS: usize = 322;
const MIMC5_HASH_ROUNDS: usize = 110;

// 延迟函数计算结果所占比特数
const Y_SIZE: usize = 256;
// 等于空间声明的N
const YN_SIZE: usize = 20;

// 需要证明的事情：

// 1.证明延迟函数计算的正确性：y = mimc_vde(key + x, m)

// 2.证明y的二进制展开每一位都是0或者1（Y_SIZE位）
// 3.证明y的二进制展开的正确性：y_bits = y

// 4.证明yn的二进制展开每一位都是0或者1（YN_SIZE位）
// 5.证明yn的二进制展开的正确性：yn_bits = yn

// 6.证明yn_bits等于y_bits的前n位：yn_bits = y_bits[0..YN_SIZE]

// 7.证明x_hash是x的哈希结果：x_hash = hash(x[0], x[1], .. , x[n-1])

pub struct PosDemo<'a> {
    pub key: Option<Fr>, // 验证者input1
    pub x: &'a [Option<Fr>],
    pub m: Option<Fr>, // 验证者input2
    pub df_constants: &'a [Fr],

    pub yn: &'a [Option<Fr>], // 验证者input3

    pub x_hash: Option<Fr>, // 验证者input4
    pub hash_constants: &'a [Fr],
}

impl<'a> ConstraintSynthesizer<Fr> for PosDemo<'a> {
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        assert_eq!(self.df_constants.len(), MIMC5_DF_ROUNDS);
        assert_eq!(self.hash_constants.len(), MIMC5_HASH_ROUNDS);

        let key_val = self.key;
        let key = cs.new_input_variable(|| key_val.ok_or(SynthesisError::AssignmentMissing))?;

        let m_val = self.m;
        cs.new_input_variable(|| m_val.ok_or(SynthesisError::AssignmentMissing))?;
    
        for i in 0..self.x.len() {
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

            // 1.证明延迟函数计算的正确性：y = mimc_vde(key + x, m)
            for j in 0..MIMC5_DF_ROUNDS {
                let tmp1_val = xl_val.map(|mut e| {
                    e.add_assign(&self.df_constants[j]);
                    e.square_in_place();
                    e
                });
                let tmp1 = cs.new_witness_variable(|| tmp1_val.ok_or(SynthesisError::AssignmentMissing))?;
                cs.enforce_constraint(
                    lc!() + xl + (self.df_constants[j], Variable::One),
                    lc!() + xl + (self.df_constants[j], Variable::One),
                    lc!() + tmp1,
                )?;
                
                let tmp2_val = tmp1_val.map(|mut e| {
                    e.square_in_place();
                    e
                });
                let tmp2 = cs.new_witness_variable(|| tmp2_val.ok_or(SynthesisError::AssignmentMissing))?;
                cs.enforce_constraint(
                    lc!() + tmp1,
                    lc!() + tmp1,
                    lc!() + tmp2,
                )?;
    
                let new_xl_val = xl_val.map(|mut e| {
                    e.add_assign(&self.df_constants[j]);
                    e.mul_assign(&tmp2_val.unwrap());
                    e.add_assign(&xr_val.unwrap());
                    e
                });
                let new_xl = cs.new_witness_variable(|| new_xl_val.ok_or(SynthesisError::AssignmentMissing))?;
                cs.enforce_constraint(
                    lc!() + tmp2,
                    lc!() + xl + (self.df_constants[j], Variable::One),
                    lc!() + new_xl - xr,
                )?;

                xr = xl;
                xr_val = xl_val;
                xl = new_xl;
                xl_val = new_xl_val;
    
                // 3.证明y的二进制展开的正确性：y_bits = y
                if j == MIMC5_DF_ROUNDS - 1 {
                    let y_bits_val = {
                        if xl_val == None {
                            vec![None; Y_SIZE]
                        }
                        else {
                            let tmp: BigInteger256 = xl_val.unwrap().into();
                            let tmp = tmp.to_bits_le();
                            (0..Y_SIZE).map(|i| Some(tmp[i])).collect()
                        }
                    };
                    
                    let mut y_val = Some(Fr::zero());
                    let mut y = cs.new_witness_variable(|| y_val.ok_or(SynthesisError::AssignmentMissing))?;

                    let mut two_val = Some(Fr::one());
                    let mut two = cs.new_witness_variable(|| two_val.ok_or(SynthesisError::AssignmentMissing))?;

                    for k in 0..y_bits_val.len() {
                        // 2.证明y的二进制展开每一位都是0或者1（Y_SIZE位）
                        let bit_val = {
                            if y_bits_val[k] == Some(true) {
                                Some(Fr::one())
                            }
                            else if y_bits_val[k] == Some(false){
                                Some(Fr::zero())
                            }
                            else {
                                None
                            }
                        };
                        let bit = cs.new_witness_variable(|| bit_val.ok_or(SynthesisError::AssignmentMissing))?;
                        cs.enforce_constraint(
                            lc!() + bit,
                            lc!() + bit,
                            lc!() + bit
                        )?;
                        
                        // tmp1 = yi * 2^i
                        let tmp_val = bit_val.map(|mut e| {
                            e.mul_assign(&two_val.unwrap());
                            e.add_assign(&y_val.unwrap());
                            e
                        });
                        let tmp = cs.new_witness_variable(|| tmp_val.ok_or(SynthesisError::AssignmentMissing))?;
                        cs.enforce_constraint(
                            lc!() + bit,
                            lc!() + two,
                            lc!() + tmp - y
                        )?;
                        
                        // newtwo = two * 2
                        let newtwo_val = two_val.map(|mut e| {
                            e.mul_assign(Fr::one().add(Fr::one()));
                            e
                        });
                        let newtwo = cs.new_witness_variable(|| newtwo_val.ok_or(SynthesisError::AssignmentMissing))?;
                        cs.enforce_constraint(
                            lc!() + two,
                            lc!() + (Fr::one().add(Fr::one()), Variable::One),
                            lc!() + newtwo
                        )?;

                        two = newtwo;
                        two_val = newtwo_val;
                        y = tmp;
                        y_val = tmp_val;
                        
                        if k == y_bits_val.len() - 1 {
                            let yy_val = xl_val;
                            let yy = cs.new_witness_variable(|| yy_val.ok_or(SynthesisError::AssignmentMissing))?;
                            cs.enforce_constraint(
                                lc!() + y,
                                lc!() + Variable::One,
                                lc!() + yy
                            )?;
                        }

                        // 5.证明yn的二进制展开的正确性：yn_bits = yn
                        if k == YN_SIZE - 1 {
                            let yn_bits_val = {
                                if self.yn[i] == None {
                                    vec![None; YN_SIZE]
                                }
                                else {
                                    let tmp: BigInteger256 = self.yn[i].unwrap().into();
                                    let tmp = tmp.to_bits_le();
                                    (0..YN_SIZE).map(|i| Some(tmp[i])).collect()
                                }
                            };
                            
                            let mut yn_val = Some(Fr::zero());
                            let mut yn = cs.new_witness_variable(|| yn_val.ok_or(SynthesisError::AssignmentMissing))?;
                            
                            let mut two_val = Some(Fr::one());
                            let mut two = cs.new_witness_variable(|| two_val.ok_or(SynthesisError::AssignmentMissing))?;
    
                            for m in 0..yn_bits_val.len() {

                                // 4.证明yn的二进制展开每一位都是0或者1（YN_SIZE位）
                                let bit_val = {
                                    if yn_bits_val[m] == Some(true) {
                                        Some(Fr::one())
                                    }
                                    else if yn_bits_val[m] == Some(false){
                                        Some(Fr::zero())
                                    }
                                    else {
                                        None
                                    }
                                };
                                let bit = cs.new_witness_variable(|| bit_val.ok_or(SynthesisError::AssignmentMissing))?;
                                cs.enforce_constraint(
                                    lc!() + bit,
                                    lc!() + bit,
                                    lc!() + bit
                                )?;
                                
                                // tmp = yn[i] * 2^i
                                let tmp_val = bit_val.map(|mut e| {
                                    e.mul_assign(&two_val.unwrap());
                                    e.add_assign(&yn_val.unwrap());
                                    e
                                });
                                let tmp = cs.new_witness_variable(|| tmp_val.ok_or(SynthesisError::AssignmentMissing))?;
                                cs.enforce_constraint(
                                    lc!() + bit,
                                    lc!() + two,
                                    lc!() + tmp - yn
                                )?;
                            
                                let newtwo_val = two_val.map(|mut e| {
                                    e.mul_assign(Fr::one().add(Fr::one()));
                                    e
                                });
                                let newtwo = cs.new_witness_variable(|| newtwo_val.ok_or(SynthesisError::AssignmentMissing))?;
                                cs.enforce_constraint(
                                    lc!() + two,
                                    lc!() + (Fr::one().add(Fr::one()), Variable::One),
                                    lc!() + newtwo
                                )?;
    
                                two = newtwo;
                                two_val = newtwo_val;
                                yn = tmp;
                                yn_val = tmp_val;

                                if m == yn_bits_val.len() - 1 {
                                    let yyn_val = self.yn[i];
                                    let yyn = cs.new_input_variable(|| yyn_val.ok_or(SynthesisError::AssignmentMissing))?;
                                    cs.enforce_constraint(
                                        lc!() + yyn,
                                        lc!() + Variable::One,
                                        lc!() + yn
                                    )?;
                                    
                                    // 6.证明yn_bits等于y_bits的前n位：yn_bits = y_bits[0..YN_SIZE]
                                    cs.enforce_constraint(
                                        lc!() + yn,
                                        lc!() + Variable::One,
                                        lc!() + y
                                    )?;
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut res_val  = self.key;
        let mut res = cs.new_witness_variable(|| res_val.ok_or(SynthesisError::AssignmentMissing))?;

        // 7.证明x_hash是x的哈希结果：x_hash = hash(x[0], x[1], .. , x[n-1])
        for i in 0..self.x.len() {
            let x_in_val= self.x[i];
            let x_in = cs.new_witness_variable(|| x_in_val.ok_or(SynthesisError::AssignmentMissing))?;

            let key_val = res_val;
            let key = cs.new_witness_variable(|| key_val.ok_or(SynthesisError::AssignmentMissing))?;    
            
            let mut h_val = Some(Fr::zero());
            let mut h = cs.new_witness_variable(|| h_val.ok_or(SynthesisError::AssignmentMissing))?;

            // 计算rounds轮
            for j in 0..MIMC5_HASH_ROUNDS {
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
                
                // t5 = t4 * t
                let t5_val = t4_val.map(|mut e| {
                    e.mul_assign(&t_val.unwrap());
                    e
                });
                let t5 = cs.new_witness_variable(|| t5_val.ok_or(SynthesisError::AssignmentMissing))?;
                cs.enforce_constraint(
                    lc!() + t4,
                    lc!() + t,
                    lc!() + t5
                )?;

                h = t5;
                h_val = t5_val;
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