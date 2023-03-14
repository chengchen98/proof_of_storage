use rand::Rng;
use ark_bls12_381::Fr;
use std::time::Instant;

use super::prover::{prepare_space, mark_space, response_1, response_2};
use super::verifier::{create_challenge, verify};

// 所声明的存储空间的位置
pub const DATA_DIR: [&str; 4] = [r"src", "proof_of_space", "data", "pos_data"];
// 验证者生成的挑战个数
pub const CHALLENGE_COUNT: usize = 20;
// 证明者需要响应的挑战个数
pub const RESPONSE_COUNT: usize = 10;
// x的取值范围[0..2^N]
pub const N: usize = 10;
// 延迟函数的轮数
pub const MIMC5_DF_ROUNDS: usize = 322;
// 哈希函数的轮数
pub const MIMC5_HASH_ROUNDS: usize = 110;

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

pub fn pospace(path: &str, df_constants: Vec<Fr>, hash_constants: Vec<Fr>, key: Fr, m: Fr) {

    // 先划分一定大小的存储空间，并用0填满
    let start = Instant::now();
    prepare_space(&path).unwrap();
    println!("Prepare storage: {:?}", start.elapsed());

    // 通过计算延迟函数标定存储空间
    let start = Instant::now();
    mark_space(&path, key, m, &df_constants).unwrap();
    println!("Create pos: {:?}", start.elapsed());

    // 验证者随机生成挑战
    let start = Instant::now();
    let challenges = create_challenge(CHALLENGE_COUNT);
    println!("Create challenge: {:?}", start.elapsed());

    // 第一次应答：
    let start = Instant::now();
    let (x_response, idx_response, x_hash_response) = response_1(&path, &challenges, key, &hash_constants);
    assert_eq!(x_response.len(), RESPONSE_COUNT);
    println!("--Response 1: {:?}", start.elapsed());

    // 计算成功率
    println!("-Success rate: {:?} / {:?}", idx_response.len(), RESPONSE_COUNT);

    // 第二次应答：生成零知识证明
    let start = Instant::now();
    let (pvk, proof) = response_2(key, &x_response, m,  &df_constants,  &challenges, &idx_response, x_hash_response, &hash_constants);
    println!("--Response 2: {:?}", start.elapsed());
    
    // 验证
    let start = Instant::now();
    verify(pvk, proof, key, m, &challenges, &idx_response, x_hash_response);
    println!("Verify: {:?}", start.elapsed());
}

#[test]
fn test() {
    use std::path::PathBuf;
    
    println!("data len (byte): {:?}  |  challenge count: {:?}  | response count: {:?}", (N + 1) * 2_usize.pow(N.try_into().unwrap()), CHALLENGE_COUNT, RESPONSE_COUNT);
    let path: PathBuf = DATA_DIR.iter().collect();
    let path = path.to_str().unwrap();

    const SAMPLES: usize = 1;
    for i in 0..SAMPLES {
        println!("Sample: {:?}", i);
        let (df_constants, hash_constants, key, m) = prepare_params();
        pospace(path, df_constants, hash_constants, key, m);
        println!("-------------------------------------");
    }
}