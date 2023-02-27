use ff_uint::{Num, PrimeField};

use crate::native::fast_poseidon::matrix::{apply_matrix, invert};

use super::mds::MdsMatrices;

pub fn calc_equivalent_constants<Fr: PrimeField>(
    rc: Vec<Vec<Num<Fr>>>,
    mds: MdsMatrices<Fr>,
    f: usize,
    p: usize,
    t: usize,
) -> Vec<Vec<Num<Fr>>> {
    let m_transpose = mds.m_transpose;
    let num_rounds = f + p;
    let half_f = f >> 1;
    let mut constants = rc.clone();

    //for i in range(num_rounds - 2 - R_f, R_f - 1, -1):
    let mut i = num_rounds - 2 - half_f;
    while i > half_f - 1 {
        //inv_cip1 = list(vector(constants_temp[i+1]) * MDS_matrix_field_transpose.inverse())
        //constants_temp[i] = list(vector(constants_temp[i]) + vector([0] + inv_cip1[1:]))
        //constants_temp[i+1] = [inv_cip1[0]] + [0] * (t-1)
        let inv_cip1 = apply_matrix(&invert(&m_transpose).unwrap(), &constants[i + 1]);
        constants[i] = constants[i]
            .iter()
            .zip([Num::ZERO].iter().chain(inv_cip1.iter().skip(1)))
            .map(|(a, b)| *a + *b)
            .collect();
        constants[i + 1] = inv_cip1
            .into_iter()
            .take(1)
            .chain(vec![Num::ZERO; t - 1].into_iter())
            .collect();
        i -= 1;
    }

    constants
}
