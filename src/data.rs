use std::{fs::File, io::Write};
use serde::{Serialize, Deserialize};

use rand::{thread_rng, Rng};

pub const DATA_DIR: &str = r"D:\graduation\code\proof_of_storage\src\data.json";

#[derive(Serialize, Deserialize, Debug)]
pub struct Data<T> {
    pub content: T
}

pub fn padding(data: &Vec<u8>, n: usize) -> Vec<u8> {
    let mut res = vec![];
    if data.len() % n != 0 {
        res = vec![0u8; n - data.len() % n];
        res.append(&mut data.clone());
    }
    res
}

pub fn to_block(data: &Vec<u8>, len: usize) -> Vec<Vec<u8>> {
    let mut data_blocks = vec![];
    for i in (0..data.len()).step_by(len) {
        data_blocks.push(data[i .. i + len].to_vec());
    }
    data_blocks
}

pub fn vecu8_xor(left: &Vec<u8>, right: &Vec<u8>) -> Vec<u8> {
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

pub fn gen_random_data(n: usize) -> Vec<u8> {
    let mut rng = thread_rng();
    const CHARSET: &[u8] = b"0123456789abcdef";

    let data: Vec<u8> = (0..n)
    .map(|_| {
        let idx = rng.gen_range(0..CHARSET.len());
        CHARSET[idx]
    }).collect();
    data
}

pub fn gen_random_str(n: usize) -> String {
    let mut rng = thread_rng();
    const CHARSET: &[u8] = b"0123456789abcdef";

    let data: String = (0..n)
    .map(|_| {
        let idx = rng.gen_range(0..CHARSET.len());
        CHARSET[idx] as char
    }).collect();
    data
}

pub fn write_file(data: &Vec<u8>, dir: &str) -> std::io::Result<()> {
    let data = Data { content: data.clone() };
    let data = serde_json::to_string(&data).unwrap();
    
    let mut file = File::create(dir).expect("Create file failed!");
    file.write_all(&data.as_bytes()).expect("Write failed!");
    Ok(())
}

pub fn read_file(dir: &str) -> Vec<u8> {
    let file = File::open(dir).unwrap();
    let data: Data<Vec<u8>> = serde_json::from_reader(file).unwrap();
    data.content
}

#[cfg(test)]
mod test {
    use super::*;
    use std::mem;

    #[test]
    fn test_data() {
        let n = 20;
        let x = gen_random_data(n);
        println!("{:?}", mem::size_of_val(&x));
        let x = String::from_utf8(x).unwrap();
        println!("{:?}", mem::size_of_val(&x));
    }

    #[test]
    fn test_write() {
        let n = 256;
        let x = gen_random_str(n);
        write_file(&x.as_bytes().to_vec(), DATA_DIR).unwrap();
    }

    #[test]
    fn test_read() {
        let x = read_file(DATA_DIR);
        let x = String::from_utf8(x).unwrap();
        println!("{:?}", x);
    }

    #[test]
    fn test_to_block() {
        let data = vec![0u8; 32];
        let n1 = 16;
        let n2 = 4;
        let blocks1 = to_block(&data, n1);
        let blocks2 = blocks1.iter().map(|x| to_block(&x, n2)).collect::<Vec<Vec<Vec<_>>>>();
        println!("{:?}", blocks2.len());
        println!("{:?}", blocks2[0].len());
    }

    #[test]
    fn test_vecu8_xor() {
        let left = vec![1, 1, 1];
        let right = vec![1, 1, 1];
        let res = vecu8_xor(&left, &right);
        println!("{:?}", res);
    }
}