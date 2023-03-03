use ff_uint::{Num, PrimeField};

use crate::constants::PREALLOC_SIZE;

use super::params::PoseidonParams;

fn ark<Fr: PrimeField>(state: &mut [Num<Fr>], c: &[Num<Fr>]) {
    state.iter_mut().zip(c.iter()).for_each(|(s, c)| *s += c)
}

// assuming (r - 1) % 5 != 0
fn sigma<Fr: PrimeField>(a: Num<Fr>) -> Num<Fr> {
    a.square().square() * a
}

fn mix<Fr: PrimeField>(state: &mut [Num<Fr>], params: &PoseidonParams<Fr>) {
    let statelen = state.len();

    // Don't allocate memory in heap if statelen is less or equal to PREALLOC_SIZE
    let mut arr: [Num<Fr>; PREALLOC_SIZE] = [Num::ZERO; PREALLOC_SIZE];
    let mut vec: Vec<Num<Fr>> = Vec::new();
    let new_state = match statelen {
        size if size <= PREALLOC_SIZE => &mut arr[..size],
        size => {
            vec.resize(size, Num::ZERO);
            &mut vec[..]
        }
    };

    for i in 0..statelen {
        for j in 0..statelen {
            new_state[i] += params.m[i][j] * state[j];
        }
    }

    (0..statelen).for_each(|i| state[i] = new_state[i]);
}

fn mix_m_i<Fr: PrimeField>(state: &mut [Num<Fr>], params: &PoseidonParams<Fr>) {
    let statelen = state.len();

    // Don't allocate memory in heap if statelen is less or equal to PREALLOC_SIZE
    let mut arr: [Num<Fr>; PREALLOC_SIZE] = [Num::ZERO; PREALLOC_SIZE];
    let mut vec: Vec<Num<Fr>> = Vec::new();
    let new_state = match statelen {
        size if size <= PREALLOC_SIZE => &mut arr[..size],
        size => {
            vec.resize(size, Num::ZERO);
            &mut vec[..]
        }
    };

    for i in 0..statelen {
        for j in 0..statelen {
            new_state[i] += params.mds_matrices.m_i[j][i] * state[j];
        }
    }

    (0..statelen).for_each(|i| state[i] = new_state[i]);
}

fn cheap_mix<Fr: PrimeField>(state: &mut [Num<Fr>], params: &PoseidonParams<Fr>, k: usize) {
    let statelen = state.len();

    let state_0 = state[0];
    let mut new_state_0 = Num::ZERO;
    (0..statelen).for_each(|i| {
        let tmp = if i == 0 {
            params.mds_matrices.m_0_0
        } else {
            params.mds_matrices.w_hat_collection[k][i - 1]
        };
        new_state_0 += tmp * state[i]
    });
    state[0] = new_state_0;

    (0..statelen - 1)
        .for_each(|i| state[i + 1] += state_0 * params.mds_matrices.v_collection[k][i]);
}

// Reference implementation: https://extgit.iaik.tugraz.at/krypto/hadeshash/-/blob/master/code/poseidonperm_x3_64_24_optimized.sage#L133
pub fn perm<Fr: PrimeField>(state: &mut [Num<Fr>], params: &PoseidonParams<Fr>) {
    assert!(state.len() == params.t);
    let half_f = params.f >> 1;
    let mut round = 0;

    // First full rounds
    for _ in 0..half_f {
        // Round constants, nonlinear layer, matrix multiplication
        ark(state, &params.round_constants[round]);
        (0..params.t).for_each(|j| state[j] = sigma(state[j]));
        mix(state, &params);
        round += 1;
    }

    // Middle partial rounds

    // Initial constants addition
    ark(state, &params.round_constants[round]);

    // First full matrix multiplication
    mix_m_i(state, &params);

    for r in 0..params.p {
        // Round constants, nonlinear layer, matrix multiplication
        state[0] = sigma(state[0]);

        // Moved constants addition
        if r < params.p - 1 {
            round += 1;
            state[0] = state[0] + params.round_constants[round][0];
        }
        cheap_mix(state, params, params.p - r - 1);
    }
    round += 1;

    // Last full rounds
    for _ in 0..half_f {
        // Round constants, nonlinear layer, matrix multiplication
        ark(state, &params.round_constants[round]);
        (0..params.t).for_each(|j| state[j] = sigma(state[j]));
        mix(state, &params);
        round += 1;
    }
}

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

    perm(&mut state[..], params);
    state[0]
}
