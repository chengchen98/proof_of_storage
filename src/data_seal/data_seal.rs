use std::fs::File;
use std::io::{Seek, Read, SeekFrom, Write};

use ark_bls12_381::Fr;
use ark_ff::{BigInteger256, BigInteger, PrimeField};
use num_bigint::BigUint;
use std::{fs::OpenOptions, str::FromStr, time::Instant};
use rand::Rng;

use super::depend::{long_depend, short_depend};
use crate::common::data::vecu8_xor;
use crate::common::mimc_hash::multi_mimc5_hash;
use crate::vde::vde::{vde, vde_inv};
use std::path::PathBuf; 

const DATA_DIR: [&str; 3] = [r"src", "data_seal", "seal_data"]; 

// 单位：bytes
const DATA_L: usize = 128 * 1024;
const L2: usize = 16 * 1024;
const L1: usize = 1024;
const SEAL_ROUNDS: usize = 1;
const MIMC5_HASH_ROUNDS: usize = 110;

pub fn create_random_file(file: &mut File) -> std::io::Result<()> {
    //! 假设原始数据大小为DATA_L字节，随机生成文件保存
    let mut rng = rand::thread_rng();
    for i in 0..DATA_L {
        let buf: [u8; 1] = [rng.gen_range(0u8..=255u8)];
        file.seek(SeekFrom::Start(i.try_into().unwrap())).unwrap();
        file.write_all(&buf).unwrap();
    }
    Ok(())
}

pub fn create_depend(block_num2: usize, block_num1: usize, idx_count_l: usize, idx_count_s: usize, mode_l: usize, mode_s: usize) -> (Vec<Vec<Vec<usize>>>, Vec<Vec<Vec<usize>>>) {
    //! 创建数据块之间的依赖关系，包括短程依赖和长程依赖
    let mut idx_l = vec![];
    let mut idx_s = vec![];

    for i in 0..block_num2 {
        let mut cur_idx_l = vec![];
        let mut cur_idx_s = vec![];
        for j in 0..block_num1 {
            cur_idx_l.push(long_depend(i, idx_count_l, mode_l));
            cur_idx_s.push(short_depend(block_num1, j, idx_count_s, mode_s));
        }
        idx_l.push(cur_idx_l);
        idx_s.push(cur_idx_s);
    }

    (idx_l, idx_s)
}

pub fn seal(file: &mut File, block_num2: usize, block_num1: usize, idx_l: &Vec<Vec<Vec<usize>>>, idx_s: &Vec<Vec<Vec<usize>>>, constants: &Vec<Fr>, hash_key: Fr, vde_key: &BigUint, vde_mode: &str) {
    //! 数据封装，正向编码
    for _ in 0..SEAL_ROUNDS {
        // 每次从硬盘中取出一个二级数据块到内存
        let mut block2 = [0; L2];
        // 用来从硬盘读取长程依赖
        let mut buf = [0; L1];

        for i in 0..block_num2 {
            // 读取当前二级数据块
            file.seek(SeekFrom::Start((i * L2).try_into().unwrap())).unwrap();
            file.read(&mut block2).unwrap();

            for j in 0..block_num1 {
                // 收集依赖块的数据
                let mut depend_data = vec![];
                {
                    for &k in &idx_l[i][j] {
                        file.seek(SeekFrom::Start((k * L2 + j * L1).try_into().unwrap())).unwrap();
                        file.read(&mut buf).unwrap();
                        depend_data.append(&mut buf.to_vec());
                    }
    
                    for &k in &idx_s[i][j] {
                        depend_data.append(&mut block2[k * L1 .. k * L1 + L1].to_vec());
                    }
                }

                // 计算依赖数据块的哈希
                let depend_data_hash = {
                    let mut x_input = vec![];
                    for i in (0..depend_data.len()).step_by(32) {
                        let x_in = Fr::from_le_bytes_mod_order(&depend_data[i .. i + 32]);
                        x_input.push(x_in);
                    }
                    let res = multi_mimc5_hash(&x_input, hash_key, &constants);
                    let res: BigInteger256 = res.into();
                    res.to_bytes_le()
                };

                // 取出当前一级数据块
                let cur_block = &block2[j * L1 .. j * L1 + L1].to_vec();
                // 异或
                let data_xor = vecu8_xor(&depend_data_hash, &cur_block)[..L1].to_vec();
                // 延迟编码
                let new_block = vde(&data_xor, vde_key, &vde_mode);
                
                // 更新当前一级数据块
                for idx in 0 .. L1 {
                    block2[idx + j * L1] = new_block[idx];
                }
            }

            // 更新当前二级数据块，并写入文件
            file.seek(SeekFrom::Start((i * L2).try_into().unwrap())).unwrap();
            file.write_all(&block2).unwrap();
        }
    }
}

