use std::fs::File;
use std::io::{Seek, Read, SeekFrom, Write};

use ark_bls12_381::Fr;
use ark_ff::{PrimeField, BigInteger256, BigInteger};
use num_bigint::BigUint;

use super::depend::{long_depend, short_depend};
use crate::common::data::vecu8_xor;
use crate::common::mimc_hash::multi_mimc7_hash;
use crate::vde::vde::{vde, vde_inv};

pub const DATA_DIR: &str = r"src\data_seal\seal_data.txt";

// the unit of len is byte
pub const DATA_L: usize = 256 * 1024 * 1024; // 256 MB
pub const L2: usize = 32 * 1024 * 1024; // 32 MB
pub const L1: usize = 1024 * 1024; // 1 MB
pub const SEAL_ROUND: usize = 3;
pub const MIMC_HASH_ROUNDS: usize = 10;

pub fn create_depend(block_num2: usize, block_num1: usize, index_count_l: usize, index_count_s: usize, mode_l: usize, mode_s: usize) -> (Vec<Vec<Vec<usize>>>, Vec<Vec<Vec<usize>>>) {
    //! Collect all the depended indexs of every data blocks.
    let mut index_l_collect = vec![];
    let mut index_s_collect = vec![];

    for i in 0..block_num2 {
        let mut index_l = vec![];
        let mut index_s = vec![];
        for j in 0..block_num1 {
            index_l.push(long_depend(block_num2, i, index_count_l, mode_l));
            index_s.push(short_depend(block_num1, j, index_count_s, mode_s));
        }
        index_l_collect.push(index_l);
        index_s_collect.push(index_s);
    }

    (index_l_collect, index_s_collect)
}

pub fn seal(round: usize, file: &mut File, block_num2: usize, block_num1: usize, index_l_collect: &Vec<Vec<Vec<usize>>>, index_s_collect: &Vec<Vec<Vec<usize>>>, constants: &Vec<Fr>, hash_key: Fr, vde_key: &BigUint, vde_mode: &str) {
    //! Seal data block by block for n round.
    for _ in 0..round {
        let mut buf = [0; L1];

        for i in 0..block_num2 {
            for j in 0..block_num1 {
                // collect the depended data
                let mut depend_data = vec![];
                {
                    for k in 0..index_l_collect[i][j].len() {
                        // move the file pointer
                        file.seek(SeekFrom::Start((k * L2 + j * L1).try_into().unwrap())).unwrap();
                        file.read(&mut buf).unwrap();
                        depend_data.append(&mut buf.to_vec());
                    }
    
                    for k in 0..index_s_collect[i][j].len() {
                        file.seek(SeekFrom::Start((i * L2 + k * L1).try_into().unwrap())).unwrap();
                        file.read(&mut buf).unwrap();
                        depend_data.append(&mut buf.to_vec());
                    }
                }

                let depend_data_hash = {
                    let mut x_input = vec![];
                    for i in (0..depend_data.len()).step_by(32) {
                        let x_in = Fr::from_le_bytes_mod_order(&depend_data[i .. i + 32]);
                        x_input.push(x_in);
                    }
                    let res = multi_mimc7_hash(&x_input, hash_key, &constants);
                    let res: BigInteger256 = res.into();
                    res.to_bytes_le()
                };

                let cur_block = {
                    file.seek(SeekFrom::Start((i * L2 + j * L1).try_into().unwrap())).unwrap();
                    file.read(&mut buf).unwrap();
                    buf.to_vec()
                };

                let data_xor = vecu8_xor(&depend_data_hash, &cur_block);
                let new_block = vde(&data_xor, &vde_key, &vde_mode);

                buf.copy_from_slice(&new_block);
                file.seek(SeekFrom::Start((i * L2 + j * L1).try_into().unwrap())).unwrap();
                file.write_all(&buf).unwrap();
            }
        }
    }
}

