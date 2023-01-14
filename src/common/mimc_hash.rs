use ark_ff::Field;

const MIMC_HASH_ROUNDS: usize = 10;

pub fn mimc7_hash<F: Field>(x_in: F, key: F, constants: &[F]) -> F {
    let mut h: F = F::zero();
    for i in 0..MIMC_HASH_ROUNDS {
        let mut t: F;
        if i == 0 {
            t = x_in.clone();
            t.add_assign(key);
        } else {
            t = h.clone(); 
            t.add_assign(&key);
            t.add_assign(&constants[i]);
        }
        let mut t2 = t.clone();
        t2.square_in_place();
        let mut t7 = t2.clone();
        t7.square_in_place();
        t7.mul_assign(&t2);
        t7.mul_assign(&t);
        h = t7.clone();
    }
    h.add_assign(&key);
    h
}

pub fn multi_mimc7_hash<F: Field>(x_inputs: &Vec<F>, key: F, constants: &[F]) -> F {
    let mut r = key.clone();
    for i in 0..x_inputs.len() {
        let h = mimc7_hash(x_inputs[i], r, constants);
        r.add_assign(&x_inputs[i]);
        r.add_assign(&h);
    }
    r
}

#[test]
fn test_mimc7_hash() {
    use ark_bls12_381::Fr;
    use ark_std::rand::Rng;
    use ark_std::test_rng;

    let rng = &mut test_rng();
    let x_in: Fr = rng.gen();
    let key: Fr = rng.gen();
    let constants = (0..MIMC_HASH_ROUNDS).map(|_| rng.gen()).collect::<Vec<_>>();
    let res = mimc7_hash(x_in, key, &constants);
    println!("{:?}", res);
}
#[test]
fn test_multi_mimc7_hash() {
    use ark_bls12_381::Fr;
    use ark_std::rand::Rng;
    use ark_std::test_rng;

    let rng = &mut test_rng();
    let x_inputs = (0..3).map(|_| rng.gen()).collect::<Vec<Fr>>();
    let key = rng.gen();
    let constants = (0..MIMC_HASH_ROUNDS).map(|_| rng.gen()).collect::<Vec<_>>();
    let res = multi_mimc7_hash(&x_inputs, key, &constants);
    println!("{:?}", res);
}