use std::fs::File;
use std::io::{Seek, Read, SeekFrom};
use md5::{Md5, Digest};
use rug::Integer;
use rug::integer::Order;
use blake3;

use super::postorage::PosPara;

pub const HASH_RES_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "hash"];

pub fn md5_hash(message: &Vec<u8>) -> Vec<u8> {
    let mut hasher = Md5::new();
    hasher.update(message);
    let res = hasher.finalize();
    res.as_slice().to_vec()
}

pub fn blake3_hash(message: &Vec<u8>) -> Vec<u8> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(message);
    let res = hasher.finalize();
    res.as_bytes().to_vec()
}

#[test]
fn test() {
    use std::time::Instant;
    use rand::Rng;
    use std::path::PathBuf;
    use std::fs::OpenOptions;
    use std::io::Write;

    let path: PathBuf = HASH_RES_DIR.iter().collect();
    let path = path.to_str().unwrap();
    let mut save_file = OpenOptions::new()
    .read(true)
    .write(true)
    .append(true)
    .create(true) 
    .open(path)
    .unwrap();

    let should_save = true;

    let message_len = 1024;
    let mut md5_cost = 0.0;
    let mut blake3_cost = 0.0;

    const SAMPLES: usize = 10000;
    for _ in 0..SAMPLES {
        let message = {
            let mut rng = rand::thread_rng();
            let mut res = vec![];
            for _ in 0..message_len {
                let buf: u8 = rng.gen_range(0u8..=255u8);
                res.push(buf);
            }
            res
        };
    
        let start = Instant::now();
        let _ = md5_hash(&message);
        md5_cost += start.elapsed().as_secs_f32();
    
        let start = Instant::now();
        let _ = blake3_hash(&message);
        blake3_cost += start.elapsed().as_secs_f32();
    }

    if should_save == true {
        md5_cost /= SAMPLES as f32;
        blake3_cost /= SAMPLES as f32;
        save_file.write_all(["message len (bytes), ", &message_len.to_string(), ", md5, ", &md5_cost.to_string(), ", blake3, ", &blake3_cost.to_string(), "\n\n"].concat().as_bytes()).unwrap();
    }
}

pub fn read_file(file: &mut File, begin_idx: usize, len: usize) -> Vec<u8> {
    //! 从 file 的 begin_idx 字节开始，读取 len 字节
    let mut buf = vec![0; len];
    file.seek(SeekFrom::Start(begin_idx.try_into().unwrap())).unwrap();
    file.read(&mut buf).unwrap();
    buf
}

pub fn to_units(data: &Vec<u8>, len: usize) -> Vec<Vec<u8>> {
    //! 将一个二级数据块按固定长度 len 分成多个一级数据块
    let mut res = vec![];
    for i in (0..data.len()).step_by(len) {
        res.push(data[i .. i + len].to_vec());
    }
    res
}

pub fn com_units(data: &Vec<Vec<u8>>) -> Vec<u8> {
    //! 将多个一级数据块合并成一个二级数据块
    let mut res = vec![];
    for i in 0..data.len() {
        res.append(&mut data[i].clone());
    }
    res
}

pub fn vecu8_xor(left: &Vec<u8>, right: &Vec<u8>) -> Vec<u8> {
    //! 两个vec<u8>逐位异或，结果长度等于 max(left.len(), right.len())
    let mut res = vec![];
    let len_left = left.len();
    let len_right = right.len();

    if len_left > len_right {
        for i in 0..len_right {
            res.push(left[i] ^ right[i]);
        }
        for j in len_right..len_left {
            res.push(left[j]);
        }
    }
    else {
        for i in 0..len_left {
            res.push(left[i] ^ right[i]);
        }
        for j in len_left..len_right {
            res.push(right[j]);
        }
    }
    res
}

pub fn modadd(left: &Vec<u8>, right: &Vec<u8>, p: &Integer) -> Vec<u8> {
    let left_int = Integer::from_digits(left, Order::Lsf);
    let right_int = Integer::from_digits(right, Order::Lsf);
    let res_int = (left_int + right_int) % p;
    let mut res = res_int.to_digits::<u8>(Order::Lsf);
    res.append(&mut vec![0u8; left.len() - res.len()]);
    res
}

pub fn modsub(left: &Vec<u8>, right: &Vec<u8>, p: &Integer) -> Vec<u8> {
    let left_int = Integer::from_digits(left, Order::Lsf);
    let right_int = Integer::from_digits(right, Order::Lsf);
    let res_int = (left_int + p - right_int) % p;
    let mut res = res_int.to_digits::<u8>(Order::Lsf);
    res.append(&mut vec![0u8; left.len() - res.len()]);
    res
}

pub fn gen_posdata(l: usize) -> PosPara {
    let params = {
        if l == 1 {
            PosPara {
                data_l: 63 * 16 * 1024 * 1,
        
                unit_l: 63,
                block_l: 63 * 64,
                big_block_l: 63 * 128 * 128,
            
                unit_pl: 64,
                block_pl: 64 * 64,
                big_block_pl: 64 * 128 * 128,
            
                seal_rounds: 2,
                vde_rounds: 10,
                vde_mode: "sloth".to_string(),
        
                mode_l: 0,
                mode_s: 0,
        
                cnt_l: 0,
                cnt_s: 10,
            
                leaves_to_prove_count: 3,
            }
        }
        else if l == 4 {
            PosPara {
                data_l: 63 * 16 * 1024 * 4,
        
                unit_l: 63,
                block_l: 63 * 256,
                big_block_l: 63 * 128 * 128,
            
                unit_pl: 64,
                block_pl: 64 * 256,
                big_block_pl: 64 * 128 * 128,
            
                seal_rounds: 2,
                vde_rounds: 10,
                vde_mode: "sloth".to_string(),
        
                mode_l: 0,
                mode_s: 0,
        
                cnt_l: 0,
                cnt_s: 10,
            
                leaves_to_prove_count: 3,
            }
        }
        else if l == 16 {
            PosPara {
                data_l: 63 * 16 * 1024 * 16,
        
                unit_l: 63,
                block_l: 63 * 1024,
                big_block_l: 63 * 128 * 128,
            
                unit_pl: 64,
                block_pl: 64 * 1024,
                big_block_pl: 64 * 128 * 128,
            
                seal_rounds: 2,
                vde_rounds: 10,
                vde_mode: "sloth".to_string(),
        
                mode_l: 0,
                mode_s: 0,
        
                cnt_l: 0,
                cnt_s: 10,
            
                leaves_to_prove_count: 3,
            }
        }
        else {
            PosPara {
                data_l: 63 * 16 * 1024 * 64,
        
                unit_l: 63,
                block_l: 63 * 4 * 1024,
                big_block_l: 63 * 128 * 128,
            
                unit_pl: 64,
                block_pl: 64 * 4 * 1024,
                big_block_pl: 64 * 128 * 128,
            
                seal_rounds: 2,
                vde_rounds: 10,
                vde_mode: "sloth".to_string(),
        
                mode_l: 0,
                mode_s: 0,
        
                cnt_l: 0,
                cnt_s: 10,
            
                leaves_to_prove_count: 3,
            }
        }
    };
    params
}
