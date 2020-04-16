use ff::{Field, SqrtField, PrimeField, PrimeFieldRepr};
use linked_list::{LinkedList, Cursor};
use std::cmp::{Ordering};
use std::ops::{Add, Sub, Mul, Neg, Div, AddAssign, SubAssign, MulAssign, DivAssign};


use crate::core::cs::ConstraintSystem;
use crate::core::num::Num;

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum Index{
    Input(usize),
    Aux(usize)
}


impl PartialOrd for Index {
    fn partial_cmp(&self, other: &Index) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Index {
    fn cmp(&self, other: &Index) -> Ordering {
        match (self, other) {
            (Index::Input(a), Index::Input(b)) => a.cmp(&b),
            (Index::Input(_), Index::Aux(_)) => Ordering::Less,
            (Index::Aux(_), Index::Input(_)) => Ordering::Greater,
            (Index::Aux(a), Index::Aux(b)) => a.cmp(&b)
        }
    }
}




pub struct Signal<'a, CS:ConstraintSystem>{
    pub value:Option<Num<CS::F>>,
    pub lc:LinkedList<(Index,Num<CS::F>)>,
    pub cs:&'a CS
}

impl<'a, CS:ConstraintSystem> Clone for Signal<'a, CS> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            lc: self.lc.clone(),
            cs: self.cs
        }
    }
}
    

impl<'a, CS:ConstraintSystem> Signal<'a, CS> {
    pub fn capacity(&self) -> usize {
        self.lc.len()
    }

    pub fn get_value(&self) -> Option<Num<CS::F>> {
        self.value
    }

    pub fn as_const(&self) -> Option<Num<CS::F>> {
        if self.lc.len()==0 {
            Some(Num::zero())
        } else if self.lc.len() == 1 {
            let front = self.lc.front().unwrap();
            if front.0 == Index::Input(0) {
                Some(front.1)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn from_var(cs:&'a CS, value: Option<Num<CS::F>>, var: Index) -> Self {
        let mut lc = LinkedList::new();
        lc.push_back((var, Num::one()));
        Self {value, lc, cs}
    }

    pub fn from_const(cs:&'a CS, value: Num<CS::F>) -> Self {
        let mut lc = LinkedList::new();
        lc.push_back((Index::Input(0), value));
        let value = Some(value);
        Self {value, lc, cs}
    }

    pub fn zero(cs:&'a CS) -> Self {
        Self::from_const(cs, Num::zero())
    }

    pub fn one(cs:&'a CS) -> Self {
        Self::from_const(cs, Num::one())
    }



    pub fn alloc(cs:&'a CS, value:Option<Num<CS::F>>) -> Self {
        let var = cs.alloc(value);
        Self::from_var(cs, value, var)
    }

    pub fn inputize(&self) {
        match self.as_const() {
            Some(v) => {
                let input = self.cs.alloc_input(Some(v));
                let a = Self::from_var(&self.cs, Some(v), input);
                let b = Self::one(&self.cs);
                let c = Self::from_const(&self.cs, v);
                self.cs.enforce(&a, &b, &c);

            },
            _ => {
                let input = self.cs.alloc_input(self.get_value());
                let a = Self::from_var(&self.cs, self.get_value(), input);
                let b = Self::one(&self.cs);
                self.cs.enforce(&a, &b, self);
            },
        }
    }
}

#[derive(Eq,PartialEq)]
enum LookupAction {
    Add,
    Insert
}

#[inline]
fn ll_lookup<K:PartialEq+PartialOrd,V>(cur: &mut Cursor<(K, V)>, n: K) -> LookupAction {
    loop {
        match cur.peek_next() {
            Some((k, _)) => {
                if  *k == n {
                    return LookupAction::Add;
                } else if *k > n {
                    return  LookupAction::Insert;
                }
            },
            None => {
                return LookupAction::Insert;
            }
        }
        cur.seek_forward(1);
    }
}



impl<'l, 'a, CS:ConstraintSystem> AddAssign<&'l Signal<'a, CS>> for Signal<'a, CS> {
    #[inline]
    fn add_assign(&mut self, other: &'l Signal<CS>)  {
        self.value = match (self.get_value(), other.get_value()) {
            (Some(a), Some(b)) => Some(a+b),
            _ => None
        };

        let mut cur_a_ll = self.lc.cursor();

        for (k, v) in other.lc.iter() {
            if ll_lookup(&mut cur_a_ll, *k) == LookupAction::Add {
                let t = cur_a_ll.peek_next().unwrap();
                t.1 += *v;
                if t.1.is_zero() {
                    cur_a_ll.remove();
                }
            } else {
                cur_a_ll.insert((*k, *v))
            }
        }
    }
}


impl<'l, 'a, CS:ConstraintSystem> SubAssign<&'l Signal<'a, CS>> for Signal<'a, CS> {

    #[inline]
    fn sub_assign(&mut self, other: &'l Signal<CS>)  {
        self.value = match (self.get_value(), other.get_value()) {
            (Some(a), Some(b)) => Some(a-b),
            _ => None
        };

        let mut cur_a_ll = self.lc.cursor();

        for (k, v) in other.lc.iter() {
            if ll_lookup(&mut cur_a_ll, *k) == LookupAction::Add {
                let t = cur_a_ll.peek_next().unwrap();
                t.1 += *v;
                if t.1.is_zero() {
                    cur_a_ll.remove();
                }
            } else {
                cur_a_ll.insert((*k, *v))
            }
        }
    }
}

impl<'l, 'a, CS:ConstraintSystem> MulAssign<&'l Num<CS::F>> for Signal<'a, CS> {
    #[inline]
    fn mul_assign(&mut self, other: &'l Num<CS::F>)  {
        if other.is_zero() {
            *self = Self::zero(&self.cs)
        } else {
            self.value = self.value.map(|v| v*other);
            for (_, v) in self.lc.iter_mut() {
                *v *= other;
            }
        }
    }
}

impl<'l, 'a, CS:ConstraintSystem> DivAssign<&'l Num<CS::F>> for Signal<'a, CS> {
    #[inline]
    fn div_assign(&mut self, other: &'l Num<CS::F>)  {
        self.value = self.value.map(|v| v/other);
        for (_, v) in self.lc.iter_mut() {
            *v /= other;
        }
    }
}


forward_val_assign_ex!(impl<'a, CS:ConstraintSystem> AddAssign<Signal<'a, CS>> for Signal<'a, CS>, add_assign);
forward_val_assign_ex!(impl<'a, CS:ConstraintSystem> SubAssign<Signal<'a, CS>> for Signal<'a, CS>, sub_assign);
forward_val_assign_ex!(impl<'a, CS:ConstraintSystem> MulAssign<Num<CS::F>> for Signal<'a, CS>, mul_assign);
forward_val_assign_ex!(impl<'a, CS:ConstraintSystem> DivAssign<Num<CS::F>> for Signal<'a, CS>, div_assign);


impl<'l, 'a, CS:ConstraintSystem> Add<&'l Signal<'a, CS>> for Signal<'a, CS> {
    type Output = Signal<'a, CS>;

    #[inline]
    fn add(mut self, other: &'l Signal<'a, CS>) -> Self::Output  {
        self += other;
        self
    }
}

impl<'l, 'a, CS:ConstraintSystem> Sub<&'l Signal<'a, CS>> for Signal<'a, CS> {
    type Output = Signal<'a, CS>;

    #[inline]
    fn sub(mut self, other: &'l Signal<'a, CS>) -> Self::Output  {
        self -= other;
        self
    }
}


impl<'l, 'a, CS:ConstraintSystem> Add<&'l Num<CS::F>> for Signal<'a, CS> {
    type Output = Signal<'a, CS>;

