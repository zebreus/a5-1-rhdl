use bitvec::{array::BitArray, bitarr};
use rhdl::{bits::bits, kernel, Bits, Digital};
use rhdl_core::{note, Synchronous};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
struct UARTReceiver {
    // TODO: Crash when generating verilog and there are no fields in the struct
    dummy: bool,
}

// tag::interface[]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
struct UARTReceiverInput {
    /// Reset signal. Pull high to reset the state machine.
    reset: bool,
    /// rs232 data input
    rs232: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
struct UARTReceiverOutput {
    /// Current output data
    data: Bits<8>,
    /// Set to high, when data is valid
    ///
    /// When this is low, the data is invalid and should be ignored.
    valid: bool,
}
// end::interface[]

impl UARTReceiverInput {
    /// Create a UARTReceiverInput from a single bit of data.
    ///
    /// Reset is implicitly false.
    fn new(data: bool) -> Self {
        UARTReceiverInput {
            reset: false,
            rs232: data,
        }
    }

    /// Create a array of UARTReceiverInputs from a single byte of data.
    fn from_byte(data: u8) -> Vec<Self> {
        let mut bits = [false; 8];
        for i in 0..8 {
            bits[i] = (data >> i) & 1 == 1;
        }
        let states = bits.iter().map(|b| UARTReceiverInput::new(*b)).collect();
        return states;
    }
}

// TODO: Deriving Digital for enums requires rhdl-bits to be a explicit dependency. This is a bug
#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
enum UARTReceiverStateEnum {
    #[default]
    Ready,
    Start,
    Data(Bits<8>),
    Stop,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
struct UARTReceiverState {
    /// Current output data
    data: Bits<8>,
    // state: UARTReceiverStateEnum,
}

impl UARTReceiverState {
    const fn default() -> Self {
        UARTReceiverState {
            data: bits::<8>(0),
            // state: UARTReceiverStateEnum::Ready,
        }
    }
}

impl Synchronous for UARTReceiver {
    type Input = UARTReceiverInput;
    type Output = UARTReceiverOutput;
    type State = UARTReceiverState;
    type Update = uart_receiver_update;

    const INITIAL_STATE: Self::State = UARTReceiverState::default();
    const UPDATE: fn(Self, Self::State, Self::Input) -> (Self::State, Self::Output) =
        uart_receiver_update;
}

#[kernel]
fn uart_receiver_update(
    _params: UARTReceiver,
    state: UARTReceiverState,
    input: UARTReceiverInput,
) -> (UARTReceiverState, UARTReceiverOutput) {
    note("reset", input.reset);
    note("rs232", input.rs232);
    // let (data, _valid) = input;
    let output = UARTReceiverOutput::default();
    let new_state = state;
    note("state", new_state);
    note("valid", output.valid);
    note("data", output.data);

    (state, output)
}

#[cfg(test)]
mod test {
    use rhdl::synchronous::simulate_with_clock;
    use rhdl::{bits::bits, synchronous::simulate};
    use rhdl_core::{
        compile_design, generate_verilog, note_init_db, note_take, KernelFnKind, Synchronous,
    };
    use rhdl_core::{ClockDetails, DigitalFn};
    use rhdl_fpga::{make_constrained_verilog, Constraint, PinConstraint};

    use super::{UARTReceiver, UARTReceiverInput};

    #[test]
    fn get_blinker_fpga() {
        let blinker = UARTReceiver { dummy: true };
        // tag::constraints[]
        // Make pin constraints for the outputs
        let mut constraints = Vec::new();
        // constraints.push(PinConstraint {
        //     kind: rhdl_fpga::PinConstraintKind::Input,
        //     index: 0,
        //     constraint: Constraint::Unused,
        // });
        let top = make_constrained_verilog(
            blinker,
            constraints,
            Constraint::Location(rhdl_fpga::bsp::alchitry::cu::BASE_CLOCK_100MHZ_LOCATION),
        )
        .unwrap();
        let pcf = top.pcf().unwrap();
        std::fs::write("uart_receiver.v", &top.module).unwrap();
        std::fs::write("blink.pcf", &pcf).unwrap();
        eprintln!("{}", top.module);
        // end::constraints[]
    }
    // end::main[]

    #[test]
    fn test_uart_receiver() {
        let mut input: Vec<UARTReceiverInput> = vec![
            UARTReceiverInput {
                reset: true,
                rs232: true,
            },
            UARTReceiverInput::new(true),
            // start byte
            UARTReceiverInput::new(false),
        ];
        input.append(&mut UARTReceiverInput::from_byte(0b00010001));
        input.push(UARTReceiverInput::new(true));
        input.push(UARTReceiverInput::new(true));

        let uart_receiver = UARTReceiver { dummy: true };
        note_init_db();
        simulate_with_clock(
            uart_receiver,
            input.into_iter(),
            ClockDetails::new("clock", 1000 * 1000, 0, false),
        )
        .count();
        let mut vcd_file = std::fs::File::create("uart_receiver.vcd").unwrap();
        note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();
    }

    // #[test]
    // fn test_uart_receiver_can_receive_byte() {
    //     let mut input: Vec<UARTReceiverInput> = vec![UARTReceiverInput {
    //         reset: true,
    //         rs232: false,
    //     }];
    //     input.append(&mut UARTReceiverInput::from_byte(0x42));

    //     let uart_receiver = UARTReceiver {};
    //     note_init_db();
    //     simulate_with_clock(
    //         uart_receiver,
    //         input.into_iter(),
    //         ClockDetails::new("clock", 1000 * 1000, 0, false),
    //     )
    //     .count();
    //     let mut vcd_file = std::fs::File::create("uart_receiver.vcd").unwrap();
    //     note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();
    // }
}
