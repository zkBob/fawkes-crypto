use ff_uint::{PrimeField, Num};

use crate::constants::PREALLOC_SIZE;

use super::{params::PoseidonParams, utils::quintic_s_box, matrix::Matrix, mds::SparseMatrix};

pub fn poseidon<Fr: PrimeField>(inputs: &[Num<Fr>], params: &PoseidonParams<Fr>) -> Num<Fr> {
    let n_inputs = inputs.len();
    assert!(
        n_inputs < params.t,
        "number of inputs should be less or equal than t"
    );
    assert!(n_inputs > 0, "number of inputs should be positive nonzero");

    // Don't allocate memory in heap if statelen is less or equal to PREALLOC_SIZE
    let mut arr: [Num<Fr>; PREALLOC_SIZE] = [Num::ZERO; PREALLOC_SIZE];
    let mut vec: Vec<Num<Fr>> = Vec::new();
    let state = match params.t {
        size if size <= PREALLOC_SIZE => &mut arr[..size],
        size => {
            vec.resize(size, Num::ZERO);
            &mut vec[..]
        }
    };
    
    (0..n_inputs).for_each(|i| state[i] = inputs[i]);

    perm(state, params);
    println!("{:?}", state);
    state[0]
}

fn perm<Fr: PrimeField>(state: &mut [Num<Fr>], params: &PoseidonParams<Fr>){
    assert!(state.len() == params.t);
    let half_f = params.f >> 1;
    let mut constants_offset = 0;
    let mut round = 0;

    // The first full round should use the initial constants.
    add_round_constants(state, &params, &mut constants_offset);

    for _ in 0..half_f {
        full_round(state, params, &mut constants_offset, &mut round, false);
    }

    for _ in 0..params.p {
        partial_round(state, params, &mut constants_offset, &mut round);
    }

    // All but last full round.
    for _ in 1..half_f {
        full_round(state, params, &mut constants_offset, &mut round, false);
    }
    full_round(state, params, &mut constants_offset, &mut round, true);

    assert_eq!(
        constants_offset,
        params.compressed_round_constants.len(),
        "Constants consumed ({}) must equal preprocessed constants provided ({}).",
        constants_offset,
        params.compressed_round_constants.len()
    );
}

fn add_round_constants<Fr: PrimeField>(state: &mut [Num<Fr>], params: &PoseidonParams<Fr>, constants_offset: &mut usize) {
    for (element, round_constant) in state.iter_mut().zip(
        params.compressed_round_constants
            .iter()
            .skip(*constants_offset),
    ) {
        *element += round_constant;
    }
    *constants_offset += state.len();
}

fn full_round<Fr: PrimeField>(state: &mut [Num<Fr>], params: &PoseidonParams<Fr>, constants_offset: &mut usize, round: &mut usize, last_round: bool) {
    let to_take = state.len();
    let post_round_keys = params
        .compressed_round_constants
        .iter()
        .skip(*constants_offset)
        .take(to_take);

    if !last_round {
        let needed = *constants_offset + to_take;
        assert!(
            needed <= params.compressed_round_constants.len(),
            "Not enough preprocessed round constants ({}), need {}.",
            params.compressed_round_constants.len(),
            needed
        );
    }
    state
        .iter_mut()
        .zip(post_round_keys)
        .for_each(|(l, post)| {
            // Be explicit that no round key is added after last round of S-boxes.
            let post_key = if last_round {
                panic!(
                    "Trying to skip last full round, but there is a key here! ({:?})",
                    post
                );
            } else {
                Some(post)
            };
            quintic_s_box(l, None, post_key);
        });
    // We need this because post_round_keys will have been empty, so it didn't happen in the for_each. :(
    if last_round {
        state
            .iter_mut()
            .for_each(|l| quintic_s_box(l, None, None));
    } else {
        *constants_offset += state.len();
    }
    round_product_mds(state, params, round);
}

/// The partial round is the same as the full round, with the difference that we apply the S-Box only to the first (arity tag) poseidon leaf.
fn partial_round<Fr: PrimeField>(state: &mut [Num<Fr>], params: &PoseidonParams<Fr>, constants_offset: &mut usize, round: &mut usize) {
    let post_round_key = params.compressed_round_constants[*constants_offset];

    // Apply the quintic S-Box to the first element
    quintic_s_box(&mut state[0], None, Some(&post_round_key));
    *constants_offset += 1;

    round_product_mds(state, params, round);
}

