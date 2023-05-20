use std::fs::{OpenOptions, File};
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use rug::Integer;
use rand::Rng;

use serde::{Serialize, Deserialize};
use bincode::{serialize_into, deserialize_from};

use super::common::{gen_posdata, blake3_hash};
use super::merkle_tree::{generate_merkle_proof, generate_merkle_tree_from_file, verify_merkle_proof, generate_merkle_tree_from_data};
use super::prover::{copy_and_pad, seal, unseal, copy_and_compress};
use super::verifier::{create_random_file, create_challenges, batch_unseal_prepare, batch_unseal_and_verify, batch_unseal, batch_verify, batch_unseal_parallel, single_unseal_prepare};

use crate::vde::rug_sloth::{P_512, P_1024, P_2048};

pub const ORIGIN_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "origin_data"];
pub const SEALED_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "sealed_data"];
pub const UNSEALED_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "unsealed_data"];

pub const RUN_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "pos_result"];
pub const STAT_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "pos_result_stat.csv"];
pub const TARGET_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "target"];

// // 单位：byte
// pub const DATA_L: usize = 63 * 16 * 16 * 1024; // 16MB
// pub const DATA_PL: usize = 64 * 16 * 16 * 1024;

// pub const UNIT_L: usize = 63;
// pub const BLOCK_L: usize = UNIT_L * 128;
// pub const BIG_BLOCK_L: usize = BLOCK_L * 128;

// pub const UNIT_PL: usize = UNIT_L + 1;
// pub const BLOCK_PL: usize = UNIT_PL * 128;
// pub const BIG_BLOCK_PL: usize = BLOCK_PL * 128;

// pub const SEAL_ROUNDS: usize = 2;
// pub const VDE_ROUNDS: usize = 10;
// pub const VDE_MODE: &str = "sloth";

// // mode = 0: 随机性依赖关系，由计算哈希函数得到
// // mode > 0: 确定性依赖关系
// pub const MODE_L: usize = 0;
// pub const MODE_S: usize = 0;
// // CNT_L = 0: 表示长程依赖个数 = idx % 10 + 1
// // CNT_L > 0: 表示长程依赖个数固定
// pub const CNT_L: usize = 0;
// pub const CNT_S: usize = 5;

// pub const LEAVES_TO_PROVE_COUNT: usize = 3;

#[derive(Clone)] 
pub struct PosPara {
    pub data_l: usize,

    pub unit_l: usize,
    pub block_l: usize,
    pub big_block_l: usize,

    pub unit_pl: usize,
    pub block_pl: usize,
    pub big_block_pl: usize,

    pub seal_rounds: usize,
    pub vde_rounds: usize,
    pub vde_mode: String,

    pub mode_l: usize,
    pub cnt_l: usize,

    pub mode_s: usize,
    pub cnt_s: usize,

    pub leaves_to_prove_count: usize,
}

#[derive(Serialize, Deserialize)]
struct PubData {
    vde_key: String,
    iv: Vec<u8>,
    // blocks_id: Vec<Vec<u8>>,
}

pub fn save_data(path: &str, vde_key: &Integer, iv: &Vec<u8>) {
    let target = PubData {vde_key: vde_key.to_string(), iv: iv.to_vec()};

    let mut file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true) 
    .open(path)
    .unwrap();

    serialize_into(&mut file, &target).unwrap();
}

pub fn load_data(path: &str) -> (Integer, Vec<u8>) {
    let file = OpenOptions::new()
    .read(true)
    .open(path)
    .unwrap();

    let target: PubData = deserialize_from(&file).unwrap();
    let (vde_key, iv) = (Integer::from_str(&target.vde_key).unwrap(), target.iv);
    (vde_key, iv)
}

