use rand::Rng;
use ark_bls12_381::Fr;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
// use num_bigint::BigInt;
use rug::Integer;

use super::prover::{create_depend, copy_and_pad, seal, unseal, copy_and_compress, response};
use super::verifier::{create_random_file, create_challenges};
use super::merkle_tree::{generate_merkle_tree, generate_merkle_proof, verify_merkle_proof};
use crate::mimc::mimc_hash::MIMC5_HASH_ROUNDS;
use crate::proof_of_storage::verifier::verify;
use crate::vde::sloth::{P_1024, P_2048};

pub const SAVED_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "pos_result"];
pub const ORIGIN_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "origin_data"];
pub const SEALED_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "sealed_data"];
pub const UNSEALED_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "unsealed_data"];

// 单位：byte
pub const DATA_L: usize = 127 * 8 * 1024 * 1024;
pub const DATA_PL: usize = 128 * 8 * 1024 * 1024;

pub const L0: usize = 127; // 127B
pub const L1: usize = L0 * 8 * 1024;
pub const L2: usize = L1 * 32;

pub const PL0: usize = L0 + 1;
pub const PL1: usize = PL0 * 8 * 1024;
pub const PL2: usize = PL1 * 32;

pub const SEAL_ROUNDS: usize = 3;
pub const SLOTH_ROUNDS: usize = 3;
pub const VDE_MODE: &str = "sloth";

pub const MODE_L: usize = 1;
pub const MODE_S: usize = 1;
pub const IDX_CNT_L: usize = 3;
pub const IDX_CNT_S: usize = 3;

pub const LEAVES_TO_PROVE_COUNT: usize = 1;

pub fn prepare_params() -> (usize, Fr, Vec<Fr>, Integer, Vec<Vec<usize>>, Vec<Vec<Vec<usize>>>) {
    // 准备存储证明所需要的参数
    let mut rng = rand::thread_rng();

    // hash
    let hash_key = rng.gen();
    let hash_constants = (0..MIMC5_HASH_ROUNDS)
        .map(|_| rng.gen())
        .collect::<Vec<_>>();

    // vde key
    let vde_key = {
        if (L0 + 1) * 8 == 2048 {
            Integer::from_str(P_2048).unwrap()
        }
        else {
            Integer::from_str(P_1024).unwrap()
        }
    };
    
    // 生成数据块依赖关系
    let l2_cnt = DATA_L / L2;
    let l1_cnt = L2 / L1;
    let (idx_l, idx_s) = create_depend(l2_cnt, l1_cnt, IDX_CNT_L, IDX_CNT_S, MODE_L, MODE_S);   

    (l2_cnt, hash_key, hash_constants, vde_key, idx_l, idx_s)
}

