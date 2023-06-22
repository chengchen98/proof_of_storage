use ark_ff::Field;

pub fn legendre<F: Field>(mut x: F, p: F) -> F {
    let mut s = F::one();
    if x.clone() == F::zero() {
        return F::zero();
    }
    else if x.clone() == F::one() {
        return F::one();
    }
    else {
    }
}

// pub fn sloth<F: Field>(mut y: F, p: F, t: usize) {
//     for _ in 0..t {
//         let flag = legendre(y.clone(), p);
//         if flag == F::one() {

//         }
//     }
// }