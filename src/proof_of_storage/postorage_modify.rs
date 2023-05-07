use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use rug::Integer;
use rand::Rng;

use super::prover_modify::{copy_and_pad, seal, unseal, copy_and_compress};
use super::verifier_modify::{create_random_file, unseal_single_prepare, unseal_single_and_verify};

use crate::vde::rug_sloth::{P_512, P_1024, P_2048};

pub const SAVED_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "pos_result"];
pub const ORIGIN_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "origin_data"];
pub const SEALED_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "sealed_data"];
pub const UNSEALED_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "unsealed_data"];

// 单位：byte
pub const DATA_L: usize = 63 * 1 * 1 * 1024; // 1MB
pub const DATA_PL: usize = 64 * 1 * 1 * 1024;

pub const UNIT_L: usize = 63;
pub const BLOCK_L: usize = UNIT_L * 128;

pub const UNIT_PL: usize = UNIT_L + 1;
pub const BLOCK_PL: usize = UNIT_PL * 128;

pub const SEAL_ROUNDS: usize = 2;
pub const VDE_ROUNDS: usize = 10;
pub const VDE_MODE: &str = "sloth";

// mode = 0: 随机性依赖关系，由计算哈希函数得到
// mode > 0: 确定性依赖关系
pub const MODE_L: usize = 0;
pub const MODE_S: usize = 1;
pub const CNT_L: usize = 3;
pub const CNT_S: usize = 3;

pub const LEAVES_TO_PROVE_COUNT: usize = 1;

pub fn prepare_params() -> (Integer, Vec<u8>) {
    // 生成vde需要的key
    let vde_key = {
        if UNIT_PL * 8 == 512 {
            Integer::from_str(P_512).unwrap()
        }
        else if UNIT_PL * 8 == 1024 {
            Integer::from_str(P_1024).unwrap()
        }
        else if UNIT_PL * 8 == 2048 {
            Integer::from_str(P_2048).unwrap()
        }
        else {
            Integer::from_str(P_1024).unwrap()
        }
    };

    let mut rng = rand::thread_rng();
    let iv = {
        let mut res = vec![];
        for _ in 0.. 128 {
            let buf: u8 = rng.gen_range(0u8..=255u8);
            res.push(buf);
        }
        res
    };
    (vde_key, iv)
}

pub fn postorage(origin_path: &str, sealed_path: &str, unsealed_path: &str, vde_key: &Integer, iv: &Vec<u8>, save_file: &mut File, should_save: bool) {
    copy_and_pad(origin_path, sealed_path);

    // seal
    let start = Instant::now();
    let (seal_vde_cost, seal_file_cost, seal_depend_cost, seal_hash_cost, seal_block_cost, seal_xor_cost) = seal(sealed_path, SEAL_ROUNDS, MODE_L, CNT_L, MODE_S, CNT_S, vde_key, &iv, VDE_ROUNDS, VDE_MODE);
    let cost1 = start.elapsed();

    // Unseal
    let start = Instant::now();
    let (unseal_vde_cost, unseal_file_cost, unseal_depend_cost, unseal_hash_cost, unseal_block_cost, unseal_xor_cost) = unseal(sealed_path, SEAL_ROUNDS, MODE_L, CNT_L, MODE_S, CNT_S, vde_key, &iv, VDE_ROUNDS, VDE_MODE);
    let cost2 = start.elapsed();

    copy_and_compress(sealed_path, unsealed_path);

    // let indices_to_prove = create_challenges(LEAVES_TO_PROVE_COUNT, (0, (DATA_L / L1)));
    // println!("{:?}", indices_to_prove);
    // let (blocks_idx, blocks, depend_blocks, sealed_merkle_root, sealed_proof, sealed_leaves)
    // = response(sealed_path, &indices_to_prove, &idx_l);
    // verify(origin_path, &blocks_idx, &idx_s, blocks, depend_blocks, sealed_proof, sealed_merkle_root, &indices_to_prove, &sealed_leaves, hash_constants, hash_key, vde_key);

    if should_save == true {
        save_file.write_all(["(byte) data len, ", &DATA_L.to_string(), ", block len, ", &BLOCK_L.to_string(), ", block count, ", &(DATA_L / BLOCK_L).to_string(), ", unit len, ", &UNIT_L.to_string(), ", unit count, ", &(BLOCK_L / UNIT_L).to_string(), "\n"].concat().as_bytes()).unwrap();
        save_file.write_all(["seal round, ", &SEAL_ROUNDS.to_string(), ", vde rounds, ", &VDE_ROUNDS.to_string(), ", mode l, ", &MODE_L.to_string(), ", cnt l, ", &CNT_L.to_string(), ", mode s, ", &MODE_S.to_string(), ", cnt s, ", &CNT_S.to_string(), "\n"].concat().as_bytes()).unwrap();
        save_file.write_all(["[P] Seal, ", &cost1.as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
        save_file.write_all(["vde, ", &seal_vde_cost.to_string(), ", file, ", &seal_file_cost.to_string(), ", depend, ", &seal_depend_cost.to_string(), ", hash, ", &seal_hash_cost.to_string(), ", block, ", &seal_block_cost.to_string(), ", xor, ", &seal_xor_cost.to_string(), "\n"].concat().as_bytes()).unwrap();
        save_file.write_all(["[P] Unseal, ", &cost2.as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
        save_file.write_all(["vde inv, ", &unseal_vde_cost.to_string(), ", file, ", &unseal_file_cost.to_string(), ", depend, ", &unseal_depend_cost.to_string(), ", hash, ", &unseal_hash_cost.to_string(), ", block, ", &unseal_block_cost.to_string(), ", xor, ", &unseal_xor_cost.to_string(), "\n\n"].concat().as_bytes()).unwrap();
    }
}

pub fn challenge_and_verify(sealed_path: &str, idx_begin: usize, idx_end: usize, unsealed_path: &str) {
    let (blocks_idx, blocks, depend_blocks) = unseal_single_prepare(sealed_path, idx_begin, idx_end, MODE_L, CNT_L);
    println!("block idx: {:?}", blocks_idx);
    println!("depend: {:?}", depend_blocks[0].len());
    let (vde_key, _) = prepare_params();
    unseal_single_and_verify(unsealed_path, SEAL_ROUNDS, &blocks_idx, &blocks, &depend_blocks, MODE_S, CNT_S, &vde_key, VDE_ROUNDS, VDE_MODE);
}

pub fn test_postorage(should_save: bool) {
    println!("data len (byte): {:?}", DATA_L);

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

    const SAMPLES: usize = 1;
    for _ in 0..SAMPLES {
        let (vde_key, iv) = prepare_params();
        // create_random_file(origin_path, DATA_L).unwrap();
        postorage(origin_path, sealed_path, unsealed_path, &vde_key, &iv, &mut save_file, should_save);
        // challenge_and_verify(sealed_path, 10 * PL2, 11 * PL2-1, unsealed_path);
    }
}

#[test]
fn test() {
    let should_save = false;
    test_postorage(should_save);
}