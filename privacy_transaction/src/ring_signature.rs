// use secp256k1::{PublicKey, SecretKey, Message, Signature};
// use secp256k1::rand::thread_rng;
// use secp256k1::rand::Rng;
// use sha2::{Sha256, Digest};

// // 定义环签名结构体
// struct RingSignature {
//     c: PublicKey, // C
//     s: Vec<SecretKey>, // s1,s2,...,sn
// }

// // 定义环签名函数
// fn ring_sign(message: &Message, public_keys: &[PublicKey], secret_key: &SecretKey) -> RingSignature {
//     // 获取公钥集合的长度
//     let n = public_keys.len();
//     // 检查私钥是否在公钥集合中
//     let i = public_keys.iter().position(|pk| pk == &PublicKey::from_secret_key(secret_key)).expect("secret key not in public keys");
//     // 生成随机数k
//     let mut rng = thread_rng();
//     let k = SecretKey::random(&mut rng);
//     // 计算C = k*G
//     let c = PublicKey::from_secret_key(&k);
//     // 初始化s向量
//     let mut s = vec![SecretKey::default(); n];
//     // 从i+1开始循环
//     let mut cj = c.clone();
//     for j in (i+1)..(i+n) {
//         // 取模n
//         let j = j % n;
//         // 生成随机数sj
//         s[j] = SecretKey::random(&mut rng);
//         // 计算Cj = sj*G + H(C||j)*Pj
//         let mut hasher = Sha256::new();
//         hasher.update(c.serialize());
//         hasher.update(j.to_be_bytes());
//         let h = hasher.finalize();
//         let h = SecretKey::from_slice(&h).expect("hash is not a valid secret key");
//         cj = PublicKey::from_combination(&[PublicKey::from_secret_key(&s[j]), PublicKey::from_secret_key(&h).mul(&public_keys[j])]).expect("public key combination failed");
//     }
//     // 计算si = k - H(C||i)*xi
//     let mut hasher = Sha256::new();
//     hasher.update(c.serialize());
//     hasher.update(i.to_be_bytes());
//     let h = hasher.finalize();
//     let h = SecretKey::from_slice(&h).expect("hash is not a valid secret key");
//     s[i] = k.clone().add(&h.mul(secret_key)).expect("secret key addition failed");
//     // 返回环签名
//     RingSignature { c, s }
// }

// // 定义环签名验证函数
// fn ring_verify(message: &Message, public_keys: &[PublicKey], signature: &RingSignature) -> bool {
//     // 获取公钥集合的长度
//     let n = public_keys.len();
//     // 检查s向量的长度是否与公钥集合相同
//     if signature.s.len() != n {
//         return false;
//     }
//     // 从0开始循环
//     let mut cj = signature.c.clone();
//     for j in 0..n {
//         // 计算Cj' = sj*G + H(C||j)*Pj
//         let mut hasher = Sha256::new();
//         hasher.update(signature.c.serialize());
//         hasher.update(j.to_be_bytes());
//         let h = hasher.finalize();
//         let h = SecretKey::from_slice(&h).expect("hash is not a valid secret key");
//         let cj_prime = PublicKey::from_combination(&[PublicKey::from_secret_key(&signature.s[j]), PublicKey::from_secret_key(&h).mul(&public_keys[j])]).expect("public key combination failed");
//         // 检查是否有Cj' == Cj
//         if cj_prime != cj {
//             return false;
//         }
//         cj = cj_prime;
//     }
//     // 如果都相等，则返回true
//     true
// }

