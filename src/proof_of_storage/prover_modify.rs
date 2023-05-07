use rug::Integer;
use std::{fs::OpenOptions, io::{Write, Seek, SeekFrom}, time::Instant};

use crate::{vde::rug_vde::{vde, vde_inv}};

use super::{depend::{long_depend, short_depend, short_depend_random, long_mode_random}, common::{read_file, to_block, vecu8_xor, com_block, md5_hash, modadd, modsub, blake3_hash}};
use super::postorage_modify::{DATA_L, UNIT_L, UNIT_PL, BLOCK_L, BLOCK_PL};

pub fn create_long_depend(num: usize, count: usize, mode: usize) -> Vec<Vec<usize>> {
    let mut indices = vec![];
    for idx in 0..num {
        let cur_indices = long_depend(idx, count, mode);
        indices.push(cur_indices);
    }
    indices
}

pub fn create_short_depend(num: usize, count: usize, mode: usize) -> Vec<Vec<usize>> {
    let mut indices = vec![];
    for idx in 0..num {
        let cur_indices = short_depend(num, idx, count, mode);
        indices.push(cur_indices);
    }
    indices
}

pub fn copy_and_pad(origin_path: &str, new_path: &str) {
    //! 将原始文件按照 L1 大小逐个pad（在高位添加一个 0），再存储到新文件
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

    let block_cnt = DATA_L / UNIT_L;
    for cnt in 0..block_cnt {
        let mut buf = read_file(&mut origin_file, cnt * UNIT_L, UNIT_L);
        buf.push(0);
        new_file.write_all(&buf).unwrap();
    }
}

