//! Try to build a minimal example for the JKFF module

use rhdl::{kernel, Digital};
use rhdl_core::{note, Synchronous};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
struct JKFF {}

impl Synchronous for JKFF {
    type Input = (bool, bool);
    type Output = bool;
    type State = bool;
    type Update = jkff_update;

    const INITIAL_STATE: Self::State = false;
    const UPDATE: fn(Self, Self::State, Self::Input) -> (Self::State, Self::Output) = jkff_update;
}

#[kernel]
pub fn jkff_update(_params: JKFF, state: bool, input: (bool, bool)) -> (bool, bool) {
    let result = match input {
        (true, true) => (!state, !state),
        (true, false) => (true, true),
        (false, true) => (false, false),
        (false, false) => (state, state),
    };
    note("J", input.0);
    note("K", input.1);
    note("output", result.1);
    return result;
}

#[cfg(test)]
mod tests {
    use rhdl::synchronous::simulate;
    use rhdl_core::{note_init_db, note_take};

    use super::JKFF;
    #[test]
    fn test_start_pulse_simulation() {
        let input = vec![
            (false, false),
            (true, false),
            (false, false),
            (false, true),
            (false, false),
        ]
        .into_iter();
        let pulse = JKFF {};
        note_init_db();
        simulate(pulse, input).count();
        let mut vcd_file = std::fs::File::create("jkff.vcd").unwrap();
        note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();
    }
}