// // 测试代码
// fn main() {
//     // 生成一个消息
//     let message = Message::from_slice(b"Hello, world!").expect("32 bytes");
//     // 生成三个公私钥对
//     let mut rng = thread_rng();
//     let (sk1, pk1) = (SecretKey::random(&mut rng), PublicKey::from_secret_key(&SecretKey::random(&mut rng)));
//     let (sk2, pk2) = (SecretKey::random(&mut rng), PublicKey::from_secret_key(&SecretKey::random(&mut rng)));
//     let (sk3, pk3) = (SecretKey::random(&mut rng), PublicKey::from_secret_key(&SecretKey::random(&mut rng)));
//     // 构造公钥集合
//     let public_keys = vec![pk1, pk2, pk3];
//     // 使用第一个私钥生成环签名
//     let signature = ring_sign(&message, &public_keys, &sk1);
//     // 验证环签名
//     let valid = ring_verify(&message, &public_keys, &signature);
//     // 打印验证结果
//     println!("The signature is {}", if valid { "valid" } else { "invalid" });
// }

// // // 引入外部库
// // use libc::{c_int, size_t};
// // use crypto::sha2::Sha256;
// // use crypto::digest::Digest;
// // use rand::RngCore;

// // // 定义常量和类型
// // const N: usize = 4; // 环的大小
// // type PublicKey = [u8; 32]; // 公钥类型
// // type PrivateKey = [u8; 32]; // 私钥类型
// // type Signature = ([u8; 32], Vec<[u8; 32]>); // 签名类型，包括一个标签和一个向量

// // // 生成密钥对
// // fn generate_keypair() -> (PublicKey, PrivateKey) {
// //     // 使用随机数生成器生成私钥
// //     let mut rng = rand::thread_rng();
// //     let mut sk = [0u8; 32];
// //     rng.fill_bytes(&mut sk);

// //     // 使用ed25519算法生成公钥
// //     let pk = ed25519_dalek::PublicKey::from(&sk);

// //     // 返回公钥和私钥
// //     (pk.to_bytes(), sk)
// // }

// // // 生成签名
// // fn generate_signature(message: &[u8], ring: &[PublicKey], sk: &PrivateKey, index: usize) -> Signature {
// //     // 检查参数是否合法
// //     assert!(ring.len() == N);
// //     assert!(index < N);

// //     // 初始化哈希函数
// //     let mut hasher = Sha256::new();

// //     // 计算公钥的哈希值
// //     let mut h = Vec::new();
// //     for pk in ring {
// //         hasher.input(pk);
// //         h.push(hasher.result_reset());
// //     }

// //     // 生成一个随机数作为标签
// //     let mut rng = rand::thread_rng();
// //     let mut tag = [0u8; 32];
// //     rng.fill_bytes(&mut tag);

// //     // 初始化一个向量作为签名的一部分
// //     let mut s = vec![[0u8; 32]; N];

// //     // 计算第一个哈希值
// //     hasher.input(&tag);
// //     hasher.input(&h[index]);
// //     hasher.input(message);
// //     let mut c = hasher.result_reset();

// //     // 遍历环中的其他公钥，计算哈希值和随机数
// //     for i in (index + 1)..(index + N) {
// //         let j = i % N;

// //         // 生成一个随机数作为向量的元素
// //         rng.fill_bytes(&mut s[j]);

// //         // 计算下一个哈希值
// //         hasher.input(&c);
// //         hasher.input(&s[j]);
// //         hasher.input(&h[j]);
// //         c = hasher.result_reset()
// //     }

// //     // 使用私钥计算最后一个随机数，使得哈希值闭合
// //     s[index] = ed25519_dalek::ExpandedSecretKey::from(sk)
// //         .sign_prehashed(c, &ed25519_dalek::PublicKey::from(sk))
// //         .to_bytes();

// //     // 返回签名，包括标签和向量
// //     (tag, s)
// // }

    
// // // 验证签名
// // fn verify_signature(message: &[u8], ring: &[PublicKey], signature: &Signature) -> bool {
// //     // 检查参数是否合法
// //     assert!(ring.len() == N);

// //     // 初始化哈希函数
// //     let mut hasher = Sha256::new();

