//! Try to build a minimal example for the Adder module

use rhdl::{kernel, Digital};
use rhdl_core::{note, Synchronous};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
struct Adder {}

impl Synchronous for Adder {
    type Input = (u8, u8);
    type Output = u8;
    type State = ();
    type Update = adder_update;

    const INITIAL_STATE: Self::State = ();
    const UPDATE: fn(Self, Self::State, Self::Input) -> (Self::State, Self::Output) = adder_update;
}

#[kernel]
pub fn adder_update(_params: Adder, state: (), input: (u8, u8)) -> ((), u8) {
    // let result = match input {
    //     (true, true) => (!state, !state),
    //     (true, false) => (true, true),
    //     (false, true) => (false, false),
    //     (false, false) => (state, state),
    // };
    let output = input.0 + input.1;
    note("a", input.0);
    note("b", input.1);
    note("output", output);
    // note("output", result.1);
    // return result;
    return ((), output);
}

#[cfg(test)]
mod tests {
    use rhdl::synchronous::simulate;
    use rhdl_core::{note_init_db, note_take};

    use super::Adder;
    #[test]
    fn test_start_pulse_simulation() {
        let input = vec![(1, 1), (2, 5), (3, 1), (2, 20)].into_iter();
        let pulse = Adder {};
        note_init_db();
        simulate(pulse, input).count();
        let mut vcd_file = std::fs::File::create("adder.vcd").unwrap();
        note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();
    }
}
