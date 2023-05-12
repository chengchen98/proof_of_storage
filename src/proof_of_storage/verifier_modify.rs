use std::collections::BTreeSet;
use std::{fs::OpenOptions, io::Write};

use rand::Rng;
use rug::Integer;

use crate::vde::rug_vde::vde_inv;

use super::common::{read_file, to_units, modsub, blake3_hash};
use super::depend::{short_depend_random, long_mode_random};
use super::prover_modify::{create_short_depend, create_long_depend};

use super::postorage_modify::{DATA_L, UNIT_PL, BLOCK_PL, UNIT_L, BLOCK_L};

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

pub fn unseal_single_prepare(path: &str, idx_begin: usize, idx_end: usize) 
-> (Vec<usize>, Vec<Vec<Vec<u8>>>) {
    let mut file = OpenOptions::new()
    .read(true)
    .open(path)
    .unwrap();

    let blocks_idx = {
        let mut res = vec![];
        let mut i = 0;
        while i * BLOCK_PL + idx_begin < idx_end {
            res.push(i + idx_begin / BLOCK_PL);
            i += 1;
        }
        res
    };

    // 从文件中读出二级数据块集合
    let blocks = {
        let mut res = vec![];
        for &idx2 in &blocks_idx {
            let buf = read_file(&mut file, idx2 * BLOCK_PL, BLOCK_PL);
            let ans = to_units(&buf, UNIT_PL);
            res.push(ans);
        }
        res
    };

    (blocks_idx, blocks)
}

pub fn unseal_single_and_verify(sealed_path: &str, origin_path: &str, seal_rounds: usize, blocks_idx: &Vec<usize>, blocks: &Vec<Vec<Vec<u8>>>, mode_l: usize, cnt_l: usize, mode_s: usize, cnt_s: usize, vde_key: &Integer, vde_rounds: usize, vde_mode: &str, iv: &Vec<u8>) {
    let mut sealed_file = OpenOptions::new()
    .read(true)
    .write(false)
    .open(sealed_path)
    .unwrap();

    let idxs_l = {
        if mode_l != 0 {
            create_long_depend(DATA_L / BLOCK_L, cnt_l, mode_l)
        }
        else {
            vec![]
        }
    };
    let idxs_s = {
        if mode_s != 0 {
            create_short_depend(BLOCK_L / UNIT_L, cnt_s, mode_s)
        }
        else {
            vec![]
        }
    };
    
    // 逐个解封装二级数据块
    for i in 0..blocks.len() {
        // 当前二级数据块编号
        let idx2 = blocks_idx[i];
        // 当前二级数据块内容
        let mut cur_block = blocks[i].clone();

        let before_block_id = {
            if idx2 != 0 {
                let before_block = read_file(&mut sealed_file, (idx2 - 1) * BLOCK_PL, BLOCK_PL);
                blake3_hash(&before_block)
            }
            else {
                vec![]
            }
        };

        let depend_blocks = {
            let mut res = vec![];
            if mode_l == 0 {
                let cur_idxs_l = {
                    if idx2 == 0 {
                        vec![]
                    }
                    else {
                        long_mode_random(&before_block_id, idx2, cnt_l)
                    }
                };

                for &idx in &cur_idxs_l {
                    let buf = read_file(&mut sealed_file, idx * BLOCK_PL, BLOCK_PL);
                    let ans = to_units(&buf, UNIT_PL);
                    res.push(ans);
                }
            }
            else {
                for &i in &idxs_l[idx2] {
                    let buf = read_file(&mut sealed_file, i * BLOCK_PL, BLOCK_PL);  
                    let ans = to_units(&buf, UNIT_PL);
                    res.push(ans);
                }
            }
            res
        };

        for _ in 0..seal_rounds {
            for j in 0..cur_block.len() {
                let idx1 = cur_block.len() - 1 - j;

                let mut depend_data = {
                    let mut res = vec![];
                    for i in 0..depend_blocks.len() {
                        res.append(&mut depend_blocks[i][idx1].clone());
                    }
                    if mode_s != 0 {
                        for &idx in &idxs_s[idx1] {
                            res.append(&mut cur_block[idx].clone());
                        }
                    }
                    else {
                        let ans = {
                            if idx1 == 0 {
                                short_depend_random(BLOCK_L / UNIT_L, &vec![], idx1, cnt_s)
                            }
                            else {
                                short_depend_random(BLOCK_L / UNIT_L, &cur_block[idx1-1], idx1, cnt_s)
                            }
                        };

                        for idx in ans {
                            res.append(&mut cur_block[idx].clone());
                        }
                    }
                    res
                };

                if idx1 == 0 {
                    if idx2 == 0 {
                        depend_data.append(&mut iv.clone());
                    }
                    else {
                        depend_data.append(&mut before_block_id.clone());
                    }
                }

                let depend_data_hash = blake3_hash(&depend_data);
                let cur_unit = &cur_block[idx1].to_vec();
                let vde_inv_res = vde_inv(&cur_unit, vde_key, vde_rounds, vde_mode, UNIT_PL);
                let new_unit = modsub(&vde_inv_res, &depend_data_hash, &vde_key);
                cur_block[idx1] = new_unit;
            }
        }

        let mut origin_file = OpenOptions::new()
        .read(true)
        .open(origin_path)
        .unwrap();

        let origin_block = read_file(&mut origin_file, idx2 * BLOCK_L, BLOCK_L);
        let origin_block = to_units(&origin_block, UNIT_L);
        for j in 0.. origin_block.len() {
            let mut k_ori = 0;
            let mut k_cur = 0;
            let mut k = 0;
            while k_ori < origin_block[j].len() {
                if origin_block[j][k_ori] != cur_block[j][k_cur] {
                    println!("{:?}", idx2);
                    println!("{:?}", j);
                    println!("{:?}", k_cur);
                    println!("{:?}", k_ori);
                }
                assert_eq!(origin_block[j][k_ori], cur_block[j][k_cur]);
                k_ori += 1;
                k_cur += 1;
                k += 1;

                if k == UNIT_L {
                    k_cur += 1;
                    k = 0;
                }
            }
        }
    }
}