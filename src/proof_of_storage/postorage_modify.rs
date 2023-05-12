use std::fs::{File, OpenOptions};
use std::io::{Write, BufReader};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use rug::Integer;
use rand::Rng;

use serde::{Serialize, Deserialize};
use bincode::{serialize_into, deserialize_from};

use super::merkle_tree::{generate_merkle_proof, generate_merkle_tree_from_file, verify_merkle_proof};
use super::prover_modify::{copy_and_pad, seal, unseal, copy_and_compress};
use super::verifier_modify::{create_random_file, unseal_single_prepare, unseal_single_and_verify};

use crate::proof_of_storage::verifier_modify::create_challenges;
use crate::vde::rug_sloth::{P_512, P_1024, P_2048};

pub const ORIGIN_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "origin_data"];
pub const SEALED_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "sealed_data"];
pub const UNSEALED_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "unsealed_data"];

pub const RUN_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "pos_result"];
pub const TARGET_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "target"];

// 单位：byte
pub const DATA_L: usize = 63 * 16 * 1 * 1024; // 1MB
pub const DATA_PL: usize = 64 * 16 * 1 * 1024;

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
pub const MODE_S: usize = 0;
pub const CNT_L: usize = 3;
pub const CNT_S: usize = 50;

pub const LEAVES_TO_PROVE_COUNT: usize = 3;

#[derive(Serialize, Deserialize)]
struct SaveData {
    vde_key: String,
    iv: Vec<u8>,
    blocks_id: Vec<Vec<Vec<u8>>>,
}

pub fn save_data(path: &str, vde_key: &Integer, iv: &Vec<u8>, blocks_id: &Vec<Vec<Vec<u8>>>) {
    let target = SaveData {vde_key: vde_key.to_string(), iv: iv.to_vec(), blocks_id: blocks_id.to_vec()};

    let mut file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true) 
    .open(path)
    .unwrap();

    serialize_into(&mut file, &target).unwrap();
}

pub fn load_data(path: &str) -> (Integer, Vec<u8>, Vec<Vec<Vec<u8>>>) {
    let file = OpenOptions::new()
    .read(true)
    .open(path)
    .unwrap();

    let target: SaveData = deserialize_from(&file).unwrap();
    let (vde_key, iv, blocks_id) = (Integer::from_str(&target.vde_key).unwrap(), target.iv, target.blocks_id);
    (vde_key, iv, blocks_id)
}

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
        for _ in 0..128 {
            let buf: u8 = rng.gen_range(0u8..=255u8);
            res.push(buf);
        }
        res
    };

    (vde_key, iv)
}

