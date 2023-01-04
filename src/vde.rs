use bls12_381::Scalar;
use ff::PrimeField;

use crate::pow::pow;
use crate::convert::s_to_bits;

pub fn sloth(x: Scalar, p: Scalar) -> Scalar {
    let mut y;

    let flag = pow(x, s_to_bits(p.sub(&Scalar::one()).mul(&Scalar::from_str_vartime("2").unwrap().invert().unwrap())));

    if flag == Scalar::one() {
        y = pow(x, s_to_bits(p.add(&Scalar::one()).mul(&Scalar::from_str_vartime("4").unwrap().invert().unwrap())));

        if y.is_odd().unwrap_u8() == 0 {
            y = p.sub(&y);
        }
    }
    else {
        let ans = p.sub(&x);

        y = pow(ans, s_to_bits(p.add(&Scalar::one()).mul(&Scalar::from_str_vartime("4").unwrap().invert().unwrap())));

        if y.is_even().unwrap_u8() == 0 {
            y = p.sub(&y);
        }
    }

    if y.is_odd().unwrap_u8() == 0 {
        return y.add(&Scalar::one());
    }
    else {
        return y.sub(&Scalar::one());
    }
}

pub fn sloth_inv(y: Scalar, p: Scalar) -> Scalar {
    let ans;
    if y.is_odd().unwrap_u8() == 0 {
        ans = y.add(&Scalar::one());
    }
    else {
        ans = y.sub(&Scalar::one());
    }

    if ans.is_odd().unwrap_u8() == 0 {
        return p.sub(&ans.square());
    }
    else {
        return ans.square();
    }
}

pub fn vde(x: Scalar, p: Scalar, mode: &str) -> Scalar {
    if mode == "sloth" {
        return sloth(x, p);
    }
    else {
        return sloth(x, p);
    }
}

pub fn vde_inv(y: Scalar, p: Scalar, mode: &str) -> Scalar {
    if mode == "sloth" {
        return sloth_inv(y, p);
    }
    else {
        return sloth_inv(y, p);
    }
}


#[cfg(test)]
mod test {
    use bls12_381::Scalar;

    use super::*;

    #[test]
    fn test_sloth() {
        let x = Scalar::from_str_vartime("2").unwrap();
        println!("x: {:?}", x);
        let p = Scalar::from_str_vartime("4").unwrap();
        let mode = "sloth";
        let y = vde(x, p, mode);
        println!("y: {:?}", y);
        let x = vde_inv(y, p, mode);
        println!("x: {:?}", x);
    }
}