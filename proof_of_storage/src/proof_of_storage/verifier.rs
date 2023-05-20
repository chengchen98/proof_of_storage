use std::collections::BTreeSet;
use std::sync::{Arc, RwLock};
use std::{fs::OpenOptions, io::Write};

use rand::Rng;
use rug::Integer;
use threadpool::ThreadPool;

use crate::vde::rug_vde::vde_inv;

use super::common::{read_file, to_units, modsub, blake3_hash};
use super::depend::{short_depend_random, long_mode_random};
use super::postorage::PosPara;
use super::prover::{create_short_depend, create_long_depend};

pub fn create_random_file(path: &str, data_len: usize) -> std::io::Result<()> {
    //! 随机创建长度为 params.data_l 字节的文件
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

pub fn single_unseal_prepare(sealed_path: &str, block_idx: usize, params: &PosPara) 
-> (Vec<Vec<u8>>, Vec<u8>, Vec<Vec<Vec<u8>>>) {
    let mut sealed_file = OpenOptions::new()
    .read(true)
    .open(sealed_path)
    .unwrap();

    // 从文件中读出二级数据块集合
    let block = {
        let buf = read_file(&mut sealed_file, block_idx * params.block_pl, params.block_pl);
        let ans = to_units(&buf, params.unit_pl);
        ans
    };

    let block_cnt = params.data_l / params.block_l;

    let idxs_l = {
        if params.mode_l != 0 {
            create_long_depend(block_cnt, params.cnt_l, params.mode_l)
        }
        else {
            vec![]
        }
    };

    let before_block_id = {
        if block_idx != 0 {
            let before_block = read_file(&mut sealed_file, (block_idx - 1) * params.block_pl, params.block_pl);
            blake3_hash(&before_block)
        }
        else {
            vec![]
        }
    };

    let depend_blocks = {
        let mut res = vec![];
        if params.mode_l == 0 {
            let cur_idxs_l = {
                if block_idx == 0 {
                    vec![]
                }
                else {
                    if params.cnt_l == 0 {
                        long_mode_random(block_cnt, &before_block_id, block_idx, block_idx / 10 + 1)
                    }
                    else {
                        long_mode_random(block_cnt, &before_block_id, block_idx, params.cnt_l)
                    }
                }
            };

            for &idx in &cur_idxs_l {
                let buf = read_file(&mut sealed_file, idx * params.block_pl, params.block_pl);
                let ans = to_units(&buf, params.unit_pl);
                res.push(ans);
            }
        }
        else {
            for &i in &idxs_l[block_idx] {
                let buf = read_file(&mut sealed_file, i * params.block_pl, params.block_pl);  
                let ans = to_units(&buf, params.unit_pl);
                res.push(ans);
            }
        }
        res
    };

    (block, before_block_id, depend_blocks)
}

pub fn batch_unseal_prepare(sealed_path: &str, idx_begin: usize, idx_end: usize, params: &PosPara) 
-> (Vec<usize>, Vec<Vec<Vec<u8>>>, Vec<Vec<u8>>, Vec<Vec<Vec<Vec<u8>>>>) {
    let mut sealed_file = OpenOptions::new()
    .read(true)
    .open(sealed_path)
    .unwrap();

    let blocks_idx = {
        let mut res = vec![];
        let mut i = 0;
        while i * params.block_pl + idx_begin < idx_end {
            res.push(i + idx_begin / params.block_pl);
            i += 1;
        }
        res
    };

    // 从文件中读出二级数据块集合
    let blocks = {
        let mut res = vec![];
        for &idx2 in &blocks_idx {
            let buf = read_file(&mut sealed_file, idx2 * params.block_pl, params.block_pl);
            let ans = to_units(&buf, params.unit_pl);
            res.push(ans);
        }
        res
    };

    let block_cnt = params.data_l / params.block_l;

    let idxs_l = {
        if params.mode_l != 0 {
            create_long_depend(block_cnt, params.cnt_l, params.mode_l)
        }
        else {
            vec![]
        }
    };

    let mut before_block_ids = vec![];
    let mut depend_blocks = vec![];

    for &idx2 in &blocks_idx {
        let single_before_block_id = {
            if idx2 != 0 {
                let before_block = read_file(&mut sealed_file, (idx2 - 1) * params.block_pl, params.block_pl);
                blake3_hash(&before_block)
            }
            else {
                vec![]
            }
        };

        let single_depend_blocks = {
            let mut res = vec![];
            if params.mode_l == 0 {
                let cur_idxs_l = {
                    if idx2 == 0 {
                        vec![]
                    }
                    else {
                        if params.cnt_l == 0 {
                            long_mode_random(block_cnt, &single_before_block_id, idx2, idx2 / 10 + 1)
                        }
                        else {
                            long_mode_random(block_cnt, &single_before_block_id, idx2, params.cnt_l)
                        }
                    }
                };

                for &idx in &cur_idxs_l {
                    let buf = read_file(&mut sealed_file, idx * params.block_pl, params.block_pl);
                    let ans = to_units(&buf, params.unit_pl);
                    res.push(ans);
                }
            }
            else {
                for &i in &idxs_l[idx2] {
                    let buf = read_file(&mut sealed_file, i * params.block_pl, params.block_pl);  
                    let ans = to_units(&buf, params.unit_pl);
                    res.push(ans);
                }
            }
            res
        };
        before_block_ids.push(single_before_block_id);
        depend_blocks.push(single_depend_blocks);
    }

    (blocks_idx, blocks, before_block_ids, depend_blocks)
}

pub fn batch_unseal_and_verify(params: &PosPara, origin_path: &str, blocks_idx: &Vec<usize>, blocks: &Vec<Vec<Vec<u8>>>, before_block_ids: &Vec<Vec<u8>>, depend_blocks: &Vec<Vec<Vec<Vec<u8>>>>, vde_key: &Integer, iv: &Vec<u8>) {
    let idxs_s = {
        if params.mode_s != 0 {
            create_short_depend(params.data_l / params.block_l, params.cnt_s, params.mode_s)
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
        let unit_cnt = cur_block.len();

        for _ in 0..params.seal_rounds {
            for j in 0..unit_cnt {
                let idx1 = unit_cnt - 1 - j;

                let mut depend_data = {
                    let mut res = vec![];
                    for k in 0..depend_blocks[i].len() {
                        res.append(&mut depend_blocks[i][k][idx1].clone());
                    }
                    if params.mode_s != 0 {
                        for &idx in &idxs_s[idx1] {
                            res.append(&mut cur_block[idx].clone());
                        }
                    }
                    else {
                        let ans = {
                            if idx1 == 0 {
                                short_depend_random(unit_cnt, &vec![], idx1, params.cnt_s)
                            }
                            else {
                                short_depend_random(unit_cnt, &cur_block[idx1-1], idx1, params.cnt_s)
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
                        depend_data.append(&mut before_block_ids[i].clone());
                    }
                }

                let depend_data_hash = blake3_hash(&depend_data);
                let cur_unit = &cur_block[idx1].to_vec();
                let vde_inv_res = vde_inv(&cur_unit, vde_key, params.vde_rounds, &params.vde_mode, params.unit_pl);
                let new_unit = modsub(&vde_inv_res, &depend_data_hash, &vde_key);
                cur_block[idx1] = new_unit;
            }
        }

        let mut origin_file = OpenOptions::new()
        .read(true)
        .open(origin_path)
        .unwrap();

        let origin_block = read_file(&mut origin_file, idx2 * params.block_l, params.block_l);
        let origin_block = to_units(&origin_block, params.unit_l);
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

                if k == params.unit_l {
                    k_cur += 1;
                    k = 0;
                }
            }
        }
    }
}

pub fn batch_unseal(params: &PosPara, blocks_idx: &Vec<usize>, blocks: &mut Vec<Vec<Vec<u8>>>, before_block_ids: &Vec<Vec<u8>>, depend_blocks: &Vec<Vec<Vec<Vec<u8>>>>, vde_key: &Integer, iv: &Vec<u8>) 
-> Vec<Vec<Vec<u8>>> {
    let idxs_s = {
        if params.mode_s != 0 {
            create_short_depend(params.block_l / params.unit_l, params.cnt_s, params.mode_s)
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
        let unit_cnt = cur_block.len();

        for _ in 0..params.seal_rounds {
            for j in 0..unit_cnt {
                let idx1 = unit_cnt - 1 - j;

                let mut depend_data = {
                    let mut res = vec![];
                    for k in 0..depend_blocks[i].len() {
                        res.append(&mut depend_blocks[i][k][idx1].clone());
                    }
                    if params.mode_s != 0 {
                        for &idx in &idxs_s[idx1] {
                            res.append(&mut cur_block[idx].clone());
                        }
                    }
                    else {
                        let ans = {
                            if idx1 == 0 {
                                short_depend_random(unit_cnt, &vec![], idx1, params.cnt_s)
                            }
                            else {
                                short_depend_random(unit_cnt, &cur_block[idx1-1], idx1, params.cnt_s)
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
                        depend_data.append(&mut before_block_ids[i].clone());
                    }
                }

                let depend_data_hash = blake3_hash(&depend_data);
                let cur_unit = &cur_block[idx1].to_vec();
                let vde_inv_res = vde_inv(&cur_unit, vde_key, params.vde_rounds, &params.vde_mode, params.unit_pl);
                let new_unit = modsub(&vde_inv_res, &depend_data_hash, &vde_key);
                cur_block[idx1] = new_unit;
            }
        }
        blocks[i] = cur_block;
    }
    blocks.to_vec()
}

pub fn batch_unseal_parallel(params: &PosPara, blocks_idx: &Vec<usize>, blocks: &Vec<Vec<Vec<u8>>>, before_block_ids: &Vec<Vec<u8>>, depend_blocks: &Vec<Vec<Vec<Vec<u8>>>>, vde_key: &Integer, iv: &Vec<u8>, parallel_num: usize) -> Vec<Vec<Vec<u8>>> {
    let idxs_s = {
        if params.mode_s != 0 {
            create_short_depend(params.data_l / params.block_l, params.cnt_s, params.mode_s)
        }
        else {
            vec![]
        }
    };

    let pool = ThreadPool::new(parallel_num);

    let iv_arc = Arc::new(RwLock::new(iv.clone()));
    let blocks_idx_arc = Arc::new(RwLock::new(blocks_idx.clone()));
    let blocks_arc = Arc::new(RwLock::new(blocks.clone()));
    let before_block_ids_arc = Arc::new(RwLock::new(before_block_ids.clone()));
    let depend_blocks_arc = Arc::new(RwLock::new(depend_blocks.clone()));
    let vde_key_arc = Arc::new(RwLock::new(vde_key.clone()));
    let idxs_s_arc = Arc::new(RwLock::new(idxs_s.clone()));

    let params_arc = Arc::new(RwLock::new(params.clone()));

    // 逐个解封装一级数据块
    for i in 0..blocks.len() {

        let iv_copy = iv_arc.clone();
        let blocks_idx_copy = blocks_idx_arc.clone();
        let blocks_copy = blocks_arc.clone();
        let before_block_idxs_copy = before_block_ids_arc.clone();
        let depend_blocks_copy = depend_blocks_arc.clone();
        let vde_key_copy = vde_key_arc.clone();
        let idxs_s_copy = idxs_s_arc.clone();

        let params_copy = params_arc.clone();
        
        pool.execute(move || {
            // 当前二级数据块编号
            let idx2 = blocks_idx_copy.read().unwrap()[i];
            // 当前二级数据块内容
            let mut cur_block = blocks_copy.read().unwrap()[i].clone();
            let unit_cnt = cur_block.len();

            for _ in 0..params_copy.read().unwrap().seal_rounds {
                for j in 0..unit_cnt {
                    let idx1 = unit_cnt - 1 - j;

                    let mut depend_data = {
                        let mut res = vec![];
                        for k in 0..depend_blocks_copy.read().unwrap()[i].len() {
                            res.append(&mut depend_blocks_copy.read().unwrap()[i][k][idx1].clone());
                        }
                        if params_copy.read().unwrap().mode_s != 0 {
                            for &idx in &idxs_s_copy.read().unwrap()[idx1] {
                                res.append(&mut cur_block[idx].clone());
                            }
                        }
                        else {
                            let ans = {
                                if idx1 == 0 {
                                    short_depend_random(unit_cnt, &vec![], idx1, params_copy.read().unwrap().cnt_s)
                                }
                                else {
                                    short_depend_random(unit_cnt, &cur_block[idx1-1], idx1, params_copy.read().unwrap().cnt_s)
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
                            depend_data.append(&mut iv_copy.read().unwrap().clone());
                        }
                        else {
                            depend_data.append(&mut before_block_idxs_copy.read().unwrap()[i].clone());
                        }
                    }
                    let depend_data_hash = blake3_hash(&depend_data);
                    let cur_unit = &cur_block[idx1].to_vec();
                    let vde_inv_res = vde_inv(&cur_unit, &vde_key_copy.read().unwrap(), params_copy.read().unwrap().vde_rounds, &params_copy.read().unwrap().vde_mode, params_copy.read().unwrap().unit_pl);
                    let new_unit = modsub(&vde_inv_res, &depend_data_hash, &vde_key_copy.read().unwrap());
                    cur_block[idx1] = new_unit;
                }
            }
            blocks_copy.write().unwrap()[i] = cur_block;
        });
    }
    pool.join();

    blocks_arc.clone().read().unwrap().to_vec()
}

pub fn batch_verify(origin_path: &str, idx2: usize, unseal_block: &Vec<Vec<u8>>, block_l: usize, unit_l: usize) {
    let mut origin_file = OpenOptions::new()
    .read(true)
    .open(origin_path)
    .unwrap();

    let origin_block = read_file(&mut origin_file, idx2 * block_l, block_l);
    let origin_block = to_units(&origin_block, unit_l);
    for j in 0.. origin_block.len() {
        let mut k_ori = 0;
        let mut k_cur = 0;
        let mut k = 0;
        while k_ori < origin_block[j].len() {
            if origin_block[j][k_ori] != unseal_block[j][k_cur] {
                println!("idx2: {:?}", idx2);
                println!("j: {:?}", j);
                println!("k_cur: {:?}", k_cur);
                println!("k_ori: {:?}", k_ori);
            }
            assert_eq!(origin_block[j][k_ori], unseal_block[j][k_cur]);
            k_ori += 1;
            k_cur += 1;
            k += 1;

            if k == unit_l {
                k_cur += 1;
                k = 0;
            }
        }
    }
}