// Allow `&Matrix` in function signatures.
#![allow(clippy::ptr_arg)]

use ff_uint::Num;
use ff_uint::PrimeField;

/// Matrix functions here are, at least for now, quick and dirty — intended only to support precomputation of poseidon optimization.

/// Matrix represented as a Vec of rows, so that m[i][j] represents the jth column of the ith row in Matrix, m.
pub type Matrix<T> = Vec<Vec<T>>;

pub fn rows<T>(matrix: &Matrix<T>) -> usize {
    matrix.len()
}

/// Panics if `matrix` is not actually a matrix. So only use any of these functions on well-formed data.
/// Only use during constant calculation on matrices known to have been constructed correctly.
fn columns<T>(matrix: &Matrix<T>) -> usize {
    if matrix.is_empty() {
        0
    } else {
        let column_length = matrix[0].len();
        for row in matrix {
            if row.len() != column_length {
                panic!("not a matrix");
            }
        }
        column_length
    }
}

// This wastefully discards the actual inverse, if it exists, so in general callers should
// just call `invert` if that result will be needed.
pub(crate) fn is_invertible<Fr: PrimeField>(matrix: &Matrix<Num<Fr>>) -> bool {
    is_square(matrix) && invert(matrix).is_some()
}

fn scalar_mul<Fr: PrimeField>(scalar: Num<Fr>, matrix: &Matrix<Num<Fr>>) -> Matrix<Num<Fr>> {
    matrix
        .iter()
        .map(|row| {
            row.iter()
                .map(|val| {
                    let mut prod = scalar;
                    prod *= val;
                    prod
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}

fn scalar_vec_mul<Fr: PrimeField>(scalar: Num<Fr>, vec: &[Num<Fr>]) -> Vec<Num<Fr>> {
    vec.iter()
        .map(|val| {
            let mut prod = scalar;
            prod *= val;
            prod
        })
        .collect::<Vec<_>>()
}

pub fn mat_mul<Fr: PrimeField>(a: &Matrix<Num<Fr>>, b: &Matrix<Num<Fr>>) -> Option<Matrix<Num<Fr>>> {
    if rows(a) != columns(b) {
        return None;
    };

    let b_t = transpose(b);

    let res = a
        .iter()
        .map(|input_row| {
            b_t.iter()
                .map(|transposed_column| vec_mul(input_row, transposed_column))
                .collect()
        })
        .collect();

    Some(res)
}

fn vec_mul<Fr: PrimeField>(a: &[Num<Fr>], b: &[Num<Fr>]) -> Num<Fr> {
    a.iter().zip(b).fold(Num::ZERO, |mut acc, (v1, v2)| {
        let mut tmp = *v1;
        tmp *= v2;
        acc += tmp;
        acc
    })
}

pub fn vec_add<Fr: PrimeField>(a: &[Num<Fr>], b: &[Num<Fr>]) -> Vec<Num<Fr>> {
    a.iter()
        .zip(b.iter())
        .map(|(a, b)| {
            let mut res = *a;
            res += b;
            res
        })
        .collect::<Vec<_>>()
}

pub fn vec_sub<Fr: PrimeField>(a: &[Num<Fr>], b: &[Num<Fr>]) -> Vec<Num<Fr>> {
    a.iter()
        .zip(b.iter())
        .map(|(a, b)| {
            let mut res = *a;
            res -= b;
            res
        })
        .collect::<Vec<_>>()
}

/// Left-multiply a vector by a square matrix of same size: MV where V is considered a column vector.
pub fn left_apply_matrix<Fr: PrimeField>(m: &Matrix<Num<Fr>>, v: &[Num<Fr>]) -> Vec<Num<Fr>> {
    assert!(is_square(m), "Only square matrix can be applied to vector.");
    assert_eq!(
        rows(m),
        v.len(),
        "Matrix can only be applied to vector of same size."
    );

    let mut result = vec![Num::ZERO; v.len()];

    for (result, row) in result.iter_mut().zip(m.iter()) {
        for (mat_val, vec_val) in row.iter().zip(v) {
            let mut tmp = *mat_val;
            tmp *= vec_val;
            *result += tmp;
        }
    }
    result
}

/// Right-multiply a vector by a square matrix  of same size: VM where V is considered a row vector.
pub fn apply_matrix<Fr: PrimeField>(m: &Matrix<Num<Fr>>, v: &[Num<Fr>]) -> Vec<Num<Fr>> {
    assert!(is_square(m), "Only square matrix can be applied to vector.");
    assert_eq!(
        rows(m),
        v.len(),
        "Matrix can only be applied to vector of same size."
    );

    let mut result = vec![Num::ZERO; v.len()];
    for (j, val) in result.iter_mut().enumerate() {
        for (i, row) in m.iter().enumerate() {
            let mut tmp = row[j];
            tmp *= v[i];
            *val += tmp;
        }
    }

    result
}

#[allow(clippy::needless_range_loop)]
pub fn transpose<Fr: PrimeField>(matrix: &Matrix<Num<Fr>>) -> Matrix<Num<Fr>> {
    let size = rows(matrix);
    let mut new = Vec::with_capacity(size);
    for j in 0..size {
        let mut row = Vec::with_capacity(size);
        for i in 0..size {
            row.push(matrix[i][j])
        }
        new.push(row);
    }
    new
}

#[allow(clippy::needless_range_loop)]
pub fn make_identity<Fr: PrimeField>(size: usize) -> Matrix<Num<Fr>> {
    let mut result = vec![vec![Num::ZERO; size]; size];
    for i in 0..size {
        result[i][i] = Num::ONE;
    }
    result
}

pub fn kronecker_delta<Fr: PrimeField>(i: usize, j: usize) -> Num<Fr> {
    if i == j {
        Num::ONE
    } else {
        Num::ZERO
    }
}

pub fn is_identity<Fr: PrimeField>(matrix: &Matrix<Num<Fr>>) -> bool {
    for i in 0..rows(matrix) {
        for j in 0..columns(matrix) {
            if matrix[i][j] != kronecker_delta(i, j) {
                return false;
            }
        }
    }
    true
}

pub fn is_square<T>(matrix: &Matrix<T>) -> bool {
    rows(matrix) == columns(matrix)
}

pub fn minor<Fr: PrimeField>(matrix: &Matrix<Num<Fr>>, i: usize, j: usize) -> Matrix<Num<Fr>> {
    assert!(is_square(matrix));
    let size = rows(matrix);
    assert!(size > 0);
    let new = matrix
        .iter()
        .enumerate()
        .filter_map(|(ii, row)| {
            if ii == i {
                None
            } else {
                let mut new_row = row.clone();
                new_row.remove(j);
                Some(new_row)
            }
        })
        .collect();
    assert!(is_square(&new));
    new
}

// Assumes matrix is partially reduced to upper triangular. `column` is the column to eliminate from all rows.
// Returns `None` if either:
//   - no non-zero pivot can be found for `column`
//   - `column` is not the first
fn eliminate<Fr: PrimeField>(
    matrix: &Matrix<Num<Fr>>,
    column: usize,
    shadow: &mut Matrix<Num<Fr>>,
) -> Option<Matrix<Num<Fr>>> {
    let zero = Num::ZERO;
    let pivot_index = (0..rows(matrix))
        .find(|&i| matrix[i][column] != zero && (0..column).all(|j| matrix[i][j] == zero))?;

    let pivot = &matrix[pivot_index];
    let pivot_val = pivot[column];

    // This should never fail since we have a non-zero `pivot_val` if we got here.
    let inv_pivot = pivot_val.checked_inv()?;
    let mut result = Vec::with_capacity(matrix.len());
    result.push(pivot.clone());

    for (i, row) in matrix.iter().enumerate() {
        if i == pivot_index {
            continue;
        };
        let val = row[column];
        if val == zero {
            // Value is already eliminated.
            result.push(row.to_vec());
        } else {
            let mut factor = val;
            factor *= inv_pivot;

            let scaled_pivot = scalar_vec_mul(factor, pivot);
            let eliminated = vec_sub(row, &scaled_pivot);
            result.push(eliminated);

            let shadow_pivot = &shadow[pivot_index];
            let scaled_shadow_pivot = scalar_vec_mul(factor, shadow_pivot);
            let shadow_row = &shadow[i];
            shadow[i] = vec_sub(shadow_row, &scaled_shadow_pivot);
        }
    }

    let pivot_row = shadow.remove(pivot_index);
    shadow.insert(0, pivot_row);

    Some(result)
}

// `matrix` must be square.
fn upper_triangular<Fr: PrimeField>(
    matrix: &Matrix<Num<Fr>>,
    shadow: &mut Matrix<Num<Fr>>,
) -> Option<Matrix<Num<Fr>>> {
    assert!(is_square(matrix));
    let mut result = Vec::with_capacity(matrix.len());
    let mut shadow_result = Vec::with_capacity(matrix.len());

    let mut curr = matrix.clone();
    let mut column = 0;
    while curr.len() > 1 {
        let initial_rows = curr.len();

        curr = eliminate(&curr, column, shadow)?;
        result.push(curr[0].clone());
        shadow_result.push(shadow[0].clone());
        column += 1;

        curr = curr[1..].to_vec();
        *shadow = shadow[1..].to_vec();
        assert_eq!(curr.len(), initial_rows - 1);
    }
    result.push(curr[0].clone());
    shadow_result.push(shadow[0].clone());

    *shadow = shadow_result;

    Some(result)
}

// `matrix` must be upper triangular.
fn reduce_to_identity<Fr: PrimeField>(
    matrix: &Matrix<Num<Fr>>,
    shadow: &mut Matrix<Num<Fr>>,
) -> Option<Matrix<Num<Fr>>> {
    let size = rows(matrix);
    let mut result: Matrix<Num<Fr>> = Vec::new();
    let mut shadow_result: Matrix<Num<Fr>> = Vec::new();

    for i in 0..size {
        let idx = size - i - 1;
        let row = &matrix[idx];
        let shadow_row = &shadow[idx];

        let val = row[idx];
        let inv = {
            let inv = val.checked_inv();
            // If `val` is zero, then there is no inverse, and we cannot compute a result.
            if inv.is_none().into() {
                return None;
            }
            inv.unwrap()
        };

        let mut normalized = scalar_vec_mul(inv, row);
        let mut shadow_normalized = scalar_vec_mul(inv, shadow_row);

        for j in 0..i {
            let idx = size - j - 1;
            let val = normalized[idx];
            let subtracted = scalar_vec_mul(val, &result[j]);
            let result_subtracted = scalar_vec_mul(val, &shadow_result[j]);

            normalized = vec_sub(&normalized, &subtracted);
            shadow_normalized = vec_sub(&shadow_normalized, &result_subtracted);
        }

        result.push(normalized);
        shadow_result.push(shadow_normalized);
    }

    result.reverse();
    shadow_result.reverse();

    *shadow = shadow_result;
    Some(result)
}

//
pub(crate) fn invert<Fr: PrimeField>(matrix: &Matrix<Num<Fr>>) -> Option<Matrix<Num<Fr>>> {
    let mut shadow = make_identity(columns(matrix));
    let ut = upper_triangular(matrix, &mut shadow);

    ut.and_then(|x| reduce_to_identity(&x, &mut shadow))
        .and(Some(shadow))
}

#[cfg(test)]
mod tests {
    use crate::engines::bn256::Fr;

    use super::*;

    #[test]
    fn test_minor() {
        let one: Num<Fr> = Num::from(1);
        let two = Num::from(2);
        let three = Num::from(3);
        let four = Num::from(4);
        let five = Num::from(5);
        let six = Num::from(6);
        let seven = Num::from(7);
        let eight = Num::from(8);
        let nine = Num::from(9);

        let m = vec![
            vec![one, two, three],
            vec![four, five, six],
            vec![seven, eight, nine],
        ];

        let cases = [
            (0, 0, vec![vec![five, six], vec![eight, nine]]),
            (0, 1, vec![vec![four, six], vec![seven, nine]]),
            (0, 2, vec![vec![four, five], vec![seven, eight]]),
            (1, 0, vec![vec![two, three], vec![eight, nine]]),
            (1, 1, vec![vec![one, three], vec![seven, nine]]),
            (1, 2, vec![vec![one, two], vec![seven, eight]]),
            (2, 0, vec![vec![two, three], vec![five, six]]),
            (2, 1, vec![vec![one, three], vec![four, six]]),
            (2, 2, vec![vec![one, two], vec![four, five]]),
        ];
        for (i, j, expected) in &cases {
            let result = minor(&m, *i, *j);

            assert_eq!(*expected, result);
        }
    }

    #[test]
    fn test_scalar_mul() {
        let zero: Num<Fr> = Num::from(0);
        let one = Num::from(1);
        let two = Num::from(2);
        let three = Num::from(3);
        let four = Num::from(4);
        let six = Num::from(6);

        let m = vec![vec![zero, one], vec![two, three]];
        let res = scalar_mul(two, &m);

        let expected = vec![vec![zero, two], vec![four, six]];

        assert_eq!(expected, res);
    }

    #[test]
    fn test_vec_mul() {
        let one: Num<Fr> = Num::from(1);
        let two = Num::from(2);
        let three = Num::from(3);
        let four = Num::from(4);
        let five = Num::from(5);
        let six = Num::from(6);

        let a = vec![one, two, three];
        let b = vec![four, five, six];
        let res = vec_mul(&a, &b);

        let expected = Num::from(32);

        assert_eq!(expected, res);
    }

    #[test]
    fn test_transpose() {
        let one: Num<Fr> = Num::from(1);
        let two = Num::from(2);
        let three = Num::from(3);
        let four = Num::from(4);
        let five = Num::from(5);
        let six = Num::from(6);
        let seven = Num::from(7);
        let eight = Num::from(8);
        let nine = Num::from(9);

        let m = vec![
            vec![one, two, three],
            vec![four, five, six],
            vec![seven, eight, nine],
        ];

        let expected = vec![
            vec![one, four, seven],
            vec![two, five, eight],
            vec![three, six, nine],
        ];

        let res = transpose(&m);
        assert_eq!(expected, res);
    }

    #[test]
    fn test_inverse() {
        let zero: Num<Fr> = Num::from(0);
        let one = Num::from(1);
        let two = Num::from(2);
        let three = Num::from(3);
        let four = Num::from(4);
        let five = Num::from(5);
        let six = Num::from(6);
        let seven = Num::from(7);
        let eight = Num::from(8);
        let nine = Num::from(9);

        let m = vec![
            vec![one, two, three],
            vec![four, three, six],
            vec![five, eight, seven],
        ];

        let m1 = vec![
            vec![one, two, three],
            vec![four, five, six],
            vec![seven, eight, nine],
        ];

        assert!(!is_invertible(&m1));
        assert!(is_invertible(&m));

        let m_inv = invert(&m).unwrap();

        let computed_identity = mat_mul(&m, &m_inv).unwrap();
        assert!(is_identity(&computed_identity));

        // S
        let some_vec = vec![six, five, four];

        // M^-1(S)
        let inverse_applied = super::apply_matrix(&m_inv, &some_vec);

        // M(M^-1(S))
        let m_applied_after_inverse = super::apply_matrix(&m, &inverse_applied);

        // S = M(M^-1(S))
        assert_eq!(
            some_vec, m_applied_after_inverse,
            "M(M^-1(V))) = V did not hold"
        );

        //panic!();
        // B
        let base_vec = vec![eight, two, five];

        // S + M(B)
        let add_after_apply = vec_add(&some_vec, &apply_matrix(&m, &base_vec));

        // M(B + M^-1(S))
        let apply_after_add = apply_matrix(&m, &vec_add(&base_vec, &inverse_applied));

        // S + M(B) = M(B + M^-1(S))
        assert_eq!(add_after_apply, apply_after_add, "breakin' the law");

        let m = vec![vec![zero, one], vec![one, zero]];
        let m_inv = invert(&m).unwrap();
        let computed_identity = mat_mul(&m, &m_inv).unwrap();
        assert!(is_identity(&computed_identity));
        let computed_identity = mat_mul(&m_inv, &m).unwrap();
        assert!(is_identity(&computed_identity));
    }

    #[test]
    fn test_eliminate() {
        //let one = Fr::from(1);
        let two: Num<Fr> = Num::from(2);
        let three = Num::from(3);
        let four = Num::from(4);
        let five = Num::from(5);
        let six = Num::from(6);
        let seven = Num::from(7);
        let eight = Num::from(8);
        //        let nine = Fr::from(9);

        let m = vec![
            vec![two, three, four],
            vec![four, five, six],
            vec![seven, eight, eight],
        ];

        for i in 0..rows(&m) {
            let mut shadow = make_identity(columns(&m));
            let res = eliminate(&m, i, &mut shadow);
            if i > 0 {
                assert!(res.is_none());
                continue;
            } else {
                assert!(res.is_some());
            }

            assert_eq!(
                1,
                res.unwrap()
                    .iter()
                    .filter(|&row| row[i] != Num::ZERO)
                    .count()
            );
        }
    }
    #[test]
    fn test_upper_triangular() {
        //        let one = Fr::from(1);
        let two: Num<Fr> = Num::from(2);
        let three = Num::from(3);
        let four = Num::from(4);
        let five = Num::from(5);
        let six = Num::from(6);
        let seven = Num::from(7);
        let eight = Num::from(8);
        //        let nine = Fr::from(9);

        let m = vec![
            vec![two, three, four],
            vec![four, five, six],
            vec![seven, eight, eight],
        ];

        let mut shadow = make_identity(columns(&m));
        let _res = upper_triangular(&m, &mut shadow);

        // Actually assert things.
    }

    #[test]
    fn test_reduce_to_identity() {
        //        let one = Fr::from(1);
        let two: Num<Fr> = Num::from(2);
        let three = Num::from(3);
        let four = Num::from(4);
        let five = Num::from(5);
        let six = Num::from(6);
        let seven = Num::from(7);
        let eight = Num::from(8);
        //        let nine = Fr::from(9);

        let m = vec![
            vec![two, three, four],
            vec![four, five, six],
            vec![seven, eight, eight],
        ];

        let mut shadow = make_identity(columns(&m));
        let ut = upper_triangular(&m, &mut shadow);

        let res = ut
            .and_then(|x| reduce_to_identity(&x, &mut shadow))
            .unwrap();

        assert!(is_identity(&res));
        let prod = mat_mul(&m, &shadow).unwrap();

        assert!(is_identity(&prod));
    }
}