use fawkes_crypto_zkbob::{native::{fast_poseidon::{params::PoseidonParams as FastPoseidonParams, self}, poseidon::{PoseidonParams, poseidon}}, engines::bn256::Fr};
use ff_uint::Num;

#[test]
fn test_fast_poseidon() {
    let salt = "salt";
    let (t, f, p) = (3, 8, 56);
    let params = PoseidonParams::<Fr>::new_with_salt(t, f, p, salt);
    let fast_params = FastPoseidonParams::<Fr>::new_with_salt(t, f, p, salt);
    
    let m = [Num::from(2), Num::from(3)];
    let hash = poseidon(&m, &params);

    let fast_hash = fast_poseidon::poseidon::poseidon(&m, &fast_params);

    println!("{:?}", hash);
    println!("{:?}", fast_hash);
}