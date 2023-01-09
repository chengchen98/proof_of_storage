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

use crate::convert::hex_to_s;
use crate::mimc::mimc;
use crate::circuit::pos_circuit::PosDemo;

pub const DATA_DIR: &str = r"src\pos_data.txt";
pub const MIMC_ROUNDS: usize = 322;

pub fn create_pos(n: usize, key: Scalar, m: Scalar, constants: &Vec<Scalar>) -> std::io::Result<()> {
    //! Prover: create proof-of-space by computing mimc function n times using incremental x.
    //! 
    //! y = mimc(key + x, m)
    let mut file = File::create(DATA_DIR).expect("Create file failed...");

    // Write y by line
    for x in 0..n {
        let y = mimc(Scalar::from_str_vartime(&x.to_string()).unwrap().add(&key), m, &constants).to_string();
        file.write_all(&y.as_bytes()[2..]).expect("Write failed!");
        file.write_all("\n".as_bytes()).expect("Write failed!");
    }
    Ok(())
}

pub fn create_challenge(challenge_count: usize, difficulty: usize) -> Vec<String> {
    //! Verifier: create CHALLENGE_COUNT challenges(hex string) by random.
    //! 
    //! The length of single challenge is DIFFICULTY.
    let mut rng = thread_rng();
    let mut challenges = vec![];
    const CHARSET: &[u8] = b"0123456789abcdef";

    for _ in 0..challenge_count {
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
    //! Prover: prepare hashmap to responsing for challenge.
    //! 
    //! k: y\[0..n\]; v: x
    let mut y_x_map = HashMap::new();
    let mut x = 0;

    let file = File::open(DATA_DIR).unwrap();
    let reader = BufReader::new(file);

    // Read file by line
    for line in reader.lines() {
        let line = line.unwrap();
        y_x_map.insert(line[line.len() - difficulty ..].to_string(), x);
        x += 1;
    }
    y_x_map
}

pub fn response_1(challenges: &Vec<String>, y_x_map: HashMap<String, usize>, constants: &Vec<Scalar>, response_count: usize) -> (Vec<usize>, Vec<usize>, Scalar) {
    //! Prover: choose samples of challenges
    //! 
    //! reseponse: (1) the index collection of samples; (2) the hash of the x_sum of samples.
    //! 
    //! Prover saves the x chosen by himself.
    let mut x_collect  = vec![];
    let mut index_response = vec![];
    let mut x_sum = 0;
    let x_hash_response;

    for i in 0..challenges.len() {
        let key = &challenges[i][..];

        if y_x_map.contains_key(key) {
            let x = y_x_map.get(key).unwrap().clone();

            x_collect.push(x);
            index_response.push(i);
            x_sum += x;

            // Only return RESPONSE_COUNT of challenges if enough.
            if x_collect.len() == response_count {
                break;
            }
        }
    }
    x_hash_response = mimc(Scalar::from_str_vartime(&x_sum.to_string()).unwrap(), Scalar::from_str_vartime(&x_sum.to_string()).unwrap(), &constants);
    (x_collect, index_response, x_hash_response)
}

pub fn response_2(x_collect: &Vec<usize>, key: Scalar, m: Scalar, constants: &Vec<Scalar>, index_response: &Vec<usize>, challenges: &Vec<String>, response_count: usize, difficulty: usize) 
-> (PreparedVerifyingKey<Bls12>, Proof<Bls12>) {
    //! Prover: create zk-proof
    //! 
    //! (1) y = mimc(key + x, m); (2) yn = y\[0..n\]; (3) x_hash = mimc(xi + xj + ..)
    let mut rng = thread_rng();

    let mut yn = vec![];
    for i in 0..response_count {
        yn.push(Some(hex_to_s(&challenges[index_response[i]])));
    }

    // Create parameters for our circuit.
    let params = {
        let c = PosDemo {
            key: Some(key),
            x: &x_collect,
            m: Some(m),
            constants: &constants,
            yn: &yn,
            difficulty: difficulty * 4
        };

        generate_random_parameters::<Bls12, _, _>(c, &mut rng).unwrap()
    };
    
    let pvk = prepare_verifying_key(&params.vk);

    let c = PosDemo {
        key: Some(key),
        x: &x_collect,
        m: Some(m),
        constants: &constants,
        yn: &yn,
        difficulty: difficulty * 4
    };

    let proof = create_random_proof(c, &params, &mut rng).unwrap();
    (pvk, proof)
}

pub fn verify(pvk: PreparedVerifyingKey<Bls12>, proof: Proof<Bls12>, key: Scalar, m: Scalar, challenges: &Vec<String>, index_response: &Vec<usize>, x_hash: Scalar, response_count: usize) {
    //! Verifier: verify the zk proof by 
    //! 
    //! (1) key; (2) m; (3) challenges and indexs; (4) x_hash
    let mut verify_input = vec![];
    verify_input.push(key);
    verify_input.push(m);

    for i in 0.. response_count {
        verify_input.push(hex_to_s(&challenges[index_response[i]]));
    }
    verify_input.push(x_hash);

    assert!(verify_proof(&pvk, &proof, &verify_input).is_ok());
}

#[cfg(test)]
mod test {
    use std::time::Instant;
    use ff::Field;

    use super::*;

    #[test]
    fn test_proof() {
        let mut rng = thread_rng();
        
        // pos data len: n = 64n bytes
        const N: usize = 1024;
        // difficulty: n = 4n bits
        const DIFFICULTY: usize = 2;
        // the count of challenges verifier sends
        const CHALLENGE_COUNT: usize = 20;
        // the count of responses prover responses
        const RESPONSE_COUNT: usize = 10;

        // Prepare shared constants for mimc.
        let constants = (0..MIMC_ROUNDS)
        .map(|_| Scalar::random(&mut rng))
        .collect::<Vec<_>>();

        // If n = 1, the data size is 65B.
        let key: Scalar = Scalar::random(&mut rng);
        let m: Scalar = Scalar::random(&mut rng);

        // Create the proof of space by computing mimc.
        let start = Instant::now();
        create_pos(N, key, m, &constants).unwrap();
        println!("Create pos: {:?}", start.elapsed());

        // Set difficulty(4 bits) for challenge and response.
        // y1(challenge) = y2(result of mimc)[0..difficulty].
        println!("Set difficulty: {:?} bits", DIFFICULTY * 4);

        // Create challenges.
        let start = Instant::now();
        let challenges = create_challenge(CHALLENGE_COUNT, DIFFICULTY);
        println!("Create challenge: {:?}", start.elapsed());

        // Response 1
        let start = Instant::now();
        let y_x_map = prepare_hashmap(DIFFICULTY);
        let (x_response,index_response, x_hash) = response_1(&challenges, y_x_map, &constants, RESPONSE_COUNT);
        assert_eq!(x_response.len(), RESPONSE_COUNT);
        println!("Response 1: {:?}", start.elapsed());
        
        // Compute the success rate
        println!("Success rate: {:?} / {:?}", index_response.len(), RESPONSE_COUNT);

        // Response 2
        let start = Instant::now();
        let (pvk, proof) = response_2(&x_response, key, m,  &constants, &index_response, &challenges, RESPONSE_COUNT, DIFFICULTY);
        println!("Response 2: {:?}", start.elapsed());
        
        // Verify
        let start = Instant::now();
        verify(pvk, proof, key, m, &challenges, &index_response, x_hash, RESPONSE_COUNT);
        println!("Verify: {:?}", start.elapsed());
    }
}