use rand::Rng;
use std::str::FromStr;
use std::time::Instant;
use rug::{Integer, integer::Order};
use num_bigint::{BigInt, ToBigInt, Sign};

pub const P_64: &str = "13758676365741467507";
pub const P_128: &str = "284966011836017917039797442435648636163";
pub const P_256: &str = "79128031240076844063259589759962924441255910968111729611693920152825864722707";
pub const P_512: &str = "10711734159436774894171334484137626675507759979749407253125221261168087448899876831488509454695461974257751111853456275453329348448922191916590010377596767";
pub const P_1024: &str = "158297696608074679654124946564912202999139663277505984894261981349837992769596165683700437968679604111373729258655046764462137227577322861762501627230418997487671809885760928375348392323002752945263359796693275288611323927303851169352900910708127230034239565388759941444235878668699843286794016470366892082267";
pub const P_2048: &str = "22287360226908822233992819736392944434475043692265646916055930477587645696682024041890820611728835974780990571065838330253841354283867699159271588286101147370436450708147936416639540332373863814027801664774471436354150618315722661359913455362721373024713389259210331115681727749894367904502907551083219287819263090154675250911168607561882294815102877332366368477130120481174929478405004375083454233478408080520257325818925705871467706311605717341130286381719809389913520035118471758580658821155908577746981648167876884576360004782560776732442189914352788858257527373771629598261282997979720455015240977446412661775607";

pub const DATA_DIR: [&str; 4] = [r"src", "vde", "data", "modpow"];

pub fn fast_modpow(mut x: BigInt, mut t: BigInt, p: BigInt)-> BigInt {
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

pub fn test_bigint(p_bits: usize, samples: usize) -> f32 {
    let mut rng = rand::thread_rng();

    let p;

    if p_bits == 64 {
        p = BigInt::from_str(P_64).unwrap();
    }
    else if p_bits == 128 {
        p = BigInt::from_str(P_128).unwrap();
    }
    else if p_bits == 256 {
        p = BigInt::from_str(P_256).unwrap();
    }
    else if p_bits == 512 {
        p = BigInt::from_str(P_512).unwrap();
    }
    else if p_bits == 1024 {
        p = BigInt::from_str(P_1024).unwrap();
    }
    else {
        p = BigInt::from_str(P_2048).unwrap();
    }

    let mut cost = 0.0;
    for _ in 0..samples {
        let x_str = (0..p_bits/8-1)
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

    cost / (samples as f32)
}

pub fn test_rug(p_size: usize, samples: usize) -> f32 {
    let mut rng = rand::thread_rng();
    
    let p;

    if p_size == 64 {
        p = Integer::from_str(P_64).unwrap();
    }
    else if p_size == 128 {
        p = Integer::from_str(P_128).unwrap();
    }
    else if p_size == 256 {
        p = Integer::from_str(P_256).unwrap();
    }
    else if p_size == 512 {
        p = Integer::from_str(P_512).unwrap();
    }
    else if p_size == 1024 {
        p = Integer::from_str(P_1024).unwrap();
    }
    else {
        p = Integer::from_str(P_2048).unwrap();
    }

    // let p = &p_str.parse::<Integer>().unwrap();

    let mut cost = 0.0;
    for _ in 0..samples {
        let x_str = (0..p_size/8-1)
        .map(|_| {
            let idx = rng.gen_range(0..=255);
            idx
        }).collect::<Vec<u8>>();
        let x = Integer::from_digits(&x_str, Order::Lsf);

        let start = Instant::now();
        let _ = x.clone().pow_mod(&((p.clone() - Integer::from(1)) / Integer::from(2)), &p).unwrap();
        cost += start.elapsed().as_secs_f32();
    }

    cost / (samples as f32)
}

#[test]
fn test_comp() {
    use std::{path::PathBuf, fs::OpenOptions, io::Write};

    let path: PathBuf = DATA_DIR.iter().collect();
    let save_path = path.to_str().unwrap();
    let mut save_file = OpenOptions::new()
    .read(true)
    .write(true)
    .append(true)
    .create(true) 
    .open(save_path)
    .unwrap();

    let should_save: bool = true;

    const P_BITS: usize = 1024;
    const SAMPLES: usize = 10000;

    let cost1 = test_bigint(P_BITS, SAMPLES);
    let cost2 = test_rug(P_BITS, SAMPLES);

    if should_save == true {
        save_file.write_all(["p size, ", &P_BITS.to_string(), ", samples, ", &SAMPLES.to_string(), ", num_bigint, ", &cost1.to_string(), ", rug, ", &cost2.to_string(), ", num_bigint / rug, ", &(cost1 / cost2).to_string(), "\n\n"].concat().as_bytes()).unwrap();
    }
}