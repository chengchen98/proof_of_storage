use std::fs::OpenOptions;
use rs_merkle::{MerkleTree, MerkleProof, Hasher, algorithms::Sha256};

use super::common::read_file;

pub fn generate_merkle_tree(path: &str, data_len: usize, leaf_len: usize) -> (Vec<[u8; 32]>, MerkleTree<Sha256>, [u8; 32]) {
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

pub fn generate_merkle_proof(indices_to_prove: &[usize], merkle_tree: MerkleTree<Sha256>) -> MerkleProof<Sha256> {
    let merkle_proof = merkle_tree.proof(&indices_to_prove);
    let proof_bytes = merkle_proof.to_bytes();
    let proof = MerkleProof::<Sha256>::try_from(proof_bytes).unwrap();
    proof
}

pub fn verify_merkle_proof(proof: MerkleProof<Sha256>, merkle_root: [u8; 32], indices_to_prove: &[usize], leaves: Vec<[u8; 32]>) {
    let mut leaves_to_prove = vec![];
    for i in 0..indices_to_prove.len() {
        let leaf = leaves.get(indices_to_prove[i]).ok_or("can't get leaves to prove").unwrap();
        leaves_to_prove.push(*leaf);
    }
    assert!(proof.verify(merkle_root, &indices_to_prove, leaves_to_prove.as_slice(), leaves.len()));
}