pub fn seal(path: &str, seal_rounds: usize, mode_l: usize, cnt_l: usize, mode_s: usize, cnt_s: usize, vde_key: &Integer, iv: &Vec<u8>, vde_rounds: usize, vde_mode: &str) 
-> (f32, f32, f32, f32, f32, f32) {
    let mut file = OpenOptions::new()
    .read(true)
    .write(true)
    .open(path)
    .unwrap();

    let mut vde_cost = 0.0;
    let mut file_cost = 0.0;
    let mut depend_cost = 0.0;
    let mut hash_cost = 0.0;
    let mut block_cost = 0.0;
    let mut xor_cost = 0.0;

    // block_cnt: 二级数据块个数
    let block_cnt = DATA_L / BLOCK_L;

    let start = Instant::now();
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
    depend_cost += start.elapsed().as_secs_f32();

    let mut blocks_id = vec![vec![]; block_cnt];

    // 封装seal_rounds轮
    for _ in 0..seal_rounds {
        // 逐个封装二级数据块
        for idx2 in 0..block_cnt {
            let mut cur_block = {
                let start = Instant::now();
                let buf = read_file(&mut file, idx2 * BLOCK_PL, BLOCK_PL);
                file_cost += start.elapsed().as_secs_f32();

                let start = Instant::now();
                let block = to_block(&buf, UNIT_PL);
                block_cost += start.elapsed().as_secs_f32();
                block
            };

            // 当前二级数据块长程依赖的二级数据块集合
            let depend_blocks = {
                let mut res = vec![];
                if mode_l == 0 {
                    let before_unit = {
                        if idx2 != 0 {
                            let start = Instant::now();
                            let buf = read_file(&mut file, idx2 * BLOCK_PL - UNIT_PL, UNIT_PL);
                            file_cost += start.elapsed().as_secs_f32();
                            buf
                        }
                        else {
                            vec![]
                        }
                    };
                    let start = Instant::now();
                    let cur_idxs_l = long_mode_random(&before_unit, idx2, cnt_l);
                    depend_cost += start.elapsed().as_secs_f32();

                    for i in 0..cur_idxs_l.len() {
                        let start = Instant::now();
                        let buf = read_file(&mut file, cur_idxs_l[i] * BLOCK_PL, BLOCK_PL);
                        file_cost += start.elapsed().as_secs_f32();
    
                        let start = Instant::now();
                        let ans = to_block(&buf, UNIT_PL);
                        block_cost += start.elapsed().as_secs_f32();

                        res.push(ans);
                    }
                }
                else {
                    for i in 0..idxs_l[idx2].len() {
                        let start = Instant::now();
                        let buf = read_file(&mut file, i * BLOCK_PL, BLOCK_PL);
                        file_cost += start.elapsed().as_secs_f32();
    
                        let start = Instant::now();
                        let ans = to_block(&buf, UNIT_PL);
                        block_cost += start.elapsed().as_secs_f32();

                        res.push(ans);
                    }
                }
                res
            };

            // 对当前二级数据块中的一级数据块逐个封装
            for idx1 in 0..cur_block.len() {
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
                        let start = Instant::now();
                        let ans = {
                            if idx1 == 0 {
                                short_depend_random(BLOCK_L / UNIT_L, &vec![], idx1, cnt_s)
                            }
                            else {
                                short_depend_random(BLOCK_L / UNIT_L, &cur_block[idx1-1], idx1, cnt_s)
                            }
                        };
                        depend_cost += start.elapsed().as_secs_f32();

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
                        depend_data.append(&mut blocks_id[idx2 - 1].clone());
                    }
                }

                // 长程依赖及短程依赖数据的哈希值
                let start = Instant::now();
                let depend_data_hash = blake3_hash(&depend_data);
                hash_cost += start.elapsed().as_secs_f32();

                // 当前一级数据块记为cur_unit
                let cur_unit = &cur_block[idx1].to_vec();
                // 哈希值与一级数据块异或
                let start = Instant::now();
                // let unit_xor = vecu8_xor(&depend_data_hash, &cur_unit)[..PL1].to_vec();
                let unit_xor = modadd(&cur_unit, &depend_data_hash, &vde_key);
                xor_cost += start.elapsed().as_secs_f32();

                // 将异或结果带入vde计算得到new_unit
                let start = Instant::now();
                let new_unit = vde(&unit_xor, &vde_key, vde_rounds, vde_mode, UNIT_PL);
                vde_cost += start.elapsed().as_secs_f32();

                // 更新unit的值
                cur_block[idx1] = new_unit;
            }

            let start = Instant::now();
            let cur_block = com_block(&cur_block);
            block_cost += start.elapsed().as_secs_f32();

            // let start = Instant::now();
            // blocks_id[idx2] = blake3_hash(&cur_block);
            // hash_cost += start.elapsed().as_secs_f32();

            let start = Instant::now();
            file.seek(SeekFrom::Start((idx2 * BLOCK_PL).try_into().unwrap())).unwrap();
            file.write_all(&cur_block).unwrap();
            file_cost += start.elapsed().as_secs_f32();
        }
    }

    (vde_cost, file_cost, depend_cost, hash_cost, block_cost, xor_cost)
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

    let block_cnt = DATA_L / UNIT_L;
    for cnt in 0..block_cnt {
        let buf = read_file(&mut origin_file, cnt * UNIT_PL, UNIT_PL);
        new_file.write_all(&buf[0..UNIT_L]).unwrap();
    }
}

