use std::{fs::File, io::Write};
use ark_std::{test_rng, rand::Rng};

pub fn padding(data: &Vec<u8>, n: usize) -> Vec<u8> {
    let mut res = data.clone();
    if data.len() < n {
        res.append(&mut vec![0u8; n - data.len()]);
    }
    res
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

pub fn create_random_file(n: usize, dir: &str) -> std::io::Result<()> {
    let mut file = File::create(dir).expect("Create file failed!");
    
    // fill file with random byte
    let mut rng = test_rng();
    const CHARSET: &[u8] = b"0123456789abcdef";
    for _ in 0..n {
        file.write_all(
            {
                let idx = rng.gen_range(0..CHARSET.len());
                &[CHARSET[idx]]
            }
        ).expect("Write failed!");
    }
    Ok(())
}

#[test]
fn test_data() {
    const DATA_DIR: &str = r"src\data.txt";
    let n = 1024 * 1024;
    create_random_file(n, DATA_DIR).unwrap();
}

#[test]
fn test_bits_xor() {
    let left = vec![1, 1, 1];
    let right = vec![0, 0, 0];
    let res = vecu8_xor(&left, &right);
    println!("{:?}", res);
}