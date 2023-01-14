// use std::fs::File;
// use std::io::{Seek, Read, SeekFrom, Write};

// use ark_bls12_381::Fr;
// use ark_ff::{PrimeField, BigInteger256, BigInteger};

// use super::depend::{long_depend, short_depend};
// use crate::common::convert::bits_to_vecu8;
// use crate::common::data::vecu8_xor;
// use crate::common::mimc_hash::mimc7_hash;
// // use crate::vde::vde::{vde, vde_inv};

// pub const DATA_DIR: &str = r"src\seal_data.txt";
// pub const L2: usize = 256;
// pub const L1: usize = 64;
// pub const SEAL_ROUND: usize = 3;
// pub const MIMC_HASH_ROUNDS: usize = 10;

// pub fn create_depend(block_num2: usize, block_num1: usize, index_count_l: usize, index_count_s: usize, mode_l: usize, mode_s: usize) -> (Vec<Vec<Vec<usize>>>, Vec<Vec<Vec<usize>>>) {
//     //! Collect all the depended indexs of every data blocks.
//     let mut index_l_collect = vec![];
//     let mut index_s_collect = vec![];

//     for i in 0..block_num2 {
//         let mut index_l = vec![];
//         let mut index_s = vec![];
//         for j in 0..block_num1 {
//             index_l.push(long_depend(block_num2, i, index_count_l, mode_l));
//             index_s.push(short_depend(block_num1, j, index_count_s, mode_s));
//         }
//         index_l_collect.push(index_l);
//         index_s_collect.push(index_s);
//     }

//     (index_l_collect, index_s_collect)
// }

// pub fn seal(round: usize, file: &mut File, block_num2: usize, block_num1: usize, index_l_collect: &Vec<Vec<Vec<usize>>>, index_s_collect: &Vec<Vec<Vec<usize>>>, constants: &Vec<Fr>, hash_key: Fr, vde_key: Fr, mode_vde: &str) {
//     //! Seal data block by block for n round.
//     for _ in 0..round {
//         let mut buf = [0; L1];

//         for i in 0..block_num2 {
//             for j in 0..block_num1 {

//                 // collect the depended data
//                 let mut depend_data = vec![];
//                 for k in 0..index_l_collect[i][j].len() {

//                     // move the file pointer
//                     file.seek(SeekFrom::Start((k * L2 + j * L1).try_into().unwrap())).unwrap();
//                     file.read(&mut buf).unwrap();
//                     depend_data.append(&mut buf.to_vec());
//                 }

//                 for k in 0..index_s_collect[i][j].len() {
//                     file.seek(SeekFrom::Start((i * L2 + k * L1).try_into().unwrap())).unwrap();
//                     file.read(&mut buf).unwrap();
//                     depend_data.append(&mut buf.to_vec());
//                 }

//                 let mut depend_hash = vec![];
//                 {
//                     for i in (0..depend_data.len()).step_by(32) {
//                         let x_in = Fr::from_le_bytes_mod_order(&depend_data[i .. i + 32]);
//                         let res = mimc7_hash(x_in, hash_key, &constants);
//                         let res: BigInteger256 = res.into();
//                         let res = res.to_bits_le();
//                         let mut res = bits_to_vecu8(&res);
//                         depend_hash.append(&mut res);
//                     }
//                 }

//                 file.seek(SeekFrom::Start((i * L2 + j * L1).try_into().unwrap())).unwrap();
//                 file.read(&mut buf).unwrap();
//                 let cur_block = buf.to_vec();

//                 let data_xor = vecu8_xor(&depend_hash, &cur_block);
//                 // let new_block = vde(&depend_data, vde_key, mode_vde);

//                 buf.copy_from_slice(&data_xor);
//                 file.seek(SeekFrom::Start((i * L2 + j * L1).try_into().unwrap())).unwrap();
//                 file.write_all(&buf).unwrap();
//             }
//         }
//     }
// }

