use std::{marker::PhantomData, rc::Rc};

use borsh::BorshDeserialize;
use byteorder::{ByteOrder, LittleEndian};
use ff_uint::{Num, PrimeField};

use crate::circuit::{cs::Gate, lc::Index};

use super::{engines::Engine, Parameters};

pub struct PrecomputedData<E: Engine> {
    pub(crate) gates: Rc<Vec<Gate<E::Fr>>>,
}

impl<E: Engine> PrecomputedData<E> {
    pub(crate) fn prepare(params: &Parameters<E>) -> Self {
        PrecomputedData { 
            gates: PrecomputedData::parse_gates(params)
        }
    }

    pub(crate) fn parse_gates(params: &Parameters<E>) -> Rc<Vec<Gate<E::Fr>>> {
        Rc::new(
            GateStreamedIterator(
                brotli::Decompressor::new(&params.2 as &[u8], 4096),
                PhantomData::<E::Fr>,
            )
            .collect::<Vec<_>>(),
        )
    }
}

struct GateStreamedIterator<Fr: PrimeField, R: std::io::Read>(pub R, pub PhantomData<Fr>);

fn read_u32<R: std::io::Read>(r: &mut R) -> std::io::Result<u32> {
    let mut b = [0; 4];
    r.read_exact(&mut b)?;
    Ok(LittleEndian::read_u32(&b))
}

fn read_gate_part<Fr: PrimeField, R: std::io::Read>(
    r: &mut R,
) -> std::io::Result<Vec<(Num<Fr>, Index)>> {
    let sz = read_u32(r)? as usize;

    let item_size =
        std::mem::size_of::<Fr>() + std::mem::size_of::<u8>() + std::mem::size_of::<u32>();
    let mut buf = vec![0; sz * item_size];
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
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "enum elements overflow",
                ))
            }
        };
        gate_part.push((a, b));
    }
    Ok(gate_part)
}

impl<Fr: PrimeField, R: std::io::Read> Iterator for GateStreamedIterator<Fr, R> {
    type Item = Gate<Fr>;
    fn next(&mut self) -> Option<Self::Item> {
        let a = read_gate_part(&mut self.0).ok()?;
        let b = read_gate_part(&mut self.0).ok()?;
        let c = read_gate_part(&mut self.0).ok()?;
        Some(Gate(a, b, c))
    }
}
