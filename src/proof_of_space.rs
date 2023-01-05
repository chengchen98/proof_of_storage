             use std::{fs::File, io::Write, ops::AddAssign};
use std::collections::HashMap;
 
// We're going to use the Groth16 proving system.
use bellman::groth16::{
    create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
    Proof, PreparedVerifyingKey,
};

use bls12_381::{Scalar, Bls12};
use ff::Field;
use rand::thread_rng;

use crate::equal::EqualDemo;
use crate::pow::{PowDemo, INPUT_SIZE, pow};
use crate::convert::{bits_to_s, s_to_bits};
use crate::data::Data;

pub const Y_DIR: &str = r"D:\graduation\code\proof_of_storage\src\y_collect.json";


pub fn create_pos(key: Scalar, m_bit: [Scalar; INPUT_SIZE], n: u32) -> std::io::Result<()> {
    let base: i32 = 2;
    let count: i32 = base.pow(n);
    let mut x = Scalar::zero();

    let mut file = File::create(Y_DIR).expect("Create file failed...");
    let mut y_collect = vec![];

    for _ in 0..count {
        let y = pow(x.add(&key), m_bit);
        let y = y.to_bytes().to_vec();
        y_collect.push(y);

       x.add_assign(&Scalar::one());
    }
    let y_collect = Data { content: y_collect };
    let y_collect = serde_json::to_string(&y_collect).unwrap();
    file.write_all(&y_collect.as_bytes()).expect("Write failed!");
    Ok(())
}

pub fn create_challenge(count: usize) -> Vec<Scalar> {
    let mut rng = thread_rng();
    let challenge = (0..count)
        .map(|_| Scalar::random(&mut rng))
        .collect::<Vec<_>>();
    challenge
}

pub fn response_1(challenge: &Vec<Scalar>, difficulty: usize) -> (Vec<Scalar>, Vec<Scalar>, Vec<Scalar>) {
    let mut y_xy_map = HashMap::new();
    {
        let file = File::open(Y_DIR).unwrap();
        let data: Data<Vec<Vec<u8>>> = serde_json::from_reader(file).unwrap();
        let data = data.content;
        let mut x = Scalar::zero();

        for i in 0..data.len() {
            let mut buf = [0u8; 32];
            buf.copy_from_slice(&data[i]);
            let y = Scalar::from_bytes(&buf).unwrap();
            let k = y.to_string();
            y_xy_map.insert(k[k.len() - difficulty ..].to_string(), (x, y));

            x.add_assign(&Scalar::one());
        }
    }
    println!("Hashmap length: {:?}", y_xy_map.len());

    let mut x_response = vec![];
    let mut y_response = vec![];
    let mut c_response = vec![];
    for &c in challenge {
        let key = &c.to_string();
        let key = &key[key.len() - difficulty ..];
        if y_xy_map.contains_key(key) {
            let x_y = y_xy_map.get(key).unwrap().clone();
            x_response.push(x_y.0);
            y_response.push(x_y.1);
            c_response.push(c);
        }
    }
    (x_response, y_response, c_response)
}

