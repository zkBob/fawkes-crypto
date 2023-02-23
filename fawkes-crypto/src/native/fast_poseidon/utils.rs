use ff_uint::{PrimeField, Num};

/// Apply the quintic S-Box (s^5) to a given item
pub(crate) fn quintic_s_box<Fr: PrimeField>(l: &mut Num<Fr>, pre_add: Option<&Num<Fr>>, post_add: Option<&Num<Fr>>) {
    if let Some(x) = pre_add {
        *l += x;
    }
    let mut tmp = *l;
    tmp = tmp.square(); // l^2
    tmp = tmp.square(); // l^4
    *l *= tmp; // l^5
    if let Some(x) = post_add {
        *l += x;
    }
}