use rand::Rng;
use ark_bls12_381::Fr;
use std::str::FromStr;
use std::time::Instant;
use num_bigint::BigUint;

use super::seal::{copy_and_pad, seal};
use super::unseal::{unseal, copy_and_compress};
use super::merkle_tree::{generate_merkle_tree, generate_merkle_proof, verify_merkle_proof};
use super::common::{create_random_file, create_depend, generate_sorted_unique_random_numbers};

pub const ORIGIN_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "origin_data"];
pub const SEALED_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "sealed_data"];
pub const UNSEALED_DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "unsealed_data"];

// 单位：byte
pub const DATA_L: usize = 127 * 1024; // 127KB
pub const DATA_PL: usize = 128 * 1024; // 128KB

pub const L0: usize = 127;
pub const L1: usize = 8 * L0; // 1KB
pub const L2: usize = 8 * L1; // 8KB

pub const PL0: usize = L0 + 1;
pub const PL1: usize = 8 * PL0;
pub const PL2: usize = 8 * PL1;

pub const SEAL_ROUNDS: usize = 3;
pub const MIMC5_HASH_ROUNDS: usize = 110;
pub const VDE_MODE: &str = "sloth";

pub const MODE_L: usize = 1;
pub const MODE_S: usize = 1;
pub const IDX_CNT_L: usize = 3;
pub const IDX_CNT_S: usize = 3;

pub const ORIGIN_LEAVES_COUNT: usize = 10;
pub const SEALED_LEAVES_COUNT: usize = 10;

pub fn prepare_parms() -> (Fr, Vec<Fr>, BigUint, Vec<Vec<Vec<usize>>>, Vec<Vec<Vec<usize>>>) {
    // 准备存储证明所需要的参数
    let mut rng = rand::thread_rng();

    // hash
    let hash_key = rng.gen();
    let hash_constants = (0..MIMC5_HASH_ROUNDS)
        .map(|_| rng.gen())
        .collect::<Vec<_>>();

    // vde
    let vde_key = BigUint::from_str("114814770432560997405734776484772649052342276989403295241799079775037429533136378312048423016912262690538326297849170244315795472085749637431992466797211295067626594614696499619970513658328822796782917146952131671224321524616875912503115070404484695218722843806545049037501838881300899940575739494347050015467").unwrap();

    // 生成数据块依赖关系
    let (idx_l, idx_s) = create_depend(IDX_CNT_L, IDX_CNT_S, MODE_L, MODE_S);   

    (hash_key, hash_constants, vde_key, idx_l, idx_s)
}

pub fn postorage(origin_path: &str, sealed_path: &str, unsealed_path: &str, hash_key: Fr, hash_constants: &Vec<Fr>, vde_key: &BigUint, idx_l: Vec<Vec<Vec<usize>>>, idx_s: Vec<Vec<Vec<usize>>>) {

    // 随机创建原始数据文件
    create_random_file(origin_path).unwrap();
    
    // Seal
    copy_and_pad(origin_path, sealed_path);

    let start = Instant::now();
    seal(sealed_path, &idx_l, &idx_s, &hash_constants, hash_key, &vde_key);
    println!("Seal: {:?}", start.elapsed());

    // Unseal
    let start = Instant::now();
    unseal(sealed_path, &idx_l, &idx_s, &hash_constants, hash_key, &vde_key);
    println!("Unseal: {:?}", start.elapsed());

    copy_and_compress(sealed_path, unsealed_path);
}

pub fn merkle_tree(path: &str, data_len: usize, leaf_len: usize, leaves_to_prove_count: usize) {
    // 生成merkle tree，并随机选取n个叶子结点进行验证
    let (leaves, merkle_tree, merkle_root) = generate_merkle_tree(&path, data_len, leaf_len);
    println!("leaf number: {:?}  | tree depth: {:?}  |  prove count: {:?}", leaves.len(), merkle_tree.depth(), leaves_to_prove_count);

    let indices_to_prove = generate_sorted_unique_random_numbers(leaves_to_prove_count, (0, leaves.len()));
    println!("indices to prove: {:?}", indices_to_prove);
    
    let proof = generate_merkle_proof(&indices_to_prove, merkle_tree);
    verify_merkle_proof(proof, merkle_root, &indices_to_prove, leaves);
}


#[test]
fn test() {
    use std::path::PathBuf;

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

    const SAMPLES: usize = 10;
    for i in 0..SAMPLES {
        println!("Sample: {:?}", i);
        // let (hash_key, hash_constants, vde_key, idx_l, idx_s) = prepare_parms();
        // postorage(origin_path, sealed_path, unsealed_path, hash_key, &hash_constants, &vde_key, idx_l, idx_s);
        merkle_tree(origin_path, DATA_L, L1, ORIGIN_LEAVES_COUNT);
        merkle_tree(sealed_path, DATA_PL, PL1, SEALED_LEAVES_COUNT);
        println!("----------------------------------------------------------------");
    }
}