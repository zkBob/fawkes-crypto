use std::time::Instant;

use fawkes_crypto_zkbob::{native::poseidon::{unoptimized, optimized}, engines::bn256::Fr};
use ff_uint::Num;

#[test]
fn test_fast_poseidon() {
    let params: unoptimized::params::PoseidonParams<Fr> = serde_json::from_str(include_str!("./res/poseidon_params_t_3.json")).unwrap();
    let fast_params = optimized::params::PoseidonParams::<Fr>::from_original_params(params.clone());

    let m = [Num::from(5), Num::from(3)];
    let hash = unoptimized::poseidon::poseidon(&m, &params);
    let fast_hash = optimized::poseidon::poseidon(&m, &fast_params);
    assert_eq!(hash, fast_hash);
}

#[test]
fn test_fast_poseidon_params_serialization() {
    let t = Instant::now();
    let params: unoptimized::params::PoseidonParams<Fr> = serde_json::from_str(include_str!("./res/poseidon_params_t_3.json")).unwrap();
    println!("Original: {:?}", t.elapsed());

    let t = Instant::now();
    let fast_params: optimized::params::PoseidonParams<Fr> = serde_json::from_str(include_str!("./res/poseidon_params_t_3.json")).unwrap();
    println!("Fast: {:?}", t.elapsed());

    let m = [Num::from(5), Num::from(3)];
    let hash = unoptimized::poseidon::poseidon(&m, &params);
    let fast_hash = optimized::poseidon::poseidon(&m, &fast_params);
    assert_eq!(hash, fast_hash);
}

#[test]
fn test_fast_poseidon_comp() {
    let params: unoptimized::params::PoseidonParams<Fr> = serde_json::from_str(include_str!("./res/poseidon_params_t_3.json")).unwrap();
    let fast_params = optimized::params::PoseidonParams::<Fr>::from_original_params(params.clone());

    let m = [Num::from(2), Num::from(3)];
    let iter_count = 10000;
    let t = Instant::now();
    for _ in 0..iter_count {
        unoptimized::poseidon::poseidon(&m, &params);
    }
    println!("Original: {:?}", t.elapsed());

    let t = Instant::now();
    for _ in 0..iter_count {
        optimized::poseidon::poseidon(&m, &fast_params);
    }
    println!("Fast: {:?}", t.elapsed());
}