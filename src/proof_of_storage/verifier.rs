use rand::Rng;
use ark_bls12_381::Fr;
use ark_ff::{PrimeField, BigInteger256, BigInteger};
use rug::Integer;
use rs_merkle::{MerkleProof, algorithms::Sha256};
use std::{fs::OpenOptions, io::Write, collections::BTreeSet};
use crate::{proof_of_storage::postorage::{L2, L1}, mimc::mimc_hash::multi_mimc5_hash, vde::rug_vde::vde_inv};
use super::{common::{read_file, to_block, vecu8_xor}, merkle_tree::verify_merkle_proof, postorage::{PL0, PL1, VDE_MODE, SEAL_ROUNDS, SLOTH_ROUNDS, L0}};

pub fn create_random_file(path: &str, data_len: usize) -> std::io::Result<()> {
    //! 随机创建长度为 DATA_L 字节的文件
    let mut file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .truncate(true)
    .open(path)
    .unwrap();

    let mut rng = rand::thread_rng();
    for _ in 0..data_len {
        let buf: [u8; 1] = [rng.gen_range(0u8..=255u8)];
        file.write_all(&buf).unwrap();
    }
    Ok(())
}

pub fn create_challenges(n: usize, range: (usize, usize)) -> Vec<usize> {
    //! 生成 count 个随机数，范围是 [left, right)
    let mut rng = rand::thread_rng();
    let mut set = BTreeSet::new();

    while set.len() < n as usize {
        let num = rng.gen_range(range.0 .. range.1);
        set.insert(num);
    }

    set.into_iter().collect()
}

pub fn verify(path: &str, blocks_idx: &Vec<usize>, idx_s: &Vec<Vec<Vec<usize>>>, blocks: Vec<Vec<Vec<u8>>>, depend_blocks: Vec<Vec<Vec<Vec<u8>>>>, proof: MerkleProof<Sha256>, root: [u8; 32], indices_to_prove: &Vec<usize>, leaves: &Vec<[u8; 32]>, hash_constants: &Vec<Fr>, hash_key: Fr, vde_key: &Integer) {
    verify_merkle_proof(proof, root, &indices_to_prove, leaves);

    for i in 0..blocks.len() {
        // 当前二级数据块
        let idx2 = blocks_idx[i];
        let mut block = blocks[i].clone();

        for _ in 0..SEAL_ROUNDS {
            // 对一级数据块逐个unseal
            for idx1_inv in 0..block.len() {
                let idx1 = block.len() - 1 - idx1_inv;

                let depend_data = {
                    let mut res = vec![];
                    for j in 0..depend_blocks[i].len() {
                        res.append(&mut depend_blocks[i][j][idx1].clone());
                    }

                    for &idx in &idx_s[idx2][idx1] {
                        res.append(&mut block[idx].clone());
                    }
                    res
                };
                
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
                        let mut vde_inv_res = vde_inv(&input, vde_key, SLOTH_ROUNDS, VDE_MODE, PL0);
                        res.append(&mut vde_inv_res);
                    }
                    res
                };
                let new_block = vecu8_xor(&vde_inv_res, &depend_data_hash)[..PL1].to_vec();

                block[idx1] = new_block;
            }
        }

        let mut file = OpenOptions::new()
        .read(true)
        .open(path)
        .unwrap();
        let origin_block = read_file(&mut file, idx2 * L2, L2);
        let origin_block = to_block(&origin_block, L1);
        for j in 0.. origin_block.len() {
            let mut k_ori = 0;
            let mut k_cur = 0;
            let mut k = 0;
            println!("ori: {:?}", origin_block[j]);
            println!("cur: {:?}", block[j]);
            while k_ori < origin_block[j].len() {
                assert_eq!(origin_block[j][k_ori], block[j][k_cur]);
                k_ori += 1;
                k_cur += 1;
                k += 1;

                if k == L0 {
                    k_cur += 1;
                    k = 0;
                }
            }
        }
    }
}

// pub fn generate_origin_merkle_tree(path: &str, block_cnt: usize) -> [u8; 32] {
//     let mut file = OpenOptions::new()
//     .read(true)
//     .open(path)
//     .unwrap();

//     let mut leaf_values = vec![];

//     for idx2 in 0..block_cnt {
//         // 对每个二级数据块生成 merkle tree，叶子结点为一级数据块
//         let leaf_values_cur = {
//             let buf = read_file(&mut file, idx2 * L2, L2);
//             to_block(&buf, L1)
//         };
//         let leaves: Vec< [u8; 32]> = leaf_values_cur.iter().map(|x| Sha256::hash(x)).collect();
//         let merkle_tree = MerkleTree::<Sha256>::from_leaves(&leaves);
//         let merkle_root = merkle_tree.root().ok_or("can't get the merkle root").unwrap();
//         leaf_values.push(merkle_root);
//     }

//     let leaves: Vec< [u8; 32]> = leaf_values.iter().map(|x| Sha256::hash(x)).collect();
//     let merkle_tree = MerkleTree::<Sha256>::from_leaves(&leaves);
//     let merkle_root = merkle_tree.root().ok_or("can't get the merkle root").unwrap();
//     merkle_root
// }