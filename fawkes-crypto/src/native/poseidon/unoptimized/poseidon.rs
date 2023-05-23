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

pub fn perm<Fr: PrimeField>(state: &mut [Num<Fr>], params: &PoseidonParams<Fr>) {
    assert!(state.len() == params.t);
    let half_f = params.f >> 1;

    for i in 0..params.f + params.p {
        ark(state, &params.c[i]);
        if i < half_f || i >= half_f + params.p {
            for j in 0..params.t {
                state[j] = sigma(state[j]);
            }
        } else {
            state[0] = sigma(state[0]);
        }
        mix(state, params);
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
