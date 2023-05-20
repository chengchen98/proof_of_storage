use std::fs::OpenOptions;
use rs_merkle::{MerkleTree, MerkleProof, Hasher, algorithms::Sha256};

use super::common::read_file;

pub const DATA_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "merkle_tree_data"];
pub const MERKLE_TREE_DIR: [&str; 4] = [r"src", "proof_of_storage", "data", "merkle_tree_result"];

pub fn generate_merkle_tree_from_file(path: &str, data_len: usize, leaf_len: usize) -> (Vec<[u8; 32]>, MerkleTree<Sha256>, [u8; 32]) {
    let mut file = OpenOptions::new()
    .read(true)
    .open(path)
    .unwrap();

    let mut leaf_values = vec![];
    for i in (0..data_len).step_by(leaf_len) {
        let buf = read_file(&mut file, i + leaf_len, leaf_len);
        leaf_values.push(buf);
    }
    let leaves: Vec< [u8; 32]> = leaf_values.iter().map(|x| Sha256::hash(x)).collect();
    let merkle_tree = MerkleTree::<Sha256>::from_leaves(&leaves);
    let merkle_root = merkle_tree.root().ok_or("can't get the merkle root").unwrap();
    (leaves, merkle_tree, merkle_root)
}

pub fn generate_merkle_tree_from_data(leaf_values: &Vec<Vec<u8>>) -> (Vec<[u8; 32]>, MerkleTree<Sha256>, [u8; 32]) {
    let leaves: Vec< [u8; 32]> = leaf_values.iter().map(|x| Sha256::hash(x)).collect();
    let merkle_tree = MerkleTree::<Sha256>::from_leaves(&leaves);
    let merkle_root = merkle_tree.root().ok_or("can't get the merkle root").unwrap();
    (leaves, merkle_tree, merkle_root)
}

pub fn generate_merkle_proof(indices_to_prove: &[usize], merkle_tree: &MerkleTree<Sha256>) -> MerkleProof<Sha256> {
    let merkle_proof = merkle_tree.proof(&indices_to_prove);
    let proof_bytes = merkle_proof.to_bytes();
    let proof = MerkleProof::<Sha256>::try_from(proof_bytes).unwrap();
    proof
}

pub fn verify_merkle_proof(proof: MerkleProof<Sha256>, merkle_root: [u8; 32], indices_to_prove: &[usize], leaves: &Vec<[u8; 32]>) {
    let mut leaves_to_prove = vec![];
    for i in 0..indices_to_prove.len() {
        let leaf = leaves.get(indices_to_prove[i]).ok_or("can't get leaves to prove").unwrap();
        leaves_to_prove.push(*leaf);
    }
    assert!(proof.verify(merkle_root, &indices_to_prove, leaves_to_prove.as_slice(), leaves.len()));
}

// pub fn test_merkle_tree_prove_and_verify(path: &str, data_len: usize, leaf_len: usize, leaves_to_prove_count: usize) {
//     // 生成merkle tree，并随机选取n个叶子结点进行验证
//     let (leaves, merkle_tree, merkle_root) = generate_merkle_tree(&path, data_len, leaf_len);
//     println!("leaf number: {:?}  | tree depth: {:?}  |  prove count: {:?}", leaves.len(), merkle_tree.depth(), leaves_to_prove_count);

//     let indices_to_prove = generate_random_indices_to_prove(leaves_to_prove_count, (0, leaves.len()));
//     println!("indices to prove: {:?}", indices_to_prove);
    
//     let proof = generate_merkle_proof(&indices_to_prove, merkle_tree);
//     verify_merkle_proof(proof, merkle_root, &indices_to_prove, &leaves);
// }


#[test]
pub fn test(){
    use std::{fs::OpenOptions, path::PathBuf, time::Instant, io::Write};
    use super::verifier::{create_random_file, create_challenges};

    let path: PathBuf = DATA_DIR.iter().collect();
    let path = path.to_str().unwrap();

    let save_path: PathBuf = MERKLE_TREE_DIR.iter().collect();
    let save_path = save_path.to_str().unwrap();
    let mut save_file = OpenOptions::new()
    .read(true)
    .write(true)
    .append(true)
    .create(true) 
    .open(save_path)
    .unwrap();

    const DATA_L: usize = 1024 * 1024 * 1024;
    const LEAVE_L: usize = 1024 * 1024;
    const COUNT: usize = 10;
    create_random_file(path, DATA_L).unwrap();

    const SAMPLES: usize = 10;
    let mut t1 = 0.0;
    let mut t2 = 0.0;
    let mut t3 = 0.0;
    for _ in 0..SAMPLES {
        let start = Instant::now();
        let (leaves, merkle_tree, merkle_root) = generate_merkle_tree_from_file(&path, DATA_L, LEAVE_L);
        t1 += start.elapsed().as_secs_f32();

        let indices_to_prove = create_challenges(COUNT, (0, leaves.len()));
        
        let start = Instant::now();
        let proof = generate_merkle_proof(&indices_to_prove, &merkle_tree);
        t2 += start.elapsed().as_secs_f32();

        let start = Instant::now();
        verify_merkle_proof(proof, merkle_root, &indices_to_prove, &leaves);
        t3 += start.elapsed().as_secs_f32();
    }
    t1 = t1 / (SAMPLES as f32);
    t2 = t2 / (SAMPLES as f32);
    t3 = t3 / (SAMPLES as f32);

    save_file.write_all(["data size, ", &DATA_L.to_string(), ", leave size, ", &LEAVE_L.to_string(), ", leave count, ", &(DATA_L / LEAVE_L).to_string(), " , challenges, ", &COUNT.to_string(), "\n"].concat().as_bytes()).unwrap();
    save_file.write_all(["samples, ", &SAMPLES.to_string(), " , challenge count, ", &COUNT.to_string(), "\n"].concat().as_bytes()).unwrap();
    save_file.write_all(["generate tree, ", &t1.to_string(), ", create proof, ", &t2.to_string(), ", verify, ", &t3.to_string(), "\n\n"].concat().as_bytes()).unwrap();
}