    #[inline]
    fn add(mut self, other: &'l Num<CS::F>) -> Self::Output  {
        self += Signal::from_const(self.cs, *other);
        self
    }
}


impl<'l, 'a, CS:ConstraintSystem> Sub<&'l Num<CS::F>> for Signal<'a, CS> {
    type Output = Signal<'a, CS>;

    #[inline]
    fn sub(mut self, other: &'l Num<CS::F>) -> Self::Output  {
        self -= Signal::from_const(self.cs, *other);
        self
    }
}

impl<'l, 'a, CS:ConstraintSystem> Sub<&'l Signal<'a, CS>> for Num<CS::F> {
    type Output = Signal<'a, CS>;

    #[inline]
    fn sub(self, other: &'l Signal<'a, CS>) -> Self::Output  {
        Signal::from_const(other.cs, self) - other
    }
}


impl<'l, 'a, CS:ConstraintSystem> Mul<&'l Num<CS::F>> for Signal<'a, CS> {
    type Output = Signal<'a, CS>;

    #[inline]
    fn mul(mut self, other: &'l Num<CS::F>) -> Self::Output  {
        self *= other;
        self
    }
}

impl<'l, 'a, CS:ConstraintSystem> Div<&'l Num<CS::F>> for Signal<'a, CS> {
    type Output = Signal<'a, CS>;