pub fn response_2(x_response: &Vec<Scalar>, y_response: &Vec<Scalar>, c_response: &Vec<Scalar>, key: Scalar, m_bit: [Scalar; INPUT_SIZE], difficulty: usize) 
-> (PreparedVerifyingKey<Bls12>, Vec<Proof<Bls12>>, PreparedVerifyingKey<Bls12>, Vec<Proof<Bls12>>, Vec<Scalar>) {
    let mut rng = thread_rng();

    let params = {
        let c = PowDemo {
            g: None,
            x_bit: [None; INPUT_SIZE],
        };

        generate_random_parameters::<Bls12, _, _>(c, &mut rng).unwrap()
    };

    let pow_pvk = prepare_verifying_key(&params.vk);

    let samples = x_response.len();
    let mut pow_proof_collect = vec![];
    {
        for sample in 0..samples {
            let x = x_response[sample].add(&key);
            
            let mut new_m_bit: [Option<Scalar>; INPUT_SIZE] = [None; INPUT_SIZE];
            for i in 0..m_bit.len() {
                new_m_bit[i] = Some(m_bit[i]);
            }
    
            let c = PowDemo {
                g: Some(x),
                x_bit: new_m_bit
            };
    
            let proof = create_random_proof(c, &params, &mut rng).unwrap();
            pow_proof_collect.push(proof);
        }
    }

    let params = {
        let c = EqualDemo {
            y1: None,
            y2: None,
            x_bit: [None; INPUT_SIZE],
            n: difficulty
        };

        generate_random_parameters::<Bls12, _, _>(c, &mut rng).unwrap()
    };
    
    let equal_pvk = prepare_verifying_key(&params.vk);    
    let mut equal_proof_collect = vec![];
    let mut y1_collect = vec![];
    {
        for sample in 0..samples {
            let y1 = c_response[sample];
            let y1_bit = s_to_bits(y1);
            let y1 = bits_to_s(y1_bit, difficulty);
            y1_collect.push(y1);

            let y2 = y_response[sample];
            let x_bit = s_to_bits(y2);
            
            let mut new_x_bit: [Option<Scalar>; INPUT_SIZE] = [None; INPUT_SIZE];
            for i in 0..x_bit.len() {
                new_x_bit[i] = Some(x_bit[i]);
            }
    
            let c = EqualDemo {
                y1: Some(y1),
                y2: Some(y2),
                x_bit: new_x_bit,
                n: difficulty
            };
    
            let proof = create_random_proof(c, &params, &mut rng).unwrap();
            assert!(verify_proof(&equal_pvk, &proof, &[y1, y2]).is_ok());
            equal_proof_collect.push(proof);
        }
    }

    (pow_pvk, pow_proof_collect, equal_pvk, equal_proof_collect, y1_collect)
}

pub fn verify_1(x_response: &Vec<Scalar>, y_response: &Vec<Scalar>, key: Scalar, m_bit: [Scalar; INPUT_SIZE]) -> std::io::Result<()> {
    for i in 0..x_response.len() {
        let x = x_response[i].add(&key);
        let y = y_response[i];
        assert_eq!(y, pow(x, m_bit));
    }
    Ok(())
}

pub fn verify_2(pow_pvk: PreparedVerifyingKey<Bls12>, pow_proof_collect: Vec<Proof<Bls12>>, equal_pvk: PreparedVerifyingKey<Bls12>, equal_proof_collect: Vec<Proof<Bls12>>, y_collect: Vec<Scalar>, y1_collect: Vec<Scalar>) -> std::io::Result<()> {
    for i in 0..pow_proof_collect.len() {
        assert!(verify_proof(&pow_pvk, &pow_proof_collect[i], &[y_collect[i]]).is_ok());
    }

    for i in 0..equal_proof_collect.len() {
        assert!(verify_proof(&equal_pvk, &equal_proof_collect[i], &[y1_collect[i], y_collect[i]]).is_ok());
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::time::Instant;

    use super::*;

    #[test]
    fn test_proof() {
        let mut rng = thread_rng();

        let n = 10;
        let key = Scalar::random(&mut rng);
        let m = Scalar::random(&mut rng);
        let m_bit = s_to_bits(m);

        let start = Instant::now();
        create_pos(key, m_bit, n).unwrap();
        println!("Create pos: {:?}", start.elapsed());

        let samples = 1000;
        
        let start = Instant::now();
        let challenge = create_challenge(samples);
        println!("Create challenge: {:?}", start.elapsed());

        // DIFFICULTY: byte size = 4 bit
        const DIFFICULTY: usize = 2;
        println!("Set difficulty: {:?}", DIFFICULTY * 4);

        let start = Instant::now();
        let (x_response, y_response, c_response) = response_1(&challenge, DIFFICULTY);
        println!("Response 1: {:?}", start.elapsed());
        
        println!("Success rate: {:?} / {:?}", x_response.len(), samples);

        let start = Instant::now();
        verify_1(&x_response, &y_response, key, m_bit).unwrap();
        println!("Verify 1: {:?}", start.elapsed());

        let start = Instant::now();
        let (pow_pvk, pow_proof_collect, equal_pvk, equal_proof_collect, y1_collect) 
        = response_2(&x_response, &y_response, &c_response, key, m_bit, DIFFICULTY * 4);
        println!("Response 2: {:?}", start.elapsed());
        
        let start = Instant::now();
        verify_2(pow_pvk, pow_proof_collect, equal_pvk, equal_proof_collect, y_response, y1_collect).unwrap();
        println!("Verify 2: {:?}", start.elapsed());
    }
}