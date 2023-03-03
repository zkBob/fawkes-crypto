use ff_uint::{
    seedbox::{SeedBox, SeedBoxGen, SeedboxChaCha20},
    Num, PrimeField,
};

#[cfg(feature = "serde_support")]
use crate::serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde_support",
    serde(bound(serialize = "", deserialize = ""))
)]
pub struct PoseidonParams<Fr: PrimeField> {
    pub c: Vec<Vec<Num<Fr>>>,
    pub m: Vec<Vec<Num<Fr>>>,
    pub t: usize,
    pub f: usize,
    pub p: usize,
}

impl<Fr: PrimeField> PoseidonParams<Fr> {
    pub fn new(t: usize, f: usize, p: usize) -> Self {
        Self::new_with_salt(t, f, p, "")
    }

    // All generated parameters should be additionally checked according to reference implementation
    // https://extgit.iaik.tugraz.at/krypto/hadeshash/-/blob/659de89cd207e19b92852458dce92adf83ad7cf7/code/generate_parameters_grain.sage#L167-285
    //
    pub fn new_with_salt(t: usize, f: usize, p: usize, salt: &str) -> Self {
        fn m<Fr: PrimeField>(n: usize, seedbox: &mut SeedboxChaCha20) -> Vec<Vec<Num<Fr>>> {
            let x = (0..n).map(|_| seedbox.gen()).collect::<Vec<_>>();
            let y = (0..n).map(|_| seedbox.gen()).collect::<Vec<_>>();
            (0..n)
                .map(|i| (0..n).map(|j| Num::ONE / (x[i] + y[j])).collect())
                .collect()
        }

        let mut seedbox = SeedboxChaCha20::new_with_salt(
            format!("fawkes_poseidon(t={},f={},p={},salt={})", t, f, p, salt).as_bytes(),
        );

        let c = (0..f + p)
            .map(|_| (0..t).map(|_| seedbox.gen()).collect())
            .collect();
        let m = m(t, &mut seedbox);
        PoseidonParams { c, m, t, f, p }
    }
}
