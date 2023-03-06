use ark_bls12_381::Fr;
use num_bigint::BigUint;
use std::{fs::File, io::{Write, Seek, SeekFrom}};
use ark_ff::{BigInteger256, BigInteger, PrimeField};

use crate::common::data::vecu8_xor;
use crate::common::mimc_hash::multi_mimc5_hash;
use crate::vde::vde::vde;

use super::common::{read_file, to_block, com_block, pad_block};
use super::data_seal::{L2, L1, SEAL_ROUNDS};

pub fn seal_0(origin_file: &mut File, sealed_file: &mut File, block_num2: usize, block_num1: usize, idx_l: &Vec<Vec<Vec<usize>>>, idx_s: &Vec<Vec<Vec<usize>>>, constants: &Vec<Fr>, hash_key: Fr, vde_key: &BigUint, vde_mode: &str) {
    //! round = 0，即做第一轮编码时，需要从origin_file中读取数据块，并将seal完的数据存入新的文件sealed_file中

    for i in 0..block_num2 {
        // 读取当前二级数据块，并将其分成多个一级数据块
        let mut block2 = {
            let buf = read_file(origin_file, i * L2, L2);
            let block = to_block(&buf, L1);
            pad_block(&block)
        };

        for j in 0..block_num1 {
            // 收集依赖块的数据
            let mut depend_data = vec![];
            {
                for &k in &idx_l[i][j] {
                    let mut buf = {
                        if k >= i {
                            let mut block = read_file(origin_file, k * L2 + j * L1, L1);
                            block.push(0);
                            block
                        }
                        else {
                            let block = read_file(sealed_file, k * block_num1 * (L1 + 1) + j * (L1 + 1), L1 + 1);
                            block
                        }
                    };
                    depend_data.append(&mut buf);
                }

                for &k in &idx_s[i][j] {
                    // 错误写法：depend_data.append(&mut block2[k]);
                    // 不写clone的话，会将 block2[k] 的内容清空
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
            // 异或
            let data_xor = vecu8_xor(&depend_data_hash, &cur_block)[..L1].to_vec();
            // 延迟编码
            let new_block = vde(&data_xor, vde_key, &vde_mode);
            
            // 更新当前一级数据块
            for idx in 0 .. new_block.len() {
                block2[j][idx] = new_block[idx];
            }

            if new_block.len() < L1 + 1 {
                block2[j][L1] = 0;
            }
        }

        let block2 = com_block(&block2);

        // 更新当前二级数据块，并写入文件
        sealed_file.seek(SeekFrom::Start((i * block_num1 * (L1 + 1)).try_into().unwrap())).unwrap();
        sealed_file.write_all(&block2).unwrap();
    }
}

pub fn seal_1(sealed_file: &mut File, block_num2: usize, block_num1: usize, idx_l: &Vec<Vec<Vec<usize>>>, idx_s: &Vec<Vec<Vec<usize>>>, constants: &Vec<Fr>, hash_key: Fr, vde_key: &BigUint, vde_mode: &str) {
    //! round > 0，从第二轮编码开始，只需要在seal_file中读取数据并做原地处理即可

    for _ in 1..SEAL_ROUNDS {
        for i in 0..block_num2 {
            // 读取当前二级数据块，并将其分成多个一级数据块
            let mut block2 = {
                let buf = read_file(sealed_file, i * block_num1 * (L1 + 1), block_num1 * (L1 + 1));
                to_block(&buf, L1 + 1)
            };
    
            for j in 0..block_num1 {
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
                // 异或
                let data_xor = vecu8_xor(&depend_data_hash, &cur_block)[..L1].to_vec();
                // 延迟编码
                let new_block = vde(&data_xor, vde_key, &vde_mode);
                
                // 更新当前一级数据块
                for idx in 0 .. new_block.len() {
                    block2[j][idx] = new_block[idx];
                }

                if new_block.len() < L1 + 1 {
                    block2[j][L1] = 0;
                }
            }
    
            let block2 = com_block(&block2);
    
            // 更新当前二级数据块，并写入文件
            sealed_file.seek(SeekFrom::Start((i * block_num1 * (L1 + 1)).try_into().unwrap())).unwrap();
            sealed_file.write_all(&block2).unwrap();
        }
    }
}