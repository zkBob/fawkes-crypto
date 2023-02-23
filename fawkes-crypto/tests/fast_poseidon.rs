use std::time::Instant;

use fawkes_crypto_zkbob::{native::{fast_poseidon::{params::PoseidonParams as FastPoseidonParams, self}, poseidon::{PoseidonParams, poseidon}}, engines::bn256::Fr};
use ff_uint::Num;

#[test]
fn test_fast_poseidon() {
    let salt = "asdfasdf";
    let (t, f, p) = (3, 8, 56);
    let params = PoseidonParams::<Fr>::new_with_salt(t, f, p, salt);
    let fast_params = FastPoseidonParams::<Fr>::new_with_salt(t, f, p, salt);
    
    let m = [Num::from(2), Num::from(3)];
    let hash = poseidon(&m, &params);

    let fast_hash = fast_poseidon::poseidon::poseidon(&m, &fast_params);

    println!("{:?}", hash);
    println!("{:?}", fast_hash);
    assert_eq!(hash, fast_hash);
}

#[test]
fn test_fast_poseidon_bench() {
    let salt = "asdfasdf";
    let (t, f, p) = (3, 8, 56);
    let params = PoseidonParams::<Fr>::new_with_salt(t, f, p, salt);
    let fast_params = FastPoseidonParams::<Fr>::new_with_salt(t, f, p, salt);

    let m = [Num::from(2), Num::from(3)];
    let iter_count = 10000;
    let t = Instant::now();
    for _ in 0..iter_count {
        let hash = poseidon(&m, &params);
    }
    println!("Original: {:?}", t.elapsed());

    let t = Instant::now();
    for _ in 0..iter_count {
        let hash = fast_poseidon::poseidon::poseidon(&m, &fast_params);
    }
    println!("Fast: {:?}", t.elapsed());
}