    #[inline]
    fn div(mut self, other: &'l Num<CS::F>) -> Self::Output  {
        self /= other;
        self
    }
}

forward_all_binop_to_val_ref_commutative_ex!(impl<'a, CS:ConstraintSystem> Add for Signal<'a, CS>, add);
forward_all_binop_to_val_ref_ex!(impl<'a, CS:ConstraintSystem> Sub<Signal<'a, CS>> for Signal<'a, CS>, sub -> Signal<'a, CS>);
forward_all_binop_to_val_ref_ex!(impl<'a, CS:ConstraintSystem> Add<Num<CS::F>> for Signal<'a, CS>, add -> Signal<'a, CS>);
forward_all_binop_to_val_ref_ex!(impl<'a, CS:ConstraintSystem> Sub<Num<CS::F>> for Signal<'a, CS>, sub -> Signal<'a, CS>);
forward_all_binop_to_val_ref_ex!(impl<'a, CS:ConstraintSystem> Sub<Signal<'a, CS>> for Num<CS::F>, sub -> Signal<'a, CS>);

forward_all_binop_to_val_ref_ex!(impl<'a, CS:ConstraintSystem> Mul<Num<CS::F>> for Signal<'a, CS>, mul -> Signal<'a, CS>);
forward_all_binop_to_val_ref_ex!(impl<'a, CS:ConstraintSystem> Div<Num<CS::F>> for Signal<'a, CS>, div -> Signal<'a, CS>);

swap_commutative!(impl<'a, CS:ConstraintSystem> Add<Num<CS::F>> for Signal<'a, CS>, add);
swap_commutative!(impl<'a, CS:ConstraintSystem> Mul<Num<CS::F>> for Signal<'a, CS>, mul);

impl<'l, 'a, CS:ConstraintSystem> MulAssign<&'l Signal<'a, CS>> for Signal<'a, CS> {
    #[inline]
    fn mul_assign(&mut self, other: &'l Signal<'a, CS>)  {
        let res = match (self.as_const(), other.as_const()) {
            (Some(a), _) => other*a,
            (_, Some(b)) => &*self*b,
            _ => {
                let value = match(self.get_value(), other.get_value()) {
                    (Some(a), Some(b)) => Some(a*b),
                    _ => None
                };

                let a_mul_b = Signal::alloc(self.cs, value);
                self.cs.enforce(self, other, &a_mul_b);
                a_mul_b
            }
        };
        *self = res;
    }
}


impl<'l, 'a, CS:ConstraintSystem> DivAssign<&'l Signal<'a, CS>> for Signal<'a, CS> {
    #[inline]
    fn div_assign(&mut self, other: &'l Signal<'a, CS>)  {
        let res = match (self.as_const(), other.as_const()) {
            (Some(a), _) => other/a,
            (_, Some(b)) => &*self/b,
            _ => {
                let value = match(self.get_value(), other.get_value()) {
                    (Some(a), Some(b)) => Some(a/b),
                    _ => None
                };


                let a_div_b = Signal::alloc(self.cs, value);
                self.cs.enforce(&a_div_b, other, self);
                a_div_b
            }
        };
        *self = res;
    }
}

forward_val_assign_ex!(impl<'a, CS:ConstraintSystem> MulAssign<Signal<'a, CS>> for Signal<'a, CS>, mul_assign);
forward_val_assign_ex!(impl<'a, CS:ConstraintSystem> DivAssign<Signal<'a, CS>> for Signal<'a, CS>, div_assign);


impl<'l, 'a, CS:ConstraintSystem> Mul<&'l Signal<'a, CS>> for Signal<'a, CS> {
    type Output = Signal<'a, CS>;

    #[inline]
    fn mul(mut self, other: &'l Signal<'a, CS>) -> Self::Output  {
        self *= other;
        self
    }
}


impl<'l, 'a, CS:ConstraintSystem> Div<&'l Signal<'a, CS>> for Signal<'a, CS> {
    type Output = Signal<'a, CS>;

    #[inline]
    fn div(mut self, other: &'l Signal<'a, CS>) -> Self::Output  {
        self /= other;
        self
    }
}

forward_all_binop_to_val_ref_ex!(impl<'a, CS:ConstraintSystem> Mul<Signal<'a, CS>> for Signal<'a, CS>, mul -> Signal<'a, CS>);
forward_all_binop_to_val_ref_ex!(impl<'a, CS:ConstraintSystem> Div<Signal<'a, CS>> for Signal<'a, CS>, div -> Signal<'a, CS>);

#[cfg(test)]
mod signal_test {
    use super::*;
    use bellman::pairing::bn256::{Fr};
    use rand::{Rng, thread_rng};


    #[test]
    fn add_test() {
        let mut rng = thread_rng();
        let ref cs = crate::core::cs::TestCS::<Fr>::new();
        let n_a = rng.gen();
        let n_b = rng.gen();

        let a = Signal::from_const(cs, n_a);
        let b = Signal::from_const(cs, n_b);
        let c = a+b;
        assert!(c.get_value().unwrap()==n_a+n_b);
    }

    #[test]
    fn add_mixed() {
        let mut rng = thread_rng();
        let ref cs = crate::core::cs::TestCS::<Fr>::new();
        let n_a = rng.gen();
        let n_b: Num<_> = rng.gen();

        let a = Signal::from_const(cs, n_a);
        let c = a+n_b;
        assert!(c.get_value().unwrap()==n_a+n_b);
    }

}