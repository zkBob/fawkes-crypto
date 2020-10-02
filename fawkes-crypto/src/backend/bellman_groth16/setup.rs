use super::osrng::OsRng;
use super::*;

pub fn setup<E: Engine, Pub: Signal<E::Fr>, Sec: Signal<E::Fr>, C: Fn(Pub, Sec)>(
    circuit: C,
) -> Parameters<E> {
    let ref rcs = CS::rc_new(false);
    let signal_pub = Pub::alloc(rcs, None);
    signal_pub.inputize();
    let signal_sec = Sec::alloc(rcs, None);

    circuit(signal_pub, signal_sec);

    let bcs = BellmanCS::<E>(rcs.clone());

    let ref mut rng = OsRng::new();
    Parameters(bellman::groth16::generate_random_parameters(bcs, rng).unwrap())
}
