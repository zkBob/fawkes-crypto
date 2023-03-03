use ff_uint::{Num, PrimeField};

use super::matrix::{invert, left_apply_matrix, make_identity, mat_mul, minor, transpose, Matrix};

#[cfg(feature = "serde_support")]
use crate::serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde_support", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde_support",
    serde(bound(serialize = "", deserialize = ""))
)]
pub struct MdsMatrices<Fr: PrimeField> {
    pub m_i: Matrix<Num<Fr>>,
    pub v_collection: Vec<Vec<Num<Fr>>>,
    pub w_hat_collection: Vec<Vec<Num<Fr>>>,
    pub m_0_0: Num<Fr>,
}

// Reference implementation: https://extgit.iaik.tugraz.at/krypto/hadeshash/-/blob/master/code/poseidonperm_x3_64_24_optimized.sage#L61
pub fn calc_equivalent_matrices<Fr: PrimeField>(
    m: &Matrix<Num<Fr>>,
    p: usize,
    t: usize,
) -> MdsMatrices<Fr> {
    let m_transpose = transpose(m);
    let mut w_hat_collection = vec![];
    let mut v_collection = vec![];

    let mut m_mul = m_transpose.clone();
    let mut m_i = vec![vec![Num::ZERO; t]; t];

    for _ in 0..p {
        let m_hat = minor(&m_mul, 0, 0);
        let m_hat_inv = invert(&m_hat).unwrap();

        let v = m_mul[0][1..].to_vec();
        v_collection.push(v);

        let w = m_mul
            .iter()
            .skip(1)
            .map(|column| column[0])
            .collect::<Vec<_>>();
        let w_hat = left_apply_matrix(&m_hat_inv, &w);
        w_hat_collection.push(w_hat);

        m_i = make_identity(t);
        m_i = (0..t)
            .map(|i| {
                (0..t)
                    .map(|j| {
                        if i == 0 || j == 0 {
                            m_i[i][j]
                        } else {
                            m_hat[i - 1][j - 1]
                        }
                    })
                    .collect()
            })
            .collect();

        m_mul = mat_mul(&m_transpose, &m_i).unwrap();
    }

    MdsMatrices {
        m_i,
        v_collection,
        w_hat_collection,
        m_0_0: m_transpose[0][0],
    }
}
