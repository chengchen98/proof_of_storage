// // 引入外部库
// use libc::{c_int, size_t};
// use crypto::sha2::Sha256;
// use crypto::digest::Digest;
// use rand::RngCore;

// // 定义常量和类型
// const N: usize = 4; // 环的大小
// type PublicKey = [u8; 32]; // 公钥类型
// type PrivateKey = [u8; 32]; // 私钥类型
// type Signature = ([u8; 32], Vec<[u8; 32]>); // 签名类型，包括一个标签和一个向量

// // 生成密钥对
// fn generate_keypair() -> (PublicKey, PrivateKey) {
//     // 使用随机数生成器生成私钥
//     let mut rng = rand::thread_rng();
//     let mut sk = [0u8; 32];
//     rng.fill_bytes(&mut sk);

//     // 使用ed25519算法生成公钥
//     let pk = ed25519_dalek::PublicKey::from(&sk);

//     // 返回公钥和私钥
//     (pk.to_bytes(), sk)
// }

// // 生成签名
// fn generate_signature(message: &[u8], ring: &[PublicKey], sk: &PrivateKey, index: usize) -> Signature {
//     // 检查参数是否合法
//     assert!(ring.len() == N);
//     assert!(index < N);

//     // 初始化哈希函数
//     let mut hasher = Sha256::new();

//     // 计算公钥的哈希值
//     let mut h = Vec::new();
//     for pk in ring {
//         hasher.input(pk);
//         h.push(hasher.result_reset());
//     }

//     // 生成一个随机数作为标签
//     let mut rng = rand::thread_rng();
//     let mut tag = [0u8; 32];
//     rng.fill_bytes(&mut tag);

//     // 初始化一个向量作为签名的一部分
//     let mut s = vec![[0u8; 32]; N];

//     // 计算第一个哈希值
//     hasher.input(&tag);
//     hasher.input(&h[index]);
//     hasher.input(message);
//     let mut c = hasher.result_reset();

//     // 遍历环中的其他公钥，计算哈希值和随机数
//     for i in (index + 1)..(index + N) {
//         let j = i % N;

//         // 生成一个随机数作为向量的元素
//         rng.fill_bytes(&mut s[j]);

//         // 计算下一个哈希值
//         hasher.input(&c);
//         hasher.input(&s[j]);
//         hasher.input(&h[j]);
//         c = hasher.result_reset()
//     }

//     // 使用私钥计算最后一个随机数，使得哈希值闭合
//     s[index] = ed25519_dalek::ExpandedSecretKey::from(sk)
//         .sign_prehashed(c, &ed25519_dalek::PublicKey::from(sk))
//         .to_bytes();

//     // 返回签名，包括标签和向量
//     (tag, s)
// }

    
// // 验证签名
// fn verify_signature(message: &[u8], ring: &[PublicKey], signature: &Signature) -> bool {
//     // 检查参数是否合法
//     assert!(ring.len() == N);

//     // 初始化哈希函数
//     let mut hasher = Sha256::new();

//     // 计算公钥的哈希值
//     let mut h = Vec::new();
//     for pk in ring {
//         hasher.input(pk);
//         h.push(hasher.result_reset());
//     }

//     // 解析签名，得到标签和向量
//     let (tag, s) = signature;

//     // 计算第一个哈希值
//     hasher.input(tag);
//     hasher.input(&h[0]);
//     hasher.input(message);
//     let mut c = hasher.result_reset();

//     // 遍历环中的所有公钥，计算哈希值和验证签名
//     for i in 0..N {
//         // 使用ed25519算法验证签名
//         let sig = ed25519_dalek::Signature::from(s[i]);
//         if let Err(_) = ed25519_dalek::PublicKey::from(ring[i]).verify_prehashed(c, None, &sig) {
//             return false;
//         }

//         // 计算下一个哈希值
//         hasher.input(&c);
//         hasher.input(&s[i]);
//         hasher.input(&h[i]);
//         c = hasher.result_reset();
//     }

//     // 检查最后一个哈希值是否等于标签，如果是，返回真，否则返回假
//     c == *tag
// }
    
// // 测试函数
// fn test() {
//     // 生成一个消息
//     let message = b"Hello, world!";

//     // 生成四个密钥对，作为环的元素
//     let (pk1, sk1) = generate_keypair();
//     let (pk2, sk2) = generate_keypair();
//     let (pk3, sk3) = generate_keypair();
//     let (pk4, sk4) = generate_keypair();

//     // 构造一个环，包含四个公钥
//     let ring = [pk1, pk2, pk3, pk4];

//     // 使用第一个私钥生成一个签名
//     let signature = generate_signature(message, &ring, &sk1, 0);

//     // 验证签名是否有效，应该返回真
//     println!("Signature is valid: {}", verify_signature(message, &ring, &signature));

//     // 修改消息，验证签名是否有效，应该返回假
//     println!("Signature is valid: {}", verify_signature(b"Hello, world?", &ring, &signature));

//     // 修改环，验证签名是否有效，应该返回假
//     println!("Signature is valid: {}", verify_signature(message, &[pk2, pk3, pk4, pk1], &signature));
//     }
    

// #[test]
// pub fn test_ring() {
//     use fujisaki_ringsig::{gen_keypair, sign, trace, verify, Tag, Trace};

//     let msg1 = b"now that the party is jumping";
//     let msg2 = b"magnetized by the mic while I kick my juice";
//     let issue = b"testcase 12345".to_vec();

//     let mut rng = rand::thread_rng();
    
//     // Make some keypairs for our ring. Pretend we only have the private key of the first keypair
//     let (my_privkey, pubkey1) = gen_keypair(&mut rng);
//     let (_, pubkey2) = gen_keypair(&mut rng);
//     let (_, pubkey3) = gen_keypair(&mut rng);
//     let pubkeys = vec![pubkey1.clone(), pubkey2, pubkey3];
    
//     // Make the tag corresponding to this issue and ring
//     let tag = Tag {
//         issue,
//         pubkeys,
//     };
    
//     // Make two signatures. Sign different messages with the same key and the same tag. This is
//     // a no-no. We will get caught.
//     let sig1 = sign(&mut rng, &*msg1, &tag, &my_privkey);
//     let sig2 = sign(&mut rng, &*msg2, &tag, &my_privkey);
    
//     // The signatures are all valid
//     assert!(verify(&*msg1, &tag, &sig1));
//     assert!(verify(&*msg2, &tag, &sig2));
    
//     // Can't mix signatures
//     assert!(!verify(&*msg1, &tag, &sig2));
    
//     // But we have been caught double-signing!
//     assert_eq!(trace(&*msg1, &sig1, &*msg2, &sig2, &tag), Trace::Revealed(&pubkey1));
// }