pub fn prepare_params(unit_pl: usize) -> (Integer, Vec<u8>) {
    // 生成vde需要的key
    let vde_key = {
        if unit_pl * 8 == 512 {
            Integer::from_str(P_512).unwrap()
        }
        else if unit_pl * 8 == 1024 {
            Integer::from_str(P_1024).unwrap()
        }
        else if unit_pl * 8 == 2048 {
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

pub fn seal_and_unseal(params: &PosPara, origin_path: &str, sealed_path: &str, unsealed_path: &str, pubdata_path: &str, run_data_file: &mut File, should_save_run_data: bool, should_unseal: bool, stat_data_file: &mut File) {
    
    copy_and_pad(origin_path, sealed_path, params.data_l, params.unit_l);

    // params
    let (vde_key, iv) = prepare_params(params.unit_pl);

    // seal
    let start = Instant::now();
    let (_, seal_vde_cost, seal_file_cost, seal_depend_cost, seal_hash_cost, seal_block_cost, seal_modadd_cost) = seal(params, sealed_path, &vde_key, &iv);
    let cost1 = start.elapsed();

    save_data(pubdata_path, &vde_key, &iv);

    if should_unseal == true {
        // Unseal
        let start = Instant::now();
        let (unseal_vde_cost, unseal_file_cost, unseal_depend_cost, unseal_hash_cost, unseal_block_cost, unseal_modsub_cost) = unseal(&params, sealed_path, &vde_key, &iv);
        let cost2 = start.elapsed();

        copy_and_compress(sealed_path, unsealed_path, params.data_l, params.unit_l, params.unit_pl);

        if should_save_run_data == true {
            run_data_file.write_all(["[P] Seal, ", &cost1.as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
            run_data_file.write_all(["vde, ", &seal_vde_cost.to_string(), ", file, ", &seal_file_cost.to_string(), ", depend, ", &seal_depend_cost.to_string(), ", hash, ", &seal_hash_cost.to_string(), ", block, ", &seal_block_cost.to_string(), ", modadd, ", &seal_modadd_cost.to_string(), "\n"].concat().as_bytes()).unwrap();
            run_data_file.write_all(["[P] Unseal, ", &cost2.as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
            run_data_file.write_all(["vde inv, ", &unseal_vde_cost.to_string(), ", file, ", &unseal_file_cost.to_string(), ", depend, ", &unseal_depend_cost.to_string(), ", hash, ", &unseal_hash_cost.to_string(), ", block, ", &unseal_block_cost.to_string(), ", modsub, ", &unseal_modsub_cost.to_string(), "\n\n"].concat().as_bytes()).unwrap();

            stat_data_file.write_all([
                &params.data_l.to_string(), ", ", &params.block_l.to_string(), ", ", &(params.data_l / params.block_l).to_string(), ", ", &params.unit_l.to_string(), ", ", &(params.block_l / params.unit_l).to_string(), ", ", 
                &params.seal_rounds.to_string(), ", ", &params.vde_mode.to_string(), ", ", &params.mode_l.to_string(), ", ", &params.cnt_l.to_string(), ", ", &params.mode_s.to_string(), ", ", &params.cnt_s.to_string(), ", ",
                &cost1.as_secs_f32().to_string(), ", ",
                &seal_vde_cost.to_string(), ", ", &seal_file_cost.to_string(), ", ", &seal_depend_cost.to_string(), ", ", &seal_hash_cost.to_string(), ", ", &seal_block_cost.to_string(), ", ", &seal_modadd_cost.to_string(), ", ",
                &cost2.as_secs_f32().to_string(), ", ",
                &unseal_vde_cost.to_string(), ", ", &unseal_file_cost.to_string(), ", ", &unseal_depend_cost.to_string(), ", ", &unseal_hash_cost.to_string(), ", ", &unseal_block_cost.to_string(), ", ", &unseal_modsub_cost.to_string(), ",",
                "\n"].concat().as_bytes()).unwrap();
        }
    }
}

pub fn test_unseal_single_and_verify(params: &PosPara, pubdata_path: &str, origin_path: &str, sealed_path: &str, parallel_num: usize) {
    // unseal single
    let (vde_key, iv) = load_data(&pubdata_path);

    let range = (0 * params.block_pl, 10 * params.block_pl);
    let start = Instant::now();
    let (blocks_idx, mut blocks, before_block_ids, depend_blocks) = batch_unseal_prepare(sealed_path, range.0, range.1, params);
    let unsealed_blocks = {
        if parallel_num == 0 {
            batch_unseal(&params, &blocks_idx, &mut blocks, &before_block_ids, &depend_blocks, &vde_key, &iv)
        }
        else {
            batch_unseal_parallel(&params, &blocks_idx, &blocks, &before_block_ids, &depend_blocks, &vde_key, &iv, parallel_num)
        }
    };
    println!("{:?}", start.elapsed());

    for i in 0..blocks_idx.len() {
        batch_verify(origin_path, blocks_idx[i], &unsealed_blocks[i], params.block_l, params.unit_l);
    }
}

pub fn merkle_tree_proof(origin_path: &str, unsealed_path: &str, data_l: usize, block_l: usize, leaves_count: usize) {
    let (_, _, merkle_root) = generate_merkle_tree_from_file(&origin_path, data_l, block_l);
    let (leaves, merkle_tree, _) = generate_merkle_tree_from_file(&unsealed_path, data_l, block_l);
    let indices_to_prove = create_challenges(leaves_count, (0, (data_l / block_l)));
    let proof = generate_merkle_proof(&indices_to_prove, &merkle_tree);
    verify_merkle_proof(proof, merkle_root, &indices_to_prove, &leaves);
}

pub fn test_postorage(params: PosPara, should_save_run_data: bool, should_seal: bool, should_unseal: bool, should_challenge_leaves: bool, should_unseal_single: bool, parallel_num: usize) {
    println!("data len (byte): {:?}", params.data_l);

    // 原始文件所在位置
    let origin_path: PathBuf = ORIGIN_DATA_DIR.iter().collect();
    let origin_path = origin_path.to_str().unwrap();

    // 用来存储seal后的数据
    let sealed_path: PathBuf = SEALED_DATA_DIR.iter().collect();
    let sealed_path = sealed_path.to_str().unwrap();

    // 用来存储unseal后的数据
    let unsealed_path: PathBuf = UNSEALED_DATA_DIR.iter().collect();
    let unsealed_path = unsealed_path.to_str().unwrap();
    
    // 保存 PubData 相关数据
    let pubdata_path: PathBuf = TARGET_DIR.iter().collect();
    let pubdata_path = pubdata_path.to_str().unwrap();

    // 保存实验数据
    let run_data_path: PathBuf = RUN_DATA_DIR.iter().collect();
    let run_data_path = run_data_path.to_str().unwrap();
    let mut run_data_file = OpenOptions::new()
    .read(true)
    .write(true)
    .append(true)
    .create(true) 
    .open(run_data_path)
    .unwrap();

    let stat_data_path: PathBuf = STAT_DATA_DIR.iter().collect();
    let stat_data_path = stat_data_path.to_str().unwrap();
    let mut stat_data_file = OpenOptions::new()
    .read(true)
    .write(true)
    .append(true)
    .create(true) 
    .open(stat_data_path)
    .unwrap();

    const SAMPLES: usize = 1;
    if should_save_run_data == true {
        run_data_file.write_all(["-- SAMPLES, ", &SAMPLES.to_string(), "\n\n"].concat().as_bytes()).unwrap();
        run_data_file.write_all(["(byte) data len, ", &params.data_l.to_string(), ", block len, ", &params.block_l.to_string(), ", block count, ", &(params.data_l / params.block_l).to_string(), ", unit len, ", &params.unit_l.to_string(), ", unit count, ", &(params.block_l / params.unit_l).to_string(), "\n"].concat().as_bytes()).unwrap();
        run_data_file.write_all(["seal round, ", &params.seal_rounds.to_string(), ", vde rounds, ", &params.vde_rounds.to_string(), ", mode l, ", &params.mode_l.to_string(), ", cnt l, ", &params.cnt_l.to_string(), ", mode s, ", &params.mode_s.to_string(), ", cnt s, ", &params.cnt_s.to_string(), "\n\n"].concat().as_bytes()).unwrap();
    }

    for i in 0..SAMPLES {
        println!("sample: {:?}", i);
        if should_seal == true {
            create_random_file(origin_path, params.data_l).unwrap();
            seal_and_unseal(&params, origin_path, sealed_path, unsealed_path, pubdata_path, &mut run_data_file, should_save_run_data, should_unseal, &mut stat_data_file);
        }

        if should_unseal_single == true {
            test_unseal_single_and_verify(&params, pubdata_path, origin_path, sealed_path, parallel_num);
        }

        if should_challenge_leaves == true {
            merkle_tree_proof(origin_path, unsealed_path, params.data_l, params.block_l, params.leaves_to_prove_count);
        }
    }
}

#[test]
fn test() {
    let should_save_run_data = false;

    // 原地 unseal
    // seal = true, unseal = true, else = false, run

    // 并行 unseal single
    // 先 seal = true, else = false, run
    // 再 unseal_single = true, else = false, run

    let should_seal = false;
    let should_unseal = false;
    let should_challenge_leaves = false;
    let should_unseal_single = true;

    let params = gen_posdata(1);
    let parallel_num = 0;

    test_postorage(params, should_save_run_data, should_seal, should_unseal, should_challenge_leaves, should_unseal_single, parallel_num);
}

#[test]
fn test_pipeline() {
    let origin_path: PathBuf = ORIGIN_DATA_DIR.iter().collect();
    let origin_path = origin_path.to_str().unwrap();

    // 用来存储seal后的数据
    let sealed_path: PathBuf = SEALED_DATA_DIR.iter().collect();
    let sealed_path = sealed_path.to_str().unwrap();

    // 用来存储unseal后的数据
    let unsealed_path: PathBuf = UNSEALED_DATA_DIR.iter().collect();
    let unsealed_path = unsealed_path.to_str().unwrap();
    
    // 保存 PubData 相关数据
    let pubdata_path: PathBuf = TARGET_DIR.iter().collect();
    let pubdata_path = pubdata_path.to_str().unwrap();

    // 保存实验数据
    let run_data_path: PathBuf = RUN_DATA_DIR.iter().collect();
    let run_data_path = run_data_path.to_str().unwrap();
    let mut run_data_file = OpenOptions::new()
    .read(true)
    .write(true)
    .append(true)
    .create(true) 
    .open(run_data_path)
    .unwrap();

    // let stat_data_path: PathBuf = STAT_DATA_DIR.iter().collect();
    // let stat_data_path = stat_data_path.to_str().unwrap();
    // let mut stat_data_file = OpenOptions::new()
    // .read(true)
    // .write(true)
    // .append(true)
    // .create(true) 
    // .open(stat_data_path)
    // .unwrap();
    

    // 预先设定参数
    let params = gen_posdata(4);
    let parallel_num = 10;
    let challenges = 10;
    let challenge_single_count = 10;


    // 创建指定长度的原始文件
    create_random_file(origin_path, params.data_l).unwrap();

    // 验证者：构建原始数据merkle树，私有保存root
    let start = Instant::now();
    let (_, _, origin_merkle_root) = generate_merkle_tree_from_file(&origin_path, params.data_l, params.block_l);
    run_data_file.write_all(["[V] Generate origin merkle tree: ", &start.elapsed().as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
    
    // 证明者：seal
    copy_and_pad(origin_path, sealed_path, params.data_l, params.unit_l);
    let (vde_key, iv) = prepare_params(params.unit_pl);
    let start = Instant::now();
    let (blocks_id, seal_vde_cost, _, _, _, _, _) = seal(&params, sealed_path, &vde_key, &iv);
    run_data_file.write_all(["[P] Seal: ", &start.elapsed().as_secs_f32().to_string(), ", Vde: ", &seal_vde_cost.to_string(), "\n"].concat().as_bytes()).unwrap();
    save_data(pubdata_path, &vde_key, &iv);

    // 证明者：对封装完的数据构建merkle树，仅公开root，其他私有保存
    let start = Instant::now();
    let (sealed_leaves, sealed_merkle_tree, sealed_merkle_root) = generate_merkle_tree_from_data(&blocks_id);
    run_data_file.write_all(["[P] Generate sealed merkle tree: ", &start.elapsed().as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
    
    // 短期多次挑战
    for i in 0..challenges {
        run_data_file.write_all(["\n-- CHALLENGE ", &i.to_string(), " --\n\n"].concat().as_bytes()).unwrap();

        // 验证者：随机生成n个挑战及随机数r，并发送给证明者    
        let start = Instant::now();    
        let r = rand::thread_rng().gen::<u8>();
        let indices_to_prove = create_challenges(challenge_single_count, (0, (params.data_l / params.block_l)));
        run_data_file.write_all(["[V] Create challenges: ", &start.elapsed().as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
    
        let mut block_collect = vec![];
        let mut before_block_id_collect = vec![];
        let mut depend_block_collect = vec![];

        let mut response_data = vec![r];

        // 证明者：第一次响应，计算指定数据块的哈希值，并发送给验证者
        let start = Instant::now();
        for &idx2 in &indices_to_prove {
            let (block, before_block_id, depend_block) = single_unseal_prepare(sealed_path, idx2, &params);

            for k in 0..block.len() {
                response_data.append(&mut block[k].clone());
            }
            block_collect.push(block);
            before_block_id_collect.push(before_block_id);
            depend_block_collect.push(depend_block);
        }
        let response_data_hash_p = blake3_hash(&response_data);
        run_data_file.write_all(["[P] Response 1 (hash): ", &start.elapsed().as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
    
        // 验证者：保存 response_data_hash_p
        // 证明者：第二次响应，将 block_collect、before_block_id_collect、depend_block_collect 发送给验证者

        // 证明者：第二次响应，同时将 挑战的叶子结点的验证路径 发送给验证者
        let start = Instant::now();
        let proof = generate_merkle_proof(&indices_to_prove, &sealed_merkle_tree);
        run_data_file.write_all(["[P] Response 2 (create proof): ", &start.elapsed().as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
    
        // 验证者：验证 response_data_hash_p 的正确性
        let start = Instant::now();
        let response_data_hash_v = {
            let mut response_data = vec![r];
            for i in 0..block_collect.len() {
                for k in 0..block_collect[i].len() {
                    response_data.append(&mut block_collect[i][k].clone());
                }
            }
            blake3_hash(&response_data)
        };
        assert_eq!(response_data_hash_p, response_data_hash_v);
        run_data_file.write_all(["[V] Verify hash: ", &start.elapsed().as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
    
        // 验证者：验证验证路径
        let start = Instant::now();
        verify_merkle_proof(proof, sealed_merkle_root, &indices_to_prove, &sealed_leaves);
        run_data_file.write_all(["[V] Verify path: ", &start.elapsed().as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
    
        // 验证者：batch_unseal
        let start = Instant::now();
        let (vde_key, iv) = load_data(&pubdata_path);
        let unsealed_blocks = {
            if parallel_num == 0 {
                batch_unseal(&params, &indices_to_prove, &mut block_collect, &before_block_id_collect, &depend_block_collect, &vde_key, &iv)
            }
            else {
                batch_unseal_parallel(&params, &indices_to_prove, &block_collect, &before_block_id_collect, &depend_block_collect, &vde_key, &iv, parallel_num)
            }
        };
        run_data_file.write_all(["[V] Batch unseal: ", &start.elapsed().as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
    
        // 验证者：batch_verify
        let start = Instant::now();
        for i in 0..indices_to_prove.len() {
            batch_verify(origin_path, indices_to_prove[i], &unsealed_blocks[i], params.block_l, params.unit_l);
        }
        run_data_file.write_all(["[V] Batch verify: ", &start.elapsed().as_secs_f32().to_string(), "\n"].concat().as_bytes()).unwrap();
    }

    // 长期完整unseal
    let start = Instant::now();
    let (unseal_vde_cost, _, _, _, _, _) = unseal(&params, sealed_path, &vde_key, &iv);
    run_data_file.write_all(["\n[V] Complete unseal: ", &start.elapsed().as_secs_f32().to_string(), ", Vde: ", &unseal_vde_cost.to_string(), "\n"].concat().as_bytes()).unwrap();
    copy_and_compress(sealed_path, unsealed_path, params.data_l, params.unit_l, params.unit_pl);
    
    let start = Instant::now();
    let (_, _, unsealed_merkle_root) = generate_merkle_tree_from_file(&unsealed_path, params.data_l, params.block_l);
    assert_eq!(origin_merkle_root, unsealed_merkle_root);
    run_data_file.write_all(["[V] Verify unsealed merkle tree: ", &start.elapsed().as_secs_f32().to_string(), "\n\n\n\n"].concat().as_bytes()).unwrap();
}