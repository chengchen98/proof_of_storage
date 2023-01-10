// use std::ops::{Div, Sub, Add};

// use num_bigint::{BigInt, ToBigInt};

// pub fn sloth(x: &BigInt, p: &BigInt) -> BigInt {
//     let flag = x.modpow(&p.sub(1.to_bigint().unwrap()).div(2.to_bigint().unwrap()), p);
//     let mut y;
//     if flag == 1.to_bigint().unwrap() {
//         let yy = &x.modpow(&p.add(1.to_bigint().unwrap()).div(4.to_bigint().unwrap()), p);
//         if yy % 2 == 1.to_bigint().unwrap() {
//             y = p.sub(yy) % p;
//         }
//     }
//     else {
//         let x = (p - x) % p;
//         y = x.modpow(&p.add(1.to_bigint().unwrap()).div(4.to_bigint().unwrap()), p);
//         if y % 2 == 0.to_bigint().unwrap() {
//             y = (p - y) % p;
//         }
//     }

//     if y % 2 == 1.to_bigint().unwrap() {
//         return (y + 1) % p;
//     }
//     else {
//         return (y - 1) % p;
//     }
// }

// // pub fn sloth_inv(y: &BigInt, p: &BigInt) -> BigInt {
// //     let mut y = y;
// //     if y % 2 == 1.to_bigint().unwrap() {
// //         y = (y + 1) % p;
// //     }
// //     else {
// //         y = (y - 1) % p;
// //     }

// //     if y % 2 == 1.to_bigint().unwrap() {
// //         return (p - y * y) % p;
// //     }
// //     else {
// //         return (y * y) % p;
// //     }
// // }

// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn test_sloth() {
//         let x = &2.to_bigint().unwrap();
//         let p = &13.to_bigint().unwrap();
//         let y = sloth(x, p);
//         println!("{:?}", y);
//     }
// }