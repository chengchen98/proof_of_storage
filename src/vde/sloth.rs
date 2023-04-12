use num_bigint::{BigInt, ToBigInt};

pub const P_64: &str = "13758676365741467507";
pub const P_128: &str = "284966011836017917039797442435648636163";
pub const P_256: &str = "79128031240076844063259589759962924441255910968111729611693920152825864722707";
pub const P_512: &str = "10711734159436774894171334484137626675507759979749407253125221261168087448899876831488509454695461974257751111853456275453329348448922191916590010377596767";
pub const P_1024: &str = "158297696608074679654124946564912202999139663277505984894261981349837992769596165683700437968679604111373729258655046764462137227577322861762501627230418997487671809885760928375348392323002752945263359796693275288611323927303851169352900910708127230034239565388759941444235878668699843286794016470366892082267";
pub const P_2048: &str = "22287360226908822233992819736392944434475043692265646916055930477587645696682024041890820611728835974780990571065838330253841354283867699159271588286101147370436450708147936416639540332373863814027801664774471436354150618315722661359913455362721373024713389259210331115681727749894367904502907551083219287819263090154675250911168607561882294815102877332366368477130120481174929478405004375083454233478408080520257325818925705871467706311605717341130286381719809389913520035118471758580658821155908577746981648167876884576360004782560776732442189914352788858257527373771629598261282997979720455015240977446412661775607";

pub const DATA_DIR: [&str; 4] = [r"src", "vde", "data", "sloth"];

pub fn legendre(mut x: BigInt, p: &BigInt) -> BigInt {
    let mut s = 1.to_bigint().unwrap();
    if x.clone() == 0.to_bigint().unwrap() {
        return 0.to_bigint().unwrap();
    }
    else if x.clone() == 1.to_bigint().unwrap() {
        return 1.to_bigint().unwrap();
    }
    else {
        let e = {
            let mut e = 0.to_bigint().unwrap();
            while x.clone() % 2.to_bigint().unwrap() == 0.to_bigint().unwrap() {
                x = x.clone() / 2.to_bigint().unwrap();
                e += 1.to_bigint().unwrap();
            }
            e
        };

        if e % 2.to_bigint().unwrap() == 0.to_bigint().unwrap() {
            s = 1.to_bigint().unwrap();
        }
        else {
            if p.clone() % 8.to_bigint().unwrap() == 1.to_bigint().unwrap() || p.clone() % 8.to_bigint().unwrap() == 7.to_bigint().unwrap() {
                s = 1.to_bigint().unwrap();
            }
            if p.clone() % 8.to_bigint().unwrap() == 3.to_bigint().unwrap() || p.clone() % 8.to_bigint().unwrap() == 5.to_bigint().unwrap() {
                s = -1.to_bigint().unwrap();
            }
        }

        if p.clone() % 4.to_bigint().unwrap() == 3.to_bigint().unwrap() && x.clone() % 4 == 3.to_bigint().unwrap() {
            s = -s;
        }

        let p1 = p.clone() % x.clone();
        if x.clone() == 1.to_bigint().unwrap() {
            return s;
        }
        else {
            return s * legendre(p1, &x);
        }
    }
}

pub fn single_sloth(x: &BigInt, p: &BigInt) -> BigInt {
    let flag = legendre(x.clone(), &p);
    // let flag = x.modpow(&((p - 1.to_bigint().unwrap()) / 2.to_bigint().unwrap()), &p);
    let mut y;
    if flag == 1.to_bigint().unwrap() {
        y = x.modpow(&((p + 1.to_bigint().unwrap()) / 4.to_bigint().unwrap()), &p);
        if y.clone() % 2.to_bigint().unwrap() == 1.to_bigint().unwrap() {
            y = (p - y) % p;
        }
    }
    else {
        let xx = (p - x) % p;
        y = xx.modpow(&((p + 1.to_bigint().unwrap()) / 4.to_bigint().unwrap()), &p);
        if y.clone() % 2.to_bigint().unwrap() == 0.to_bigint().unwrap() {
            y = (p - y) % p;
        }
    }

    if y.clone() % 2.to_bigint().unwrap() == 1.to_bigint().unwrap() {
        return (y + 1.to_bigint().unwrap()) % p;
    }
    else {
        return (y - 1.to_bigint().unwrap()) % p;
    }
}

pub fn sloth(y: &BigInt, p: &BigInt, t: usize) -> BigInt {
    let mut y = y.clone();
    for _ in 0..t {
        y = single_sloth(&y, &p);
    }
    y
}

pub fn single_sloth_inv(y: &BigInt, p: &BigInt) -> BigInt {
    let x;
    if y.clone() % 2.to_bigint().unwrap() == 1.to_bigint().unwrap() {
        x = (y + 1.to_bigint().unwrap()) % p;
    }
    else {
        x = (y - 1.to_bigint().unwrap()) % p;
    }

    if x.clone() % 2.to_bigint().unwrap() == 1.to_bigint().unwrap() {
        return (p - (x.pow(2) % p)) % p;
    }
    else {
        return x.pow(2) % p;
    }
}

pub fn sloth_inv(y: &BigInt, p: &BigInt, t: usize) -> BigInt {
    let mut x = y.clone();
    for _ in 0..t {
        x = single_sloth_inv(&x, p);
    }
    x
}

#[test]
fn test_legendre() {
    let x = 118.to_bigint().unwrap();
    let p = 229.to_bigint().unwrap();
    println!("{:?}", legendre(x, &p));
}

#[test]
fn test_sloth() {
    use rand::Rng;
    use num_bigint::Sign;
    use std::str::FromStr;
    use std::time::Instant;
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

    const T: usize = 10;
    const PL: usize = 1024;
    let p;

    if PL == 64 {
        p = BigInt::from_str(P_64).unwrap();
    }
    else if PL == 128 {
        p = BigInt::from_str(P_128).unwrap();
    }
    else if PL == 256 {
        p = BigInt::from_str(P_256).unwrap();
    }
    else if PL == 512 {
        p = BigInt::from_str(P_512).unwrap();
    }
    else if PL == 1024 {
        p = BigInt::from_str(P_1024).unwrap();
    }
    else {
        p = BigInt::from_str(P_2048).unwrap();
    }
    
    let mut rng = rand::thread_rng();
    
    const SAMPLES: usize = 1000;
    let mut t1 = 0.0;
    let mut t2 = 0.0;
    for _ in 0..SAMPLES {
        let x_str = (0..(PL/8-1))
        .map(|_| {
            let idx = rng.gen_range(0..=255);
            idx
        }).collect::<Vec<_>>();
        let x = BigInt::from_bytes_le(Sign::Plus, &x_str);

        let start = Instant::now();
        let y = sloth(&x, &p, T);
        t1 += start.elapsed().as_secs_f32();

        let start = Instant::now();
        let z = sloth_inv(&y, &p, T);
        t2 += start.elapsed().as_secs_f32();

        assert_eq!(x, z);
    }

    t1 = t1 / (SAMPLES as f32);
    t2 = t2 / (SAMPLES as f32);    
    save_file.write_all(["p size, ", &PL.to_string(), ", round, ", &T.to_string(), ", samples, ", &SAMPLES.to_string(), ", sloth, ", &t1.to_string(), ", sloth inv, ", &t2.to_string(), ", rate, ", &(t1/t2).to_string(), "\n\n"].concat().as_bytes()).unwrap();
}
