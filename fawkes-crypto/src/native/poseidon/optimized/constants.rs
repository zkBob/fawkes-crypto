use std::iter;

use ff_uint::{Num, PrimeField};

use super::matrix::{apply_matrix, invert, transpose};

// Reference implementation: https://extgit.iaik.tugraz.at/krypto/hadeshash/-/blob/master/code/poseidonperm_x3_64_24_optimized.sage#L43
pub fn calc_equivalent_constants<Fr: PrimeField>(
    rc: &Vec<Vec<Num<Fr>>>,
    m: &Vec<Vec<Num<Fr>>>,
    f: usize,
    p: usize,
    t: usize,
) -> Vec<Vec<Num<Fr>>> {
    let num_rounds = f + p;
    let half_f = f >> 1;
    let mut constants = rc.clone();
    let m_transpose_inv = &invert(&transpose(&m)).unwrap();

    let mut i = num_rounds - 2 - half_f;
    while i > half_f - 1 {
        let inv_cip1 = apply_matrix(m_transpose_inv, &constants[i + 1]);
        constants[i] = constants[i]
            .iter()
            .zip([Num::ZERO].iter().chain(inv_cip1.iter().skip(1)))
            .map(|(a, b)| *a + *b)
            .collect();
        constants[i + 1] = inv_cip1
            .into_iter()
            .take(1)
            .chain(iter::repeat(Num::ZERO))
            .take(t)
            .collect();
        i -= 1;
    }

    constants
}
