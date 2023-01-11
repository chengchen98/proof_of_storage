// use bls12_381::Scalar;

// pub fn sloth(x: Scalar, p: Scalar) -> Scalar {
//     return x.add(&p);
// }

// pub fn sloth_inv(y: Scalar, p: Scalar) -> Scalar {
//     return y.sub(&p);
// }

// pub fn single_vde(x: Scalar, p: Scalar, mode: &str) -> Scalar {
//     if mode == "sloth" {
//         return sloth(x, p);
//     }
//     else {
//         return sloth(x, p);
//     }
// }

// pub fn single_vde_inv(y: Scalar, p: Scalar, mode: &str) -> Scalar {
//     if mode == "sloth" {
//         return sloth_inv(y, p);
//     }
//     else {
//         return sloth_inv(y, p);
//     }
// }

// pub fn vde(x: &Vec<u8>, p: Scalar, mode: &str) -> Vec<u8> {
//     let mut res = vec![];

//     let mut buf_x = [0u8; 32];
//     for i in (0..x.len()).step_by(32) {
//         buf_x.copy_from_slice(&x[i .. i + 32]);
//         let cur_x = Scalar::from_bytes(&buf_x).unwrap();
//         let y = single_vde(cur_x, p, mode);
//         res.append(&mut y.to_bytes().to_vec());
//     }

//     res
// }

// pub fn vde_inv(y: &Vec<u8>, p: Scalar, mode: &str) -> Vec<u8> {
//     let mut res = vec![];

//     let mut buf_y = [0u8; 32];
//     for i in (0..y.len()).step_by(32) {
//         buf_y.copy_from_slice(&y[i .. i + 32]);
//         let cur_y = Scalar::from_bytes(&buf_y).unwrap();
//         let y = single_vde_inv(cur_y, p, mode);
//         res.append(&mut y.to_bytes().to_vec());
//     }

//     res
// }


// #[cfg(test)]
// mod test {
//     use bls12_381::Scalar;
//     use ff::PrimeField;

//     use super::{single_vde, single_vde_inv};

//     #[test]
//     fn test_sloth() {
//         let x = Scalar::from_str_vartime("2").unwrap();
//         println!("x: {:?}", x);
//         let p = Scalar::from_str_vartime("5").unwrap();
//         let mode = "sloth";
//         let y = single_vde(x, p, mode);
//         println!("y: {:?}", y);
//         let x = single_vde_inv(y, p, mode);
//         println!("x: {:?}", x);
//     }
// }