pub fn unseal(path: &str, seal_rounds: usize, mode_l: usize, cnt_l: usize, mode_s: usize, cnt_s: usize, vde_key: &Integer, iv: &Vec<u8>, vde_rounds: usize, vde_mode: &str) 
-> (f32, f32, f32, f32, f32, f32) {
    let mut file = OpenOptions::new()
    .read(true)
    .write(true)
    .open(path)
    .unwrap();

    let mut vde_cost = 0.0;
    let mut file_cost = 0.0;
    let mut depend_cost = 0.0;
    let mut hash_cost = 0.0;
    let mut block_cost = 0.0;
    let mut xor_cost = 0.0;

    let block_cnt = DATA_L / BLOCK_L;

    let start = Instant::now();
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
    depend_cost += start.elapsed().as_secs_f32();

    let mut blocks_id = vec![vec![]; block_cnt];

    for _ in 0..seal_rounds {
        for i in 0..block_cnt {
            let idx2 = block_cnt - 1 - i;

            let mut cur_block = {
                let start = Instant::now();
                let buf = read_file(&mut file, idx2 * BLOCK_PL, BLOCK_PL);
                file_cost += start.elapsed().as_secs_f32();
                

                let start = Instant::now();
                let block = to_block(&buf, UNIT_PL);
                block_cost += start.elapsed().as_secs_f32();
                block
            };

            let depend_blocks = {
                let mut res = vec![];
                if mode_l == 0 {
                    let before_unit = {
                        if idx2 != 0 {
                            let start = Instant::now();
                            let buf = read_file(&mut file, idx2 * BLOCK_PL - UNIT_PL, UNIT_PL);
                            file_cost += start.elapsed().as_secs_f32();
                            buf
                        }
                        else {
                            vec![]
                        }
                    };
                    let start = Instant::now();
                    let cur_idxs_l = long_mode_random(&before_unit, idx2, cnt_l);
                    depend_cost += start.elapsed().as_secs_f32();

                    for i in 0..cur_idxs_l.len() {
                        let start = Instant::now();
                        let buf = read_file(&mut file, cur_idxs_l[i] * BLOCK_PL, BLOCK_PL);
                        file_cost += start.elapsed().as_secs_f32();
    

                        let start = Instant::now();
                        let ans = to_block(&buf, UNIT_PL);
                        block_cost += start.elapsed().as_secs_f32();

                        res.push(ans);
                    }
                }
                else {
                    for i in 0..idxs_l[idx2].len() {
                        let start = Instant::now();
                        let buf = read_file(&mut file, i * BLOCK_PL, BLOCK_PL);
                        file_cost += start.elapsed().as_secs_f32();
    

                        let start = Instant::now();
                        let ans = to_block(&buf, UNIT_PL);
                        block_cost += start.elapsed().as_secs_f32();

                        res.push(ans);
                    }
                }
                res
            };

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
                        let start = Instant::now();
                        let ans = {
                            if idx1 == 0 {
                                short_depend_random(BLOCK_L / UNIT_L, &vec![], idx1, cnt_s)
                            }
                            else {
                                short_depend_random(BLOCK_L / UNIT_L, &cur_block[idx1-1], idx1, cnt_s)
                            }
                        };
                        depend_cost += start.elapsed().as_secs_f32();

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
                        depend_data.append(&mut blocks_id[idx2 - 1].clone());
                    }
                }

                let start = Instant::now();
                let depend_data_hash = blake3_hash(&depend_data);
                hash_cost += start.elapsed().as_secs_f32();

                let cur_unit = &cur_block[idx1].to_vec();

                let start = Instant::now();
                let vde_inv_res = vde_inv(&cur_unit, &vde_key, vde_rounds, vde_mode, UNIT_PL);
                vde_cost += start.elapsed().as_secs_f32();

                let start = Instant::now();
                // let new_unit = vecu8_xor(&vde_inv_res, &depend_data_hash);
                let new_unit = modsub(&vde_inv_res, &&depend_data_hash, &vde_key);
                xor_cost += start.elapsed().as_secs_f32();

                cur_block[idx1] = new_unit;
            }

            let start = Instant::now();
            let cur_block = com_block(&cur_block);
            block_cost += start.elapsed().as_secs_f32();

            // let start = Instant::now();
            // blocks_id[idx2] = blake3_hash(&cur_block);
            // hash_cost += start.elapsed().as_secs_f32();

            let start = Instant::now();
            file.seek(SeekFrom::Start((idx2 * BLOCK_PL).try_into().unwrap())).unwrap();
            file.write_all(&cur_block).unwrap();
            file_cost += start.elapsed().as_secs_f32();
        }
    }

    (vde_cost, file_cost, depend_cost, hash_cost, block_cost, xor_cost)
}