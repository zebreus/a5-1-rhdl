use itertools::Itertools;
use rhdl::{
    bits::{b3, bits},
    kernel,
    synchronous::simulate,
    Bits, Digital,
};
use rhdl_core::{note, note_init_db, note_take, ClockDetails, Synchronous, UpdateFn};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct StartPulse {}

impl Synchronous for StartPulse {
    type Input = ();
    type Output = bool;
    type State = bool;
    type Update = pulse_update;

    const INITIAL_STATE: Self::State = false;
    const UPDATE: fn(Self, Self::State, Self::Input) -> (Self::State, Self::Output) = pulse_update;
}

#[kernel]
pub fn pulse_update(_params: StartPulse, state: bool, _input: ()) -> (bool, bool) {
    note("state", state);
    note("output", !state);
    (true, !state)
}

#[test]
fn test_start_pulse_simulation() {
    let input = std::iter::repeat(()).take(100);
    let pulse = StartPulse {};
    note_init_db();
    let outputs = simulate(pulse, input).filter(|x| *x).count();
    assert_eq!(outputs, 1);
    let mut vcd_file = std::fs::File::create("start_pulse.vcd").unwrap();
    note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();
}

// #[test]
// fn test_start_pulse_simulation() {
//     let input_a = std::iter::repeat(());
//     let input_b = std::iter::repeat(());
//     let input = input_a.interleave(input_b).take(500);
//     let pulse = StartPulse {};
//     note_init_db();
//     let outputs = simulate(pulse, input).filter(|x| *x).count();
//     assert_eq!(outputs, 1);
//     let mut vcd_file = std::fs::File::create("start_pulse.vcd").unwrap();
//     let clock = ClockDetails::new("clk", 100, 0, false);
//     note_take()
//         .unwrap()
//         .dump_vcd(&[clock], &mut vcd_file)
//         .unwrap();
// }
