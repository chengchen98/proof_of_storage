use num_bigint::BigUint;
use crate::vde::sloth::{sloth, sloth_inv};

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
        let mut y_bytes = y.to_bytes_le().to_vec();

        if y_bytes.len() < STEP {
            y_bytes.append(&mut vec![0u8; STEP - y_bytes.len()]);
        }

        res.append(&mut y_bytes);
    }
    res
}

pub fn vde_inv(y: &Vec<u8>, p: &BigUint, mode: &str) -> Vec<u8> {
    let mut res = vec![];
    for i in (0..y.len()).step_by(STEP) {
        let buf_y = &y[i .. i + STEP];
        let cur_y = BigUint::from_bytes_le(&buf_y);

        let x = single_vde_inv(&cur_y, p, mode);
        let mut x_bytes = x.to_bytes_le().to_vec();

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

    let x = vec![0u8; 128];
    let p = BigUint::from_str("162892692473361111348249522320347526171207756760381512377472315857021028422689815298733972916755720242725920671690392382889161699607077776923153532250584503438515484867646456937083398184988196288738761695736655551130641678117347468224388930820784409522417624141309667471624562708798367178136545063034409853007").unwrap();
    let y = vde(&x, &p, "sloth");
    let z = vde_inv(&y, &p, "sloth");
    assert_eq!(x, z);
}