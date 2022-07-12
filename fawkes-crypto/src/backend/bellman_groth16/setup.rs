use std::io::Write;

use super::osrng::OsRng;
use super::*;
use crate::circuit::cs::BuildCS;

pub fn setup<
    E: Engine,
    Pub: Signal<BuildCS<E::Fr>>,
    Sec: Signal<BuildCS<E::Fr>>,
    C: Fn(Pub, Sec),
>(
    circuit: C,
) -> Parameters<E> {
    let ref mut rng = OsRng::new();

    let ref rcs = BuildCS::rc_new();

    let bcs = BellmanCS::<E, BuildCS<E::Fr>>::new(rcs.clone());

    let bp: bellman::groth16::Parameters<<E as Engine>::BE> =
        bellman::groth16::generate_random_parameters(bcs, rng)
            .unwrap()
            .to_owned();

    setup_with_params(circuit, bp)
}

pub fn setup_with_params<
    E: Engine,
    Pub: Signal<BuildCS<E::Fr>>,
    Sec: Signal<BuildCS<E::Fr>>,
    C: Fn(Pub, Sec),
>(
    circuit: C,
    bp: bellman::groth16::Parameters<<E as Engine>::BE>,
) -> Parameters<E> {
    let ref rcs = BuildCS::rc_new();
    let signal_pub = Pub::alloc(rcs, None);
    signal_pub.inputize();
    let signal_sec = Sec::alloc(rcs, None);

    circuit(signal_pub, signal_sec);

    // let bp = bellman::groth16::generate_random_parameters(bcs, rng).unwrap();
    let cs = rcs.borrow();

    let num_gates = cs.gates.len();

    let mut buf = std::io::Cursor::new(vec![]);
    let mut c = brotli::CompressorWriter::new(&mut buf, 4096, 9, 22);
    for g in cs.gates.iter() {
        c.write_all(&g.try_to_vec().unwrap()).unwrap();
    }

    c.flush().unwrap();
    drop(c);

    Parameters(
        bp,
        num_gates as u32,
        buf.into_inner(),
        cs.const_tracker.clone(),
    )
}
