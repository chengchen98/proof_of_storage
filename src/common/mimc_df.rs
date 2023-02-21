use ark_ff::Field;

const MIMC_DF_ROUNDS: usize = 322;

/// This is an implementation of MiMC, specifically a
/// variant named `LongsightF322p3` for BLS12-381.
/// See http://eprint.iacr.org/2016/492 for more
/// information about this construction.
///
/// ```
/// function LongsightF322p3(xL ⦂ Fp, xR ⦂ Fp) {
///     for i from 0 up to 321 {
///         xL, xR := xR + (xL + Ci)^3, xL
///     }
///     return xL
/// }
/// ```
pub fn mimc3_df<F: Field>(mut xl: F, mut xr: F, constants: &[F]) -> F {
    assert_eq!(constants.len(), MIMC_DF_ROUNDS);

    for i in 0..MIMC_DF_ROUNDS {
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
/// function LongsightF322p5(xL ⦂ Fp, xR ⦂ Fp) {
///     for i from 0 up to 321 {
///         xL, xR := xR + (xL + Ci)^5, xL
///     }
///     return xL
/// }
/// ```
pub fn mimc5_df<F: Field>(mut xl: F, mut xr: F, constants: &[F]) -> F {
    assert_eq!(constants.len(), MIMC_DF_ROUNDS);

    for i in 0..MIMC_DF_ROUNDS {
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

    let rng = &mut test_rng();
    let xl = rng.gen();
    let xr = rng.gen();
    let constants: Vec<Fr> = (0..MIMC_DF_ROUNDS)
    .map(|_| rng.gen())
    .collect::<Vec<_>>();

    let start = Instant::now();
    let _ = mimc5_df(xl, xr, &constants);
    println!("time: {:?}", start.elapsed());
}