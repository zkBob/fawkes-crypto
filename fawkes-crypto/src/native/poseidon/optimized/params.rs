use std::convert::TryFrom;

use ff_uint::{Num, PrimeField};

#[cfg(feature = "serde_support")]
use crate::serde::{Deserialize, Serialize};

use super::{
    constants::calc_equivalent_constants,
    mds::{calc_equivalent_matrices, MdsMatrices},
};
use crate::native::poseidon::unoptimized::params::PoseidonParams as OriginalPoseidonParams;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde_support",
    serde(bound(serialize = "", deserialize = ""))
)]
#[cfg_attr(
    feature = "serde_support",
    serde(try_from = "OriginalPoseidonParams<Fr>")
)]
pub struct PoseidonParams<Fr: PrimeField> {
    pub c: Vec<Vec<Num<Fr>>>,
    pub m: Vec<Vec<Num<Fr>>>,
    pub t: usize,
    pub f: usize,
    pub p: usize,

    #[cfg_attr(feature = "serde_support", serde(skip_serializing))]
    pub mds_matrices: MdsMatrices<Fr>,
    #[cfg_attr(feature = "serde_support", serde(skip_serializing))]
    pub round_constants: Vec<Vec<Num<Fr>>>,
}

impl<Fr: PrimeField> PoseidonParams<Fr> {
    pub fn new(t: usize, f: usize, p: usize) -> Self {
        Self::from_original_params(OriginalPoseidonParams::new(t, f, p))
    }

    pub fn new_with_salt(t: usize, f: usize, p: usize, salt: &str) -> Self {
        Self::from_original_params(OriginalPoseidonParams::new_with_salt(t, f, p, salt))
    }

    pub fn from_original_params(params: OriginalPoseidonParams<Fr>) -> PoseidonParams<Fr> {
        let m = &params.m;
        let (f, p, t) = (params.f, params.p, params.t);

        let mds_matrices = calc_equivalent_matrices(m, p, t);
        let round_constants = calc_equivalent_constants(&params.c, m, f, p, t);

        PoseidonParams {
            c: params.c,
            m: params.m,
            t,
            f,
            p,
            mds_matrices,
            round_constants,
        }
    }
}

impl<Fr: PrimeField> TryFrom<OriginalPoseidonParams<Fr>> for PoseidonParams<Fr> {
    type Error = &'static str;

    fn try_from(params: OriginalPoseidonParams<Fr>) -> Result<Self, Self::Error> {
        Ok(PoseidonParams::from_original_params(params))
    }
}