// pub fn unseal(round: usize, file: &mut File, block_num2: usize, block_num1: usize, index_l_collect: &Vec<Vec<Vec<usize>>>, index_s_collect: &Vec<Vec<Vec<usize>>>, constants: &Vec<Fr>, hash_key: Fr, vde_key: Fr, mode_vde: &str) {
//     //! Unseal data block by block for n round.
//     for _ in 0..round {
//         let mut buf = [0u8; L1];

//         for i in 0..block_num2 {
//             for j in 0..block_num1 {
//                 let mut depend_data = vec![];

//                 for k in 0..index_l_collect[i][j].len() {
//                     file.seek(SeekFrom::Start((k * L2 + j * L1).try_into().unwrap())).unwrap();
//                     file.read(&mut buf).unwrap();
//                     depend_data.append(&mut buf.to_vec());
//                 }

//                 for k in 0..index_s_collect[i][j].len() {
//                     file.seek(SeekFrom::Start((i * L2 + k * L1).try_into().unwrap())).unwrap();
//                     file.read(&mut buf).unwrap();
//                     depend_data.append(&mut buf.to_vec());
//                 }
                
//                 let mut depend_hash = vec![];
//                 {
//                     for i in (0..depend_data.len()).step_by(32) {
//                         let x_in = Fr::from_le_bytes_mod_order(&depend_data[i .. i + 32]);
//                         let res = mimc7_hash(x_in, hash_key, &constants);
//                         let res: BigInteger256 = res.into();
//                         let res = res.to_bits_le();
//                         let mut res = bits_to_vecu8(&res);
//                         depend_hash.append(&mut res);
//                     }
//                 }
                
//                 file.seek(SeekFrom::Start((i * L2 + j * L1).try_into().unwrap())).unwrap();
//                 file.read(&mut buf).unwrap();
//                 let cur_block = buf.to_vec();

//                 // let mut new_block = vde_inv(&cur_block, vde_key, mode_vde);
//                 let new_block = vecu8_xor(&cur_block, &depend_hash);

//                 buf.copy_from_slice(&new_block);
//                 file.seek(SeekFrom::Start((i * L2 + j * L1).try_into().unwrap())).unwrap();
//                 file.write_all(&buf).unwrap();
//             }
//         }
//     }
// }

// #[cfg(test)]
// mod test {

//     use std::fs::OpenOptions;
//     use std::time::Instant;

//     use ark_bls12_381::Fr;
//     use ark_ff::Field;
//     use ark_std::{rand::Rng, test_rng};

//     use super::*;
//     use crate::common::data::write_file;

//     #[test]
//     fn test() {
//         let mut rng = &mut test_rng();

//         // data len: n = n bytes
//         let data_len: usize = 1024 * 1024;
//         write_file(data_len, DATA_DIR).unwrap();

//         let mut file = OpenOptions::new()
//         .read(true)
//         .write(true)
//         .create(true)
//         .open(DATA_DIR)
//         .unwrap();

//         let block_num2 = data_len / L2;
//         let block_num1 = L2 / L1;

//         let index_count_l: usize = 3;
//         let index_count_s: usize = 3;

//         let mode_l: usize = 1;
//         let mode_s: usize = 1;
//         let mode_vde: &str = "sloth";

//         let constants = (0..MIMC_HASH_ROUNDS)
//             .map(|_| rng.gen())
//             .collect::<Vec<_>>();
//         let vde_key = rng.gen();
//         let hash_key = rng.gen();

//         let (index_l_collect, index_s_collect) = create_depend(block_num2, block_num1, index_count_l, index_count_s, mode_l, mode_s);
        
//         let start = Instant::now();
//         seal(SEAL_ROUND, &mut file, block_num2, block_num1, &index_l_collect, &index_s_collect, &constants, hash_key, vde_key, mode_vde);
//         println!("Seal: {:?}", start.elapsed());

//         let start = Instant::now();
//         unseal(SEAL_ROUND, &mut file, block_num2, block_num1, &index_l_collect, &index_s_collect, &constants, hash_key, vde_key, mode_vde);
//         println!("Unseal: {:?}", start.elapsed());

//     }
// }