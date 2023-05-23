use std::time::Instant;

use fawkes_crypto_zkbob::{native::poseidon::{unoptimized, optimized}, engines::bn256::Fr};
use ff_uint::Num;

#[test]
fn test_optimized_poseidon() {
    let params: unoptimized::params::PoseidonParams<Fr> = serde_json::from_str(include_str!("./res/poseidon_params_t_3.json")).unwrap();
    let optimized_params = optimized::params::PoseidonParams::<Fr>::from_original_params(params.clone());

    let m = [Num::from(5), Num::from(3)];
    let hash = unoptimized::poseidon::poseidon(&m, &params);
    let optimized_hash = optimized::poseidon::poseidon(&m, &optimized_params);
    assert_eq!(hash, optimized_hash);
}

#[test]
fn test_optimized_poseidon_params_serialization() {
    let params_str = include_str!("./res/poseidon_params_t_3.json");
    let t = Instant::now();
    let params: unoptimized::params::PoseidonParams<Fr> = serde_json::from_str(params_str).unwrap();
    println!("Unoptimized: {:?}", t.elapsed());

    let t = Instant::now();
    let optimized_params: optimized::params::PoseidonParams<Fr> = serde_json::from_str(params_str).unwrap();
    println!("Optimized: {:?}", t.elapsed());

    let params_serialized = serde_json::to_string(&optimized_params).unwrap();
    assert_eq!(&params_str.replace(" ", ""), &params_serialized);

    let m = [Num::from(5), Num::from(3)];
    let hash = unoptimized::poseidon::poseidon(&m, &params);
    let optimized_hash = optimized::poseidon::poseidon(&m, &optimized_params);
    assert_eq!(hash, optimized_hash);
}

#[test]
fn test_optimized_poseidon_comp() {
    let params: unoptimized::params::PoseidonParams<Fr> = serde_json::from_str(include_str!("./res/poseidon_params_t_3.json")).unwrap();
    let optimized_params = optimized::params::PoseidonParams::<Fr>::from_original_params(params.clone());

    let m = [Num::from(2), Num::from(3)];
    let iter_count = 10000;
    let t = Instant::now();
    for _ in 0..iter_count {
        unoptimized::poseidon::poseidon(&m, &params);
    }
    println!("Unoptimized: {:?}", t.elapsed());

    let t = Instant::now();
    for _ in 0..iter_count {
        optimized::poseidon::poseidon(&m, &optimized_params);
    }
    println!("Optimized: {:?}", t.elapsed());
}