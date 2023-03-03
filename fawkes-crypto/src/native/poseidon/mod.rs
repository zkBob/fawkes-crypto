use ff_uint::{PrimeField, Num};
use itertools::Itertools;

use crate::core::sizedvec::SizedVec;
#[cfg(feature = "serde_support")]
use crate::serde::{Deserialize, Serialize};

pub mod optimized;
pub mod unoptimized;

pub type PoseidonParams<Fr> = self::optimized::params::PoseidonParams<Fr>;

pub fn poseidon<Fr: PrimeField>(inputs: &[Num<Fr>], params: &PoseidonParams<Fr>) -> Num<Fr> {
    self::optimized::poseidon::poseidon(inputs, params)
}

pub fn perm<Fr: PrimeField>(state: &mut [Num<Fr>], params: &PoseidonParams<Fr>) {
    self::optimized::poseidon::perm(state, params)
}

pub fn poseidon_sponge<Fr: PrimeField>(inputs: &[Num<Fr>], params: &PoseidonParams<Fr>) -> Num<Fr> {
    let mut state = vec![Num::ZERO; params.t];
    let size = Num::from(inputs.len() as u64);
    core::iter::once(&size).chain(inputs.iter()).chunks(params.t-1).into_iter().for_each(|c| {
        state.iter_mut().zip(c.into_iter()).for_each(|(l, r)| *l+=*r);
        perm(&mut state, params);
    });
    state[0]
}


#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde_support", serde(bound(serialize = "", deserialize = "")))]
pub struct MerkleProof<Fr: PrimeField, const L: usize> {
    pub sibling: SizedVec<Num<Fr>, L>,
    pub path: SizedVec<bool, L>,
}

pub fn poseidon_merkle_proof_root<Fr: PrimeField, const L: usize>(
    leaf: Num<Fr>,
    proof: &MerkleProof<Fr, L>,
    params: &PoseidonParams<Fr>,
) -> Num<Fr> {
    let mut root = leaf.clone();
    for (&p, &s) in proof.path.iter().zip(proof.sibling.iter()) {
        let pair = if p { [s, root] } else { [root, s] };
        root = poseidon(pair.as_ref(), params);
    }
    root
}

pub fn poseidon_merkle_tree_root<Fr: PrimeField>(
    leaf: &[Num<Fr>],
    params: &PoseidonParams<Fr>,
) -> Num<Fr> {
    let leaf_sz = leaf.len();
    assert!(leaf_sz > 0, "should be at least one leaf in the tree");
    let proof_sz = std::mem::size_of::<usize>() * 8 - (leaf_sz - 1).leading_zeros() as usize;
    let total_leaf_sz = 1usize << proof_sz;
    let mut state = leaf.to_vec();
    state.extend_from_slice(&vec![Num::ZERO; total_leaf_sz - leaf_sz]);
    for j in 0..proof_sz {
        for i in 0..total_leaf_sz >> (j + 1) {
            state[i] = poseidon(&[state[2 * i].clone(), state[2 * i + 1].clone()], params);
        }
    }
    state[0].clone()
}