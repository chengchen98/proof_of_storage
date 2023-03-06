use std::io::SeekFrom;
use std::io::prelude::*;
use std::fs::{File, OpenOptions};
use std::ops::Add;
use std::str::FromStr;
use std::time::Instant;

use ark_bls12_381::{Fr, Bls12_381};
use ark_ff::{BigInteger, BigInteger256};
use rand::Rng;
use std::path::PathBuf; 

use ark_groth16::{
    Proof, PreparedVerifyingKey,
    create_random_proof, generate_random_parameters, prepare_verifying_key, verify_proof,
};

use crate::common::convert::{bits_to_bytes, bytes_to_bits, bits_to_usize, usize_to_bits};
use crate::common::mimc_df::mimc5_df;
use crate::common::mimc_hash::multi_mimc5_hash;
use crate::circuit::pos_circuit::PosDemo;

// 所声明的存储空间的位置
const DATA_DIR: [&str; 3] = [r"src", "proof_of_space", "pos_data"];
// 验证者生成的挑战个数
const CHALLENGE_COUNT: usize = 20;
// 证明者需要响应的挑战个数
const RESPONSE_COUNT: usize = 10;
// x的取值范围[0..2^N]
const N: usize = 20;
// 延迟函数的轮数
const MIMC5_DF_ROUNDS: usize = 322;
// 哈希函数的轮数
const MIMC5_HASH_ROUNDS: usize = 110;

pub fn prepare_storage(file: &mut File, count: usize) -> std::io::Result<()> {
    //! 提供 count = (N+1)*2^N bits 的存储空间，初始将内容全部设置为0
    let buf = [0];
    for i in 0..(N + 1) * count / 8 {
        file.seek(SeekFrom::Start(i.try_into().unwrap())).unwrap();
        file.write_all(&buf).unwrap();
    }
    Ok(())
}

pub fn mark_storage(file: &mut File, key: Fr, count: usize, m: Fr, df_constants: &Vec<Fr>) -> std::io::Result<()> {
    //! 计算延迟函数，并将结果修改进存储空间
    // y的前n位作为idx，x作为val，在y位置存储x
    for x in 0..count {
   
        // 计算df得到y，再将y转化为二进制形式，取前n位作为idx
        // let y = mimc5_df(Fr::from(x).add(&key), m, &df_constants);
        let y = mimc5_df(Fr::from_str(&x.to_string()).unwrap().add(&key), m, &df_constants);
        let y: BigInteger256 = y.into();
        let yn_bits = y.to_bits_le()[0..N].to_vec();

        // 需要修改位置的起始比特数
        let y_usize = bits_to_usize(&yn_bits);
        let yn_bit_idx = (N + 1) * y_usize;

        // 需要修改位置的起始字节数
        let mut yn_byte_idx = yn_bit_idx / 8;

        // 左边byte中不需要修改的bit个数
        let yn_left_count = yn_bit_idx % 8;

        // 将x转化为二进制形式，并在最右边添加一个标识位true，此时的x为N+1位
        let mut x_bits = usize_to_bits(x, N);
        x_bits.push(true);
        
        // 遍历文件，逐个取出字节并按比特位修改
        let mut buf = vec![0u8];
        file.seek(SeekFrom::Start(yn_byte_idx.try_into().unwrap())).unwrap();
        file.read(&mut buf).unwrap();
        let mut buf_bits = bytes_to_bits(&buf);

        // 待写入字节的比特位的idx
        let mut j = yn_left_count;

        // 遍历x_bits的比特位，逐个写入文件
        for i in 0..x_bits.len() {
            buf_bits[j] = x_bits[i];
            j += 1;

            // 当满一个字节，就将修改后的字节写入文件，并移动到下一个字节处理
            if j == 8 {
                buf = bits_to_bytes(&buf_bits);
                file.seek(SeekFrom::Start(yn_byte_idx.try_into().unwrap())).unwrap();
                file.write_all(&buf).unwrap();
                yn_byte_idx += 1;

                file.seek(SeekFrom::Start(yn_byte_idx.try_into().unwrap())).unwrap();
                file.read(&mut buf).unwrap(); 
                buf_bits = bytes_to_bits(&buf);

                j = 0;
            }
        }

        // 写入最后一个修改后的字节
        if j != 0 {
            buf = bits_to_bytes(&buf_bits);
            file.seek(SeekFrom::Start(yn_byte_idx.try_into().unwrap())).unwrap();
            file.write_all(&buf).unwrap();
        }
    }

    Ok(())
}

