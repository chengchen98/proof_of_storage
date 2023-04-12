use std::fs::File;
use std::io::{Seek, Read, SeekFrom};

pub fn read_file(file: &mut File, begin_idx: usize, len: usize) -> Vec<u8> {
    //! 从 file 的 begin_idx 字节开始，读取 len 字节
    let mut buf = vec![0; len];
    file.seek(SeekFrom::Start(begin_idx.try_into().unwrap())).unwrap();
    file.read(&mut buf).unwrap();
    buf
}

pub fn to_block(data: &Vec<u8>, len: usize) -> Vec<Vec<u8>> {
    //! 将一个二级数据块按固定长度 len 分成多个一级数据块
    let mut res = vec![];
    for i in (0..data.len()).step_by(len) {
        res.push(data[i .. i + len].to_vec());
    }
    res
}

pub fn com_block(data: &Vec<Vec<u8>>) -> Vec<u8> {
    //! 将多个一级数据块合并成一个二级数据块
    let mut res = vec![];
    for i in 0..data.len() {
        for j in 0..data[i].len() {
            res.push(data[i][j]);
        }
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