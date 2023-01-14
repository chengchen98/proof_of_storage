use ark_bls12_381::Fr;
use ark_ff::{BigInteger256, BigInteger};

pub fn fr_to_bits(x: Fr) -> String {
    let x: BigInteger256 = x.into(); // convert fp to bigint
    let x = x.to_bits_le(); // convert bigint to binary vector
    let x: String = x.iter().map(|y| { if *y == true { "1" } else { "0" }}).collect(); // convert binary vector to binary string
    x
}

pub fn bits_to_fr(x: &str) -> Fr {
    let x: Vec<bool> = x.chars().into_iter().map(|x| if x == '0' { false } else { true }).collect(); // convert the binary string to binary vector
    let x = BigInteger256::from_bits_le(&x); // convert the binary vector to bigint
    let x = Fr::from(x); // comvert the bigint to fp
    x
}

pub fn vecu8_to_bits(x: &Vec<u8>) -> Vec<bool> {
    //! Convert vec<u8> to binary expression.
    let mut res = vec![];
    for i in 0..x.len() {
        let mut xi = x[i];
        let mut cur = vec![];
        for _ in 0..8 {
            if xi % 2 == 1 {
                cur.push(true);
            }
            else {
                cur.push(false);
            }
            xi /= 2;
        }
        cur.reverse();
        res.append(&mut cur);
    }
    res
}

pub fn bits_to_vecu8(x: &Vec<bool>) -> Vec<u8> {
    //! Convert binary expression to vec<u8>.
    let mut res = vec![];
    for i in (0..x.len()).step_by(8) {
        let mut cur = 0;
        let mut base = 128;
        for j in 0..8 {
            if x[i + j] == true {
                cur += base;
            }
            base /= 2;
        }
        res.push(cur);
    }
    res
}

#[test]
fn test_fr_to_bits() {
    let x = Fr::from(1);
    let x_bits = fr_to_bits(x);
    assert_eq!(x_bits.len(), 256);
}

#[test]
fn test_bits_to_fr() {
    let x_bits= "1100";
    let x = bits_to_fr(x_bits);
    println!("{:?}", x);
}

#[test]
fn test_vecu8_to_bits() {
    let x = vec![1, 2];
    let res = vecu8_to_bits(&x);
    println!("{:?}", res);
    let y = bits_to_vecu8(&res);
    println!("{:?}", y);
}