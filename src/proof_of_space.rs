use std::io::BufReader;
use std::io::prelude::*;
use std::fs::File;
use std::collections::HashMap;

use bls12_381::{Scalar, Bls12};
use ff::PrimeField;
use rand::{thread_rng, Rng};
 
// We're going to use the Groth16 proving system.
use bellman::groth16::{
    create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
    Proof, PreparedVerifyingKey,
};

use crate::equal::EqualDemo;
use crate::mimc::{mimc, MiMCDemo};
use crate::convert::{s_to_bits, hex_to_s};

pub const INPUT_SIZE: usize = 256;

pub const Y_DIR: &str = r"src\y_collect.txt";

pub fn create_pos(n: usize, key: Scalar, m: Scalar, constants: &Vec<Scalar>) -> std::io::Result<()> {
    let mut file = File::create(Y_DIR).expect("Create file failed...");
    for i in 0..n {
        let y = mimc(Scalar::from_str_vartime(&i.to_string()).unwrap().add(&key), m, &constants).to_string();
        file.write_all(&y.as_bytes()[2..]).expect("Write failed!");
        file.write_all("\n".as_bytes()).expect("Write failed!");
    }
    Ok(())
}

pub fn create_challenge(count: usize, difficulty: usize) -> Vec<String> {
    let mut rng = thread_rng();
    let mut challenges = vec![];
    const CHARSET: &[u8] = b"0123456789abcdef";

    for _ in 0..count {
        let challenge: String = (0..difficulty)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

        challenges.push(challenge);
    }
    challenges
}

pub fn prepare_hashmap(difficulty: usize) -> HashMap<String, usize> {
    let mut y_x_map = HashMap::new();
    let mut x = 0;

    let file = File::open(Y_DIR).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        y_x_map.insert(line[line.len() - difficulty ..].to_string(), x);
        x += 1;
    }
    y_x_map
}

pub fn response_1(challenge: &Vec<String>, y_x_map: HashMap<String, usize>, samples: usize) -> (Vec<usize>, Vec<usize>) {
    let mut x_response  = vec![];
    let mut index_response = vec![];

    for i in 0..challenge.len() {
        let key = &challenge[i][..];

        if y_x_map.contains_key(key) {
            let x = y_x_map.get(key).unwrap().clone();
            
            x_response.push(x);
            index_response.push(i);

            if x_response.len() == samples {
                return (x_response, index_response);
            }
        }
    }
    (x_response, index_response)
}

pub fn response_2(x_response: &Vec<usize>, key: Scalar, m: Scalar, constants: &Vec<Scalar>, index_response: &Vec<usize>, challenge: &Vec<String>, diffculty: usize) 
-> (PreparedVerifyingKey<Bls12>, Vec<Proof<Bls12>>, PreparedVerifyingKey<Bls12>, Vec<Proof<Bls12>>, Vec<Scalar>, Vec<Scalar>) {
    let mut rng = thread_rng();
    let samples = x_response.len();

    let params = {
        let c = MiMCDemo {
            xl: None,
            xr: None,
            constants: &constants,
        };

        generate_random_parameters::<Bls12, _, _>(c, &mut rng).unwrap()
    };

    let mimc_pvk = prepare_verifying_key(&params.vk);
    let mut mimc_proof_collect = vec![];
    let mut y_collect = vec![];
    {
        for i in 0..samples {
            let xi = Scalar::from_str_vartime(&x_response[i].to_string()).unwrap().add(&key);
            let y = mimc(xi, m, &constants);
            y_collect.push(y);
    
            let c = MiMCDemo {
                xl: Some(xi),
                xr: Some(m),
                constants: &constants,
            };
    
            let proof = create_random_proof(c, &params, &mut rng).unwrap();
            mimc_proof_collect.push(proof);
        }
    }

    let params = {
        let c = EqualDemo {
            x1: None,
            x2: None,
            x_bit: [None; INPUT_SIZE],
            difficulty: diffculty * 4
        };

        generate_random_parameters::<Bls12, _, _>(c, &mut rng).unwrap()
    };
    
    let equal_pvk = prepare_verifying_key(&params.vk);
    let mut x_collect = vec![];
    let mut equal_proof_collect = vec![];
    {
        for i in 0..x_response.len() {
            let x1 = &challenge[index_response[i]][..];
            let x1 = hex_to_s(&x1);
            x_collect.push(x1);

            let x2 = y_collect[i];
            let x_bit = s_to_bits(x2, INPUT_SIZE);

            let mut new_x_bit: [Option<u8>; INPUT_SIZE] = [None; INPUT_SIZE];
            for i in 0..INPUT_SIZE {
                new_x_bit[i] = Some(x_bit[i]);
            }
    
            let c = EqualDemo {
                x1: Some(x1),
                x2: Some(x2),
                x_bit: new_x_bit,
                difficulty: diffculty * 4
            };
    
            let proof = create_random_proof(c, &params, &mut rng).unwrap();
            equal_proof_collect.push(proof);
        }
    }

    (mimc_pvk, mimc_proof_collect, equal_pvk, equal_proof_collect, x_collect, y_collect)
}

