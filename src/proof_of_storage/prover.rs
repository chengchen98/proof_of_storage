use ark_bls12_381::Fr;
use num_bigint::BigInt;
use rs_merkle::{algorithms::Sha256, MerkleProof};
use std::{fs::OpenOptions, io::{Write, Seek, SeekFrom}, time::Instant};
use ark_ff::{BigInteger256, BigInteger, PrimeField};

use crate::vde::vde::{vde, vde_inv};
use crate::mimc::mimc_hash::multi_mimc5_hash;

use super::{common::{read_file, to_block, com_block, vecu8_xor}, depend::{long_depend, short_depend}, postorage::DATA_PL, merkle_tree::{generate_merkle_tree, generate_merkle_proof}};
use super::postorage::{DATA_L, L0, PL2, PL1, PL0, SEAL_ROUNDS, SLOTH_ROUNDS, VDE_MODE};

pub fn create_depend(l2_cnt: usize, l1_cnt: usize, idx_cnt_l: usize, idx_cnt_s: usize, mode_l: usize, mode_s: usize) -> (Vec<Vec<usize>>, Vec<Vec<Vec<usize>>>) {
    let mut idx_l = vec![];
    let mut idx_s = vec![];

    for cnt2 in 0..l2_cnt {
        let cur_idx_l = long_depend(cnt2, idx_cnt_l, mode_l);
        idx_l.push(cur_idx_l);

        let mut cur_idx_s = vec![];
        for cnt1 in 0..l1_cnt {
            cur_idx_s.push(short_depend(l1_cnt, cnt1, idx_cnt_s, mode_s));
        }
        idx_s.push(cur_idx_s);
    }

    (idx_l, idx_s)
}

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

pub fn seal(path: &str, block_cnt: usize, idx_l: &Vec<Vec<usize>>, idx_s: &Vec<Vec<Vec<usize>>>, hash_constants: &Vec<Fr>, hash_key: Fr, vde_key: &BigInt) 
-> (f32, f32) {
    let mut file = OpenOptions::new()
    .read(true)
    .write(true)
    .open(path)
    .unwrap();

    let mut vde_cost = 0.0;
    let mut file_cost = 0.0;

    for _ in 0..SEAL_ROUNDS {
        for idx2 in 0..block_cnt {
            let mut block = {
                let start = Instant::now();
                let buf = read_file(&mut file, idx2 * PL2, PL2);
                file_cost += start.elapsed().as_secs_f32();
                to_block(&buf, PL1)
            };

            let depend_blocks = {
                let mut res = vec![];
                for i in 0..idx_l[idx2].len() {
                    let start = Instant::now();
                    let buf = read_file(&mut file, i * PL2, PL2);
                    file_cost += start.elapsed().as_secs_f32();
                    let ans = to_block(&buf, PL1);
                    res.push(ans);
                }
                res
            };

            for idx1 in 0..block.len() {
                // 获取长程和短程依赖数据块
                let mut depend_data = vec![];
                {
                    // for &idx in &idx_l[idx2][idx1] {
                    //     let start = Instant::now();
                    //     let mut buf = read_file(&mut file, idx * PL2 + idx1 * PL1, PL1);
                    //     file_cost += start.elapsed().as_secs_f32();
                    //     depend_data.append(&mut buf);
                    // }

                    for i in 0..depend_blocks.len() {
                        depend_data.append(&mut depend_blocks[i][idx1].clone());
                    }

                    for &idx in &idx_s[idx2][idx1] {
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
                    let res = multi_mimc5_hash(&x_input, hash_key, &hash_constants);
                    let res: BigInteger256 = res.into();
                    res.to_bytes_le()
                };
    
                let cur_block = &block[idx1].to_vec();
                let block_xor = vecu8_xor(&depend_data_hash, &cur_block)[..PL1].to_vec();
                let new_block = {
                    let mut res = vec![];
                    for idx in (0..PL1).step_by(PL0) {
                        let input = block_xor[idx .. idx + PL0].to_vec();
                        let start = Instant::now();
                        let mut vde_res = vde(&input, vde_key, SLOTH_ROUNDS, VDE_MODE);
                        vde_cost += start.elapsed().as_secs_f32();
                        res.append(&mut vde_res);
                    }
                    res
                };

                block[idx1] = new_block;
            }

            let block = com_block(&block);
            let start = Instant::now();
            file.seek(SeekFrom::Start((idx2 * PL2).try_into().unwrap())).unwrap();
            file.write_all(&block).unwrap();
            file_cost += start.elapsed().as_secs_f32();
        }
    }

    (vde_cost, file_cost)
}

pub fn copy_and_compress(origin_path: &str, new_path: &str) {
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
        let buf = read_file(&mut origin_file, cnt * PL0, PL0);
        new_file.write_all(&buf[0..L0]).unwrap();
    }
}

