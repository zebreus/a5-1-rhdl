// tag::main[]
use rhdl::{bits::bits, kernel, Bits, Digital};
use rhdl_core::{note, Synchronous};
use rhdl_std::{get_bit, set_bit};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct ShiftRegister {
    pub state: Bits<4>,
}

impl Synchronous for ShiftRegister {
    type Input = bool;
    type Output = bool;
    type State = Bits<4>;
    type Update = shift_register_update;

    const INITIAL_STATE: Self::State = bits(0b0000);
    const UPDATE: rhdl_core::UpdateFn<Self> = shift_register_update;
}

#[kernel]
pub fn shift_register_update(
    _params: ShiftRegister,
    state: Bits<4>,
    input: bool,
) -> (Bits<4>, bool) {
    // This line simulates, but fails when synthesizing
    // let mut new_state: Bits<4> = state << 1;
    // This line simulates, but fails when synthesizing, because of `invalid digit found in string`
    // let new_state: Bits<4> = state >> 1u128;
    let mut new_state: Bits<4> = state << bits::<4>(1);
    let output_bit = get_bit::<4>(state, 3);
    new_state = set_bit::<4>(new_state, 0, input);
    note("state", state);
    note("output", output_bit);
    note("input", input);
    (new_state, output_bit)
}

#[cfg(test)]
mod test {
    use rhdl::{bits::bits, synchronous::simulate};
    use rhdl_core::DigitalFn;
    use rhdl_core::{
        compile_design, generate_verilog, note_init_db, note_take, KernelFnKind, Synchronous,
    };

    use super::ShiftRegister;

    // tag::test[]
    #[test]
    fn test_shift_register() {
        let input: Vec<bool> = vec![
            true, true, false, true, false, false, false, false, true, true, false, false, true,
            true, false, false, false, false, false,
        ];
        let inverter = ShiftRegister {
            state: bits(0b0000),
        };
        note_init_db();
        simulate(inverter, input.into_iter()).count();
        let mut vcd_file = std::fs::File::create("shift_register.vcd").unwrap();
        note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();
    }
    // end::test[]
    // end::main[]

    // tag::generate_verilog[]
    #[test]
    fn test_generate_verilog() {
        let Some(KernelFnKind::Kernel(kernel)) =
            <ShiftRegister as Synchronous>::Update::kernel_fn()
        else {
            panic!("No kernel function found");
        };
        let design = &compile_design(kernel).unwrap();
        let verilog = generate_verilog(design).unwrap();
        let module_code = format!("{}", verilog);
        std::fs::write("shift_register.v", module_code).unwrap();
    }
    // end::generate_verilog[]

    // tag::generate_verilog_module[]
    #[test]
    fn test_generate_verilog_module() {
        let Some(KernelFnKind::Kernel(kernel)) =
            <ShiftRegister as Synchronous>::Update::kernel_fn()
        else {
            panic!("No kernel function found");
        };
        let design = &compile_design(kernel).unwrap();
        let verilog = generate_verilog(design).unwrap();
        let module_code = format!("{}", verilog);
        std::fs::write("shift_register.v", module_code).unwrap();
    }
    // end::generate_verilog_module[]

    // tag::main[]
}
// end::main[]
