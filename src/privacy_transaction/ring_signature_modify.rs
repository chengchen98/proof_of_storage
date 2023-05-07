// use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
// use curve25519_dalek::scalar::Scalar;
// use curve25519_dalek::ristretto::RistrettoPoint;
// use rand::rngs::OsRng;
// use rand::thread_rng;
// use rand::RngCore;
// use sha3::{Digest, Sha3_512};

// // 签名
// pub struct Signature {
//     pub c: Scalar,
//     pub r: Vec<Scalar>,
//     pub s: Scalar,
// }

// impl Signature {
//     pub fn new() -> Self {
//         Signature {
//             c: Scalar::zero(),
//             r: Vec::new(),
//             s: Scalar::zero(),
//         }
//     }
// }

// // 环签名
// pub struct RingSignature {
//     pub public_keys: Vec<RistrettoPoint>,
//     pub signature: Signature,
// }

// impl RingSignature {
//     // 签名
//     pub fn sign(&mut self, message: &[u8], private_key: &Scalar, index: usize) -> Result<(), &'static str> {
//         // 确定密钥环的大小
//         let n = self.public_keys.len();

//         // 随机生成 r 值
//         let mut rng = thread_rng();
//         let r = Scalar::random(&mut rng);

//         // 计算 R = r * B
//         let R = &r * &RISTRETTO_BASEPOINT_POINT;

//         // 计算 c = H(R || I || M)
//         let mut hasher = Sha3_512::new();
//         hasher.update(R.compress().as_bytes());
//         hasher.update(self.public_keys[index].compress().as_bytes());
//         hasher.update(message);
//         let c = Scalar::from_hash(hasher);

//         // 计算 s_i = r - c * x_i
//         let s_i = &r - &c * private_key;

//         // 保存签名信息
//         self.signature.c = c;
//         self.signature.r.push(r);
//         self.signature.s = s_i;

//         // 递归计算 s_{i+1 mod n}, ..., s_{i+n-1 mod n}
//         for j in 1..n {
//             let k = (index + j) % n;
//             let pk_j = &self.public_keys[k];
//             let r_j = &self.signature.r[j - 1];
//             let s_j = &self.signature.s;
//             let R_j = &r_j * &RISTRETTO_BASEPOINT_POINT + &s_j * pk_j;
//             let c_j = {
//                 let mut hasher = Sha3_512::new();
//                 hasher.update(R_j.compress().as_bytes());
//                 hasher.update(self.public_keys[k].compress().as_bytes());
//                 hasher.update(message);
//                 Scalar::from_hash(hasher)
//             };
//             let s_k = &r_j - &c_j * s_j;
//             self.signature.s = s_k;
//         }

//         Ok(())
//     }

//     // 验证签名
//     pub fn verify(&self, message: &[u8]) -> bool {
//         // 确定密钥环的大小
//         let n = self.public_keys.len();

//         // 检查签名是否为空
//         if self.signature.r.len() != n || self.signature.s == Scalar::zero() {
//             return false;
//         }

//         // 计算 R = s_i * B + c_i * P_i
//         let mut R = &self.signature.s * &RISTRETTO_BASEPOINT_POINT + &self.signature.c * self.signature.r[0];

//         let mut hasher = Sha3_512::new();
//         hasher.update(self.signature.r[0].compress().as_bytes());
//         hasher.update(self.public_keys[0].compress().as_bytes());
//         hasher.update(message);
//         let mut c = Scalar::from_hash(hasher);
//         let mut P = &self.signature.s * &RISTRETTO_BASEPOINT_POINT + &c * &self.public_keys[0];
    
//         // 递归计算 R_{i+1 mod n}, ..., R_{i+n-1 mod n}
//         for i in 1..n {
//             let pk_i = &self.public_keys[i];
//             let r_i = &self.signature.r[i - 1];
//             let s_i = &self.signature.s;
//             let mut hasher = Sha3_512::new();
//             hasher.update(r_i.compress().as_bytes());
//             hasher.update(pk_i.compress().as_bytes());
//             hasher.update(message);
//             c = Scalar::from_hash(hasher);
//             R = &r_i * &RISTRETTO_BASEPOINT_POINT + &s_i * pk_i;
//             P = &c * &R + &self.signature.s * P;
//         }
    
//         // 验证签名是否有效
//         P.compress() == R.compress()
//     }
// }

// #[test]
// fn test() {
//     let mut rng = OsRng;
//     let secret_key = Scalar::random(&mut rng);
//     let public_key = &secret_key * &RISTRETTO_BASEPOINT_POINT;

//     // 构造密钥环
//     let mut public_keys = Vec::new();
//     for _ in 0..4 {
//         let mut rng = OsRng;
//         public_keys.push(&Scalar::random(&mut rng) * &RISTRETTO_BASEPOINT_POINT);
//     }
//     public_keys.push(public_key);

//     // 签名
//     let mut signature = Signature::new();
//     let mut ring_signature = RingSignature {
//         public_keys: public_keys.clone(),
//         signature: signature.clone(),
//     };
//     ring_signature.sign(b"hello world", &secret_key, 4).unwrap();
//     println!("Signature: {:?}", ring_signature.signature);

//     // 验证签名
//     assert!(ring_signature.verify(b"hello world"));

//     // 修改消息
//     assert!(!ring_signature.verify(b"hello rust"));

// }