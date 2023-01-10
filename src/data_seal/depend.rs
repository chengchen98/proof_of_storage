pub fn long_mode_1(num: usize, index: usize, count: usize) -> Vec<usize> {
    //! rule: -1-2\*0, -1-2\*1, -1-2\*2, -1-2\*3...
    //! 
    //!           -1,     -3,     -5,     -7...
    let mut long_index = vec![];
    for i in 0..count {
        if index >= 1 + 2 * i {
            let idx = index - 1 - 2 * i;
            if idx < num {
                long_index.push(idx);
            } 
        }
    }
    long_index
}

pub fn long_mode_2(num: usize, index: usize, count: usize) -> Vec<usize> {
    //! rule: -1-2\^0, -1-2\^1, -1-2\^2, -1-2\^3...
    //! 
    //!       -2,     -3,     -5,     -9...
    let mut long_index = vec![];
    for i in 0..count {
        let idx = index - 1 - usize::pow(2, i.try_into().unwrap());
        if idx < num {
            long_index.push(idx);
        }
    }
    long_index
}

pub fn long_depend(num: usize, index: usize, count: usize, mode: usize) -> Vec<usize> {
    //! Generate indexs of long depended.
    //! 
    //! num: the number of all blocks
    //! 
    //! index: the index of the current block
    //! 
    //! count: the count of depended indexs
    //! 
    //! mode: choose the rule
    let mut res = vec![];
    if mode == 1 {
        res = long_mode_1(num, index, count);
    }
    else if mode == 2 {
        res = long_mode_2(num, index, count);
    }
    res
}

pub fn short_mode_1(num: usize, index: usize, count: usize) -> Vec<usize> {
    //! rule: -1-2\*0, +1+2\*0, -1-2\*1, +1+2\*1, ...
    //! 
    //!       -1,     1,      -3,     3...
    let mut short_index = vec![];
    let mut epoch = 0;

    loop {
        if short_index.len() == count {
            break;
        }

        let dis = 1 + 2 * epoch;

        if index < dis && index + dis >= num {
            break;
        }

        if index >= dis {
            short_index.push(index - dis);
        }

        if index + dis < num {
            short_index.push(index + dis);
        }
        
        epoch += 1;
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
        if flag == true {
            idx = index - 1 + usize::pow(2, i.try_into().unwrap());
        }
        else {
            idx = index - 1 - usize::pow(2, i.try_into().unwrap());
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
    let mut res = vec![];
    if mode == 1 {
        res = short_mode_1(num, index, count);
    }
    else if mode == 2 {
        res = short_mode_2(num, index, count);
    }
    res
}


#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn test_long_mode_1() {
        let indexs = vec![5, 3, 1];
        let res = long_mode_1(7, 6, 3);
        assert_eq!(indexs[0], res[0]);
        assert_eq!(indexs[1], res[1]);
        assert_eq!(indexs[2], res[2]);
    }

    #[test]
    fn test_long_mode_2() {
        let indexs = vec![5, 4, 2];
        let res = long_mode_2(8, 7, 3);
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
