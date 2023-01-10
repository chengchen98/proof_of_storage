use bls12_381::Scalar;
use ff::{PrimeField, PrimeFieldBits};

// pub fn s_to_bits(x: Scalar) -> String {
//     let mut x_bit = String::new();
//     let x_vec = x.to_bytes();

//     for i in 0..32 {
//         let mut xi = x_vec[i];
//         for _ in 0..8 {
//             if xi % 2 == 1 {
//                 x_bit.push('1');
//             }
//             else {
//                 x_bit.push('0');
//             }
//             xi = xi / 2;
//         }
//     }
//     x_bit
// }

pub fn s_to_bits(x: Scalar) -> String {
    let x_bits = x.to_le_bits();
    let mut res = String::new();
    for i in 0..x_bits.len() {
        if x_bits[i] == true {
            res.push('1');
        }
        else {
            res.push('0');
        }
    }
    res
}

pub fn bits_to_s(x_bits: &str) -> Scalar {
    let mut x = Scalar::zero();
    let mut two = Scalar::one();
    let base = Scalar::from_str_vartime("2").unwrap();

    for i in 0..x_bits.len() {
        if x_bits.chars().nth(i).unwrap() == '1' {
            x = x.add(&two);
        }
        two = two.mul(&base);
    }
    x
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test() {
        let x = Scalar::from_str_vartime("1").unwrap();
        let bits = s_to_bits(x);
        let y = bits_to_s(&bits);
        assert_eq!(x, y);
    }
}