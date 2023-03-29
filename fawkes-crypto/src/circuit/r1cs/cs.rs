use crate::{
    circuit::{
        num::CNum,
        lc::{LC, Index, AbstractLC, ZeroLC}
    },
    core::signal::Signal,
    ff_uint::{Num, PrimeField}
};

use std::{cell::RefCell, marker::PhantomData, rc::Rc};
use bit_vec::BitVec;

pub type RCS<C> = Rc<RefCell<C>>;

#[cfg(feature="borsh_support")]
use crate::borsh::{BorshSerialize, BorshDeserialize};


#[derive(Clone, Debug)]
#[cfg_attr(feature = "borsh_support", derive(BorshSerialize, BorshDeserialize))]
pub struct Gate<Fr:PrimeField>(
    pub Vec<(Num<Fr>, Index)>,
    pub Vec<(Num<Fr>, Index)>,
    pub Vec<(Num<Fr>, Index)>,
);

pub trait CS: Clone {
    type Fr: PrimeField;
    type LC: AbstractLC<Self::Fr>;
    type GateIterator<'g>: Iterator<Item=&'g Gate<Self::Fr>> where Self: 'g;

    fn num_gates(&self) -> usize;
    fn num_input(&self) -> usize;
    fn num_aux(&self) -> usize;
    fn get_value(&self, index:Index) -> Option<Num<Self::Fr>>;
    fn get_gate_iterator<'g>(&'g self) -> Self::GateIterator<'g>;

    // a*b === c
    fn enforce(a: &CNum<Self>, b: &CNum<Self>, c: &CNum<Self>);
    fn inputize(n: &CNum<Self>);
    fn alloc(cs: &RCS<Self>, value: Option<&Num<Self::Fr>>) -> CNum<Self>;

    fn const_tracker_before(&mut self) -> Option<bool> {
        None
    }

    fn const_tracker_after(&mut self, _:bool) {}
}


#[derive(Clone, Debug)]
pub struct DebugCS<Fr: PrimeField> {
    pub num_input:usize,
    pub num_aux:usize,
    pub num_gates: usize,
    pub phantom: PhantomData<Fr>
}

impl<Fr: PrimeField> DebugCS<Fr> {
    pub fn new() -> Self {
        Self {
            num_input: 1,
            num_aux: 0,
            num_gates: 0,
            phantom: PhantomData
        }
    }

    pub fn rc_new() -> RCS<Self> {
        Rc::new(RefCell::new(Self::new()))
    }
}

#[derive(Clone, Debug)]
pub struct BuildCS<Fr: PrimeField> {
    pub num_input:usize,
    pub num_aux:usize,
    pub gates: Vec<Gate<Fr>>,
    pub const_tracker: BitVec
}

impl<Fr: PrimeField> BuildCS<Fr> {
    pub fn new() -> Self {
        Self {
            num_input: 1,
            num_aux: 0,
            gates: vec![],
            const_tracker: BitVec::new()
        }
    }

    pub fn rc_new() -> RCS<Self> {
        Rc::new(RefCell::new(Self::new()))
    }
}

#[derive(Clone, Debug)]
pub struct WitnessCS<'a, Fr: PrimeField> {
    pub values_input: Vec<Num<Fr>>,
    pub values_aux: Vec<Num<Fr>>,
    pub num_gates: usize,
    pub gates_data: Rc<Vec<Gate<Fr>>>,
    pub const_tracker: &'a BitVec,
    pub const_tracker_index: usize
}

impl<'a, Fr: PrimeField> WitnessCS<'a, Fr> {
    pub fn new(num_gates:usize, gates_data: Rc<Vec<Gate<Fr>>>, const_tracker: &'a BitVec) -> Self {
        Self {
            values_input: vec![Num::ONE],
            values_aux: vec![],
            num_gates,
            gates_data,
            const_tracker,
            const_tracker_index: 0
        }
    }

    pub fn rc_new(num_gates:usize, gates_data: Rc<Vec<Gate<Fr>>>, const_tracker: &'a BitVec) -> RCS<Self> {
        Rc::new(RefCell::new(Self::new(num_gates, gates_data, const_tracker)))
    }
}


impl<Fr: PrimeField>  CS for DebugCS<Fr> {
    type Fr = Fr;
    type LC = LC<Fr>;
    type GateIterator<'g> = core::iter::Empty<&'g Gate<Self::Fr>> where Self: 'g;

    fn num_gates(&self) -> usize {
        self.num_gates
    }

    fn num_input(&self) -> usize {
        self.num_input
    }
    fn num_aux(&self) -> usize {
        self.num_aux
    }

    fn get_value(&self, _:Index) -> Option<Num<Fr>> {
        None
    }

