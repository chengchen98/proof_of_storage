use ark_bls12_381::Fr;
use num_bigint::BigUint;
use std::{fs::OpenOptions, io::{Write, Seek, SeekFrom}};
use ark_ff::{BigInteger256, BigInteger, PrimeField};

use crate::vde::vde::vde;
use crate::common::mimc_hash::multi_mimc5_hash;

use super::common::{read_file, to_block, com_block, vecu8_xor};
use super::postorage::{DATA_L, L2, L1, L0, PL2, PL1, PL0, SEAL_ROUNDS};

pub fn copy_and_pad(origin_path: &str, new_path: &str) {
    //! 将原始文件按照 L0 大小逐个pad（在高位添加一个 0），再存储到新文件
    let mut origin_file = OpenOptions::new()
    .read(true)
    .open(origin_path)
    .unwrap();

    let mut new_file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .truncate(true)
    .open(new_path)
    .unwrap();

    let block_cnt = DATA_L / L0;
    for cnt in 0..block_cnt {
        let mut buf = read_file(&mut origin_file, cnt * L0, L0);
        buf.push(0);
        new_file.write_all(&buf).unwrap();
    }
}

pub fn seal(path: &str, idx_l: &Vec<Vec<Vec<usize>>>, idx_s: &Vec<Vec<Vec<usize>>>, hash_cts: &Vec<Fr>, hash_key: Fr, vde_key: &BigUint, vde_mode: &str) {
    // 原始文件按照 L2 长度分块的个数，即原始数据中二级数据块的个数
    let l2_cnt = DATA_L / L2;
    // 一个 L2 块按照 L1 长度分块的个数，即每个二级数据块中一级数据块的个数
    let l1_cnt = L2 / L1;

    let mut file = OpenOptions::new()
    .read(true)
    .write(true)
    .open(path)
    .unwrap();

    for _ in 0..SEAL_ROUNDS {
        for cnt2 in 0..l2_cnt {
            let mut block = {
                let buf = read_file(&mut file, cnt2 * PL2, PL2);
                to_block(&buf, PL1)
            };

            for cnt1 in 0..l1_cnt {
                // 获取长程和短程依赖数据块
                let mut depend_data = vec![];
                {
                    for &idx in &idx_l[cnt2][cnt1] {
                        let mut buf = read_file(&mut file, idx * PL2 + cnt1 * PL1, PL1);
                        depend_data.append(&mut buf);
                    }

                    for &idx in &idx_s[cnt2][cnt1] {
                        depend_data.append(&mut block[idx].clone());
                    }
                }
            
                // 计算依赖数据块的哈希
                let depend_data_hash = {
                    let mut x_input = vec![];
                    for i in (0..depend_data.len()).step_by(32) {
                        let x_in = Fr::from_le_bytes_mod_order(&depend_data[i .. i + 32]);
                        x_input.push(x_in);
                    }
                    let res = multi_mimc5_hash(&x_input, hash_key, &hash_cts);
                    let res: BigInteger256 = res.into();
                    res.to_bytes_le()
                };
    
                let cur_block = &block[cnt1].to_vec();
                let block_xor = vecu8_xor(&depend_data_hash, &cur_block)[..PL1].to_vec();
                let new_block = {
                    let mut res = vec![];
                    for idx in (0..PL1).step_by(PL0) {
                        let input = block_xor[idx .. idx + PL0].to_vec();
                        let mut vde_res = vde(&input, vde_key, &vde_mode);
                        res.append(&mut vde_res);
                    }
                    res
                };

                block[cnt1] = new_block;
            }

            let block = com_block(&block);
            file.seek(SeekFrom::Start((cnt2 * PL2).try_into().unwrap())).unwrap();
            file.write_all(&block).unwrap();
        }
    }
}

