use itertools::Itertools;
use rhdl::{
    bits::{b3, bits},
    kernel,
    synchronous::simulate,
    Bits, Digital,
};
use rhdl_core::{note, note_init_db, note_take, Synchronous, UpdateFn};

// To make a blinker, we want to blink at a rate of 1 Hz. The clock is 100 MHz, so we want to
// toggle the output every 50 million clock cycles. We can use a Strobe with a period of 50
// million to do this.  We want the LED to be on for 1/5th of a second, which is 10 million
// clock cycles. We can use a OneShot with a duration of 10 million to do this.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct Blinker {
    pub state: Bits<3>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct BlinkerState {
    pub state: Bits<3>,
}

impl Synchronous for Blinker {
    type Input = ();
    type Output = Bits<3>;
    type State = BlinkerState;
    type Update = blinker_update;

    const INITIAL_STATE: Self::State = BlinkerState { state: bits(0b000) };
    const UPDATE: UpdateFn<Self> = blinker_update;
}

#[kernel]
pub fn blinker_update(params: Blinker, state: BlinkerState, _input: ()) -> (BlinkerState, b3) {
    // let (q_pulser, pulser_output) = pulser_update::<26>(params.pulser, state.pulser, true);
    let output = match state.state {
        Bits::<3>(0b000) => b3(0b100),
        Bits::<3>(0b100) => b3(0b010),
        Bits::<3>(0b010) => b3(0b001),
        Bits::<3>(0b001) => b3(0b000),
        _ => b3(0b000),
    };
    (BlinkerState { state: output }, output)
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct StartPulse {}

impl Synchronous for StartPulse {
    type Input = bool;
    type Output = bool;
    type State = bool;
    type Update = pulse_update;

    const INITIAL_STATE: Self::State = false;
    const UPDATE: fn(Self, Self::State, Self::Input) -> (Self::State, Self::Output) = pulse_update;
}

#[kernel]
pub fn pulse_update(_params: StartPulse, state: bool, input: bool) -> (bool, bool) {
    note("state", state);
    note("output", !state);
    note("input", input);
    (true, !state)
}

#[test]
fn test_start_pulse_simulation() {
    let input_a = std::iter::repeat(false);
    let input_b = std::iter::repeat(true);
    let input = input_a.interleave(input_b).take(100);
    let pulse = StartPulse {};
    note_init_db();
    let outputs = simulate(pulse, input).filter(|x| *x).count();
    assert_eq!(outputs, 1);
    let mut vcd_file = std::fs::File::create("sdfsdf.vcd").unwrap();
    note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();
}
