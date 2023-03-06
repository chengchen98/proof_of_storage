use num_bigint::BigUint;
use rand::Rng;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;

use super::common::{create_random_file, create_depend};
use super::seal::{seal_0, seal_1};
use super::unseal::{unseal_0, unseal_1};

const ORIGIN_DATA_DIR: [&str; 4] = [r"src", "data_seal", "data", "origin_data"];
const SEALED_DATA_DIR: [&str; 4] = [r"src", "data_seal", "data", "sealed_data"];
const UNSEALED_DATA_DIR: [&str; 4] = [r"src", "data_seal", "data", "unsealed_data"];

// 单位：bytes
pub const DATA_L: usize = 128 * 127;
pub const L2: usize = 16 * 127;
pub const L1: usize = 127;
pub const SEAL_ROUNDS: usize = 3;
pub const MIMC5_HASH_ROUNDS: usize = 110;

pub fn test_deal_seal() {
    let origin_path: PathBuf = ORIGIN_DATA_DIR.iter().collect();
    let origin_path = origin_path.to_str().unwrap();

    let sealed_path: PathBuf = SEALED_DATA_DIR.iter().collect();
    let sealed_path = sealed_path.to_str().unwrap();

    let unsealed_path: PathBuf = UNSEALED_DATA_DIR.iter().collect();
    let unsealed_path = unsealed_path.to_str().unwrap();

    // 随机创建原始数据文件
    let mut origin_file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .truncate(true)
    .open(origin_path)
    .unwrap();
    create_random_file(&mut origin_file).unwrap();

    // 原始文件可以分为几个二级数据块
    let block_num2 = DATA_L / L2;
    // 每个二级数据块又可以分为几个一级数据块
    let block_num1 = L2 / L1;

    // 依赖数据块的个数
    let idx_cnt_l: usize = 3;
    let idx_cnt_s: usize = 3;

    let mode_l: usize = 1;
    let mode_s: usize = 1;

    let mut rng = rand::thread_rng();
    let constants = (0..MIMC5_HASH_ROUNDS)
        .map(|_| rng.gen())
        .collect::<Vec<_>>();

    let vde_mode: &str = "sloth";
    let vde_key = BigUint::from_str("162892692473361111348249522320347526171207756760381512377472315857021028422689815298733972916755720242725920671690392382889161699607077776923153532250584503438515484867646456937083398184988196288738761695736655551130641678117347468224388930820784409522417624141309667471624562708798367178136545063034409853007").unwrap();
    let hash_key = rng.gen();

    // 生成数据块依赖关系
    let (idx_l, idx_s) = create_depend(block_num2, block_num1, idx_cnt_l, idx_cnt_s, mode_l, mode_s);   

    // 以只读方式打开原始数据文件
    let mut origin_file = OpenOptions::new()
    .read(true)
    .open(origin_path)
    .unwrap();
    
    // 创建新的文件，用来存储seal后的数据
    let mut sealed_file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .truncate(true)
    .open(sealed_path)
    .unwrap();

    // Seal_0
    let start = Instant::now();
    seal_0(&mut origin_file, &mut sealed_file, block_num2, block_num1, &idx_l, &idx_s, &constants, hash_key, &vde_key, vde_mode);
    println!("Seal_0: {:?}", start.elapsed());

    // Seal_1
    seal_1(&mut sealed_file, block_num2, block_num1, &idx_l, &idx_s, &constants, hash_key, &vde_key, vde_mode);
    println!("Seal_1: {:?}", start.elapsed());

    // 创建新的文件，用来存储unseal后的数据
    let mut unsealed_file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .truncate(true)
    .open(unsealed_path)
    .unwrap();

    // Unseal
    let start = Instant::now();
    unseal_1(&mut sealed_file, block_num2, block_num1, &idx_l, &idx_s, &constants, hash_key, &vde_key, vde_mode);
    println!("Unseal: {:?}", start.elapsed());

    let start = Instant::now();
    unseal_0(&mut sealed_file, &mut unsealed_file, block_num2, block_num1, &idx_l, &idx_s, &constants, hash_key, &vde_key, vde_mode);
    println!("Unseal: {:?}", start.elapsed());
}

#[test]
fn test() {
    test_deal_seal();
}