use num_bigint::BigUint;
use crate::{vde::sloth::{sloth, sloth_inv}, common::data::padding};

const STEP: usize = 128;

pub fn single_vde(x: &BigUint, p: &BigUint, mode: &str) -> BigUint {
    if mode == "sloth" {
        return sloth(x, p);
    }
    else {
        return sloth(x, p);
    }
}

pub fn single_vde_inv(y: &BigUint, p: &BigUint, mode: &str) -> BigUint {
    if mode == "sloth" {
        return sloth_inv(y, p);
    }
    else {
        return sloth_inv(y, p);
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

pub fn vde(x: &Vec<u8>, p: &BigUint, mode: &str) -> Vec<u8> {
    let mut res = vec![];
    for i in (0..x.len()).step_by(STEP) {
        let buf_x = &x[i .. i + STEP];
        let cur_x = BigUint::from_bytes_le(&buf_x);

        let y = single_vde(&cur_x, p, mode);
        let y_bytes = y.to_bytes_le().to_vec();

        let mut y_bytes_pad = padding(&y_bytes, STEP);

        res.append(&mut y_bytes_pad);
    }
    res
}

pub fn vde_inv(y: &Vec<u8>, p: &BigUint, mode: &str) -> Vec<u8> {
    let mut res = vec![];
    for i in (0..y.len()).step_by(STEP) {
        let buf_y = &y[i .. i + STEP];
        let cur_y = BigUint::from_bytes_le(&buf_y);

        let x = single_vde_inv(&cur_y, p, mode);
        let x_bytes = x.to_bytes_le().to_vec();

        let mut x_bytes_pad = padding(&x_bytes, STEP);

        res.append(&mut x_bytes_pad);
    }
    res
}

#[test]
fn test_vde() {
    use std::str::FromStr;

    let x = vec![1u8; 1024];
    let p = BigUint::from_str("276945728797634137489847193533935566200901110872557999805088095083433912915081929876610085556888176394277441945470579512610156696848456080099840453319124321877455883488948246054067984322844955398390786946509577100886479649428068281092367813035032036823204874960913543086692263648390252658950393200040464000839").unwrap();
    let y = vde(&x, &p, "sloth");
    let z = vde_inv(&y, &p, "sloth");
    assert_eq!(x, z);
}