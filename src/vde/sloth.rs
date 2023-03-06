use num_bigint::{BigUint, ToBigUint};

pub fn sloth(x: &BigUint, p: &BigUint) -> BigUint {
    let flag = x.modpow(&((p - 1.to_biguint().unwrap()) / 2.to_biguint().unwrap()), &p);
    let mut y;
    if flag == 1.to_biguint().unwrap() {
        y = x.modpow(&((p + 1.to_biguint().unwrap()) / 4.to_biguint().unwrap()), &p);
        if y.clone() % 2.to_biguint().unwrap() == 1.to_biguint().unwrap() {
            y = (p - y) % p;
        }
    }
    else {
        let x = (p - x) % p;
        y = x.modpow(&((p + 1.to_biguint().unwrap()) / 4.to_biguint().unwrap()), &p);
        if y.clone() % 2.to_biguint().unwrap() == 0.to_biguint().unwrap() {
            y = (p - y) % p;
        }
    }

    if y.clone() % 2.to_biguint().unwrap() == 1.to_biguint().unwrap() {
        return (y + 1.to_biguint().unwrap()) % p;
    }
    else {
        return (y - 1.to_biguint().unwrap()) % p;
    }
}
                   
pub fn sloth_inv(y: &BigUint, p: &BigUint) -> BigUint {
    let x;
    if y % 2.to_biguint().unwrap() == 1.to_biguint().unwrap() {
        x = (y + 1.to_biguint().unwrap()) % p;
    }
    else {
        x = (y - 1.to_biguint().unwrap()) % p;
    }

    if x.clone() % 2.to_biguint().unwrap() == 1.to_biguint().unwrap() {
        return (p - (x.pow(2) % p)) % p;
    }
    else {
        return x.pow(2) % p;
    }
}

#[test]
fn test_sloth() {
    use std::str::FromStr;
    
    let x = BigUint::from_str("15829769660807467965412494656491220299913966327750598489426198134983799276959616568370043796867960411137372925865504676446213722757732286176250162723041899748767180988576092837534839232300275294526335979669327528861132392730385116935290091070812723003423956538875994144423587866869984328679401647036689208226").unwrap();
    let p = BigUint::from_str("158297696608074679654124946564912202999139663277505984894261981349837992769596165683700437968679604111373729258655046764462137227577322861762501627230418997487671809885760928375348392323002752945263359796693275288611323927303851169352900910708127230034239565388759941444235878668699843286794016470366892082267").unwrap();
    let y = sloth(&x, &p);
    let z = sloth_inv(&y, &p);
    assert_eq!(x, z);
}

// use num_bigint::{BigInt, ToBigInt};

// pub fn sloth(x: &BigInt, p: &BigInt) -> BigInt {
//     let flag = x.modpow(&((p - 1.to_bigint().unwrap()) / 2.to_bigint().unwrap()), &p);
//     let mut y;
//     if flag == 1.to_bigint().unwrap() {
//         y = x.modpow(&((p + 1.to_bigint().unwrap()) / 4.to_bigint().unwrap()), &p);
//         if y.clone() % 2.to_bigint().unwrap() == 1.to_bigint().unwrap() {
//             y = (p - y) % p;
//         }
//     }
//     else {
//         let x = (p - x) % p;
//         y = x.modpow(&((p + 1.to_bigint().unwrap()) / 4.to_bigint().unwrap()), &p);
//         if y.clone() % 2.to_bigint().unwrap() == 0.to_bigint().unwrap() {
//             y = (p - y) % p;
//         }
//     }

//     if y.clone() % 2.to_bigint().unwrap() == 1.to_bigint().unwrap() {
//         return (y + 1.to_bigint().unwrap()) % p;
//     }
//     else {
//         return (y - 1.to_bigint().unwrap()) % p;
//     }
// }

// pub fn sloth_inv(y: &BigInt, p: &BigInt) -> BigInt {
//     let x;
//     if y % 2.to_bigint().unwrap() == 1.to_bigint().unwrap() {
//         x = (y + 1.to_bigint().unwrap()) % p;
//     }
//     else {
//         x = (y - 1.to_bigint().unwrap()) % p;
//     }

//     // println!("{:?}", x);
//     if x.clone() % 2.to_bigint().unwrap() == 0.to_bigint().unwrap() {
//         // println!("{:?}", (p - x.pow(2)) % p);
//         return x.pow(2) % p;
//     }
//     else {
//         return p - (x.pow(2) % p);
//     }
// }

// #[test]
// fn test_sloth() {
//     use std::str::FromStr;
    
//     let x = BigInt::from_str("10651882343749550841037844891129757296580376557000959074933115083942565070951529429273334642355394500665871657348910588903229690409866409087077223171328038243313190345431827485209520660018133114670476774403030737676988389514760120856118111726257419635876913384445746006681465910069131753735840456243734046070").unwrap();
//     let p = BigInt::from_str("106518823437495508410378448911297572965803765570009590749331150839425650709515294292733346423553945006658716573489105889032296904098664090870772231713280382433131903454318274852095206600181331146704767744030307376769883895147601208561181117262574196358769133844457460066814659100691317537358404562437340460703").unwrap();
//     let y = sloth(&x, &p);

//     let z = sloth_inv(&y, &p);
//     assert_eq!(x, z);
// }
