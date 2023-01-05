use bls12_381::Scalar;
use ff::PrimeField;

pub const INPUT_SIZE: usize = 256;

pub fn bits_to_s(x_bit: [Scalar; INPUT_SIZE], n: usize) -> Scalar {
    //! Convert binary to Scalar.
    //! 
    //! n: only convert data\[0..n\] to Scalar
    let mut x = Scalar::zero();
    let mut two = Scalar::one();
    let base = Scalar::from_str_vartime("2").unwrap();

    for i in 0..n {
        if x_bit[i] == Scalar::one() {
            x = x.add(&two);
        }
        two = two.mul(&base);
    }
    x
}

pub fn s_to_bits(x: Scalar) -> [Scalar; INPUT_SIZE] {
    //! Convert Scalar to binary.
    let mut x_bit = [Scalar::zero(); INPUT_SIZE];
    let x_vec = x.to_bytes();

    for i in 0..32 {
        let mut xi = x_vec[i];
        for j in 0..8 {
            if xi % 2 == 0 {
                x_bit[i * 8 + j] = Scalar::zero();
            }
            else {
                x_bit[i * 8 + j] = Scalar::one();
            }
            xi = xi / 2;
        }
    }
    x_bit
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test() {
        let x = Scalar::from_str_vartime("256").unwrap();
        let bits = s_to_bits(x);
        let y = bits_to_s(bits, 256);
        assert_eq!(x, y);
    }
}