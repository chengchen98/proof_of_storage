// def fastModular(x): #快速幂的实现
// 	"""x[0] = base """
// 	"""x[1] = power"""
// 	"""x[2] = modulus"""
// 	result = 1
// 	while(x[1] > 0):
// 		if(x[1] & 1): # 位运算加快判断奇偶
// 			result = result * x[0] % x[2]
// 		x[1] = int(x[1]/2)
// 		x[0] = x[0] * x[0] % x[2]
// 	return result

use num_bigint::{BigInt, ToBigInt};

pub fn fast_modular(mut x: BigInt, mut t: BigInt, p: BigInt)-> BigInt {
    let mut res = 1.to_bigint().unwrap();
    while t.clone() > 0.to_bigint().unwrap() {
        if t.clone() & 1.to_bigint().unwrap() == 1.to_bigint().unwrap() {
            res = res * x.clone() % p.clone();
        }

        t = t.clone() / 2.to_bigint().unwrap();
        x = x.clone() * x % p.clone();
    }
    res
}

pub fn fast_mod(mut x: u128, mut t: u128, p: u128) -> u128 {
    let mut res = 1;
    while t > 0 {
        if t & 1 == 1 {
            res = res * x % p;
        }

        t = t / 2;
        x = x * x % p;
    }
    res
}
    
#[test]
fn test_bigint() {
    use rand::Rng;
    use num_bigint::Sign;
    use std::str::FromStr;
    use std::time::Instant;
    use num_bigint::{BigInt, ToBigInt};
    
    let mut rng = rand::thread_rng();
    
    // const P_1024: &str = "158297696608074679654124946564912202999139663277505984894261981349837992769596165683700437968679604111373729258655046764462137227577322861762501627230418997487671809885760928375348392323002752945263359796693275288611323927303851169352900910708127230034239565388759941444235878668699843286794016470366892082267";
    // let p = BigInt::from_str(P_1024).unwrap();

    // const P_512: &str = "10711734159436774894171334484137626675507759979749407253125221261168087448899876831488509454695461974257751111853456275453329348448922191916590010377596767";
    // let p = BigInt::from_str(P_512).unwrap();

    const P_128: &str = "284966011836017917039797442435648636163";
    let p = BigInt::from_str(P_128).unwrap();

    const SAMPLES: usize = 10;
    let mut cost = 0.0;
    for _ in 0..SAMPLES {
        let x_str = (0..8)
        .map(|_| {
            let idx = rng.gen_range(0..=255);
            idx
        }).collect::<Vec<_>>();
        let x = BigInt::from_bytes_le(Sign::Plus, &x_str);
        
        let start = Instant::now();
        let _ = x.modpow(&((&p - 1.to_bigint().unwrap()) / 2.to_bigint().unwrap()), &p);
        // let _ = fast_modular(x, (p.clone()-1)/2, p.clone());
        cost += start.elapsed().as_secs_f32();
    }

    cost = cost / (SAMPLES as f32);
    println!("{:?}", cost);
}

// 0.0009

#[test]
fn test() {
    use rand::Rng;
    use std::time::Instant;
    
    let mut rng = rand::thread_rng();
    let x = rng.gen_range(0..2_u128.pow(64));
    let p = 284966011836017917039797442435648636163;

    const SAMPLES: usize = 100;
    let mut cost = 0.0;
    for _ in 0..SAMPLES {
        let start = Instant::now();
        let _ = fast_mod(x, (p-1)/2, p);
        cost += start.elapsed().as_secs_f32();
    }

    cost = cost / (SAMPLES as f32);
    println!("{:?}", cost);
}

// 0.00000928