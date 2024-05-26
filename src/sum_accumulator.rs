use rhdl::{kernel, Digital};
use rhdl_core::{note, Synchronous};

/// Every clock cycle, the input is added to the sum of the previous inputs.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
struct SumAccumulator {}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
struct SumAccumulatorState {
    sum: u8,
}

impl Synchronous for SumAccumulator {
    type Input = u8;
    type Output = u8;
    type State = SumAccumulatorState;
    type Update = sum_accumulator_update;

    const INITIAL_STATE: Self::State = SumAccumulatorState { sum: 0 };
    const UPDATE: fn(Self, Self::State, Self::Input) -> (Self::State, Self::Output) =
        sum_accumulator_update;
}

#[kernel]
pub fn sum_accumulator_update(
    _params: SumAccumulator,
    state: SumAccumulatorState,
    input: u8,
) -> (SumAccumulatorState, u8) {
    note("input", input);
    note("state", state);
    let output = input + state.sum;
    note("output", output);
    return (SumAccumulatorState { sum: output }, output);
}

#[cfg(test)]
mod tests {
    use rhdl::synchronous::simulate;
    use rhdl_core::{note_init_db, note_take};

    use super::SumAccumulator;
    #[test]
    fn test_start_pulse_simulation() {
        let inputs = vec![4, 6, 8, 12, 3].into_iter();
        let dut = SumAccumulator {};
        note_init_db();
        simulate(dut, inputs).count();
        let mut vcd_file = std::fs::File::create("sum_accumulator.vcd").unwrap();
        note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();
    }
}
