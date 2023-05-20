use rand::Rng;
use ark_bls12_381::Fr;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::time::Instant;
use std::io::{self, Write};

use super::prover::{prepare_space, mark_space, response_1, response_2, construct_circuit};
use super::verifier::{create_challenges, verify};
use crate::mimc::{mimc_df::MIMC5_DF_ROUNDS, mimc_hash::MIMC5_HASH_ROUNDS};

// 实验数据存储的地址
pub const SAVED_DATA_DIR: [&str; 4] = [r"src", "proof_of_space", "data", "result"];
// 所声明的存储空间的地址
pub const DATA_DIR: [&str; 4] = [r"src", "proof_of_space", "data", "pos_data"];
// 验证者生成的挑战个数
pub const CHALLENGE_COUNT: usize = 100;
// 证明者需要响应的挑战个数
pub const RESPONSE_COUNT: usize = 50;
// x的取值范围[0..2^N]
pub const N: usize = 20;

pub fn prepare_params() -> (Vec<Fr>, Vec<Fr>, Fr, Fr) {
    // 准备计算DF和Hash的参数
    let mut rng = rand::thread_rng();

    let df_constants: Vec<Fr> = (0..MIMC5_DF_ROUNDS)
    .map(|_| rng.gen())
    .collect::<Vec<_>>();

    let hash_constants: Vec<Fr> = (0..MIMC5_HASH_ROUNDS)
    .map(|_| rng.gen())
    .collect::<Vec<_>>();

    let key: Fr = rng.gen();
    let m: Fr = rng.gen();

    (df_constants, hash_constants, key, m)
}

pub fn pospace(path: &str, df_constants: Vec<Fr>, hash_constants: Vec<Fr>, key: Fr, m: Fr, save_file: &mut File) {
    // 先划分一定大小的存储空间，并用0填满
    let start = Instant::now();
    prepare_space(&path, N).unwrap();
    let cost1 = start.elapsed();

    // 通过计算延迟函数标定存储空间
    let start = Instant::now();
    let (df_cost, file_cost) = mark_space(&path, key, m, &df_constants, N);
    let cost2 = start.elapsed();

    // 验证者随机生成挑战
    let start = Instant::now();
    let challenges = create_challenges(CHALLENGE_COUNT, N);
    let cost3 = start.elapsed();

    // 第一次应答
    let start = Instant::now();
    let (x_response, idx_response, x_hash_response) = response_1(&path, &challenges, key, &hash_constants, N);
    let cost4 = start.elapsed();

    // 第二次应答：生成零知识证明
    let start = Instant::now();
    let params = construct_circuit(&df_constants, &hash_constants);
    let cost5 = start.elapsed();

    let start = Instant::now();
    let (pvk, proof) = response_2(params, key, &x_response, m,  &df_constants,  &challenges, &idx_response, x_hash_response, &hash_constants);
    let cost6 = start.elapsed();
    
    // 验证
    let start = Instant::now();
    verify(pvk, proof, key, m, &challenges, &idx_response, x_hash_response);
    let cost7 = start.elapsed();

    save_file.write_all(["N, ", &N.to_string(), ", data len (byte), ", &((N + 1) * 2_usize.pow(N.try_into().unwrap()) / 8).to_string(), ", vde round, ", &MIMC5_DF_ROUNDS.to_string(), "\n"].concat().as_bytes()).unwrap();
    save_file.write_all(["challenge count, ", &CHALLENGE_COUNT.to_string(), ", response count, ", &RESPONSE_COUNT.to_string(), ", success rate, ", &idx_response.len().to_string(), "/", &RESPONSE_COUNT.to_string(), "\n"].concat().as_bytes()).unwrap();
    
    save_file.write_all(["[P] Prepare space, ", &cost1.as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
    save_file.write_all(["[P] Mark space, ", &cost2.as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
    save_file.write_all(["df cost, ", &df_cost.to_string(), ", file cost, ", &file_cost.to_string(), "\n"].concat().as_bytes()).unwrap();
    save_file.write_all(["[V] Create challenges, ", &cost3.as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
    save_file.write_all(["[P] -- Response 1 (return index and hash), ", &cost4.as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
    save_file.write_all(["[P] -- Response 2 (create params), ", &cost5.as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
    save_file.write_all(["[P] -- Response 2 (create proof), ", &cost6.as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
    save_file.write_all(["[V] Verify, ", &cost7.as_secs_f32().to_string(), "\n\n"].concat().as_bytes()).unwrap();
}

pub fn test_pospace() {
    println!("data len (byte): {:?}  |  challenge count: {:?}  | response count: {:?}", (N + 1) * 2_usize.pow(N.try_into().unwrap()) / 8, CHALLENGE_COUNT, RESPONSE_COUNT);
    let path: PathBuf = DATA_DIR.iter().collect();
    let path = path.to_str().unwrap();

    let save_path: PathBuf = SAVED_DATA_DIR.iter().collect();
    let save_path = save_path.to_str().unwrap();
    let mut save_file = OpenOptions::new()
    .read(true)
    .write(true)
    .append(true)
    .create(true) 
    .open(save_path)
    .unwrap();

    const SAMPLES: usize = 5;
    for i in 0..SAMPLES {
        println!("Sample: {:?}", i);
        let (df_constants, hash_constants, key, m) = prepare_params();
        pospace(path, df_constants, hash_constants, key, m, &mut save_file);
        println!("-------------------------------------");
    }
}

#[test]
fn test() {
    test_pospace();
}