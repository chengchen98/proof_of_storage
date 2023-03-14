use rand::Rng;
use std::collections::BTreeSet;
use std::fs::{File, OpenOptions};
use std::io::{Seek, Read, SeekFrom, Write};

use super::depend::{long_depend, short_depend};
use super::postorage::{DATA_L, L2, L1};

pub fn create_random_file(path: &str) -> std::io::Result<()> {
    //! 随机创建长度为 DATA_L 字节的文件
    let mut file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .truncate(true)
    .open(path)
    .unwrap();

    let mut rng = rand::thread_rng();
    for _ in 0..DATA_L {
        let buf: [u8; 1] = [rng.gen_range(0u8..=255u8)];
        file.write_all(&buf).unwrap();
    }
    Ok(())
}

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

pub fn create_depend(idx_cnt_l: usize, idx_cnt_s: usize, mode_l: usize, mode_s: usize) -> (Vec<Vec<Vec<usize>>>, Vec<Vec<Vec<usize>>>) {
    //! 创建数据块之间的依赖关系，包括短程依赖和长程依赖
    let l2_cnt = DATA_L / L2;
    let l1_cnt = L2 / L1;

    let mut idx_l = vec![];
    let mut idx_s = vec![];

    for cnt2 in 0..l2_cnt {
        let mut cur_idx_l = vec![];
        let mut cur_idx_s = vec![];
        for cnt1 in 0..l1_cnt {
            cur_idx_l.push(long_depend(cnt2, idx_cnt_l, mode_l));
            cur_idx_s.push(short_depend(l1_cnt, cnt1, idx_cnt_s, mode_s));
        }
        idx_l.push(cur_idx_l);
        idx_s.push(cur_idx_s);
    }

    (idx_l, idx_s)
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

pub fn generate_sorted_unique_random_numbers(count: usize, range: (usize, usize)) -> Vec<usize> {
    //! 生成 count 个随机数，范围是 [left, right)
    let mut rng = rand::thread_rng();
    let mut set = BTreeSet::new();

    while set.len() < count as usize {
        let num = rng.gen_range(range.0 .. range.1);
        set.insert(num);
    }

    set.into_iter().collect()
}