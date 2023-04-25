use rug::{Integer, integer::Order};
use super::rug_sloth::{sloth, sloth_inv};
use crate::proof_of_storage::postorage::L0;

pub const STEP: usize = L0 + 1;

pub fn single_vde(x: &Integer, p: &Integer, t: usize, mode: &str) -> Integer {
    if mode == "sloth" {
        return sloth(x, p, t);
    }
    else {
        return sloth(x, p, t);
    }
}

pub fn single_vde_inv(y: &Integer, p: &Integer, t: usize, mode: &str) -> Integer {
    if mode == "sloth" {
        return sloth_inv(y, p, t);
    }
    else {
        return sloth_inv(y, p, t);
    }
}

pub fn vde(x: &Vec<u8>, p: &Integer, t: usize, mode: &str) -> Vec<u8> {
    let mut res = vec![];
    for i in (0..x.len()).step_by(STEP) {
        let buf_x = &x[i .. i + STEP];
        let cur_x = Integer::from_digits(&buf_x, Order::Lsf);
        let y = single_vde(&cur_x, p, t, mode);
        let mut y_bytes = y.to_digits::<u8>(Order::Lsf);

        if y_bytes.len() < STEP {
            y_bytes.append(&mut vec![0u8; STEP - y_bytes.len()]);
        }

        res.append(&mut y_bytes);
    }
    res
}

pub fn vde_inv(y: &Vec<u8>, p: &Integer, t: usize, mode: &str) -> Vec<u8> {
    let mut res = vec![];
    for i in (0..y.len()).step_by(STEP) {
        let buf_y = &y[i .. i + STEP];
        let cur_y = Integer::from_digits(&buf_y, Order::Lsf);

        let x = single_vde_inv(&cur_y, p, t, mode);
        let mut x_bytes = x.to_digits::<u8>(Order::Lsf);

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
    let p = &Integer::from_str(P_1024).unwrap();

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