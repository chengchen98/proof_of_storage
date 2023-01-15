use std::ops::{Div, Sub, Add};
use num_bigint::{BigUint, ToBigUint};

pub fn sloth(x: &BigUint, p: &BigUint) -> BigUint {
    let flag = x.modpow(&p.sub(1.to_biguint().unwrap()).div(2.to_biguint().unwrap()), &p);
    let mut y;
    if flag == 1.to_biguint().unwrap() {
        y = x.modpow(&p.add(1.to_biguint().unwrap()).div(4.to_biguint().unwrap()), &p);
        if y.clone() % 2.to_biguint().unwrap() == 1.to_biguint().unwrap() {
            y = &p.sub(y) % p;
        }
    }
    else {
        let x = &p.sub(x) % p;
        y = x.modpow(&p.add(1.to_biguint().unwrap()).div(4.to_biguint().unwrap()), &p);
        if y.clone() % 2.to_biguint().unwrap() == 0.to_biguint().unwrap() {
            y = &p.sub(y) % p;
        }
    }

    if y.clone() % 2.to_biguint().unwrap() == 1.to_biguint().unwrap() {
        return y.add(1.to_biguint().unwrap()) % p;
    }
    else {
        return y.sub(1.to_biguint().unwrap()) % p;
    }
}

pub fn sloth_inv(y: &BigUint, p: &BigUint) -> BigUint {
    let x;
    if y % 2.to_biguint().unwrap() == 1.to_biguint().unwrap() {
        x = &y.add(1.to_biguint().unwrap()) % p;
    }
    else {
        x = &y.sub(1.to_biguint().unwrap()) % p;
    }

    if x.clone() % 2.to_biguint().unwrap() == 1.to_biguint().unwrap() {
        return p.sub(x.pow(2) % p);
    }
    else {
        return x.pow(2) % p;
    }
}

#[test]
fn test_sloth() {
    use std::str::FromStr;
    
    let x = 2.to_biguint().unwrap();
    let p = BigUint::from_str("340282366920938463463374607431768211507").unwrap();
    let y = sloth(&x, &p);
    let z = sloth_inv(&y, &p);
    assert_eq!(x, z);
}