pub fn create_challenge(n: usize) -> Vec<usize> {
    //! 验证者随机生成挑战
    let mut rng = rand::thread_rng();
    let mut challenges = vec![];
    const CHOICE: &[bool] = &[false, true];

    // 生成n个N比特的挑战
    for _ in 0..n {
        let challenge: Vec<bool> = (0..N)
        .map(|_| { 
            let idx = rng.gen_range(0..2);
            CHOICE[idx]
         }).collect();
        challenges.push(bits_to_usize(&challenge));
    }
    challenges
}

pub fn response_1(file: &mut File, challenges: &Vec<usize>, key: Fr, hash_constants: &Vec<Fr>) -> (Vec<Fr>, Vec<usize>, Fr) {
    //! 响应挑战
    let mut x_response  = vec![];
    let mut idx_response = vec![];
    let x_hash_response;

    let mut count = 0; 

    // 逐个响应挑战，根据challenge[i]（即yn）找到对应的x
    for i in 0..challenges.len() {
        // 一个挑战对应一个yn
        let yn = challenges[i];
        let yn_bit_idx = (N + 1) * yn;
        let mut yn_byte_idx = yn_bit_idx / 8;
        let yn_left_count = yn_bit_idx % 8;

        let mut buf = vec![0u8];
        file.seek(SeekFrom::Start(yn_byte_idx.try_into().unwrap())).unwrap();
        file.read(&mut buf).unwrap(); 
        let mut buf_bits = bytes_to_bits(&buf);

        // 待读出字节的比特位的idx
        let mut k = yn_left_count;

        // 待读出的x
        let mut x_bits = vec![false; N + 1];

        // 从yn_bit_idx开始，连续读出N+1个比特
        for j in 0..x_bits.len() {
            x_bits[j] = buf_bits[k];
            k += 1;

            // 当读满一个字节，移动到下一个字节读出
            if k == 8 {
                yn_byte_idx += 1;

                file.seek(SeekFrom::Start(yn_byte_idx.try_into().unwrap())).unwrap();
                file.read(&mut buf).unwrap();
                buf_bits = bytes_to_bits(&buf);

                k = 0;
            }
        }

        // 若标识位为true，表示当前y是由x计算得来的，否则说明当前y没有原像
        // 去掉最右边的标识位，恢复x
        if x_bits[N] == true {
            let x_usize = bits_to_usize(&x_bits[0..N].to_vec());
            let x = Fr::from_str(&x_usize.to_string()).unwrap();
            x_response.push(x);
            idx_response.push(i);
            count += 1;
        }
        
        // 只需要响应指定个数的挑战
        if count == RESPONSE_COUNT {
            break;
        }
    }

    x_hash_response = multi_mimc5_hash(&x_response, key, &hash_constants);
    (x_response, idx_response, x_hash_response)
}

pub fn response_2(key: Fr, x_response: &Vec<Fr>, m: Fr, df_constants: &Vec<Fr>, challenges: &Vec<usize>, idx_response: &Vec<usize>, x_hash: Fr, hash_constants: &Vec<Fr>) 
-> (PreparedVerifyingKey<Bls12_381>, Proof<Bls12_381>) {
    //! 生成零知识证明
    let mut rng = rand::thread_rng();

    // 构建电路，生成参数
    let start = Instant::now();
    let params = {
        let c = PosDemo {
            key: None,
            x: &[None; RESPONSE_COUNT],
            m: None,
            df_constants: &df_constants,
    
            yn: &[None; RESPONSE_COUNT],
    
            x_hash: None,
            hash_constants: &hash_constants
        };
        generate_random_parameters::<Bls12_381, _, _>(c, &mut rng).unwrap()
    };
    println!("(1) Generate params: {:?}", start.elapsed());
 
    let pvk = prepare_verifying_key(&params.vk);
    
    // 生成证明
    let start = Instant::now();
    let proof = {
        let x = (0..x_response.len()).map(|i| Some(x_response[i])).collect::<Vec<_>>();
        let x = x.as_slice();
        let yn = (0..idx_response.len()).map(|i| Some(Fr::from_str(&challenges[idx_response[i]].to_string()).unwrap())).collect::<Vec<_>>();
        let yn = yn.as_slice();

        let c = PosDemo {
            key: Some(key),
            x: x,
            m: Some(m),
            df_constants: &df_constants,

            yn: yn,

            x_hash: Some(x_hash),
            hash_constants: &hash_constants
        };
        
        create_random_proof(c, &params, &mut rng).unwrap()
    };
    println!("(2) Create proof: {:?}", start.elapsed());

    (pvk, proof)
}

