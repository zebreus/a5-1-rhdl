use rhdl::{bits::bits, kernel, Bits, Digital};
use rhdl_core::{note, Synchronous};
use rhdl_std::set_bit;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct UARTReceiver {
    // TODO: Crashes when generating verilog and there are no fields in the struct
    /// Duration of a single bit in clock cycles
    bitlength: Bits<32>,
    /// Duration of a half bit in clock cycles
    half_bitlength: Bits<32>,
}

impl UARTReceiver {
    /// Create a new UARTReceiver with a given clock speed and bit rate.
    pub fn new(clock_speed: u128, bit_rate: u128) -> Self {
        UARTReceiver {
            bitlength: Bits(clock_speed / bit_rate),
            half_bitlength: Bits(((clock_speed / bit_rate) - 1) / 2),
        }
    }
}

// tag::interface[]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct UARTReceiverInput {
    /// Reset signal. Pull high to reset the state machine.
    reset: bool,
    /// rs232 data input
    rs232: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct UARTReceiverOutput {
    /// Current output data
    data: Bits<8>,
    /// Set to high, when data is valid
    ///
    /// When this is low, the data is invalid and should be ignored.
    valid: bool,
}
// end::interface[]

// TODO: Deriving Digital for enums requires rhdl-bits to be a explicit dependency. This is a bug
// tag::state[]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub enum UARTReceiverStateEnum {
    #[default]
    Ready,
    Data(u8),
    Stop,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct UARTReceiverState {
    // TODO: Digital is missing for u32 and u 64
    counter: Bits<32>,
    /// Current output data
    data: Bits<8>,
    state: UARTReceiverStateEnum,
}
// end::state[]

impl UARTReceiverState {
    const fn default() -> Self {
        UARTReceiverState {
            data: bits::<8>(0),
            state: UARTReceiverStateEnum::Ready,
            counter: Bits(0),
        }
    }
}

// tag::synchronous[]
impl Synchronous for UARTReceiver {
    type Input = UARTReceiverInput;
    type Output = UARTReceiverOutput;
    type State = UARTReceiverState;
    type Update = uart_receiver_update;

    const INITIAL_STATE: Self::State = UARTReceiverState::default();
    const UPDATE: fn(Self, Self::State, Self::Input) -> (Self::State, Self::Output) =
        uart_receiver_update;
}
// end::synchronous[]

// TODO: Figure out how to use set_bit with a Bits index
// tag::update[]
#[kernel]
pub fn uart_receiver_update(
    params: UARTReceiver,
    state: UARTReceiverState,
    input: UARTReceiverInput,
) -> (UARTReceiverState, UARTReceiverOutput) {
    note("rs232", input.rs232);

    let next_state: UARTReceiverState = match state.state {
        UARTReceiverStateEnum::Ready => {
            if input.rs232 == false {
                UARTReceiverState {
                    data: bits::<8>(0),
                    state: UARTReceiverStateEnum::Data(0),
                    counter: params.bitlength + params.bitlength - 2,
                }
            } else {
                UARTReceiverState {
                    data: state.data,
                    state: UARTReceiverStateEnum::Ready,
                    counter: Bits::<32>(0),
                }
            }
        }
        UARTReceiverStateEnum::Data(index) => {
            let new_data = if state.counter == (params.half_bitlength) {
                set_bit::<8>(state.data, index, input.rs232)
            } else {
                state.data
            };

            if state.counter == 0 {
                if index == 7 {
                    UARTReceiverState {
                        data: new_data,
                        state: UARTReceiverStateEnum::Stop,
                        counter: state.counter,
                    }
                } else {
                    UARTReceiverState {
                        data: new_data,
                        state: UARTReceiverStateEnum::Data(index + 1),
                        counter: params.bitlength - 1,
                    }
                }
            } else {
                UARTReceiverState {
                    data: new_data,
                    state: UARTReceiverStateEnum::Data(index),
                    counter: state.counter - 1,
                }
            }
        }
        UARTReceiverStateEnum::Stop => UARTReceiverState {
            data: state.data,
            state: UARTReceiverStateEnum::Ready,
            counter: Bits::<32>(0),
        },
    };

    let output = UARTReceiverOutput {
        data: next_state.data,
        valid: state.state == UARTReceiverStateEnum::Stop,
    };

    note("next_state", next_state);
    note("valid", output.valid);
    note("data", output.data);

    (next_state, output)
}
// end::update[]

