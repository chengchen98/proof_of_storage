use std::ops::Add;
use std::str::FromStr;
use std::fs::OpenOptions;
use std::io::{SeekFrom, prelude::*};
use std::time::Instant;

use ark_bls12_381::{Fr, Bls12_381, Parameters};
use ark_ec::bls12::Bls12;
use ark_ff::{BigInteger, BigInteger256};

use ark_groth16::{
    Proof, PreparedVerifyingKey,
    create_random_proof, generate_random_parameters, prepare_verifying_key, ProvingKey,
};

use crate::mimc::mimc_df::mimc5_df;
use crate::mimc::mimc_hash::multi_mimc5_hash;
use crate::circuit::pospace_circuit::PospaceDemo;

use super::convert::{bits_to_bytes, bytes_to_bits, bits_to_usize, usize_to_bits};

use super::pospace::RESPONSE_COUNT;

pub fn prepare_space(path: &str, n: usize) -> std::io::Result<()> {
    //! 提供 count = (N+1)*2^N bits 的存储空间，初始将内容全部设置为0
    let mut file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .truncate(true)
    .open(path)
    .unwrap();

    let base: usize = 2;
    let count = base.pow(n.try_into().unwrap());

    let buf = [0u8; 1];
    for _ in 0..(n + 1) * count / 8 {
        file.write_all(&buf).unwrap();
    }
    Ok(())
}

pub fn mark_space(path: &str, key: Fr, m: Fr, df_constants: &Vec<Fr>, n: usize) -> (f32, f32) {
    //! 计算延迟函数，并将结果修改进存储空间
    // y的前n位作为idx，x作为val，在y位置存储x
    let mut file = OpenOptions::new()
    .read(true)
    .write(true)
    .open(path)
    .unwrap();

    // 计算需要计算延迟函数的次数
    let base: usize = 2;
    let count = base.pow(n.try_into().unwrap());

    let mut df_cost = 0.0;
    let mut file_cost = 0.0;

    for x in 0..count {
   
        // 计算df得到y，再将y转化为二进制形式，取前n位作为idx
        let start = Instant::now();
        let y = mimc5_df(Fr::from_str(&x.to_string()).unwrap().add(&key), m, &df_constants);
        df_cost += start.elapsed().as_secs_f32();
        let y: BigInteger256 = y.into();
        let yn_bits = y.to_bits_le()[0..n].to_vec();

        // 需要修改位置的起始比特数
        let y_usize = bits_to_usize(&yn_bits);
        let yn_bit_idx = (n + 1) * y_usize;

        // 需要修改位置的起始字节数
        let mut yn_byte_idx = yn_bit_idx / 8;

        // 左边byte中不需要修改的bit个数
        let yn_left_count = yn_bit_idx % 8;

        // 将x转化为二进制形式，并在最右边添加一个标识位true，此时的x为N+1位
        let mut x_bits = usize_to_bits(x, n);
        x_bits.push(true);
        
        // 遍历文件，逐个取出字节并按比特位修改
        let mut buf = vec![0u8];
        let start = Instant::now();
        file.seek(SeekFrom::Start(yn_byte_idx.try_into().unwrap())).unwrap();
        file.read(&mut buf).unwrap();
        file_cost += start.elapsed().as_secs_f32();
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
                let start = Instant::now();
                file.seek(SeekFrom::Start(yn_byte_idx.try_into().unwrap())).unwrap();
                file.write_all(&buf).unwrap();
                file_cost += start.elapsed().as_secs_f32();
                yn_byte_idx += 1;

                let start = Instant::now();
                file.seek(SeekFrom::Start(yn_byte_idx.try_into().unwrap())).unwrap();
                file.read(&mut buf).unwrap(); 
                file_cost += start.elapsed().as_secs_f32();
                buf_bits = bytes_to_bits(&buf);

                j = 0;
            }
        }

        // 写入最后一个修改后的字节
        if j != 0 {
            buf = bits_to_bytes(&buf_bits);
            let start = Instant::now();
            file.seek(SeekFrom::Start(yn_byte_idx.try_into().unwrap())).unwrap();
            file.write_all(&buf).unwrap();
            file_cost += start.elapsed().as_secs_f32();
        }
    }
    println!("df cost: {:?}", df_cost);
    println!("file cost: {:?}", file_cost);

    (df_cost, file_cost)
}

pub fn response_1(path: &str, challenges: &Vec<usize>, key: Fr, hash_constants: &Vec<Fr>, n: usize) -> (Vec<Fr>, Vec<usize>, Fr) {
    //! 响应挑战
    let mut file = OpenOptions::new()
    .read(true)
    .write(true)
    .open(path)
    .unwrap();

    let mut x_response  = vec![];
    let mut idx_response = vec![];
    let x_hash_response;

    let mut count = 0; 

    // 逐个响应挑战，根据challenge[i]（即yn）找到对应的x
    for i in 0..challenges.len() {
        // 一个挑战对应一个yn
        let yn = challenges[i];
        let yn_bit_idx = (n + 1) * yn;
        let mut yn_byte_idx = yn_bit_idx / 8;
        let yn_left_count = yn_bit_idx % 8;

        let mut buf = vec![0u8];
        file.seek(SeekFrom::Start(yn_byte_idx.try_into().unwrap())).unwrap();
        file.read(&mut buf).unwrap(); 
        let mut buf_bits = bytes_to_bits(&buf);

        // 待读出字节的比特位的idx
        let mut k = yn_left_count;

        // 待读出的x
        let mut x_bits = vec![false; n + 1];

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
        if x_bits[n] == true {
            let x_usize = bits_to_usize(&x_bits[0..n].to_vec());
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

pub fn construct_circuit(df_constants: &Vec<Fr>, hash_constants: &Vec<Fr>) 
-> ProvingKey<Bls12<Parameters>> {
    //! 生成零知识证明
    let mut rng = rand::thread_rng();

    // 构建电路，生成参数，电路可重复使用
    let params = {
        let c = PospaceDemo {
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
    params
}

pub fn response_2(params: ProvingKey<Bls12<Parameters>>, key: Fr, x_response: &Vec<Fr>, m: Fr, df_constants: &Vec<Fr>, challenges: &Vec<usize>, idx_response: &Vec<usize>, x_hash: Fr, hash_constants: &Vec<Fr>) 
-> (PreparedVerifyingKey<Bls12_381>, Proof<Bls12_381>) {
    //! 生成零知识证明
    let mut rng = rand::thread_rng();
    let pvk = prepare_verifying_key(&params.vk);
    
    // 针对本次任务生成证明
    let proof = {
        let x = (0..x_response.len()).map(|i| Some(x_response[i])).collect::<Vec<_>>();
        let x = x.as_slice();
        let yn = (0..idx_response.len()).map(|i| Some(Fr::from_str(&challenges[idx_response[i]].to_string()).unwrap())).collect::<Vec<_>>();
        let yn = yn.as_slice();

        let c = PospaceDemo {
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

    (pvk, proof)
}