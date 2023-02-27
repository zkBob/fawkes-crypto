use std::time::Instant;

use fawkes_crypto_zkbob::{native::{fast_poseidon::{params::PoseidonParams as FastPoseidonParams, self}, poseidon::{PoseidonParams, poseidon}}, engines::bn256::Fr};
use ff_uint::Num;

#[test]
fn test_fast_poseidon() {
    let params: PoseidonParams<Fr> = serde_json::from_str(include_str!("./res/poseidon_params_t_6.json")).unwrap();
    let fast_params = FastPoseidonParams::<Fr>::from_original_params(params.clone());

    let m = [Num::from(5), Num::from(3)];
    let hash = poseidon(&m, &params);

    let fast_hash = fast_poseidon::poseidon::poseidon(&m, &fast_params);

    println!("{:?}", hash);
    println!("{:?}", fast_hash);
    assert_eq!(hash, fast_hash);
}

#[test]
fn test_fast_poseidon_comp() {
    let params: PoseidonParams<Fr> = serde_json::from_str(include_str!("./res/poseidon_params_t_3.json")).unwrap();
    let fast_params = FastPoseidonParams::<Fr>::from_original_params(params.clone());

    let m = [Num::from(2), Num::from(3)];
    let iter_count = 10000;
    let t = Instant::now();
    for _ in 0..iter_count {
        poseidon(&m, &params);
    }
    println!("Original: {:?}", t.elapsed());

    let t = Instant::now();
    for _ in 0..iter_count {
        fast_poseidon::poseidon::poseidon(&m, &fast_params);
    }
    println!("Fast: {:?}", t.elapsed());
}


#[test]
fn test_fast_poseidon_bench() {
    let salt = "asdfasdf";
    let (t, f, p) = (3, 8, 56);
    let params = PoseidonParams::<Fr>::new_with_salt(t, f, p, salt);
    let fast_params = FastPoseidonParams::<Fr>::from_original_params(params.clone());

    let m = [Num::from(2), Num::from(3)];
    let iter_count = 100000;
    for _ in 0..iter_count {
        fast_poseidon::poseidon::poseidon(&m, &fast_params);
    }
}