#[cfg(test)]
mod test {
    use super::{UARTReceiver, UARTReceiverInput};
    use itertools::{repeat_n, Itertools};
    use rhdl::synchronous::simulate_with_clock;
    use rhdl_core::ClockDetails;
    use rhdl_core::{note_init_db, note_take};
    use rhdl_fpga::{make_constrained_verilog, Constraint};

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
        fn from_byte(data: u8) -> impl Iterator<Item = UARTReceiverInput> {
            (0..8)
                .map(move |i| (data >> i) & 1 == 1)
                .map(|b| UARTReceiverInput::new(b))
        }
    }

    impl UARTReceiver {
        fn test_input_bit(&self, data: bool) -> impl Iterator<Item = UARTReceiverInput> {
            let bitlength = self.bitlength.0 as usize;
            // Start bit
            repeat_n(UARTReceiverInput::new(data), bitlength)
        }
        fn test_input_byte(&self, data: u8) -> impl Iterator<Item = UARTReceiverInput> {
            let bitlength = self.bitlength.0 as usize;
            // Start bit
            UARTReceiverInput::from_byte(data).flat_map(move |bit| repeat_n(bit, bitlength))
        }

        fn test_reset(&self) -> impl Iterator<Item = UARTReceiverInput> {
            [
                UARTReceiverInput {
                    reset: true,
                    rs232: true,
                },
                UARTReceiverInput::new(true),
            ]
            .into_iter()
        }

        // TODO: Implement As<usize> for Bits
        fn test_transmission(&self, data: u8) -> impl Iterator<Item = UARTReceiverInput> + '_ {
            // Start bit
            self.test_input_bit(false)
                // Data bits
                .chain(self.test_input_byte(data))
                // Stop bit
                .chain(self.test_input_bit(true))
        }
    }

    #[test]
    fn get_blinker_fpga() {
        let blinker = UARTReceiver::new(19200 /*12000000*/, 9600);
        let constraints = Vec::new();
        let top = make_constrained_verilog(
            blinker,
            constraints,
            Constraint::Location(rhdl_fpga::bsp::alchitry::cu::BASE_CLOCK_100MHZ_LOCATION),
        )
        .unwrap();
        let pcf = top.pcf().unwrap();
        std::fs::write("uart_receiver.v", &top.module).unwrap();
        std::fs::write("uart_receiver.pcf", &pcf).unwrap();
        eprintln!("{}", top.module);
    }

    fn test_uart_receiver_at_speed(speed: u128) {
        let uart_receiver = UARTReceiver::new(9600 * speed /*12000000*/, 9600);
        let input = uart_receiver
            .test_reset()
            .chain(uart_receiver.test_transmission(0b00010001))
            .chain(uart_receiver.test_input_bit(true))
            .chain(uart_receiver.test_input_bit(true))
            .chain(uart_receiver.test_input_bit(true));

        note_init_db();
        let results = simulate_with_clock(
            uart_receiver,
            input,
            ClockDetails::new("clock", 1000 * 1000, 0, false),
        )
        .collect_vec();
        let mut vcd_file = std::fs::File::create(format!("uart_receiver_{}.vcd", speed)).unwrap();
        note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();

        let valid_data = results.iter().find(|r| r.0.valid);
        let result = valid_data.unwrap();
        // Assert that the data is correct
        assert_eq!(
            result.0.data, 0b00010001,
            "Result is not {}, but {}",
            0b00010001, result.0.data
        );
        // Assert that the valid pulse comes at the right time
        assert_eq!(result.1, (1000 / 2) + 1000 * (2 + speed as u64 * 9));
    }

    #[test]
    fn test_uart_receiver_speed_1() {
        test_uart_receiver_at_speed(1);
    }
    #[test]
    fn test_uart_receiver_speed_2() {
        test_uart_receiver_at_speed(2);
    }
    #[test]
    fn test_uart_receiver_speed_3() {
        test_uart_receiver_at_speed(3);
    }
    #[test]
    fn test_uart_receiver_speed_4() {
        test_uart_receiver_at_speed(4);
    }

    // #[test]
    // fn test_uart_receiver() {
    //     let mut input: Vec<UARTReceiverInput> = vec![
    //         UARTReceiverInput {
    //             reset: true,
    //             rs232: true,
    //         },
    //         UARTReceiverInput::new(true),
    //         // start bit
    //         UARTReceiverInput::new(false),
    //     ];
    //     input.append(&mut UARTReceiverInput::from_byte(0b00010001));
    //     // stop bit
    //     input.push(UARTReceiverInput::new(true));
    //     //  Idle a bit more for a better trace
    //     input.push(UARTReceiverInput::new(true));
    //     input.push(UARTReceiverInput::new(true));
    //     input.push(UARTReceiverInput::new(true));

    //     let uart_receiver = UARTReceiver::new(19200 /*12000000*/, 9600);

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
