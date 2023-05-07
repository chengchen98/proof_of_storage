// use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
// use curve25519_dalek::ristretto::RistrettoPoint;
// use curve25519_dalek::scalar::Scalar;
// use rand::{CryptoRng, RngCore};
// use sha3::{Digest, Sha3_512};

// // 定义签名结构体
// #[derive(Debug, Clone)]
// struct Signature {
//     r: Vec<RistrettoPoint>,
//     s: Vec<Scalar>,
// }

// impl Signature {
//     // 生成空签名
//     fn new() -> Signature {
//         Signature { r: Vec::new(), s: Vec::new() }
//     }

//     // 添加 r 值和 s 值
//     fn add(&mut self, r: RistrettoPoint, s: Scalar) {
//         self.r.push(r);
//         self.s.push(s);
//     }
// }

// // 定义环签名结构体
// #[derive(Debug)]
// struct RingSignature {
//     public_keys: Vec<RistrettoPoint>,
//     signature: Signature,
// }

// impl RingSignature {
//     // 签名函数
//     fn sign<R: RngCore + CryptoRng>(&mut self, message: &[u8], secret_key: &Scalar, index: usize, rng: &mut R) {
//         let n = self.public_keys.len();

//         // 生成随机值 r_i
//         let mut r_i = Scalar::random(rng);
//         let mut R = &r_i * &RISTRETTO_BASEPOINT_POINT;

//         // 计算 s_i
//         let mut c;
//         let mut s_i;
//         let mut r_sum = r_i;
//         for i in 0..n {
//             if i == index {
//                 continue;
//             }
//             let pk_i = &self.public_keys[i];
//             let mut hasher = Sha3_512::new();
//             hasher.update(R.compress().as_bytes());
//             hasher.update(pk_i.compress().as_bytes());
//             hasher.update(message);
//             c = Scalar::from_hash(hasher);
//             s_i = &r_i - &c * secret_key;
//             self.signature.add(R, s_i);
//             r_sum += s_i;
//             r_i = Scalar::random(rng);
//             R = &r_i * &RISTRETTO_BASEPOINT_POINT + &c * pk_i;
//         }
//         s_i = &Scalar::one() - &r_sum;
//         self.signature.add(R, s_i);
//     }

//     // 验证签名函数
//     fn verify(&self, message: &[u8]) -> bool {
//         let n = self.public_keys.len();

//         // 确认签名中包含 n 个 r 值和 n 个 s 值
//         if self.signature.r.len() != n || self.signature.s.len() != n {
//             return false;
//         }

//         // 计算 P
//         let mut hasher = Sha3_512::new();
//         for i in 0..n {
//             let r_i = &self.signature.r[i];
//             let pk_i = &self.public_keys[i];
//             hasher.update(r_i.compress().as_bytes());
//             hasher.update(pk_i.compress().as_bytes());
//             hasher.update(message);
//         }
//         let c = Scalar::from_hash(hasher);
//         let mut P = &c * &self.public_keys[0] + &self.signature.s[0] * &RISTRETTO_BASEPOINT_POINT;

//         // 递归计算 P'
//         for i in 1..n {
//             let r_i = &self.signature.r[i];
//             let pk_i = &self.public_keys[i];
//             P = &c * pk_i + &self.signature.s[i] * r_i + P;
//         }

//         // 验证 P 是否等于 R
//         P.compress() == self.signature.r[0].compress()
//     }
// }

// fn generate_keypair<R: RngCore + CryptoRng>(rng: &mut R) -> (RistrettoPoint, Scalar) {
//     let secret_key = Scalar::random(rng);
//     let public_key = &secret_key * &RISTRETTO_BASEPOINT_POINT;
//     (public_key, secret_key)
// }

// #[test]
// fn test() {
//     // 生成公钥集合
//     let mut public_keys = Vec::new();
//     for _ in 0..5 {
//         let (public_key, _) = generate_keypair(&mut rand::thread_rng());
//         public_keys.push(public_key);
//     }

//     // 签名者选定签名位置和对应的私钥
//     let index = 3;
//     let (_, secret_key) = generate_keypair(&mut rand::thread_rng());

//     // 签名
//     let mut signature = Signature::new();
//     let mut ring_signature = RingSignature {
//         public_keys: public_keys,
//         signature: signature,
//     };
//     ring_signature.sign(b"message", &secret_key, index, &mut rand::thread_rng());

//     // 验证签名
//     assert_eq!(true, ring_signature.verify(b"message"));
// }
