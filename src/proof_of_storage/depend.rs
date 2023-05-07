use md5::{Md5, Digest};
use rug::{Integer, integer::Order};
use blake3;

pub fn long_mode_1(index: usize, count: usize) -> Vec<usize> {
    //! rule: -1-2\*0, -1-2\*1, -1-2\*2, -1-2\*3...
    //! 
    //!           -1,     -3,     -5,     -7...
    let mut long_index = vec![];
    for i in 0..count {
        let dis = 1 + 2 * i;
        if index >= dis {
            long_index.push(index - dis);
        }
    }
    long_index
}

pub fn long_mode_2(index: usize, count: usize) -> Vec<usize> {
    //! rule: -1-2\^0, -1-2\^1, -1-2\^2, -1-2\^3...
    //! 
    //!       -2,     -3,     -5,     -9...
    let mut long_index = vec![];
    for i in 0..count {
        // dis = 1 + 2^i
        // dis = 2, 3, 5, 9..
        let dis = 1 + usize::pow(2, i.try_into().unwrap());
        if index >= dis {
            long_index.push(index - dis);
        }
    }
    long_index
}

pub fn long_mode_3(index: usize) -> Vec<usize> {
    // 回溯个数与当前数据块编号有关，=编号%10
    let mut long_index = vec![];
    let count = index / 10;
    for i in 0..count {
        // dis = 1 + 2^i
        // dis = 2, 3, 5, 9..
        let dis = 1 + usize::pow(2, i.try_into().unwrap());
        if index >= dis {
            long_index.push(index - dis);
        }
    }
    long_index
}

pub fn long_mode_random(data: &Vec<u8>, index: usize, count: usize) -> Vec<usize> {   
    let mut long_index = vec![];
    if index == 0 {
        return long_index;
    }

    // if index == 2 {
    //     println!("data: {:?}", data);
    // }

    let mut hasher = blake3::Hasher::new();
    hasher.update(data);
    let mut blake3_res = hasher.finalize();

    let mut data_hash: [u8; 16] = blake3_res.as_bytes().as_slice()[..16].try_into().unwrap();
    let mut data_idx = u128::from_be_bytes(data_hash);
    let mut depend_idx: u128 = data_idx % index as u128;
    long_index.push(depend_idx as usize);

    for _ in 1..count {
        hasher = blake3::Hasher::new();
        hasher.update(&data_hash);
        blake3_res = hasher.finalize();

        data_hash = blake3_res.as_bytes().as_slice()[..16].try_into().unwrap();
        data_idx = u128::from_be_bytes(data_hash);
        depend_idx = data_idx % index as u128;
        long_index.push(depend_idx as usize);
    }
    long_index
}

pub fn long_depend(index: usize, count: usize, mode: usize) -> Vec<usize> {
    //! Generate indexs of long depended.
    //! 
    //! num: the number of all blocks
    //! 
    //! index: the index of the current block
    //! 
    //! count: the count of depended indexs
    //! 
    //! mode: choose the rule
    if mode == 1 {
        long_mode_1(index, count)
    }
    else if mode == 2 {
        long_mode_2(index, count)
    }
    else if mode == 3 {
        long_mode_3(index)
    }
    else {
        vec![]
    }
}

pub fn short_mode_1(num: usize, index: usize, count: usize) -> Vec<usize> {
    //! rule: -1-2\*0, +1+2\*0, -1-2\*1, +1+2\*1, ...
    //! 
    //!       -1,     1,      -3,     3...
    let mut short_index = vec![];
    let mut epoch = 0;

    loop {
        // 当找到指定个数的依赖时，停止循环
        if short_index.len() >= count {
            break;
        }

        // dis = 1, 3, 5, 7..
        let dis = 1 + 2 * epoch;

        // 超出寻址范围时，停止循环
        if index < dis && index + dis >= num {
            break;
        }

        // 依赖在左侧
        if index >= dis {
            short_index.push(index - dis);
        }

        if short_index.len() >= count {
            break;
        }

        // 依赖在右侧
        if index + dis < num {
            short_index.push(index + dis);
        }
        
        epoch += 1;
    }

    for i in 0..short_index.len() {
        if short_index[i] == index {
            println!("error");
        }
    }
    short_index
}

