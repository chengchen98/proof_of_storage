use std::io::BufReader;
use std::io::prelude::*;
use std::fs::File;
use std::collections::HashMap;
use std::ops::Add;
use std::str::FromStr;

use ark_bls12_381::{Fr, Bls12_381};
use ark_ff::BigInteger;
use ark_ff::BigInteger256;
use ark_ff::One;
use ark_ff::Zero;
use ark_std::{rand::Rng, test_rng};
 
// We're going to use the Groth-Maller17 proving system.
use ark_groth16::{
    Proof, PreparedVerifyingKey,
    create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
};

use crate::common::mimc_df::mimc_df;
use crate::circuit::pos_circuit::PosDemo;
use crate::common::convert::{bits_to_fr, fr_to_bits};
use crate::common::mimc_hash::multi_mimc7_hash;

const DATA_DIR: &str = r"src\proof_of_space\pos_data.txt";
pub const MIMC_DF_ROUNDS: usize = 322;
pub const MIMC_HASH_ROUNDS: usize = 10;

pub fn create_pos(n: usize, key: Fr, m: Fr, df_constants: &Vec<Fr>) -> std::io::Result<()> {
    //! Prover: create proof-of-space by computing mimc function n times using incremental x.
    //! 
    //! y = mimc_df(key + x, m)
    let mut file = File::create(DATA_DIR).expect("Create file failed...");

    // Write y by line
    for x in 0..n {
        let y = mimc_df(Fr::from_str(&x.to_string()).unwrap().add(&key), m, &df_constants); // compute mimc
        let y = fr_to_bits(y);
        file.write_all(&y.as_bytes()).expect("Write failed!");
        file.write_all("\n".as_bytes()).expect("Write failed!");
    }
    Ok(())
}

pub fn create_challenge(challenge_count: usize, difficulty: usize) -> Vec<String> {
    //! Verifier: create CHALLENGE_COUNT challenges(hex string) by random.
    //! 
    //! The length of single challenge is DIFFICULTY.
    let mut rng = test_rng();
    let mut challenges = vec![];
    const CHARSET: &[u8] = b"01";

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
        y_x_map.insert(line[.. difficulty].to_string(), x);
        x += 1;
    }
    y_x_map
}

pub fn response_1(challenges: &Vec<String>, y_x_map: HashMap<String, usize>, key: Fr, hash_constants: &Vec<Fr>, response_count: usize) -> (Vec<Fr>, Vec<usize>, Fr) {
    //! Prover: choose samples of challenges
    //! 
    //! reseponse: (1) the index collection of samples; (2) the hash of the x_vec of samples.
    //! 
    //! Prover saves the x chosen by himself.
    let mut x_response  = vec![];
    let mut index_response = vec![];
    let x_hash_response;

    for i in 0..challenges.len() {
        let key = &challenges[i][..];

        if y_x_map.contains_key(key) {
            let x = y_x_map.get(key).unwrap().clone();

            x_response.push(Fr::from_str(&x.to_string()).unwrap());
            index_response.push(i);

            // Only return RESPONSE_COUNT of challenges if enough.
            if x_response.len() == response_count {
                break;
            }
        }
    }

    // input of mimc hash is x vector
    x_hash_response = multi_mimc7_hash(&x_response, key, &hash_constants);
    (x_response, index_response, x_hash_response)
}

