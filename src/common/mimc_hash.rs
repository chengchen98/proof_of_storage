use bls12_381::Scalar;

use crate::common::{mimc::mimc, data::padding};

pub const MIMC_ROUNDS: usize = 322;

pub fn mimc_hash(input: &Vec<u8>, constants: &Vec<Scalar>) -> Vec<u8> {
    //! Provide a method to compute mimc hash.
    //! 
    //! input: 64n
    //! 
    //! output: 32n
    let input = padding(&input, 64);
    let mut output = vec![];

    let mut buf_xl = [0u8; 32];
    let mut buf_xr = [0u8; 32];
    for i in (0..input.len()).step_by(64) {
        buf_xl.copy_from_slice(&input[i .. i + 32]);
        buf_xr.copy_from_slice(&input[i + 32 .. i + 64]);
        let xl = Scalar::from_bytes(&buf_xl).unwrap();
        let xr = Scalar::from_bytes(&buf_xr).unwrap();
        let image = mimc(xl, xr, &constants);

        output.append(&mut image.to_bytes().to_vec());
    }

    output
}