pub fn unseal(file: &mut File, block_num2: usize, block_num1: usize, idx_l: &Vec<Vec<Vec<usize>>>, idx_s: &Vec<Vec<Vec<usize>>>, constants: &Vec<Fr>, hash_key: Fr, vde_key: &BigUint, vde_mode: &str) {
    //! 数据解封装，逆向解码
    for _ in 0..SEAL_ROUNDS {
        // 每次从硬盘中取出一个二级数据块到内存
        let mut block2 = [0; L2];
        // 用来从硬盘读取长程依赖
        let mut buf = [0; L1];

        for bn2 in 0..block_num2 {
            let i = block_num2 - 1 - bn2;

            // 读取当前二级数据块
            file.seek(SeekFrom::Start((i * L2).try_into().unwrap())).unwrap();
            file.read(&mut block2).unwrap();

            for bn1 in 0..block_num1 {
                let j = block_num1 - 1 - bn1;

                // 收集依赖块的数据
                let mut depend_data = vec![];
                {
                    for &k in &idx_l[i][j] {
                        file.seek(SeekFrom::Start((k * L2 + j * L1).try_into().unwrap())).unwrap();
                        file.read(&mut buf).unwrap();
                        depend_data.append(&mut buf.to_vec());
                    }
    
                    for &k in &idx_s[i][j] {
                        depend_data.append(&mut block2[k * L1 .. k * L1 + L1].to_vec());
                    }
                }
                
                // 计算依赖数据块的哈希
                let depend_data_hash = {
                    let mut x_input = vec![];
                    for i in (0..depend_data.len()).step_by(32) {
                        let x_in = Fr::from_le_bytes_mod_order(&depend_data[i .. i + 32]);
                        x_input.push(x_in);
                    }
                    let res = multi_mimc5_hash(&x_input, hash_key, &constants);
                    let res: BigInteger256 = res.into();
                    res.to_bytes_le()
                };

                // 取出当前一级数据块
                let cur_block = &block2[j * L1 .. j * L1 + L1].to_vec();
                // 延迟编码的逆
                let vde_res = vde_inv(&cur_block, vde_key, &vde_mode);
                // 异或
                let new_block = vecu8_xor(&vde_res, &depend_data_hash)[0..L1].to_vec();
                
                // 更新当前一级数据块
                for idx in 0 .. L1 {
                    block2[j * L1 + idx] = new_block[idx];
                }
            }
            // 更新当前二级数据块，并写入文件
            file.seek(SeekFrom::Start((i * L2).try_into().unwrap())).unwrap();
            file.write_all(&block2).unwrap();
        }
    }
}

pub fn test_deal_seal() {
    let path: PathBuf = DATA_DIR.iter().collect();

    let mut file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    // .truncate(true)
    .open(path.to_str().unwrap())
    .unwrap();

    // create_random_file(&mut file).unwrap();

    // 原始文件可以分为几个二级数据块
    let block_num2 = DATA_L / L2;
    // 每个二级数据块又可以分为几个一级数据块
    let block_num1 = L2 / L1;

    // 依赖数据块的个数
    let idx_count_l: usize = 3;
    let idx_count_s: usize = 3;

    let mode_l: usize = 1;
    let mode_s: usize = 1;

    let mut rng = rand::thread_rng();
    let constants = (0..MIMC5_HASH_ROUNDS)
        .map(|_| rng.gen())
        .collect::<Vec<_>>();

    let vde_mode: &str = "sloth";
    let vde_key = BigUint::from_str("276945728797634137489847193533935566200901110872557999805088095083433912915081929876610085556888176394277441945470579512610156696848456080099840453319124321877455883488948246054067984322844955398390786946509577100886479649428068281092367813035032036823204874960913543086692263648390252658950393200040464000839").unwrap();
    let hash_key = rng.gen();

    // 生成数据块依赖关系
    let (idx_l, idx_s) = create_depend(block_num2, block_num1, idx_count_l, idx_count_s, mode_l, mode_s);   

    // Seal
    let start = Instant::now();
    seal(&mut file, block_num2, block_num1, &idx_l, &idx_s, &constants, hash_key, &vde_key, vde_mode);
    println!("Seal: {:?}", start.elapsed());

    // Unseal
    let start = Instant::now();
    unseal(&mut file, block_num2, block_num1, &idx_l, &idx_s, &constants, hash_key, &vde_key, vde_mode);
    println!("Unseal: {:?}", start.elapsed());
}

#[test]
fn test() {
    test_deal_seal();
}