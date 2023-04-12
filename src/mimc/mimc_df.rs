use ark_ff::Field;

pub const MIMC3_DF_ROUNDS: usize = 322;
pub const MIMC5_DF_ROUNDS: usize = 10;
pub const DATA_DIR: [&str; 4] = [r"src", "mimc", "data", "df"];


pub fn mimc3_df<F: Field>(mut xl: F, mut xr: F, constants: &[F]) -> F {
    assert_eq!(constants.len(), MIMC3_DF_ROUNDS);

    for i in 0..MIMC3_DF_ROUNDS {
        let mut tmp1 = xl;
        tmp1.add_assign(&constants[i]);
        let mut tmp2 = tmp1;
        tmp2.square_in_place();
        tmp2.mul_assign(&tmp1);
        tmp2.add_assign(&xr);
        xr = xl;
        xl = tmp2;
    }

    xl
}

pub fn mimc5_df<F: Field>(mut xl: F, mut xr: F, constants: &[F]) -> F {
    assert_eq!(constants.len(), MIMC5_DF_ROUNDS);

    for i in 0..MIMC5_DF_ROUNDS {
        let mut tmp1 = xl;
        tmp1.add_assign(&constants[i]);
        let mut tmp2 = tmp1.clone();
        tmp2.square_in_place();
        let mut tmp5 = tmp2.clone();
        tmp5.square_in_place();
        tmp5.mul_assign(&tmp1);
        tmp5.add_assign(&xr);
        xr = xl;
        xl = tmp5;
    }

    xl
}

#[test]
fn test_mimc5_df() {
    use std::time::Instant;
    use ark_bls12_381::Fr;
    use ark_std::{rand::Rng, test_rng};
    use std::{path::PathBuf, fs::OpenOptions, io::Write};

    let rng = &mut test_rng();
    let xl = rng.gen();
    let xr = rng.gen();
    let constants: Vec<Fr> = (0..MIMC5_DF_ROUNDS)
    .map(|_| rng.gen())
    .collect::<Vec<_>>();

    let path: PathBuf = DATA_DIR.iter().collect();
    let save_path = path.to_str().unwrap();
    let mut save_file = OpenOptions::new()
    .read(true)
    .write(true)
    .append(true)
    .create(true) 
    .open(save_path)
    .unwrap();
    const SAMPLES: usize = 10000;

    let mut t = 0.0;
    for _ in 0..SAMPLES {
        // println!("sample: {:?}  | rounds: {:?}", i, MIMC5_DF_ROUNDS);

        let start = Instant::now();
        let _ = mimc5_df(xl, xr, &constants);
        t += start.elapsed().as_secs_f32();
        // println!("mimc_df: {:?}", start.elapsed());
        // println!("-------------------------------------");
    }

    t = t / (SAMPLES as f32);
    save_file.write_all(["round, ", &MIMC5_DF_ROUNDS.to_string(), ", samples, ", &SAMPLES.to_string(), ", average time, ", &t.to_string(), "\n\n"].concat().as_bytes()).unwrap(); 
}