// pub fn verify_1(x_response: &Vec<Scalar>, y_response: &Vec<Scalar>, key: Scalar, m_bit: [Scalar; INPUT_SIZE]) -> std::io::Result<()> {
//     for i in 0..x_response.len() {
//         let x = x_response[i].add(&key);
//         let y = y_response[i];
//         assert_eq!(y, pow(x, m_bit));
//     }
//     Ok(())
// }

pub fn verify_2(pow_pvk: PreparedVerifyingKey<Bls12>, pow_proof_collect: Vec<Proof<Bls12>>, equal_pvk: PreparedVerifyingKey<Bls12>, equal_proof_collect: Vec<Proof<Bls12>>, x_collect: Vec<Scalar>, y_collect: Vec<Scalar>) -> std::io::Result<()> {
    for i in 0..pow_proof_collect.len() {
        assert!(verify_proof(&pow_pvk, &pow_proof_collect[i], &[y_collect[i]]).is_ok());
    }

    for i in 0..equal_proof_collect.len() {
        assert!(verify_proof(&equal_pvk, &equal_proof_collect[i], &[x_collect[i], y_collect[i]]).is_ok());
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::time::Instant;

    use ff::Field;

    use crate::mimc::MIMC_ROUNDS;

    use super::*;

    #[test]
    fn test_proof() {
        let mut rng = thread_rng();
        const DIFFICULTY: usize = 2;

        // Prepare constants for mimc.
        let constants = (0..MIMC_ROUNDS)
        .map(|_| Scalar::random(&mut rng))
        .collect::<Vec<_>>();

        let n: usize = 1024;
        let key: Scalar = Scalar::random(&mut rng);
        let m: Scalar = Scalar::random(&mut rng);

        // Create proof of space by computing mimc.
        let start = Instant::now();
        create_pos(n, key, m, &constants).unwrap();
        println!("Create pos: {:?}", start.elapsed());

        // Set difficulty(4 bits) for challenge and response.
        // y1(challenge) = y2(result of mimc)[0..difficulty].
        println!("Set difficulty: {:?}", DIFFICULTY);

        // Create challenges.
        const CHALLENGE_COUNT: usize = 20;
        let start = Instant::now();
        let challenge = create_challenge(CHALLENGE_COUNT, DIFFICULTY);
        println!("{:?}", challenge);
        println!("Create challenge: {:?}", start.elapsed());

        // Response 1
        // (1) choose samples of challenges
        // (2) a. response the hash of the x collect of samples; b. the index collection of samples
        const RESPONSE_COUNT: usize = 10;
        let start = Instant::now();
        let y_x_map = prepare_hashmap(DIFFICULTY);
        let (x_response,index_response) = response_1(&challenge, y_x_map, RESPONSE_COUNT);
        println!("Response 1: {:?}", start.elapsed());
        
        println!("Success rate: {:?} / {:?}", index_response.len(), RESPONSE_COUNT);

        // let start = Instant::now();
        // verify_1(&x_response, &y_response, key, m).unwrap();
        // println!("Verify 1: {:?}", start.elapsed());

        let start = Instant::now();
        let (mimc_pvk, mimc_proof_collect, equal_pvk, equal_proof_collect, x_collect, y_collect) 
        = response_2(&x_response, key, m,  &constants, &index_response, &challenge, DIFFICULTY);
        println!("Response 2: {:?}", start.elapsed());
        
        let start = Instant::now();
        verify_2(mimc_pvk, mimc_proof_collect, equal_pvk, equal_proof_collect, x_collect, y_collect).unwrap();
        println!("Verify 2: {:?}", start.elapsed());
    }
}