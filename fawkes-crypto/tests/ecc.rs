use fawkes_crypto_zkbob::{
    native::ecc::*,
    ff_uint::Num,
    engines::{bn256::JubJubBN256, bls12_381::JubJubBLS12_381},
    rand::thread_rng,
};


#[test]
fn test_compress_decompress_bn256() {
    let mut rng = thread_rng();
    let params = JubJubBN256::new();

    let p = EdwardsPoint::rand(&mut rng, &params).mul(Num::from(8), &params);
    let p_bytes = p.compress(&params);
    let p_2 = EdwardsPoint::decompress_unchecked(p_bytes, &params).unwrap();
    assert_eq!(p, p_2);
    assert!(p_2.is_in_prime_subgroup(&params))
}

#[test]
fn test_compress_decompress_bls12_381() {
    let mut rng = thread_rng();
    let params = JubJubBLS12_381::new();

    let p = EdwardsPoint::rand(&mut rng, &params).mul(Num::from(8), &params);
    let p_bytes = p.compress(&params);
    let p_2 = EdwardsPoint::decompress_unchecked(p_bytes, &params).unwrap();
    assert_eq!(p, p_2);
    assert!(p_2.is_in_prime_subgroup(&params))
}


#[test]
fn test_decompress_subgroup_decompress_bn256() {
    let mut rng = thread_rng();
    let params = JubJubBN256::new();

    let p = EdwardsPoint::rand(&mut rng, &params).mul(Num::from(8), &params);

    let (p_1, is_in_prime_subgroup_1) = {
        let p_bytes = p.compress(&params);
        let p = EdwardsPoint::decompress_unchecked(p_bytes, &params).unwrap();
        (p, p.is_in_prime_subgroup(&params))
    };

    let p_2 = {
        EdwardsPoint::subgroup_decompress(p.x, &params)
    };
    assert!(!is_in_prime_subgroup_1 && p_2.is_none() || is_in_prime_subgroup_1 && p_1 == p_2.unwrap())
}

#[test]
fn test_decompress_subgroup_decompress_bls12_381() {
    let mut rng = thread_rng();
    let params = JubJubBLS12_381::new();

    let p = EdwardsPoint::rand(&mut rng, &params).mul(Num::from(8), &params);

    let (p_1, is_in_prime_subgroup_1) = {
        let p_bytes = p.compress(&params);
        let p = EdwardsPoint::decompress_unchecked(p_bytes, &params).unwrap();
        (p, p.is_in_prime_subgroup(&params))
    };

    let p_2 = {
        EdwardsPoint::subgroup_decompress(p.x, &params)
    };
    assert!(!is_in_prime_subgroup_1 && p_2.is_none() || is_in_prime_subgroup_1 && p_1 == p_2.unwrap())
}