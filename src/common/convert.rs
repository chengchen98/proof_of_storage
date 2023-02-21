pub fn bytes_to_bits(x: &Vec<u8>) -> Vec<bool> {
    //! 字节转比特
    //! example: [1] -> [true, false, false, false, false, false, false, false]
    let mut res = vec![];
    for i in 0..x.len() {
        let mut xi = x[i];
        let mut cur = vec![];
        for _ in 0..8 {
            if xi % 2 == 1 {
                cur.push(true);
            }
            else {
                cur.push(false);
            }
            xi /= 2;
        }
        res.append(&mut cur);
    }
    res
}

pub fn bits_to_bytes(x: &Vec<bool>) -> Vec<u8> {
    //! 比特转字节
    //! example: [true] -> [1]
    let mut res = vec![];
    let mut j = 0;
    let mut tmp = 0;
    let mut base = 1;
    for i in 0..x.len() {
        tmp += base * {
            if x[i] == false {
                0
            }
            else {
                1
            }
        };
        j += 1;
        if j == 8 {
            res.push(tmp);
            tmp = 0;
            j = 0;
            base = 1;
        }
        else {
            base *= 2;
        }
    }

    if j != 0 {
        res.push(tmp);
    }
    res
}

pub fn bits_to_usize(x: &Vec<bool>) -> usize {
    //! 比特转整型
    let mut res = 0;
    let mut base = 1;
    for i in 0..x.len() {
        res += base * { 
            if x[i] == false {
                0
            }
            else {
                1
            }
        };
        base *= 2;
    }
    res
}

pub fn usize_to_bits(x: usize, n: usize) -> Vec<bool> {
    //! 整型转比特
    let mut x = x.clone();
    let mut res = vec![false; n];
    let mut i = 0;
    while x > 0 {
        if x % 2 == 1 {
            res[i] = true;
        }
        x = x / 2;
        i += 1;
        
        if i >= n {
            break;
        }
    }
    res
}

#[test]
fn test_bytes_bits_convert() {
    let x = vec![false, true];
    let y = bits_to_bytes(&x);
    let xx = bytes_to_bits(&y);
    let yy = bits_to_bytes(&xx);
    assert_eq!(y, yy);
}

#[test]
fn test_bits_usize_convert() {
    let x = vec![false, true];
    let y = bits_to_usize(&x);
    assert_eq!(y, 2);
}