use ff_uint::{PrimeField, Num};

use crate::native::fast_poseidon::{mds::{derive_mds_matrices}, constants::calc_equivalent_constants};
#[cfg(feature = "serde_support")]
use crate::serde::{Deserialize, Serialize};

use super::{mds::{MdsMatrices}};
use crate::native::poseidon::PoseidonParams as OriginalPoseidonParams;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde_support", serde(bound(serialize = "", deserialize = "")))]
pub struct PoseidonParams<Fr: PrimeField> {
    pub mds_matrices: MdsMatrices<Fr>,
    pub round_constants: Vec<Vec<Num<Fr>>>,
    pub f: usize,
    pub p: usize,
    pub t: usize,
}

impl<Fr: PrimeField> PoseidonParams<Fr> {
    pub fn from_original_params(params: OriginalPoseidonParams<Fr>) -> PoseidonParams<Fr> {
        let mds = params.m;
        let (t, f, p) = (params.t, params.f, params.p);
        
        let mds_matrices = derive_mds_matrices(mds, p, t);
        let round_constants = calc_equivalent_constants(params.c, mds_matrices.clone(), f, p, t);

        PoseidonParams { 
            mds_matrices,
            round_constants,
            f,
            p,
            t,
        }
    }
}