pub fn unseal(round: usize, file: &mut File, block_num2: usize, block_num1: usize, index_l_collect: &Vec<Vec<Vec<usize>>>, index_s_collect: &Vec<Vec<Vec<usize>>>, constants: &Vec<Fr>, hash_key: Fr, vde_key: &BigUint, vde_mode: &str) {
    //! Unseal data block by block for n round.
    for _ in 0..round {
        let mut buf = [0u8; L1];

        for i in 0..block_num2 {
            for j in 0..block_num1 {
                // collect the depended data
                let mut depend_data = vec![];
                {
                    for k in 0..index_l_collect[i][j].len() {
                        // move the file pointer
                        file.seek(SeekFrom::Start((k * L2 + j * L1).try_into().unwrap())).unwrap();
                        file.read(&mut buf).unwrap();
                        depend_data.append(&mut buf.to_vec());
                    }
    
                    for k in 0..index_s_collect[i][j].len() {
                        file.seek(SeekFrom::Start((i * L2 + k * L1).try_into().unwrap())).unwrap();
                        file.read(&mut buf).unwrap();
                        depend_data.append(&mut buf.to_vec());
                    }
                }
                
                let depend_data_hash = {
                    let mut x_input = vec![];
                    for i in (0..depend_data.len()).step_by(32) {
                        let x_in = Fr::from_le_bytes_mod_order(&depend_data[i .. i + 32]);
                        x_input.push(x_in);
                    }
                    let res = multi_mimc7_hash(&x_input, hash_key, &constants);
                    let res: BigInteger256 = res.into();
                    res.to_bytes_le()
                };

                let cur_block = {
                    file.seek(SeekFrom::Start((i * L2 + j * L1).try_into().unwrap())).unwrap();
                    file.read(&mut buf).unwrap();
                    buf.to_vec()
                };

                let vde_res = vde_inv(&cur_block, &vde_key, &vde_mode);
                let new_block = vecu8_xor(&vde_res, &depend_data_hash);

                buf.copy_from_slice(&new_block);
                file.seek(SeekFrom::Start((i * L2 + j * L1).try_into().unwrap())).unwrap();
                file.write_all(&buf).unwrap();
            }
        }
    }
}

#[cfg(test)]
mod test {

    use std::{fs::OpenOptions, str::FromStr};
    use std::time::Instant;

    // use ark_bls12_381::Fr;
    // use ark_ff::Field;
    use ark_std::{rand::Rng, test_rng};
    use num_bigint::BigUint;

    use super::*;
    use crate::common::data::write_file;

    #[test]
    fn test() {
        let rng = &mut test_rng();
        
        // Generate origin data and write to file.
        write_file(DATA_L, DATA_DIR).unwrap();

        let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(DATA_DIR)
        .unwrap();

        // Set parameters
        let block_num2 = DATA_L / L2;
        let block_num1 = L2 / L1;

        let index_count_l: usize = 3;
        let index_count_s: usize = 3;

        let mode_l: usize = 1;
        let mode_s: usize = 1;

        let constants = (0..MIMC_HASH_ROUNDS)
            .map(|_| rng.gen())
            .collect::<Vec<_>>();

        let vde_mode: &str = "sloth";
        let vde_key = BigUint::from_str("340282366920938463463374607431768211507").unwrap();
        let hash_key = rng.gen();

        // Collect all the left indexes of depended data blocks.
        let (index_l_collect, index_s_collect) = create_depend(block_num2, block_num1, index_count_l, index_count_s, mode_l, mode_s);
        
        // Seal
        let start = Instant::now();
        seal(SEAL_ROUND, &mut file, block_num2, block_num1, &index_l_collect, &index_s_collect, &constants, hash_key, &vde_key, vde_mode);
        println!("Seal: {:?}", start.elapsed());

        // Unseal
        let start = Instant::now();
        unseal(SEAL_ROUND, &mut file, block_num2, block_num1, &index_l_collect, &index_s_collect, &constants, hash_key, &vde_key, vde_mode);
        println!("Unseal: {:?}", start.elapsed());

    }
}