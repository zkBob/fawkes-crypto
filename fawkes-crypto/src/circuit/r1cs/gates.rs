use std::{marker::PhantomData, ops::Deref, io::Read};

#[cfg(feature="borsh_support")]
use borsh::{BorshSerialize, BorshDeserialize};

use byteorder::{LittleEndian, ByteOrder};
use ff_uint::{PrimeField, Num};

use super::lc::Index;


#[derive(Clone, Debug)]
#[cfg_attr(feature = "borsh_support", derive(BorshSerialize, BorshDeserialize))]
pub struct Gate<Fr: PrimeField>(
    pub Vec<(Num<Fr>, Index)>,
    pub Vec<(Num<Fr>, Index)>,
    pub Vec<(Num<Fr>, Index)>,
);

#[derive(Clone, Debug)]
pub enum GateSource<'a, Fr: PrimeField> {
    Compressed(&'a [u8]),
    Precomputed(&'a Vec<Gate<Fr>>)
}

pub enum GateIterator<'a, Fr: PrimeField> {
    Streamed(Box<GateStreamedIterator<Fr, brotli::Decompressor<&'a [u8]>>>),
    Precomputed(std::slice::Iter<'a, Gate<Fr>>)
}

impl<'a, Fr: PrimeField> GateIterator<'a, Fr> {
    pub fn new(source: &GateSource<'a, Fr>) -> Self {
        match source {
            GateSource::Compressed(bytes) => {
                Self::Streamed(Box::new(GateStreamedIterator(brotli::Decompressor::new(bytes, 4096), PhantomData)))
            },
            GateSource::Precomputed(vec) => {
                Self::Precomputed(vec.iter())
            }
        }
    }
}

pub enum GateWrapper<'a, Fr: PrimeField> {
    Value(Gate<Fr>),
    Ref(&'a Gate<Fr>)
}

impl<'a, Fr: PrimeField> GateWrapper<'a, Fr> {
    pub fn gate(self) -> Gate<Fr> {
        match self {
            Self::Value(v) => v,
            Self::Ref(r) => r.clone()
        }
    }
}

impl<'a, Fr: PrimeField> Deref for GateWrapper<'a, Fr> {
    type Target = Gate<Fr>;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Value(val) => val,
            Self::Ref(reference) => reference
        }
    }
}

impl<'a, Fr: PrimeField> Iterator for GateIterator<'a, Fr> {
    type Item = GateWrapper<'a, Fr>;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Streamed(iter) => {
                iter.next().map(|g| GateWrapper::Value(g))
            }
            Self::Precomputed(iter) => {
                iter.next().map(|g| GateWrapper::Ref(g))
            },
        }
    }
}

pub fn evaluate_gates_memory_size<Fr: PrimeField>(
    num_gates: usize,
    bytes: &[u8],
) -> std::io::Result<usize> {
    let r = &mut brotli::Decompressor::new(bytes, 4096);
    let mut memory_size = 0;
    let item_size = std::mem::size_of::<Fr>() + std::mem::size_of::<u8>() + std::mem::size_of::<u32>();
    let gate_size = std::mem::size_of::<Fr>() + std::mem::size_of::<Index>();
    for _ in 0..num_gates {
        for _ in 0..3 {
            let count = read_u32(r)? as usize;
            memory_size += count * gate_size;
            
            let mut buf = vec![0; count * item_size];
            r.read_exact(&mut buf)?;
        }
    }
    Ok(memory_size)
}

pub struct GateStreamedIterator<Fr:PrimeField, R:std::io::Read>(R, PhantomData<Fr>);

fn read_u32<R:std::io::Read>(r: &mut R) -> std::io::Result<u32> {
    let mut b = [0; 4];
    r.read_exact(&mut b)?;
    Ok(LittleEndian::read_u32(&b))
}


fn read_gate_part<Fr:PrimeField, R:std::io::Read>(r: &mut R) -> std::io::Result<Vec<(Num<Fr>, Index)>> {
    let sz = read_u32(r)? as usize;

    let item_size = std::mem::size_of::<Fr>() + std::mem::size_of::<u8>() + std::mem::size_of::<u32>();
    let mut buf = vec![0; sz*item_size];
    r.read_exact(&mut buf)?;
    let mut buf_ref = &buf[..];
    let mut gate_part = Vec::with_capacity(sz);
    for _ in 0..sz {
        let a = Num::<Fr>::deserialize(&mut buf_ref)?;
        let b1 = u8::deserialize(&mut buf_ref)?;
        let b2 = u32::deserialize(&mut buf_ref)?;
        let b = match b1 {
            0 => Index::Input(b2),
            1 => Index::Aux(b2),
            _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "enum elements overflow"))
        };
        gate_part.push((a,b));
    }
    Ok(gate_part)
}

impl<Fr:PrimeField, R:std::io::Read> Iterator for GateStreamedIterator<Fr, R> {
    type Item = Gate<Fr>;
    fn next(&mut self) -> Option<Self::Item> {
        let a = read_gate_part(&mut self.0).ok()?;
        let b = read_gate_part(&mut self.0).ok()?;
        let c = read_gate_part(&mut self.0).ok()?;
        Some(Gate(a,b,c))
    }
}