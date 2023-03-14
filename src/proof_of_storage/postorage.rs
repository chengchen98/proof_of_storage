use rand::Rng;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use num_bigint::BigUint;

use super::merkle_tree::{gen_merkle_tree, gen_merkle_proof, verify_merkle_proof};
use super::seal::{copy_and_pad, seal};
use super::unseal::{unseal, copy_and_compress};
use super::common::{create_random_file, create_depend, generate_sorted_unique_random_numbers};

const ORIGIN_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "origin_data"];
const SEALED_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "sealed_data"];
const UNSEALED_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "unsealed_data"];

// 单位：byte
pub const DATA_L: usize = 1 * 127 * 1024; // 128KB

pub const L0: usize = 127;
pub const L1: usize = 8 * L0; // 1KB
pub const L2: usize = 8 * L1; // 8KB

pub const PL0: usize = L0 + 1;
pub const PL1: usize = 8 * PL0;
pub const PL2: usize = 8 * PL1;

pub const SEAL_ROUNDS: usize = 3;
pub const MIMC5_HASH_ROUNDS: usize = 110;

pub const VDE_MODE: &str = "sloth";
pub const LEAVES_COUNT: usize = 10;

pub fn test_postorage() {
    
    // 随机创建原始数据文件
    let origin_path: PathBuf = ORIGIN_DATA_DIR.iter().collect();
    let origin_path = origin_path.to_str().unwrap();
    create_random_file(origin_path).unwrap();
    
    // 创建新的文件，用来存储seal后的数据
    let sealed_path: PathBuf = SEALED_DATA_DIR.iter().collect();
    let sealed_path = sealed_path.to_str().unwrap();

    // 创建新的文件，用来存储unseal后的数据
    let unsealed_path: PathBuf = UNSEALED_DATA_DIR.iter().collect();
    let unsealed_path = unsealed_path.to_str().unwrap();

    // 长程和短程依赖数据块的个数
    let idx_cnt_l: usize = 3;
    let idx_cnt_s: usize = 3;

    // 依赖模式选择
    let mode_l: usize = 1;
    let mode_s: usize = 1;

    let mut rng = rand::thread_rng();

    // hash
    let hash_key = rng.gen();
    let hash_cts = (0..MIMC5_HASH_ROUNDS)
        .map(|_| rng.gen())
        .collect::<Vec<_>>();

    // vde
    let vde_key = BigUint::from_str("114814770432560997405734776484772649052342276989403295241799079775037429533136378312048423016912262690538326297849170244315795472085749637431992466797211295067626594614696499619970513658328822796782917146952131671224321524616875912503115070404484695218722843806545049037501838881300899940575739494347050015467").unwrap();

    // 生成数据块依赖关系
    let (idx_l, idx_s) = create_depend(idx_cnt_l, idx_cnt_s, mode_l, mode_s);   

    println!("data len (byte): {:?}  | block2 len: {:?}  | block1 len: {:?}", DATA_L, L2, L1);
    println!("-------------------------------------");

    // Seal
    copy_and_pad(origin_path, sealed_path);

    let start = Instant::now();
    seal(sealed_path, &idx_l, &idx_s, &hash_cts, hash_key, &vde_key);
    println!("Seal: {:?}", start.elapsed());

    // Unseal
    let start = Instant::now();
    unseal(sealed_path, &idx_l, &idx_s, &hash_cts, hash_key, &vde_key);
    println!("Unseal: {:?}", start.elapsed());
    println!("-------------------------------------");

    copy_and_compress(sealed_path, unsealed_path);
}

pub fn merkle_tree() {
    let path: PathBuf = ORIGIN_DATA_DIR.iter().collect();
    let path = path.to_str().unwrap();

    let (leaves, merkle_tree, merkle_root) = gen_merkle_tree(&path);
    println!("leaves number: {:?}  | tree depth: {:?}  |  prove count: {:?}", leaves.len(), merkle_tree.depth(), leaves.len());

    let indices_to_prove = generate_sorted_unique_random_numbers(LEAVES_COUNT, (0, leaves.len()));
    println!("indices to prove: {:?}", indices_to_prove);
    
    let proof = gen_merkle_proof(&indices_to_prove, merkle_tree);
    verify_merkle_proof(proof, merkle_root, &indices_to_prove, leaves);
    println!("-------------------------------------");
}


#[test]
fn test() {
    const SAMPLES: usize = 10;
    for i in 0..SAMPLES {
        println!("Sample: {:?}", i);
        // test_postorage();
        merkle_tree();
    }
}