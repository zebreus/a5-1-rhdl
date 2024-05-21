// tag::inverter[]
use rhdl::{kernel, Digital};
use rhdl_core::{note, Synchronous};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct Inverter {}

impl Synchronous for Inverter {
    type Input = bool;
    type Output = bool;
    type State = ();
    type Update = inverter_update;

    const INITIAL_STATE: Self::State = ();
    const UPDATE: fn(Self, Self::State, Self::Input) -> (Self::State, Self::Output) =
        inverter_update;
}

#[kernel]
pub fn inverter_update(_params: Inverter, _state: (), input: bool) -> ((), bool) {
    note("input", input);
    note("output", !input);
    ((), !input)
}
// end::inverter[]

#[cfg(test)]
mod test {
    use rhdl::synchronous::simulate;
    use rhdl_core::{note_init_db, note_take};

    use super::Inverter;

    // tag::test[]
    #[test]
    fn test_inverter() {
        let input: Vec<bool> = vec![
            true, true, false, true, false, false, false, false, true, true, false, false, true,
            true,
        ];
        let inverter = Inverter {};
        note_init_db();
        simulate(inverter, input.into_iter()).count();
        let mut vcd_file = std::fs::File::create("inverter.vcd").unwrap();
        note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();
    }
    // end::test[]

    // tag::test_expected_output[]
    #[test]
    fn test_inverter_against_expected_output() {
        let input: Vec<bool> = vec![
            true, true, false, true, false, false, false, false, true, true, false, false, true,
            true,
        ];
        let inverted_input = input.clone().into_iter().map(|x| !x).collect::<Vec<_>>();
        let inverter = Inverter {};
        let output: Vec<_> = simulate(inverter, input.clone().into_iter()).collect();
        assert_eq!(output, inverted_input);
    }
    // end::test_expected_output[]
}