pub fn response_2(x_response: &Vec<Fr>, key: Fr, m: Fr, df_constants: &Vec<Fr>, hash_constants: &Vec<Fr>, index_response: &Vec<usize>, challenges: &Vec<String>, response_count: usize, difficulty: usize) 
-> (PreparedVerifyingKey<Bls12_381>, Proof<Bls12_381>) {
    //! Prover: create zk-proof
    //! 
    //! (1) y = mimc_df(key + x, m); (2) yn = y\[0..n\]; (3) x_hash = mimc_hash(xi + xj + ..)
    let mut rng = test_rng();

    let mut x_collect = vec![];
    let mut y_bits_collect = vec![];
    let mut yn_collect = vec![];
    for i in 0..response_count {
        x_collect.push(Some(x_response[i]));

        let y = mimc_df(x_response[i].add(&key), m, &df_constants);
        let y: BigInteger256 = y.into();
        let y_bits = y.to_bits_le();
        let mut buf_bits = [None; 256];
        for j in 0..y_bits.len() {
            if y_bits[j] == true {
                buf_bits[j] = Some(Fr::one());
            }
            else {
                buf_bits[j] = Some(Fr::zero());
            }
        }
        y_bits_collect.push(Some(buf_bits)); // compute mimc

        let yn = &challenges[index_response[i]]; // get one response
        let yn = bits_to_fr(&yn);
        yn_collect.push(Some(yn));
    }

    // Create parameters for our circuit.
    let params = {
        let c = PosDemo::<Fr> {
            key: Some(key),
            x: &x_collect,
            m: Some(m),
            df_constants: &df_constants,
            y_bits: &y_bits_collect,
            yn: &yn_collect,
            difficulty: difficulty,
            hash_constants: &hash_constants
        };

        generate_random_parameters::<Bls12_381, _, _>(c, &mut rng).unwrap()
    };

    let pvk = prepare_verifying_key(&params.vk);

    let c = PosDemo {
        key: Some(key),
        x: &x_collect,
        m: Some(m),
        df_constants: &df_constants,
        y_bits: &y_bits_collect,
        yn: &yn_collect,
        difficulty: difficulty,
        hash_constants: &hash_constants
    };

    let proof = create_random_proof(c, &params, &mut rng).unwrap();
    (pvk, proof)
}

pub fn verify(pvk: PreparedVerifyingKey<Bls12_381>, proof: Proof<Bls12_381>, key: Fr, m: Fr, challenges: &Vec<String>, index_response: &Vec<usize>, x_hash_response: Fr, response_count: usize) {
    //! Verifier: verify the zk proof by 
    //! 
    //! (1) key; (2) m; (3) challenges and indexs; (4) x_hash
    let mut verify_input = vec![];
    verify_input.push(key);
    verify_input.push(m);

    for i in 0.. response_count {
        let c = &challenges[index_response[i]];
        verify_input.push(bits_to_fr(&c));
    }
    verify_input.push(x_hash_response);

    assert!(verify_proof(&pvk, &proof, &verify_input).is_ok());
}

#[test]
fn test_pos() {
    use std::time::Instant;

    let rng = &mut test_rng();
    
    // pos data len: n = 64n bytes
    const N: usize = 1024;
    // difficulty: bits
    const DIFFICULTY: usize = 8;
    // the count of challenges verifier sends
    const CHALLENGE_COUNT: usize = 20;
    // the count of responses prover responses
    const RESPONSE_COUNT: usize = 10;

    // Prepare shared constants for mimc.
    let df_constants: Vec<Fr> = (0..MIMC_DF_ROUNDS)
    .map(|_| rng.gen())
    .collect::<Vec<_>>();

    let hash_constants: Vec<Fr> = (0..MIMC_HASH_ROUNDS)
    .map(|_| rng.gen())
    .collect::<Vec<_>>();

    // If n = 1, the data size is 257B.
    let key: Fr = rng.gen();
    let m: Fr = rng.gen();

    // Create the proof of space by computing mimc.
    let start = Instant::now();
    create_pos(N, key, m, &df_constants).unwrap();
    println!("Create pos: {:?}", start.elapsed());

    // Set difficulty(bit) for challenge and response.
    // y1(challenge) = y2(result of mimc)[0..difficulty].
    println!("Set difficulty: {:?} bits", DIFFICULTY);

    // Create challenges.
    let start = Instant::now();
    let challenges = create_challenge(CHALLENGE_COUNT, DIFFICULTY);
    println!("Create challenge: {:?}", start.elapsed());

    // Response 1
    let start = Instant::now();
    let y_x_map = prepare_hashmap(DIFFICULTY);
    let (x_response, index_response, x_hash_response) = response_1(&challenges, y_x_map, key, &hash_constants, RESPONSE_COUNT);
    assert_eq!(x_response.len(), RESPONSE_COUNT);
    println!("Response 1: {:?}", start.elapsed());

    // Compute the success rate
    println!("Success rate: {:?} / {:?}", index_response.len(), RESPONSE_COUNT);

    // Response 2
    let start = Instant::now();
    let (pvk, proof) = response_2(&x_response, key, m,  &df_constants,  &hash_constants, &index_response, &challenges, RESPONSE_COUNT, DIFFICULTY);
    println!("Response 2: {:?}", start.elapsed());
    
    // Verify
    let start = Instant::now();
    verify(pvk, proof, key, m, &challenges, &index_response, x_hash_response, RESPONSE_COUNT);
    println!("Verify: {:?}", start.elapsed());
}