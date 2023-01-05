use bls12_381::Scalar;

use crate::data::vecu8_xor;
use crate::depend::{long_depend, short_depend};
use crate::mimc_hash::mimc_hash;
use crate::vde::{vde, vde_inv};


pub fn create_depend(block_num_level_2: usize, block_num_level_1: usize, index_count_l: usize, index_count_s: usize, mode_l: usize, mode_s: usize) -> (Vec<Vec<Vec<usize>>>, Vec<Vec<Vec<usize>>>) {
    let mut index_l_collect = vec![];
    let mut index_s_collect = vec![];

    for i in 0..block_num_level_2 {
        let mut index_l = vec![];
        let mut index_s = vec![];
        for j in 0..block_num_level_1 {
            index_l.push(long_depend(block_num_level_2, i, index_count_l, mode_l));
            index_s.push(short_depend(block_num_level_1, j, index_count_s, mode_s));
        }
        index_l_collect.push(index_l);
        index_s_collect.push(index_s);
    }

    (index_l_collect, index_s_collect)
}

pub fn seal(round: usize, data_blocks: &mut Vec<Vec<Vec<u8>>>, index_l_collect: &Vec<Vec<Vec<usize>>>, index_s_collect: &Vec<Vec<Vec<usize>>>, constants: &Vec<Scalar>, key: Scalar, mode_vde: &str) {
    for _ in 0..round {
        for i in 0..data_blocks.len() {
            for j in 0..data_blocks[i].len() {
                let mut depend_data = vec![];

                for k in 0..index_l_collect[i][j].len() {
                    depend_data.append(&mut data_blocks[k][j].clone());
                }

                for k in 0..index_s_collect[i][j].len() {
                    depend_data.append(&mut data_blocks[i][k].clone());
                }

                depend_data = mimc_hash(&depend_data, &constants);
                depend_data = vecu8_xor(&depend_data, &data_blocks[i][j]);
                let new_data = vde(&depend_data, key, mode_vde);

                data_blocks[i][j] = new_data;
            }
        }
    }
}

pub fn unseal(round: usize, data_blocks: &mut Vec<Vec<Vec<u8>>>, index_l_collect: &Vec<Vec<Vec<usize>>>, index_s_collect: &Vec<Vec<Vec<usize>>>, constants: &Vec<Scalar>, key: Scalar, mode_vde: &str) {
    for _ in 0..round {
        for i in 0..data_blocks.len() {
            for j in 0..data_blocks[i].len() {
                let mut depend_data = vec![];

                for k in 0..index_l_collect[i][j].len() {
                    depend_data.append(&mut data_blocks[k][j].clone());
                }

                for k in 0..index_s_collect[i][j].len() {
                    depend_data.append(&mut data_blocks[i][k].clone());
                }

                depend_data = mimc_hash(&depend_data, &constants);

                let mut new_data = vde_inv(&data_blocks[i][j], key, mode_vde);
                new_data = vecu8_xor(&new_data, &depend_data);

                data_blocks[i][j] = new_data;
            }
        }
    }
}

#[cfg(test)]
mod test {

    use std::fs::File;
    use std::time::Instant;

    use bls12_381::Scalar;
    use ff::Field;
    use rand::thread_rng;

    use crate::data_seal::{create_depend, seal, unseal};
    use crate::data::{Data, to_block, DATA_DIR};

    #[test]
    fn test() {
        let mut rng = thread_rng();

        // Read data.
        let file = File::open(DATA_DIR).unwrap();
        let data: Data<Vec<u8>> = serde_json::from_reader(file).unwrap();
        let data_vec = data.content;

        let len_level_2: usize = 128;
        let len_level_1: usize = 64;

        let data_blocks = to_block(&data_vec, len_level_2);
        let mut data_blocks = data_blocks.iter().map(|x| to_block(&x, len_level_1)).collect::<Vec<Vec<Vec<_>>>>();

        let block_num_level_2 = data_blocks.len();
        let block_num_level_1 = data_blocks[0].len();

        let index_count_l: usize = 3;
        let index_count_s: usize = 3;

        let mode_l: usize = 1;
        let mode_s: usize = 1;
        let mode_vde: &str = "sloth";

        const ROUND: usize = 3;
        const MIMC_ROUNDS: usize = 322;
        
        let constants = (0..MIMC_ROUNDS)
            .map(|_| Scalar::random(&mut rng))
            .collect::<Vec<_>>();
        let key = Scalar::random(&mut rng);

        let (index_l_collect, index_s_collect) = create_depend(block_num_level_2, block_num_level_1, index_count_l, index_count_s, mode_l, mode_s);
        
        let start = Instant::now();
        seal(ROUND, &mut data_blocks, &index_l_collect, &index_s_collect, &constants, key, mode_vde);
        println!("Seal: {:?}", start.elapsed());

        let start = Instant::now();
        unseal(ROUND, &mut data_blocks, &index_l_collect, &index_s_collect, &constants, key, mode_vde);
        println!("Unseal: {:?}", start.elapsed());

        for i in 0..data_blocks.len() {
            for j in 0..data_blocks[i].len() {
                for k in 0..data_blocks[i][j].len() {
                    assert_eq!(data_blocks[i][j][k], data_vec[i * len_level_2 + j * len_level_1 + k]);
                }
            }
        }
    }
}