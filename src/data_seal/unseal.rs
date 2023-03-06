use ark_bls12_381::Fr;
use num_bigint::BigUint;
use std::{fs::File, io::{Write, Seek, SeekFrom}};
use ark_ff::{BigInteger256, BigInteger, PrimeField};

use crate::common::data::vecu8_xor;
use crate::common::mimc_hash::multi_mimc5_hash;
use crate::vde::vde::vde_inv;

use super::common::{read_file, to_block, com_block, compress_block};
use super::data_seal::{L2, L1, SEAL_ROUNDS};

pub fn unseal_1(sealed_file: &mut File, block_num2: usize, block_num1: usize, idx_l: &Vec<Vec<Vec<usize>>>, idx_s: &Vec<Vec<Vec<usize>>>, constants: &Vec<Fr>, hash_key: Fr, vde_key: &BigUint, vde_mode: &str) {
    //! 数据解封装，逆向解码，直接在sealed_file上原地解码即可

    for _ in 1..SEAL_ROUNDS {

        for bn2 in 0..block_num2 {
            let i = block_num2 - 1 - bn2;
            
            let mut block2 = {
                let buf = read_file(sealed_file, i * block_num1 * (L1 + 1), block_num1 * (L1 + 1));
                to_block(&buf, L1 + 1)
            };
    
            for bn1 in 0..block_num1 {
                let j = block_num1 - 1 - bn1;
    
                // 收集依赖块的数据
                let mut depend_data = vec![];
                {
                    for &k in &idx_l[i][j] {
                        let mut block1 = read_file(sealed_file, k * block_num1 * (L1 + 1) + j * (L1 + 1), L1 + 1);
                        depend_data.append(&mut block1);
                    }
    
                    for &k in &idx_s[i][j] {
                        depend_data.append(&mut block2[k].clone());
                    }
                }
    
                assert_eq!(depend_data.len() % 32, 0);
                
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
                let cur_block = &block2[j].to_vec();
                // 延迟编码的逆
                let vde_res = vde_inv(&cur_block, vde_key, &vde_mode);
                // 异或
                let new_block = vecu8_xor(&vde_res, &depend_data_hash)[0..vde_res.len()].to_vec();
                
                // 更新当前一级数据块
                for idx in 0 .. new_block.len() {
                    block2[j][idx] = new_block[idx];
                }

                if new_block.len() < L1 + 1 {
                    block2[j][L1] = 0;
                }
            }
            // 若当前不是最后一轮，原地操作
            let block2 = com_block(&block2);
            sealed_file.seek(SeekFrom::Start((i * block_num1 * (L1 + 1)).try_into().unwrap())).unwrap();
            sealed_file.write_all(&block2).unwrap();
        }
    }
}

pub fn unseal_0(sealed_file: &mut File, unsealed_file: &mut File, block_num2: usize, block_num1: usize, idx_l: &Vec<Vec<Vec<usize>>>, idx_s: &Vec<Vec<Vec<usize>>>, constants: &Vec<Fr>, hash_key: Fr, vde_key: &BigUint, vde_mode: &str) {
    //! 数据解封装，逆向解码，直接在sealed_file上原地解码即可

    for bn2 in 0..block_num2 {
        let i = block_num2 - 1 - bn2;
        
        let mut block2 = {
            let buf = read_file(sealed_file, i * block_num1 * (L1 + 1), block_num1 * (L1 + 1));
            to_block(&buf, L1 + 1)
        };

        for bn1 in 0..block_num1 {
            let j = block_num1 - 1 - bn1;

            // 收集依赖块的数据
            let mut depend_data = vec![];
            {
                for &k in &idx_l[i][j] {
                    let mut buf = {
                        if k >= i {
                            let block = read_file(sealed_file, k * block_num1 * (L1 + 1) + j * (L1 + 1), L1 + 1);
                            block
                        }
                        else {
                            let mut block = read_file(unsealed_file, k * L2 + j * L1, L1);
                            block.push(0);
                            block
                        }
                    };
                    depend_data.append(&mut buf);
                }

                for &k in &idx_s[i][j] {
                    depend_data.append(&mut block2[k].clone());
                }
            }

            assert_eq!(depend_data.len() % 32, 0);
            
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
            let cur_block = &block2[j].to_vec();
            // 延迟编码的逆
            let vde_res = vde_inv(&cur_block, vde_key, &vde_mode);
            // 异或
            let new_block = vecu8_xor(&vde_res, &depend_data_hash)[0..vde_res.len()].to_vec();
            
            // 更新当前一级数据块
            for idx in 0 .. new_block.len() {
                block2[j][idx] = new_block[idx];
            }

            if new_block.len() < L1 + 1 {
                block2[j][L1] = 0;
            }
        }

        // 更新当前二级数据块
        let block2 = compress_block(&block2);
        let block2 = com_block(&block2);
        unsealed_file.seek(SeekFrom::Start((i * L2).try_into().unwrap())).unwrap();
        unsealed_file.write_all(&block2).unwrap();
    }
}