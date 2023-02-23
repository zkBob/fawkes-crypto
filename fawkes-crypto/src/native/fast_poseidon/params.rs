use ff_uint::{PrimeField, seedbox::{SeedboxChaCha20, SeedBoxGen, SeedBox}, Num};

use crate::native::fast_poseidon::{mds::{derive_mds_matrices, factor_to_sparse_matrixes}, preprocessing::compress_round_constants, matrix::{transpose, is_invertible}};
#[cfg(feature = "serde_support")]
use crate::serde::{Deserialize, Serialize};

use super::{mds::{MdsMatrices, SparseMatrix}, matrix::Matrix};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde_support", serde(bound(serialize = "", deserialize = "")))]
pub struct PoseidonParams<Fr: PrimeField> {
    pub mds_matrices: MdsMatrices<Fr>,
    pub round_constants: Option<Vec<Num<Fr>>>,
    pub compressed_round_constants: Vec<Num<Fr>>,
    pub pre_sparse_matrix: Matrix<Num<Fr>>,
    pub sparse_matrices: Vec<SparseMatrix<Fr>>,
    pub f: usize,
    pub p: usize,
    pub t: usize,
}

impl<Fr: PrimeField> PoseidonParams<Fr> {
    pub fn new(t: usize, f: usize, p: usize) -> Self {
        Self::new_with_salt(t, f, p, "")
    }

    // All generated parameters should be additionally checked according to reference implementation
    // https://extgit.iaik.tugraz.at/krypto/hadeshash/-/blob/659de89cd207e19b92852458dce92adf83ad7cf7/code/generate_parameters_grain.sage#L167-285
    //
    pub fn new_with_salt(t: usize, f: usize, p: usize, salt:&str) -> Self {

        fn m<Fr: PrimeField>(n: usize, seedbox: &mut SeedboxChaCha20) -> Vec<Vec<Num<Fr>>> {
            // TODO: I modified this part to make matrix simmetric
            let x = (0..n).map(|i| Num::from(i as u64)).collect::<Vec<Num<Fr>>>();
            let y = (n..2*n).map(|i| Num::from(i as u64)).collect::<Vec<Num<Fr>>>();
            (0..n).map(|i| (0..n).map(|j| Num::ONE/(x[i] + y[j]) ).collect()).collect()
        }

        let mut seedbox = SeedboxChaCha20::new_with_salt(
            format!("fawkes_poseidon(t={},f={},p={},salt={})", t, f, p, salt).as_bytes(),
        );

        let round_constants = (0..f + p)
            .map(|_| (0..t).map(|_| seedbox.gen()).collect::<Vec<Num<Fr>>>())
            .flatten() // TODO: check it
            .collect();
        let mds: Vec<Vec<Num<Fr>>> = m(t, &mut seedbox);

        // To ensure correctness, we would check all sub-matrices for invertibility. Meanwhile, this is a simple sanity check.
        assert!(is_invertible(&mds));

        //  `poseidon::product_mds_with_matrix` relies on the constructed MDS matrix being symmetric, so ensure it is.
        assert_eq!(mds, transpose(&mds));
        
        let mds_matrices = derive_mds_matrices(mds);

        let compressed_round_constants = compress_round_constants(
            t,
            f,
            p,
            &round_constants,
            &mds_matrices,
            p,
        );

        let (pre_sparse_matrix, sparse_matrices) =
            factor_to_sparse_matrixes(mds_matrices.m.clone(), p);

        // Ensure we have enough constants for the sbox rounds
        assert!(
            t * (f + p) <= round_constants.len(),
            "Not enough round constants"
        );

        assert_eq!(
            f * t + p,
            compressed_round_constants.len()
        );


        PoseidonParams { 
            mds_matrices,
            round_constants: Some(round_constants),
            compressed_round_constants,
            pre_sparse_matrix,
            sparse_matrices,
            f,
            p,
            t,
        }
    }
}