pub fn postorage(origin_path: &str, sealed_path: &str, unsealed_path: &str, block_cnt: usize, hash_key: Fr, hash_constants: &Vec<Fr>, vde_key: &Integer, idx_l: Vec<Vec<usize>>, idx_s: Vec<Vec<Vec<usize>>>, save_file: &mut File, should_save: bool) {
    // Seal
    print!("Start to copy and pad origin data...");
    io::stdout().flush().unwrap();
    copy_and_pad(origin_path, sealed_path);
    print!("Ok\n");

    print!("Start to seal...");
    io::stdout().flush().unwrap();
    let start = Instant::now();
    let (seal_vde_cost, seal_file_cost) = seal(sealed_path, block_cnt, &idx_l, &idx_s, &hash_constants, hash_key, vde_key);
    let cost1 = start.elapsed();
    print!("Ok...{:?}\n", cost1);

    // Unseal
    print!("Start to unseal...");
    io::stdout().flush().unwrap();
    let start = Instant::now();
    let (unseal_vde_cost, unseal_file_cost) = unseal(sealed_path, block_cnt, &idx_l, &idx_s, &hash_constants, hash_key, vde_key);
    let cost2 = start.elapsed();
    print!("Ok...{:?}\n", cost2);

    // print!("Start to copy and compress unsealed data...");
    io::stdout().flush().unwrap();
    copy_and_compress(sealed_path, unsealed_path);
    print!("Ok\n");

    // let indices_to_prove = create_challenges(LEAVES_TO_PROVE_COUNT, (0, (DATA_L / L1)));
    // println!("{:?}", indices_to_prove);
    // let (blocks_idx, blocks, depend_blocks, sealed_merkle_root, sealed_proof, sealed_leaves)
    // = response(sealed_path, &indices_to_prove, &idx_l);
    // verify(origin_path, &blocks_idx, &idx_s, blocks, depend_blocks, sealed_proof, sealed_merkle_root, &indices_to_prove, &sealed_leaves, hash_constants, hash_key, vde_key);

    if should_save == true {
        save_file.write_all(["(byte) data len, ", &DATA_L.to_string(), ", block2 len, ", &L2.to_string(), ", block2 count, ", &block_cnt.to_string(), ", block1 len, ", &L1.to_string(), ", block1 count, ", &(L2 / L1).to_string(), "\n"].concat().as_bytes()).unwrap();
        save_file.write_all(["[P] Seal, ", &cost1.as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
        save_file.write_all(["vde, ", &seal_vde_cost.to_string(), ", file, ", &seal_file_cost.to_string(), "\n"].concat().as_bytes()).unwrap();
        save_file.write_all(["[P] Unseal, ", &cost2.as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
        save_file.write_all(["vde inv, ", &unseal_vde_cost.to_string(), ", file, ", &unseal_file_cost.to_string(), "\n\n"].concat().as_bytes()).unwrap();
    }
}

pub fn challenge_and_response(origin_path: &str, sealed_path: &str, unsealed_path: &str, save_file: &mut File) {
    // 验证者生成 origin merkle tree，只保存 root
    print!("Generate origin merkle tree...");
    io::stdout().flush().unwrap();
    let start = Instant::now();
    let (_, _, origin_merkle_root) = generate_merkle_tree(&origin_path, DATA_L, L1);
    let cost1 = start.elapsed();
    print!("Ok...{:?}\n", cost1);

    // 证明者生成 sealed merkle tree，计算 root 并发送给验证者       
    print!("Generate sealed merkle tree...");
    io::stdout().flush().unwrap();
    let start = Instant::now();
    let (sealed_leaves, sealed_merkle_tree, sealed_merkle_root) = generate_merkle_tree(&sealed_path, DATA_PL, PL1);
    let cost2 = start.elapsed();
    print!("Ok...{:?}\n", cost2);

    // 验证者随机生成叶子节点下标，要求证明者恢复对应的原始数据
    print!("Generate indices to prove...");
    io::stdout().flush().unwrap();
    let start = Instant::now();
    let indices_to_prove = create_challenges(LEAVES_TO_PROVE_COUNT, (0, sealed_leaves.len()));
    let cost3 = start.elapsed();
    print!("Ok...{:?}\n", cost3);

    // 证明者生成验证路径 sealed_proof
    print!("Generate sealed proof...");
    io::stdout().flush().unwrap();
    let start = Instant::now();
    let sealed_proof = generate_merkle_proof(&indices_to_prove, sealed_merkle_tree);
    let cost4 = start.elapsed();
    print!("Ok...{:?}\n", cost4);

    // 验证者验证验证路径 sealed_proof
    print!("Verify sealed merkle proof...");
    io::stdout().flush().unwrap();
    let start = Instant::now();
    verify_merkle_proof(sealed_proof, sealed_merkle_root, &indices_to_prove, &sealed_leaves);
    let cost5 = start.elapsed();
    print!("Ok...{:?}\n", cost5);

    // 证明者生成 unsealed merkle tree
    print!("Generate unsealed merkle tree...");
    io::stdout().flush().unwrap();
    let start = Instant::now();
    let (unsealed_leaves, unsealed_merkle_tree, _) = generate_merkle_tree(&unsealed_path, DATA_L, L1);
    let cost6 = start.elapsed();
    print!("Ok...{:?}\n", cost6);

    // 证明者生成验证路径 unsealed_proof
    print!("Generate unsealed proof...");
    io::stdout().flush().unwrap();
    let start = Instant::now();
    let unsealed_proof = generate_merkle_proof(&indices_to_prove, unsealed_merkle_tree.clone());
    let cost7 = start.elapsed();
    print!("Ok...{:?}\n", cost7);

    // 验证者用 origin merkle root 验证 unsealed_proof
    print!("Verify unsealed merkle proof...");
    io::stdout().flush().unwrap();
    let start = Instant::now();
    verify_merkle_proof(unsealed_proof, origin_merkle_root, &indices_to_prove, &unsealed_leaves);
    let cost8 = start.elapsed();
    print!("Ok...{:?}\n", cost8);

    save_file.write_all(["merkle tree depth, ", &(unsealed_merkle_tree.depth().to_string()), ", leaves count, ", &unsealed_leaves.len().to_string(), ", leaves to prove count, ", &LEAVES_TO_PROVE_COUNT.to_string(), "\n"].concat().as_bytes()).unwrap();
    save_file.write_all(["merkle tree generate, ", &(cost1.as_secs_f32() + cost2.as_secs_f32()).to_string(), "\n"].concat().as_bytes()).unwrap();
    save_file.write_all(["[P] merkle tree prove, ", &(cost4.as_secs_f32() + cost6.as_secs_f32() + cost7.as_secs_f32()).to_string(), "\n"].concat().as_bytes()).unwrap();
    save_file.write_all(["[V] merkle tree verify, ", &(cost3.as_secs_f32() + cost5.as_secs_f32() + cost8.as_secs_f32()).to_string(), "\n"].concat().as_bytes()).unwrap();
    save_file.write_all("\n".as_bytes()).unwrap();
    save_file.flush().unwrap();
}


pub fn test_postorage(should_save: bool) {
    println!("data len (byte): {:?}  | block2 len: {:?}  | block1 len: {:?}", DATA_L, L2, L1);

    // 原始文件所在位置
    let origin_path: PathBuf = ORIGIN_DATA_DIR.iter().collect();
    let origin_path = origin_path.to_str().unwrap();

    // 用来存储seal后的数据
    let sealed_path: PathBuf = SEALED_DATA_DIR.iter().collect();
    let sealed_path = sealed_path.to_str().unwrap();

    // 用来存储unseal后的数据
    let unsealed_path: PathBuf = UNSEALED_DATA_DIR.iter().collect();
    let unsealed_path = unsealed_path.to_str().unwrap();
    
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

        let (block_cnt, hash_key, hash_constants, vde_key, idx_l, idx_s) = prepare_params();
        create_random_file(origin_path, DATA_L).unwrap();
        postorage(origin_path, sealed_path, unsealed_path, block_cnt, hash_key, &hash_constants, &vde_key, idx_l, idx_s, &mut save_file, should_save);
        // challenge_and_response(origin_path, sealed_path, unsealed_path, &mut save_file);
        println!("----------------------------------------------------------------");
    }
}

#[test]
fn test() {
    let should_save = true;
    test_postorage(should_save);
}