/// Set the provided elements with the result of the product between the elements and the appropriate
/// MDS matrix.
#[allow(clippy::collapsible_else_if)]
fn round_product_mds<Fr: PrimeField>(state: &mut [Num<Fr>], params: &PoseidonParams<Fr>, round: &mut usize) {
    let full_half = params.f >> 1;
    let sparse_offset = full_half - 1;
    if *round == sparse_offset {
        product_mds_with_matrix(state, &params.pre_sparse_matrix);
    } else {
        if (*round > sparse_offset)
            && (*round < full_half + params.p)
        {
            let index = *round - sparse_offset - 1;
            let sparse_matrix = &params.sparse_matrices[index];

            product_mds_with_sparse_matrix(state, sparse_matrix);
        } else {
            product_mds(state, params);
        }
    };

    *round += 1;
}

/// NOTE: This calculates a vector-matrix product (`elements * matrix`) rather than the
/// expected matrix-vector `(matrix * elements)`. This is a performance optimization which
/// exploits the fact that our MDS matrices are symmetric by construction.
#[allow(clippy::ptr_arg)]
pub(crate) fn product_mds_with_matrix<Fr: PrimeField>(state: &mut [Num<Fr>], matrix: &Matrix<Num<Fr>>) {
    // TODO: maybe replace current hack with it
    //let mut result = GenericArray::<F, A::ConstantsSize>::generate(|_| F::zero());

    // Don't allocate memory in heap if statelen is less or equal to PREALLOC_SIZE
    let mut arr: [Num<Fr>; PREALLOC_SIZE] = [Num::ZERO; PREALLOC_SIZE];
    let mut vec: Vec<Num<Fr>> = Vec::new();
    let new_state = match state.len() {
        size if size <= PREALLOC_SIZE => &mut arr[..size],
        size => {
            vec.resize(size, Num::ZERO);
            &mut vec[..]
        }
    };

    for (j, val) in new_state.iter_mut().enumerate() {
        for (i, row) in matrix.iter().enumerate() {
            let mut tmp = row[j];
            tmp *= state[i];
            *val += tmp;
        }
    }

    (0..state.len()).for_each(|i| state[i] = new_state[i]);
}

// Sparse matrix in this context means one of the form, M''.
fn product_mds_with_sparse_matrix<Fr: PrimeField>(state: &mut [Num<Fr>], sparse_matrix: &SparseMatrix<Fr>) {
    // TODO: maybe replace current hack with it
    //let mut result = GenericArray::<F, A::ConstantsSize>::generate(|_| F::zero());

    // Don't allocate memory in heap if statelen is less or equal to PREALLOC_SIZE
    let mut arr: [Num<Fr>; PREALLOC_SIZE] = [Num::ZERO; PREALLOC_SIZE];
    let mut vec: Vec<Num<Fr>> = Vec::new();
    let new_state = match state.len() {
        size if size <= PREALLOC_SIZE => &mut arr[..size],
        size => {
            vec.resize(size, Num::ZERO);
            &mut vec[..]
        }
    };

    // First column is dense.
    for (i, val) in sparse_matrix.w_hat.iter().enumerate() {
        let mut tmp = *val;
        tmp *= state[i];
        new_state[0] += &tmp;
    }

    for (j, val) in new_state.iter_mut().enumerate().skip(1) {
        // Except for first row/column, diagonals are one.
        *val += state[j];

        // First row is dense.
        let mut tmp = sparse_matrix.v_rest[j - 1];
        tmp *= state[0];
        *val += tmp;
    }

    (0..state.len()).for_each(|i| state[i] = new_state[i]);
}

/// Set the provided elements with the result of the product between the elements and the constant
/// MDS matrix.
pub(crate) fn product_mds<Fr: PrimeField>(state: &mut [Num<Fr>], params: &PoseidonParams<Fr>) {
    product_mds_with_matrix(state, &params.mds_matrices.m);
}