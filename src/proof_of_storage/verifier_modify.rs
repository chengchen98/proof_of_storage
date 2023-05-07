use std::{fs::OpenOptions, io::Write};

use rand::Rng;
use rug::Integer;

use crate::{vde::rug_vde::vde_inv};

use super::{postorage_modify::{DATA_L, UNIT_PL, BLOCK_PL, UNIT_L, BLOCK_L}, prover_modify::{create_short_depend, create_long_depend}, depend::{short_depend_random, long_mode_random}, common::{md5_hash, read_file, to_block, modsub}};

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

pub fn unseal_single_prepare(path: &str, idx_begin: usize, idx_end: usize, mode_l: usize, cnt_l: usize) 
-> (Vec<usize>, Vec<Vec<Vec<u8>>>, Vec<Vec<Vec<Vec<u8>>>>) {
    let mut file = OpenOptions::new()
    .read(true)
    .open(path)
    .unwrap();
    
    // // 所有一级数据块编号集合
    // let units_idx = {
    //     let mut res = vec![];
    //     let mut i = 0;
    //     while i * PL1 + idx_begin <= idx_end {
    //         res.push(i + idx_begin / PL1);
    //         i += 1;
    //     }
    //     res
    // };

    // // 一级数据块对应的二级数据块编号集合
    // let blocks_idx = {
    //     let mut res = vec![];
    //     for &i in &units_idx {
    //         let idx = i / (PL2 / PL1);
    //         if res.len() > 0 {
    //             if idx != res[res.len() - 1] {
    //                 res.push(idx);
    //             }
    //         }
    //     }
    //     res
    // };

    let blocks_idx = {
        let mut res = vec![];
        let mut i = 0;
        while i * BLOCK_PL + idx_begin <= idx_end {
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
            let ans = to_block(&buf, UNIT_PL);
            res.push(ans);
        }
        res
    };
    
    let idxs_l = {
        if mode_l != 0 {
            create_long_depend(DATA_L / BLOCK_L, cnt_l, mode_l)
        }
        else {
            vec![]
        }
    };

    // 长程依赖的二级数据块集合
    let depend_blocks = {
        let mut res = vec![];
        for &idx2 in &blocks_idx {
            let mut tmp = vec![];
            if mode_l == 0 {
                let before_unit = {
                    if idx2 != 0 {
                        let buf = read_file(&mut file, idx2 * BLOCK_PL - UNIT_PL, UNIT_PL);
                        buf
                    }
                    else {
                        vec![]
                    }
                };
                let cur_idxs_l = long_mode_random(&before_unit, idx2, cnt_l);
                for &j in &cur_idxs_l {
                    let buf = read_file(&mut file, j * BLOCK_PL, BLOCK_PL);
                    let ans = to_block(&buf, UNIT_PL);
                    tmp.push(ans);
                }
                res.push(tmp);
            }
            else {
                for &j in &idxs_l[idx2] {
                    let buf = read_file(&mut file, j * BLOCK_PL, BLOCK_PL);
                    let ans = to_block(&buf, UNIT_PL);
                    tmp.push(ans);
                }
                res.push(tmp);
            }
        }
        res
    };

    (blocks_idx, blocks, depend_blocks)
}

pub fn unseal_single_and_verify(path: &str, seal_rounds: usize, blocks_idx: &Vec<usize>, blocks: &Vec<Vec<Vec<u8>>>, depend_blocks: &Vec<Vec<Vec<Vec<u8>>>>, mode_s: usize, cnt_s: usize, vde_key: &Integer, vde_rounds: usize, vde_mode: &str) {
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
        let mut block = blocks[i].clone();
        // 当前二级数据块长程依赖的二级数据块集合
        let cur_depend_blocks = depend_blocks[i].clone();

        // 进行seal_rounds轮解封装
        for _ in 0..seal_rounds {
            // 按一级数据块大小逐个解封装
            for idx1_inv in 0..block.len() {
                // 从后往前解封装
                let idx1 = block.len() - 1 - idx1_inv;

                let depend_data = {
                    let mut res = vec![];
                    for i in 0..cur_depend_blocks.len() {
                        res.append(&mut cur_depend_blocks[i][idx1].clone());
                    }
                    if mode_s != 0 {
                        for &idx in &idxs_s[idx1] {
                            res.append(&mut block[idx].clone());
                        }
                    }
                    else {
                        let ans = {
                            if idx1 == 0 {
                                short_depend_random(BLOCK_L / UNIT_L, &vec![], idx1, cnt_s)
                            }
                            else {
                                short_depend_random(BLOCK_L / UNIT_L, &block[idx1-1], idx1, cnt_s)
                            }
                        };
                        for idx in ans {
                            res.append(&mut block[idx].clone());
                        }
                    }
                    res
                };

                let depend_data_hash = md5_hash(&depend_data);
                let cur_unit = &block[idx1].to_vec();
                let vde_inv_res = vde_inv(&cur_unit, vde_key, vde_rounds, vde_mode, UNIT_PL);
                // let new_unit = vecu8_xor(&vde_inv_res, &depend_data_hash);
                let new_unit = modsub(&vde_inv_res, &depend_data_hash, &vde_key);
                block[idx1] = new_unit;
            }
        }

        let mut file = OpenOptions::new()
        .read(true)
        .open(path)
        .unwrap();

        let origin_block = read_file(&mut file, idx2 * BLOCK_L, BLOCK_L);
        let origin_block = to_block(&origin_block, UNIT_L);
        for j in 0.. origin_block.len() {
            let mut k_ori = 0;
            let mut k_cur = 0;
            let mut k = 0;
            println!("j: {:?}", j);
            println!("ori: {:?}", origin_block[j]);
            println!("cur: {:?}", block[j]);
            while k_ori < origin_block[j].len() {
                assert_eq!(origin_block[j][k_ori], block[j][k_cur]);
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