// //     // 计算公钥的哈希值
// //     let mut h = Vec::new();
// //     for pk in ring {
// //         hasher.input(pk);
// //         h.push(hasher.result_reset());
// //     }

// //     // 解析签名，得到标签和向量
// //     let (tag, s) = signature;

// //     // 计算第一个哈希值
// //     hasher.input(tag);
// //     hasher.input(&h[0]);
// //     hasher.input(message);
// //     let mut c = hasher.result_reset();

// //     // 遍历环中的所有公钥，计算哈希值和验证签名
// //     for i in 0..N {
// //         // 使用ed25519算法验证签名
// //         let sig = ed25519_dalek::Signature::from(s[i]);
// //         if let Err(_) = ed25519_dalek::PublicKey::from(ring[i]).verify_prehashed(c, None, &sig) {
// //             return false;
// //         }

// //         // 计算下一个哈希值
// //         hasher.input(&c);
// //         hasher.input(&s[i]);
// //         hasher.input(&h[i]);
// //         c = hasher.result_reset();
// //     }

// //     // 检查最后一个哈希值是否等于标签，如果是，返回真，否则返回假
// //     c == *tag
// // }
    
// // // 测试函数
// // fn test() {
// //     // 生成一个消息
// //     let message = b"Hello, world!";

// //     // 生成四个密钥对，作为环的元素
// //     let (pk1, sk1) = generate_keypair();
// //     let (pk2, sk2) = generate_keypair();
// //     let (pk3, sk3) = generate_keypair();
// //     let (pk4, sk4) = generate_keypair();

// //     // 构造一个环，包含四个公钥
// //     let ring = [pk1, pk2, pk3, pk4];

// //     // 使用第一个私钥生成一个签名
// //     let signature = generate_signature(message, &ring, &sk1, 0);

// //     // 验证签名是否有效，应该返回真
// //     println!("Signature is valid: {}", verify_signature(message, &ring, &signature));

// //     // 修改消息，验证签名是否有效，应该返回假
// //     println!("Signature is valid: {}", verify_signature(b"Hello, world?", &ring, &signature));

// //     // 修改环，验证签名是否有效，应该返回假
// //     println!("Signature is valid: {}", verify_signature(message, &[pk2, pk3, pk4, pk1], &signature));
// // }
    

// // #[test]
// // pub fn test_ring() {
// //     use fujisaki_ringsig::{gen_keypair, sign, trace, verify, Tag, Trace};

// //     let msg1 = b"now that the party is jumping";
// //     let msg2 = b"magnetized by the mic while I kick my juice";
// //     let issue = b"testcase 12345".to_vec();

// //     let mut rng = rand::thread_rng();
    
// //     // Make some keypairs for our ring. Pretend we only have the private key of the first keypair
// //     let (my_privkey, pubkey1) = gen_keypair(&mut rng);
// //     let (_, pubkey2) = gen_keypair(&mut rng);
// //     let (_, pubkey3) = gen_keypair(&mut rng);
// //     let pubkeys = vec![pubkey1.clone(), pubkey2, pubkey3];
    
// //     // Make the tag corresponding to this issue and ring
// //     let tag = Tag {
// //         issue,
// //         pubkeys,
// //     };
    
// //     // Make two signatures. Sign different messages with the same key and the same tag. This is
// //     // a no-no. We will get caught.
// //     let sig1 = sign(&mut rng, &*msg1, &tag, &my_privkey);
// //     let sig2 = sign(&mut rng, &*msg2, &tag, &my_privkey);
    
// //     // The signatures are all valid
// //     assert!(verify(&*msg1, &tag, &sig1));
// //     assert!(verify(&*msg2, &tag, &sig2));
    
// //     // Can't mix signatures
// //     assert!(!verify(&*msg1, &tag, &sig2));
    
// //     // But we have been caught double-signing!
// //     assert_eq!(trace(&*msg1, &sig1, &*msg2, &sig2, &tag), Trace::Revealed(&pubkey1));
// // }