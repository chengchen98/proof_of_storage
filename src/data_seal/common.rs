use rand::Rng;
use std::fs::File;
use std::io::{Seek, Read, SeekFrom, Write};

use super::depend::{long_depend, short_depend};

const DATA_L: usize = 128 * 127;

pub fn read_file(file: &mut File, begin_idx: usize, len: usize) -> Vec<u8> {
    let mut buf = vec![0; len];
    file.seek(SeekFrom::Start(begin_idx.try_into().unwrap())).unwrap();
    file.read(&mut buf).unwrap();
    buf
}

pub fn create_random_file(file: &mut File) -> std::io::Result<()> {
    //! 假设原始数据大小为DATA_L字节，随机生成文件保存
    let mut rng = rand::thread_rng();
    for i in 0..DATA_L {
        let buf: [u8; 1] = [rng.gen_range(0u8..=255u8)];
        file.seek(SeekFrom::Start(i.try_into().unwrap())).unwrap();
        file.write_all(&buf).unwrap();
    }
    Ok(())
}

pub fn to_block(data: &Vec<u8>, n: usize) -> Vec<Vec<u8>> {
    let mut res = vec![];
    for _ in (0..data.len()).step_by(n) {
        let mut tmp = vec![];
        for i in 0..n {
            tmp.push(data[i]);
        }
        res.push(tmp);
    }
    res
}

pub fn com_block(data: &Vec<Vec<u8>>) -> Vec<u8> {
    let mut res = vec![];
    for i in 0..data.len() {
        for j in 0..data[i].len() {
            res.push(data[i][j]);
        }
    }
    res
}

pub fn pad_block(data: &Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let mut res = data.clone();
    for i in 0..data.len() {
        res[i].push(0);
    }
    res
}

pub fn compress_block(data: &Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let mut res = vec![];
    for i in 0..data.len() {
        res.push(data[i][0..data[i].len() - 1].to_vec());
    }
    res
}

pub fn create_depend(block_num2: usize, block_num1: usize, idx_cnt_l: usize, idx_cnt_s: usize, mode_l: usize, mode_s: usize) -> (Vec<Vec<Vec<usize>>>, Vec<Vec<Vec<usize>>>) {
    //! 创建数据块之间的依赖关系，包括短程依赖和长程依赖
    let mut idx_l = vec![];
    let mut idx_s = vec![];

    for i in 0..block_num2 {
        let mut cur_idx_l = vec![];
        let mut cur_idx_s = vec![];
        for j in 0..block_num1 {
            cur_idx_l.push(long_depend(i, idx_cnt_l, mode_l));
            cur_idx_s.push(short_depend(block_num1, j, idx_cnt_s, mode_s));
        }
        idx_l.push(cur_idx_l);
        idx_s.push(cur_idx_s);
    }

    (idx_l, idx_s)
}
