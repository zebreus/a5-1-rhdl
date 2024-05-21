use itertools::Itertools;
use rhdl::{
    bits::{b3, bits},
    kernel,
    synchronous::simulate,
    Bits, Digital,
};
use rhdl_core::{note, note_init_db, note_take, ClockDetails, Synchronous, UpdateFn};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
struct ClockThing {}

impl Synchronous for ClockThing {
    type Input = ();
    type Output = bool;
    type State = bool;
    type Update = pulse_update;

    const INITIAL_STATE: Self::State = false;
    const UPDATE: fn(Self, Self::State, Self::Input) -> (Self::State, Self::Output) = pulse_update;
}

#[kernel]
pub fn pulse_update(_params: ClockThing, state: bool, _input: ()) -> (bool, bool) {
    note("state", state);
    note("output", !state);
    (!state, !state)
}

#[test]
fn test_start_pulse_simulation() {
    let input = std::iter::repeat(()).take(100);
    let pulse = ClockThing {};
    note_init_db();
    let outputs = simulate(pulse, input).count();
    assert_eq!(outputs, 100);
    let mut vcd_file = std::fs::File::create("clock_thing.vcd").unwrap();
    note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();
}