pub fn verify(pvk: PreparedVerifyingKey<Bls12_381>, proof: Proof<Bls12_381>, key: Fr, m: Fr, challenges: &Vec<usize>, idx_response: &Vec<usize>, x_hash: Fr) {
    //! 验证零知识证明
    let mut verify_input = vec![];
    verify_input.push(key);
    verify_input.push(m);

    for i in 0.. idx_response.len() {
        let yn = challenges[idx_response[i]];
        verify_input.push(Fr::from_str(&yn.to_string()).unwrap());
    }

    verify_input.push(x_hash);

    assert!(verify_proof(&pvk, &proof, &verify_input).is_ok());
}

pub fn test_pos() {
    let mut rng = rand::thread_rng();

    // 准备计算DF和Hash的常数
    let df_constants: Vec<Fr> = (0..MIMC5_DF_ROUNDS)
    .map(|_| rng.gen())
    .collect::<Vec<_>>();

    let hash_constants: Vec<Fr> = (0..MIMC5_HASH_ROUNDS)
    .map(|_| rng.gen())
    .collect::<Vec<_>>();

    let key: Fr = rng.gen();
    let m: Fr = rng.gen();

    println!("-------------------------------------");

    let path: PathBuf = DATA_DIR.iter().collect();
    let path = path.to_str().unwrap();

    // 计算需要计算延迟函数的次数
    // let base: usize = 2;
    // let count = base.pow(N.try_into().unwrap());

    // 先划分一定大小的存储空间，并用0填满
    // let start = Instant::now();
    // let mut file = OpenOptions::new()
    // .read(true)
    // .write(true)
    // .create(true)
    // .truncate(true)
    // .open(path)
    // .unwrap();
    // prepare_storage(&mut file, count).unwrap();
    // println!("Prepare storage: {:?}", start.elapsed());
    // println!("-------------------------------------");

    let mut file = OpenOptions::new()
    .read(true)
    .write(true)
    .open(path)
    .unwrap();

    // // 通过计算延迟函数标定存储空间
    // let start = Instant::now();
    // mark_storage(&mut file, key, count, m, &df_constants).unwrap();
    // println!("Create pos: {:?}", start.elapsed());
    // println!("-------------------------------------");

    // 验证者随机生成挑战
    let start = Instant::now();
    let challenges = create_challenge(CHALLENGE_COUNT);
    println!("Create challenge: {:?}", start.elapsed());
    println!("-------------------------------------");

    // 第一次响应：
    let start = Instant::now();
    let (x_response, idx_response, x_hash_response) = response_1(&mut file, &challenges, key, &hash_constants);
    assert_eq!(x_response.len(), RESPONSE_COUNT);
    println!("Response 1: {:?}", start.elapsed());
    println!("-------------------------------------");

    // 计算成功率
    println!("Success rate: {:?} / {:?}", idx_response.len(), RESPONSE_COUNT);
    println!("-------------------------------------");

    // 第二次响应：生成零知识证明
    let start = Instant::now();
    let (pvk, proof) = response_2(key, &x_response, m,  &df_constants,  &challenges, &idx_response, x_hash_response, &hash_constants);
    println!("Response 2: {:?}", start.elapsed());
    println!("-------------------------------------");
    
    // 验证
    let start = Instant::now();
    verify(pvk, proof, key, m, &challenges, &idx_response, x_hash_response);
    println!("Verify: {:?}", start.elapsed());
    println!("-------------------------------------");
}

#[test]
fn test() {
    test_pos();
}