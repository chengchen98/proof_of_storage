use rug::{Integer, integer::Order};
use super::rug_sloth::{sloth, sloth_inv};

pub fn vde(x: &Vec<u8>, p: &Integer, t: usize, mode: &String, l: usize) -> Vec<u8> {
    let cur_x = Integer::from_digits(&x, Order::Lsf);
    let y;
    if mode == "sloth" {
        y = sloth(&cur_x, p, t);
    }
    else {
        y = sloth(&cur_x, p, t);
    }
    let mut y_bytes = y.to_digits::<u8>(Order::Lsf);

    if y_bytes.len() < l {
        y_bytes.append(&mut vec![0u8; l - y_bytes.len()]);
    }
    y_bytes
}

pub fn vde_inv(y: &Vec<u8>, p: &Integer, t: usize, mode: &String, l: usize) -> Vec<u8> {
    let cur_y = Integer::from_digits(&y, Order::Lsf);
    let x;

    if mode == "sloth" {
        x = sloth_inv(&cur_y, p, t);
    }
    else {
        x = sloth_inv(&cur_y, p, t);
    }
    let mut x_bytes = x.to_digits::<u8>(Order::Lsf);

    if x_bytes.len() < l {
        x_bytes.append(&mut vec![0u8; l - x_bytes.len()]);
    }
    x_bytes
}

#[test]
fn test_vde() {
    use std::str::FromStr;
    use std::time::Instant;
    use super::sloth::P_1024;

    const T: usize = 3;
    let x =vec![1u8; 120];
    let p = &Integer::from_str(P_1024).unwrap();

    const SAMPLES: usize = 10;
    for i in 0..SAMPLES {
        println!("sample: {:?}", i);
        
        let start = Instant::now();
        let y = vde(&x, p, T, &"sloth".to_string(), 1024/8);
        let cost1 = start.elapsed();
        println!("Vde: {:?}", cost1);

        let start = Instant::now();
        let z = vde_inv(&y, p, T, &"sloth".to_string(), 1024/8);
        let cost2 = start.elapsed();
        println!("Vde inv: {:?}", cost2);

        println!("vde / vde inv: {:?}", cost1.as_secs_f32() / cost2.as_secs_f32());
        assert_eq!(Integer::from_digits(&x, Order::Lsf), Integer::from_digits(&z, Order::Lsf));
        println!("-------------------------------------");
    }
}