use rand::Rng;
use std::str::FromStr;
use ark_bls12_381::{Fr, Bls12_381};

use ark_groth16::{
    Proof, PreparedVerifyingKey,
    verify_proof,
};

use super::convert::bits_to_usize;

pub fn create_challenges(n: usize, end: usize) -> Vec<usize> {
    //! 验证者随机生成挑战
    let mut rng = rand::thread_rng();
    let mut challenges = vec![];
    const CHOICE: &[bool] = &[false, true];

    // 生成n个N比特的挑战
    for _ in 0..n {
        let challenge: Vec<bool> = (0..end)
        .map(|_| { 
            let idx = rng.gen_range(0..2);
            CHOICE[idx]
         }).collect();
        challenges.push(bits_to_usize(&challenge));
    }
    challenges
}

pub fn verify(pvk: PreparedVerifyingKey<Bls12_381>, proof: Proof<Bls12_381>, key: Fr, m: Fr, challenges: &Vec<usize>, idx_response: &Vec<usize>, x_hash: Fr) {
    //! 验证零知识证明
    let mut verify_input = vec![];
    verify_input.push(key);
    verify_input.push(m);

    for i in 0.. idx_response.len() {
        let yn = challenges[idx_response[i]];
        verify_input.push(Fr::from_str(&yn.to_string()).unwrap());
    }

    verify_input.push(x_hash);

    assert!(verify_proof(&pvk, &proof, &verify_input).is_ok());
}
