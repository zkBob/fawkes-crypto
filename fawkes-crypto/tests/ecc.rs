use fawkes_crypto_zkbob::{
    native::ecc::*,
    ff_uint::Num,
    engines::bn256::{Fr, JubJubBN256},
    rand::thread_rng,
};


#[test]
fn test_compress_decompress() {
    let mut rng = thread_rng();
    let params = JubJubBN256::new();

    let p = EdwardsPoint::<Fr>::rand(&mut rng, &params).mul(Num::from(8), &params);
    let p_bytes = p.compress(&params);
    let p_2 = EdwardsPoint::<Fr>::decompress_unchecked(p_bytes, &params).unwrap();
    assert_eq!(p, p_2);
    assert!(p_2.is_in_prime_subgroup(&params))
}


#[test]
fn test_decompress_subgroup_decompress() {
    let mut rng = thread_rng();
    let params = JubJubBN256::new();

    let p = EdwardsPoint::<Fr>::rand(&mut rng, &params).mul(Num::from(8), &params);

    let (p_1, is_in_prime_subgroup_1) = {
        let p_bytes = p.compress(&params);
        let p = EdwardsPoint::<Fr>::decompress_unchecked(p_bytes, &params).unwrap();
        (p, p.is_in_prime_subgroup(&params))
    };

    let p_2 = {
        EdwardsPoint::<Fr>::subgroup_decompress(p.x, &params)
    };
    assert!(!is_in_prime_subgroup_1 && p_2.is_none() || is_in_prime_subgroup_1 && p_1 == p_2.unwrap())
}