pub fn unseal(path: &str, block_cnt: usize, idx_l: &Vec<Vec<usize>>, idx_s: &Vec<Vec<Vec<usize>>>, hash_constants: &Vec<Fr>, hash_key: Fr, vde_key: &BigInt) 
-> (f32, f32) {
    let mut file = OpenOptions::new()
    .read(true)
    .write(true)
    .open(path)
    .unwrap();

    let mut vde_cost = 0.0;
    let mut file_cost = 0.0;

    for _ in 0..SEAL_ROUNDS {
        for i in 0..block_cnt {
            let idx2 = block_cnt - 1 - i;

            let mut block = {
                let start = Instant::now();
                let buf = read_file(&mut file, idx2 * PL2, PL2);
                file_cost += start.elapsed().as_secs_f32();
                to_block(&buf, PL1)
            };

            let depend_blocks = {
                let mut res = vec![];
                for i in 0..idx_l[idx2].len() {
                    let start = Instant::now();
                    let buf = read_file(&mut file, i * PL2, PL2);
                    file_cost += start.elapsed().as_secs_f32();
                    let ans = to_block(&buf, PL1);
                    res.push(ans);
                }
                res
            };

            for j in 0..block.len() {
                let idx1 = block.len() - 1 - j;

                // 获取长程和短程依赖数据块
                let mut depend_data = vec![];
                {
                    // for &idx in &idx_l[idx2][idx1] {
                    //     let start = Instant::now();
                    //     let mut buf = read_file(&mut file, idx * PL2 + idx1 * PL1, PL1);
                    //     file_cost += start.elapsed().as_secs_f32();
                    //     depend_data.append(&mut buf);
                    // }

                    for i in 0..depend_blocks.len() {
                        depend_data.append(&mut depend_blocks[i][idx1].clone());
                    }

                    for &idx in &idx_s[idx2][idx1] {
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
                    let res = multi_mimc5_hash(&x_input, hash_key, &hash_constants);
                    let res: BigInteger256 = res.into();
                    res.to_bytes_le()
                };
    
                let cur_block = &block[idx1].to_vec();
                let vde_inv_res = {
                    let mut res = vec![];
                    for idx in (0..PL1).step_by(PL0) {
                        let input = cur_block[idx .. idx + PL0].to_vec();
                        let start = Instant::now();
                        let mut vde_inv_res = vde_inv(&input, vde_key, SLOTH_ROUNDS, VDE_MODE);
                        vde_cost += start.elapsed().as_secs_f32();
                        res.append(&mut vde_inv_res);
                    }
                    res
                };
                let new_block = vecu8_xor(&vde_inv_res, &depend_data_hash)[..PL1].to_vec();

                block[idx1] = new_block;
            }

            let block = com_block(&block);
            let start = Instant::now();
            file.seek(SeekFrom::Start((idx2 * PL2).try_into().unwrap())).unwrap();
            file.write_all(&block).unwrap();
            file_cost += start.elapsed().as_secs_f32();
        }
    }

    (vde_cost, file_cost)
}

// pub fn generate_sealed_merkle_tree(path: &str, block_cnt: usize) -> (Vec<[u8; 32]>, MerkleTree<Sha256>, [u8; 32]) {
//     // 生成二级 merkle tree
//     let mut file = OpenOptions::new()
//     .read(true)
//     .open(path)
//     .unwrap();

//     let mut leaf_values = vec![];

//     for idx2 in 0..block_cnt {
//         // 对每个二级数据块生成 merkle tree，叶子结点为一级数据块
//         let leaf_values_cur = {
//             let buf = read_file(&mut file, idx2 * PL2, PL2);
//             to_block(&buf, PL1)
//         };
//         let leaves: Vec< [u8; 32]> = leaf_values_cur.iter().map(|x| Sha256::hash(x)).collect();
//         let merkle_tree = MerkleTree::<Sha256>::from_leaves(&leaves);
//         let merkle_root = merkle_tree.root().ok_or("can't get the merkle root").unwrap();
//         leaf_values.push(merkle_root);
//     }
//     // 二级数据块本身作为叶子结点，生成 merkle tree
//     let leaves: Vec< [u8; 32]> = leaf_values.iter().map(|x| Sha256::hash(x)).collect();
//     let merkle_tree = MerkleTree::<Sha256>::from_leaves(&leaves);
//     let merkle_root = merkle_tree.root().ok_or("can't get the merkle root").unwrap();
//     (leaves, merkle_tree, merkle_root)
// }

pub fn response(path: &str, indices_to_prove: &Vec<usize>, idx_l: &Vec<Vec<usize>>) 
-> (Vec<usize>, Vec<Vec<Vec<u8>>>, Vec<Vec<Vec<Vec<u8>>>>, [u8; 32], MerkleProof<Sha256>, Vec<[u8; 32]>) {
    let (sealed_leaves, sealed_merkle_tree, sealed_merkle_root) = generate_merkle_tree(&path, DATA_PL, PL1);
    let sealed_proof = generate_merkle_proof(&indices_to_prove, sealed_merkle_tree);

    let mut file = OpenOptions::new()
    .read(true)
    .open(path)
    .unwrap();

    // 计算出所有需要证明的一级数据块对应的二级数据块编号
    let blocks_idx = {
        let mut res = vec![];
        for i in indices_to_prove {
            let idx = i / (PL2 / PL1);
            res.push(idx);
        }
        res
    };

    // 读出二级数据块集合
    let blocks = {
        let mut res = vec![];
        for &i in &blocks_idx {
            let buf = read_file(&mut file, i * PL2, PL2);
            let ans = to_block(&buf, PL1);
            res.push(ans);
        }
        res
    };

    // 长程依赖二级数据块集合
    let depend_blocks = {
        let mut res = vec![];
        for &i in &blocks_idx {
            let mut tmp = vec![];
            for &j in &idx_l[i] {
                let buf = read_file(&mut file, j * PL2, PL2);
                let ans = to_block(&buf, PL1);
                tmp.push(ans);
            }
            res.push(tmp);
        }
        res
    };

    (blocks_idx, blocks, depend_blocks, sealed_merkle_root, sealed_proof, sealed_leaves)
}