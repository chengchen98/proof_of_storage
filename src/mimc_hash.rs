use bls12_381::Scalar;

use crate::mimc::mimc;

pub fn mimc_hash_short(input: Vec<u8>, constants: &Vec<Scalar>) -> Scalar {
    let mut buf_xr = [0u8; 32];
    let mut output = Scalar::zero();
    for i in (0..input.len()).step_by(32) {
        buf_xr.copy_from_slice(&input[i .. i + 32]);
        let xr = Scalar::from_bytes(&buf_xr).unwrap();
        output = mimc(output, xr, &constants);
    }
    output
}

pub fn mimc_hash_long(input: Vec<u8>, constants: &Vec<Scalar>) -> String {
    let mut buf_xl = [0u8; 32];
    let mut buf_xr = [0u8; 32];
    let mut output = String::new();
    for i in (0..input.len()).step_by(64) {
        buf_xl.copy_from_slice(&input[i .. i + 32]);
        buf_xr.copy_from_slice(&input[i + 32 .. i + 64]);
        let xl = Scalar::from_bytes(&buf_xl).unwrap();
        let xr = Scalar::from_bytes(&buf_xr).unwrap();
        let image = mimc(xl, xr, &constants);
        output.push_str(&image.to_string()[2..]);
    }
    output
}


#[cfg(test)] 
mod test {
    use ff::Field;
    use rand::thread_rng;

    use super::*;
    
    pub const MIMC_ROUNDS: usize = 322;

    #[test]
    fn test_mimc_hash_short() {
    
        let input = vec![1; 64];
        let mut rng = thread_rng();
        
        let constants = (0..MIMC_ROUNDS)
            .map(|_| Scalar::random(&mut rng))
            .collect::<Vec<_>>();
    
        let output = mimc_hash_short(input, &constants);
        println!("mimc hash: {:?}", output);
    }

    #[test]
    fn test_mimc_hash_long() {
    
        let input = vec![1; 128];
        let mut rng = thread_rng();
        
        let constants = (0..MIMC_ROUNDS)
            .map(|_| Scalar::random(&mut rng))
            .collect::<Vec<_>>();
    
        let output = mimc_hash_long(input, &constants);
        println!("mimc hash: {:?}", output);
    }
}
    