    fn get_gate_iterator<'g>(&'g self) -> Self::GateIterator<'g> {
        std::unimplemented!();
    }

    // a*b === c
    fn enforce(a: &CNum<Self>, b: &CNum<Self>, c: &CNum<Self>) {
        let mut rcs = a.get_cs().borrow_mut();
        rcs.num_gates+=1;

        match (a.value, b.value, c.value) {
            (Some(a), Some(b), Some(c)) => {
                assert!(a * b == c, "Not satisfied constraint");
            },
            _ => {}
        }
        
    }

    fn inputize(n: &CNum<Self>) {
        let mut rcs = n.get_cs().borrow_mut();
        rcs.num_gates+=1;
        rcs.num_input+=1;
    }

    fn alloc(cs: &RCS<Self>, value: Option<&Num<Self::Fr>>) -> CNum<Self> {
        let mut rcs = cs.borrow_mut();
        let v = rcs.num_aux as u32;
        rcs.num_aux+=1;
        CNum {
            value: value.cloned(),
            lc: LC::from_index(Index::Aux(v)),
            cs: cs.clone(),
        }
    }

}

impl<'a, Fr: PrimeField> CS for WitnessCS<'a, Fr> {
    type Fr = Fr;
    type LC = ZeroLC;
    type GateIterator<'g> = core::slice::Iter<'g, Gate<Self::Fr>> where Self: 'g;

    fn num_gates(&self) -> usize {
        self.num_gates
    }

    fn num_input(&self) -> usize {
        self.values_input.len()
    }
    fn num_aux(&self) -> usize {
        self.values_aux.len()
    }

    fn get_value(&self, index:Index) -> Option<Num<Fr>> {
        match index {
            Index::Input(i) => Some(self.values_input[i as usize]),
            Index::Aux(i) => Some(self.values_aux[i as usize]),
        }
    }

    fn get_gate_iterator<'g>(&'g self) -> Self::GateIterator<'g> {
        self.gates_data.iter()
    }

    fn enforce(_: &CNum<Self>, _: &CNum<Self>, _: &CNum<Self>) {
    }

    fn inputize(n: &CNum<Self>) {
        let mut rcs = n.get_cs().borrow_mut();
        rcs.values_input.push(n.get_value().expect("value is empty"));
    }

    fn alloc(cs: &RCS<Self>, value: Option<&Num<Self::Fr>>) -> CNum<Self> {
        let mut rcs = cs.borrow_mut();
        rcs.values_aux.push(value.cloned().expect("value is empty"));
        CNum {
            value: value.cloned(),
            lc: ZeroLC,
            cs: cs.clone(),
        }
    }

    fn const_tracker_before(&mut self) -> Option<bool> {
        let i = self.const_tracker_index;
        self.const_tracker_index+=1;
        Some(self.const_tracker[i])
    }

}


impl<Fr: PrimeField> CS for BuildCS<Fr> {
    type Fr = Fr;
    type LC = LC<Fr>;
    type GateIterator<'g> = core::slice::Iter<'g, Gate<Self::Fr>> where Self: 'g;

    fn num_gates(&self) -> usize {
        self.gates.len()
    }

    fn num_input(&self) -> usize {
        self.num_input
    }
    fn num_aux(&self) -> usize {
        self.num_aux
    }

    fn get_value(&self, _:Index) -> Option<Num<Fr>> {
        None
    }

    fn get_gate_iterator<'g>(&'g self) -> Self::GateIterator<'g> {
        self.gates.iter()
    }

    // a*b === c
    fn enforce(a: &CNum<Self>, b: &CNum<Self>, c: &CNum<Self>) {
        let mut rcs = a.get_cs().borrow_mut();
        rcs.gates.push(Gate(a.lc.to_vec(), b.lc.to_vec(), c.lc.to_vec()))
    }

    fn inputize(n: &CNum<Self>) {
        let mut rcs = n.get_cs().borrow_mut();
        let v = rcs.num_input as u32;
        rcs.num_input+=1;
        rcs.gates.push(Gate(
            n.lc.to_vec(),
            vec![(Num::ONE, Index::Input(0))],
            vec![(Num::ONE, Index::Input(v))],
        ));
    }

    fn alloc(cs: &RCS<Self>, _: Option<&Num<Self::Fr>>) -> CNum<Self> {
        let mut rcs = cs.borrow_mut();
        let v = rcs.num_aux as u32;
        rcs.num_aux+=1;
        CNum {
            value: None,
            lc: LC::from_index(Index::Aux(v)),
            cs: cs.clone(),
        }
    }

    fn const_tracker_after(&mut self, v:bool) {
        self.const_tracker.push(v);
    }
}
