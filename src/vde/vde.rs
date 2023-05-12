use num_bigint::{BigInt, Sign};
use super::sloth::{sloth, sloth_inv};
use crate::proof_of_storage::postorage_modify::UNIT_L;

pub const STEP: usize = UNIT_L + 1;

pub fn single_vde(x: &BigInt, p: &BigInt, t: usize, mode: &str) -> BigInt {
    if mode == "sloth" {
        return sloth(x, p, t);
    }
    else {
        return sloth(x, p, t);
    }
}

pub fn single_vde_inv(y: &BigInt, p: &BigInt, t: usize, mode: &str) -> BigInt {
    if mode == "sloth" {
        return sloth_inv(y, p, t);
    }
    else {
        return sloth_inv(y, p, t);
    }
}

// pub fn vde(x: &Vec<u8>, p: &BigUint, mode: &str) -> Vec<u8> {
//     let cur_x = BigUint::from_bytes_le(&x);
//     let y = single_vde(&(cur_x % p), &p, mode);
//     padding(&y.to_bytes_le().to_vec(), x.len())
// }

// pub fn vde_inv(y: &Vec<u8>, p: &BigUint, mode: &str) -> Vec<u8> {
//     let cur_y = BigUint::from_bytes_le(&y);
//     let x = single_vde_inv(&cur_y, &p, mode);
//     padding(&x.to_bytes_le().to_vec(), y.len())
// }

pub fn vde(x: &Vec<u8>, p: &BigInt, t: usize, mode: &str) -> Vec<u8> {
    let mut res = vec![];
    for i in (0..x.len()).step_by(STEP) {
        let buf_x = &x[i .. i + STEP];
        let cur_x = BigInt::from_bytes_le(Sign::Plus, &buf_x);

        let y = single_vde(&cur_x, p, t, mode);
        let mut y_bytes = y.to_bytes_le().1.to_vec();

        if y_bytes.len() < STEP {
            y_bytes.append(&mut vec![0u8; STEP - y_bytes.len()]);
        }

        res.append(&mut y_bytes);
    }
    res
}

pub fn vde_inv(y: &Vec<u8>, p: &BigInt, t: usize, mode: &str) -> Vec<u8> {
    let mut res = vec![];
    for i in (0..y.len()).step_by(STEP) {
        let buf_y = &y[i .. i + STEP];
        let cur_y = BigInt::from_bytes_le(Sign::Plus,&buf_y);

        let x = single_vde_inv(&cur_y, p, t, mode);
        let mut x_bytes = x.to_bytes_le().1.to_vec();

        if x_bytes.len() < STEP {
            x_bytes.append(&mut vec![0u8; STEP - x_bytes.len()]);
        }

        res.append(&mut x_bytes);
    }
    res
}

#[test]
fn test_vde() {
    use std::str::FromStr;
    use std::time::Instant;
    use super::sloth::P_1024;

    const T: usize = 3;
    const N: usize = 10;
    let x =vec![1u8; 128 * N];
    let p = &BigInt::from_str(P_1024).unwrap();

    const SAMPLES: usize = 10;
    for i in 0..SAMPLES {
        println!("sample: {:?}  |  n: {:?}", i, N);
        
        let start = Instant::now();
        let y = vde(&x, p, T, "sloth");
        let cost1 = start.elapsed();
        println!("Vde: {:?}", cost1);

        let start = Instant::now();
        let z = vde_inv(&y, p, T, "sloth");
        let cost2 = start.elapsed();
        println!("Vde inv: {:?}", cost2);

        println!("vde / vde inv: {:?}", cost1.as_secs_f32() / cost2.as_secs_f32());
        assert_eq!(x, z);
        println!("-------------------------------------");
    }
}