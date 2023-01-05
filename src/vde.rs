use bls12_381::Scalar;
use ff::PrimeField;

use crate::pow::pow;
use crate::convert::s_to_bits;

// pub fn sloth(x: Scalar, p: Scalar) -> Scalar {
//     return x.add(&p);
// }

// pub fn sloth_inv(y: Scalar, p: Scalar) -> Scalar {
//     return y.sub(&p);
// }

pub fn compare_s(a: Scalar, b: Scalar) -> bool {
    let a_vec = a.to_bytes();
    let b_vec = b.to_bytes();
    for i in 0..32 {
        if a_vec[31 - i] >= b_vec[31 - i] {
            return true;
        }
        else {
            return false;
        }
    }
    true
}

pub fn modp_s(x: Scalar, p: Scalar) -> Scalar {
    let mut res = x;
    loop {
        if compare_s(res, p) == false {
            break;
        }
        res = res.sub(&p);
    }
    res
}

pub fn sloth(x: Scalar, p: Scalar) -> Scalar {
    let mut y;

    let flag = pow(x, s_to_bits(p.sub(&Scalar::one()).mul(&Scalar::from_str_vartime("2").unwrap().invert().unwrap())));
    let flag = modp_s(flag, p);

    if flag == Scalar::one() {
        y = pow(x, s_to_bits(p.add(&Scalar::one()).mul(&Scalar::from_str_vartime("4").unwrap().invert().unwrap())));
        y = modp_s(y, p);

        if y.is_odd().unwrap_u8() == 0 {
            y = p.sub(&y);
            y = modp_s(y, p);
        }
    }
    else {
        let ans = p.sub(&x);
        let ans = modp_s(ans, p);

        y = pow(ans, s_to_bits(p.add(&Scalar::one()).mul(&Scalar::from_str_vartime("4").unwrap().invert().unwrap())));
        y = modp_s(y, p);

        if y.is_even().unwrap_u8() == 0 {
            y = p.sub(&y);
            y = modp_s(y, p);
        }
    }

    if y.is_odd().unwrap_u8() == 0 {
        return modp_s(y.add(&Scalar::one()), p);
    }
    else {
        return modp_s(y.sub(&Scalar::one()), p);
    }
}

pub fn sloth_inv(y: Scalar, p: Scalar) -> Scalar {
    let mut x;
    if y.is_odd().unwrap_u8() == 0 {
        x = y.add(&Scalar::one());
        x = modp_s(x, p);
    }
    else {
        x = y.sub(&Scalar::one());
        x = modp_s(x, p);
    }

    if x.is_odd().unwrap_u8() == 0 {
        return modp_s(p.sub(&x.square()), p);
    }
    else {
        return modp_s(x.square(), p);
    }
}

pub fn single_vde(x: Scalar, p: Scalar, mode: &str) -> Scalar {
    if mode == "sloth" {
        return sloth(x, p);
    }
    else {
        return sloth(x, p);
    }
}

pub fn single_vde_inv(y: Scalar, p: Scalar, mode: &str) -> Scalar {
    if mode == "sloth" {
        return sloth_inv(y, p);
    }
    else {
        return sloth_inv(y, p);
    }
}

pub fn vde(x: &Vec<u8>, p: Scalar, mode: &str) -> Vec<u8> {
    let mut res = vec![];

    let mut buf_x = [0u8; 32];
    for i in (0..x.len()).step_by(32) {
        buf_x.copy_from_slice(&x[i .. i + 32]);
        let cur_x = Scalar::from_bytes(&buf_x).unwrap();
        let y = single_vde(cur_x, p, mode);
        res.append(&mut y.to_bytes().to_vec());
    }

    res
}

pub fn vde_inv(y: &Vec<u8>, p: Scalar, mode: &str) -> Vec<u8> {
    let mut res = vec![];

    let mut buf_y = [0u8; 32];
    for i in (0..y.len()).step_by(32) {
        buf_y.copy_from_slice(&y[i .. i + 32]);
        let cur_y = Scalar::from_bytes(&buf_y).unwrap();
        let y = single_vde_inv(cur_y, p, mode);
        res.append(&mut y.to_bytes().to_vec());
    }

    res
}


#[cfg(test)]
mod test {
    use bls12_381::Scalar;
    use ff::PrimeField;

    use super::{single_vde, single_vde_inv};

    #[test]
    fn test_sloth() {
        let x = Scalar::from_str_vartime("2").unwrap();
        println!("x: {:?}", x);
        let p = Scalar::from_str_vartime("5").unwrap();
        let mode = "sloth";
        let y = single_vde(x, p, mode);
        println!("y: {:?}", y);
        let x = single_vde_inv(y, p, mode);
        println!("x: {:?}", x);
    }
}