pub fn postorage(origin_path: &str, sealed_path: &str, unsealed_path: &str, save_data_path: &str, run_data_path: &str, should_save_run_data: bool, should_unseal: bool) {
    copy_and_pad(origin_path, sealed_path);

    // params
    let (vde_key, iv) = prepare_params();

    // seal
    let start = Instant::now();
    let (blocks_id, seal_vde_cost, seal_file_cost, seal_depend_cost, seal_hash_cost, seal_block_cost, seal_modadd_cost) = seal(sealed_path, SEAL_ROUNDS, MODE_L, CNT_L, MODE_S, CNT_S, &vde_key, &iv, VDE_ROUNDS, VDE_MODE);
    let cost1 = start.elapsed();

    save_data(save_data_path, &vde_key, &iv, &blocks_id);

    if should_unseal == true {
        // Unseal
        let start = Instant::now();
        let (unseal_vde_cost, unseal_file_cost, unseal_depend_cost, unseal_hash_cost, unseal_block_cost, unseal_modsub_cost) = unseal(sealed_path, SEAL_ROUNDS, MODE_L, CNT_L, MODE_S, CNT_S, &vde_key, &iv, &blocks_id, VDE_ROUNDS, VDE_MODE);
        let cost2 = start.elapsed();

        copy_and_compress(sealed_path, unsealed_path);

        if should_save_run_data == true {
            let mut run_data_file = OpenOptions::new()
            .read(true)
            .write(true)
            .append(true)
            .create(true) 
            .open(run_data_path)
            .unwrap();
        
            run_data_file.write_all(["(byte) data len, ", &DATA_L.to_string(), ", block len, ", &BLOCK_L.to_string(), ", block count, ", &(DATA_L / BLOCK_L).to_string(), ", unit len, ", &UNIT_L.to_string(), ", unit count, ", &(BLOCK_L / UNIT_L).to_string(), "\n"].concat().as_bytes()).unwrap();
            run_data_file.write_all(["seal round, ", &SEAL_ROUNDS.to_string(), ", vde rounds, ", &VDE_ROUNDS.to_string(), ", mode l, ", &MODE_L.to_string(), ", cnt l, ", &CNT_L.to_string(), ", mode s, ", &MODE_S.to_string(), ", cnt s, ", &CNT_S.to_string(), "\n"].concat().as_bytes()).unwrap();
            run_data_file.write_all(["[P] Seal, ", &cost1.as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
            run_data_file.write_all(["vde, ", &seal_vde_cost.to_string(), ", file, ", &seal_file_cost.to_string(), ", depend, ", &seal_depend_cost.to_string(), ", hash, ", &seal_hash_cost.to_string(), ", block, ", &seal_block_cost.to_string(), ", modadd, ", &seal_modadd_cost.to_string(), "\n"].concat().as_bytes()).unwrap();
            run_data_file.write_all(["[P] Unseal, ", &cost2.as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
            run_data_file.write_all(["vde inv, ", &unseal_vde_cost.to_string(), ", file, ", &unseal_file_cost.to_string(), ", depend, ", &unseal_depend_cost.to_string(), ", hash, ", &unseal_hash_cost.to_string(), ", block, ", &unseal_block_cost.to_string(), ", modsub, ", &unseal_modsub_cost.to_string(), "\n\n"].concat().as_bytes()).unwrap();
        }
    }
}

pub fn challenge_unseal_single(target_path: &str, origin_path: &str, sealed_path: &str) {
    // unseal single
    let (vde_key, iv, blocks_id) = load_data(&target_path);

    let range = (0 * BLOCK_PL, 1 * BLOCK_PL);
    let (blocks_idx, blocks) = unseal_single_prepare(sealed_path, range.0, range.1, MODE_L, CNT_L, MODE_S, CNT_S);
    unseal_single_and_verify(sealed_path, origin_path, SEAL_ROUNDS, &blocks_idx, &blocks, MODE_L, CNT_L, MODE_S, CNT_S, &vde_key, VDE_ROUNDS, VDE_MODE, &iv, &blocks_id);
}

pub fn merkle_tree_proof(origin_path: &str, unsealed_path: &str) {
    let (_, _, merkle_root) = generate_merkle_tree_from_file(&origin_path, DATA_L, BLOCK_L);
    let (leaves, merkle_tree, _) = generate_merkle_tree_from_file(&unsealed_path, DATA_L, BLOCK_L);
    let indices_to_prove = create_challenges(LEAVES_TO_PROVE_COUNT, (0, (DATA_L / BLOCK_L)));
    let proof = generate_merkle_proof(&indices_to_prove, merkle_tree);
    verify_merkle_proof(proof, merkle_root, &indices_to_prove, &leaves);
}

pub fn test_postorage(should_save_run_data: bool, should_seal: bool, should_unseal: bool, should_challenge: bool, should_unseal_single: bool) {
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
    
    // 保存 SaveData 相关数据
    let target_path: PathBuf = TARGET_DIR.iter().collect();
    let target_path = target_path.to_str().unwrap();

    // 保存实验数据
    let run_data_path: PathBuf = RUN_DATA_DIR.iter().collect();
    let run_data_path = run_data_path.to_str().unwrap();

    const SAMPLES: usize = 4;
    for _ in 0..SAMPLES {
        if should_seal == true {
            create_random_file(origin_path, DATA_L).unwrap();
            postorage(origin_path, sealed_path, unsealed_path, target_path, run_data_path, should_save_run_data, should_unseal);
        }

        if should_unseal_single == true {
            challenge_unseal_single(target_path, origin_path, sealed_path);
        }

        if should_challenge == true {
            merkle_tree_proof(origin_path, unsealed_path);
        }
    }
}

#[test]
fn test() {
    let should_save_run_data = true;

    let should_seal = true;
    let should_unseal = true;
    let should_challenge = false;
    let should_unseal_single = false;
    test_postorage(should_save_run_data, should_seal, should_unseal, should_challenge, should_unseal_single);
}