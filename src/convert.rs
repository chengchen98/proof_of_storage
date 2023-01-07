use std::collections::HashMap;

use bls12_381::Scalar;
use ff::PrimeField;

pub fn s_to_bits(x: Scalar, n: usize) -> Vec<u8> {
    //! Convert Scalar to binary vector.
    //! 
    //! n: only convert the last n bit of x to bit
    let mut x_bit = vec![];
    let x_vec = x.to_bytes();

    for i in 0..32 {
        let mut xi = x_vec[i];
        for j in 0..8 {
            if xi % 2 == 1 {
                x_bit.push(1);
            }
            else {
                x_bit.push(0);
            }

            if i * 8 + j + 1 == n {
                return x_bit;
            }
            xi = xi / 2;
        }
    }
    x_bit
}

pub fn bits_to_s(x_bit: &Vec<u8>, n: usize) -> Scalar {
    //! Convert binary vector to Scalar.
    //! 
    //! n: only convert the first n bit of x to Scalar
    //! 
    //! input example: \[0u8, 0u8, 1u8, 1u8, ...\]
    let mut x = Scalar::zero();
    let mut two = Scalar::one();
    let base = Scalar::from_str_vartime("2").unwrap();

    for i in 0..n {
        if x_bit[i] == 1 {
            x = x.add(&two);
        }
        two = two.mul(&base);
    }
    x
}

pub fn hex_to_s(x_hex: &str) -> Scalar {
    //! Convert hex string to Scalar.
    //! 
    //! input example: abc12
    let x_hex: String = x_hex.chars().rev().collect();
    
    let mut dict = HashMap::new();
    dict.insert(String::from("0"), String::from("0"));
    dict.insert(String::from("1"), String::from("1"));
    dict.insert(String::from("2"), String::from("2"));
    dict.insert(String::from("3"), String::from("3"));
    dict.insert(String::from("4"), String::from("4"));
    dict.insert(String::from("5"), String::from("5"));
    dict.insert(String::from("6"), String::from("6"));
    dict.insert(String::from("7"), String::from("7"));
    dict.insert(String::from("8"), String::from("8"));
    dict.insert(String::from("9"), String::from("9"));
    dict.insert(String::from("a"), String::from("10"));
    dict.insert(String::from("b"), String::from("11"));
    dict.insert(String::from("c"), String::from("12"));
    dict.insert(String::from("d"), String::from("13"));
    dict.insert(String::from("e"), String::from("14"));
    dict.insert(String::from("f"), String::from("15"));

    let mut res = Scalar::zero();
    let mut base = Scalar::one();
    for i in 0..x_hex.len() {
        let cur_x = dict.get(&x_hex[i..i + 1]).unwrap().clone();
        res = res + base * Scalar::from_str_vartime(&cur_x).unwrap();
        base *= Scalar::from_str_vartime("16").unwrap();
    }
    res
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test() {
        let x = Scalar::from_str_vartime("1").unwrap();
        let n = 1;
        let bits = s_to_bits(x, n);
        let y = bits_to_s(&bits, n);
        assert_eq!(x, y);
    }
}