pub fn short_mode_2(num: usize, index: usize, count: usize) -> Vec<usize> {
    //! rule: -1-2\^0, -1+2\^1, -1-2\^2, -1+2\^3...
    //! 
    //!       -2,     1,      -5,     7...
    let mut short_index = vec![];
    let mut flag = false;
    for i in 0..count {
        let idx;
        let dis1 = 1 - usize::pow(2, i.try_into().unwrap());
        let dis2 = 1 + usize::pow(2, i.try_into().unwrap());
        if flag == true {
            idx = index - dis1;
        }
        else {
            idx = index - dis2;
        }
        flag = !flag;

        if idx >= num {
            break;
        }

        if idx < num {
            short_index.push(idx);
        }
    }
    short_index
}

pub fn short_depend_random(num: usize, data: &Vec<u8>, index: usize, count: usize) -> Vec<usize> {
    let mut short_index = vec![];

    let mut hasher = blake3::Hasher::new();
    hasher.update(data);
    let mut blake3_res = hasher.finalize();

    let mut data_hash: [u8; 16] = blake3_res.as_bytes().as_slice()[..16].try_into().unwrap();
    let mut data_idx = u128::from_be_bytes(data_hash);
    let mut depend_idx: u128 = data_idx % num as u128;
    if index as u128 != depend_idx {
        short_index.push(depend_idx as usize);
    }

    for _ in 1..count {
        hasher = blake3::Hasher::new();
        hasher.update(&data_hash);
        blake3_res = hasher.finalize();

        data_hash = blake3_res.as_bytes().as_slice()[..16].try_into().unwrap();
        data_idx = u128::from_be_bytes(data_hash);
        depend_idx = data_idx % num as u128;

        if index as u128 != depend_idx {
            short_index.push(depend_idx as usize);
        }
    }
    short_index
}


pub fn short_depend(num: usize, index: usize, count: usize, mode: usize) -> Vec<usize> {
    //! Generate indexs of short depended.
    //! 
    //! num: the number of all blocks
    //! 
    //! index: the index of the current block
    //! 
    //! count: the count of depended indexs
    //! 
    //! mode: choose the rule
    if mode == 1 {
        short_mode_1(num, index, count)
    }
    else if mode == 2 {
        short_mode_2(num, index, count)
    }
    else {
        vec![]
    }
}


#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn test_long_mode_1() {
        let indexs = vec![5, 3, 1];
        let res = long_mode_1(6, 3);
        assert_eq!(indexs[0], res[0]);
        assert_eq!(indexs[1], res[1]);
        assert_eq!(indexs[2], res[2]);
    }

    #[test]
    fn test_long_mode_2() {
        let indexs = vec![5, 4, 2];
        let res = long_mode_2(7, 3);
        assert_eq!(indexs[0], res[0]);
        assert_eq!(indexs[1], res[1]);
        assert_eq!(indexs[2], res[2]);
    }
    
    #[test]
    fn test_short_mode_1() {
        let indexs = vec![2, 4, 0, 6];
        let res = short_mode_1(7, 3, 4);
        assert_eq!(indexs[0], res[0]);
        assert_eq!(indexs[1], res[1]);
        assert_eq!(indexs[2], res[2]);
        assert_eq!(indexs[3], res[3]);
    }

    #[test]
    fn test_short_mode_2() {
        let indexs = vec![3, 6, 0, 12];
        let res = short_mode_2(13, 5, 4);
        assert_eq!(indexs[0], res[0]);
        assert_eq!(indexs[1], res[1]);
        assert_eq!(indexs[2], res[2]);
        assert_eq!(indexs[3], res[3]);
    }
}
