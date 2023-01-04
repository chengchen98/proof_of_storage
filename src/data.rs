use std::{fs::File, io::Write};
use serde::{Serialize, Deserialize};

use rand::{thread_rng, Rng};

#[derive(Serialize, Deserialize, Debug)]
pub struct Data<T> {
    content: T
}

pub fn padding(input: Vec<u8>, n: usize) -> Vec<u8> {
    let mut res = vec![];
    if input.len() % n != 0 {
        res = vec![0u8; n - input.len() % n];
        res.append(&mut input.clone());
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
    const DIR: &str = r"D:\graduation\code\proof_of_storage\src\data.json";
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
        println!("{:?}", x);
        write_file(&x.as_bytes().to_vec(), DIR).unwrap();
    }

    #[test]
    fn test_read() {
        let x = read_file(DIR);
        let x = String::from_utf8(x).unwrap();
        println!("{:?}", x);
    }
}