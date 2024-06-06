use rhdl::{bits::bits, kernel, Bits, Digital};
use rhdl_core::{note, Synchronous};
use rhdl_std::get_bit;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct UartSender {
    // TODO: Crashes when generating verilog and there are no fields in the struct
    /// Duration of a single bit in clock cycles
    bitlength: Bits<32>,
    /// Duration of a half bit in clock cycles
    half_bitlength: Bits<32>,
}

impl UartSender {
    /// Create a new UartSender with a given clock speed and bit rate.
    #[allow(dead_code)]
    pub fn new(clock_speed: u128, bit_rate: u128) -> Self {
        UartSender {
            bitlength: Bits(clock_speed / bit_rate),
            half_bitlength: Bits(((clock_speed / bit_rate) - 1) / 2),
        }
    }
}

// tag::interface[]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct UartSenderInput {
    /// Reset signal. Pull high to reset the state machine.
    reset: bool,
    /// Bit to send
    data: Bits<8>,
    /// Set to high to start the transmission
    ///
    /// Only works if the sender is ready
    ready: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct UartSenderOutput {
    /// Set to high if the Sender is ready for the next byte
    ready: bool,
    /// rs232 data input
    rs232: bool,
}
// end::interface[]

// TODO: Deriving Digital for enums requires rhdl-bits to be a explicit dependency. This is a bug
// tag::state[]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub enum UartSenderStateEnum {
    #[default]
    Idle,
    Start,
    Data(u8),
    Stop,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct UartSenderState {
    /// Counter to wait for the correct length to send the next bit
    counter: Bits<32>,
    /// The data we will be sending
    data: Bits<8>,
    /// The current state of the sender
    state: UartSenderStateEnum,
}
// end::state[]

impl UartSenderState {
    const fn default() -> Self {
        UartSenderState {
            data: bits::<8>(0),
            state: UartSenderStateEnum::Idle,
            counter: Bits(0),
        }
    }
}

// tag::synchronous[]
impl Synchronous for UartSender {
    type Input = UartSenderInput;
    type Output = UartSenderOutput;
    type State = UartSenderState;
    type Update = uart_sender_update;

    const INITIAL_STATE: Self::State = UartSenderState::default();
    const UPDATE: fn(Self, Self::State, Self::Input) -> (Self::State, Self::Output) =
        uart_sender_update;
}
// end::synchronous[]

// TODO: Figure out how to use set_bit with a Bits index
// tag::update[]
#[kernel]
pub fn uart_sender_update(
    params: UartSender,
    state: UartSenderState,
    input: UartSenderInput,
) -> (UartSenderState, UartSenderOutput) {
    note("input_data", input.data);
    note("input_valid", input.ready);

    let next_state: UartSenderState = match state.state {
        UartSenderStateEnum::Idle => {
            if input.ready {
                UartSenderState {
                    data: input.data,
                    state: UartSenderStateEnum::Start,
                    counter: params.bitlength - 1,
                }
            } else {
                UartSenderState {
                    data: state.data,
                    state: state.state,
                    counter: state.counter,
                }
            }
        }
        UartSenderStateEnum::Start => {
            if state.counter == 0 {
                UartSenderState {
                    data: state.data,
                    state: UartSenderStateEnum::Data(0),
                    counter: params.bitlength - 1,
                }
            } else {
                UartSenderState {
                    data: state.data,
                    state: UartSenderStateEnum::Start,
                    counter: state.counter - 1,
                }
            }
        }
        UartSenderStateEnum::Stop => {
            if state.counter == 0 {
                UartSenderState {
                    data: state.data,
                    state: UartSenderStateEnum::Idle,
                    counter: params.bitlength - 1,
                }
            } else {
                UartSenderState {
                    data: state.data,
                    state: UartSenderStateEnum::Stop,
                    counter: state.counter - 1,
                }
            }
        }
        UartSenderStateEnum::Data(index) => {
            if state.counter == 0 {
                if index == 7 {
                    UartSenderState {
                        data: state.data,
                        state: UartSenderStateEnum::Stop,
                        counter: params.bitlength - 1,
                    }
                } else {
                    UartSenderState {
                        data: state.data,
                        state: UartSenderStateEnum::Data(index + 1),
                        counter: params.bitlength - 1,
                    }
                }
            } else {
                UartSenderState {
                    data: state.data,
                    state: UartSenderStateEnum::Data(index),
                    counter: state.counter - 1,
                }
            }
        }
    };

    let output = match next_state.state {
        UartSenderStateEnum::Idle => UartSenderOutput {
            ready: true,
            rs232: true,
        },
        UartSenderStateEnum::Start => UartSenderOutput {
            ready: false,
            rs232: false,
        },
        UartSenderStateEnum::Data(index) => UartSenderOutput {
            ready: false,
            rs232: get_bit::<8>(next_state.data, index),
        },
        UartSenderStateEnum::Stop => UartSenderOutput {
            ready: false,
            rs232: true,
        },
    };

    note("next_state", next_state);
    note("output__rs232", output.rs232);
    note("output__ready_for_data", output.ready);

    (next_state, output)
}
// end::update[]

#[cfg(test)]
mod test {
    use super::{UartSender, UartSenderInput};
    use itertools::{repeat_n, Itertools};
    use rhdl::bits::b8;
    use rhdl::synchronous::simulate_with_clock;
    use rhdl_bits::bits;
    use rhdl_core::ClockDetails;
    use rhdl_core::{note_init_db, note_take};
    use rhdl_fpga::{make_constrained_verilog, Constraint};

    impl UartSenderInput {
        /// Create a UartSenderInput from a single bit of data.
        ///
        /// Reset is implicitly false.
        fn new() -> Self {
            UartSenderInput {
                reset: false,
                data: b8::default(),
                ready: false,
            }
        }

        fn reset() -> Self {
            UartSenderInput {
                reset: true,
                data: b8::default(),
                ready: false,
            }
        }

        fn transmit(data: u8) -> Self {
            UartSenderInput {
                reset: false,
                data: bits::<8>(data as u128),
                ready: true,
            }
        }
    }

    impl UartSender {
        fn test_set_byte(&self, data: u8) -> impl Iterator<Item = UartSenderInput> {
            repeat_n(UartSenderInput::transmit(data), 1)
        }
        fn test_wait_one_cycle(&self) -> impl Iterator<Item = UartSenderInput> {
            // Start bit
            repeat_n(UartSenderInput::new(), 1)
        }
        fn test_wait_one_transmission(&self) -> impl Iterator<Item = UartSenderInput> {
            let bitlength = self.bitlength.0 as usize;
            // Start bit
            repeat_n(UartSenderInput::new(), bitlength * 10)
        }

        fn test_reset(&self) -> impl Iterator<Item = UartSenderInput> {
            [UartSenderInput::reset(), UartSenderInput::new()].into_iter()
        }

        // TODO: Implement As<usize> for Bits
        fn test_transmission(&self, data: u8) -> impl Iterator<Item = UartSenderInput> + '_ {
            // Start bit
            self.test_set_byte(data)
                // Data bits
                .chain(self.test_wait_one_transmission())
        }
    }

    #[test]
    fn synthesize_for_fpga() {
        let blinker = UartSender::new(19200 /*12000000*/, 9600);
        let constraints = Vec::new();
        let top = make_constrained_verilog(
            blinker,
            constraints,
            Constraint::Location(rhdl_fpga::bsp::alchitry::cu::BASE_CLOCK_100MHZ_LOCATION),
        )
        .unwrap();
        let pcf = top.pcf().unwrap();
        std::fs::write("uart_sender.v", &top.module).unwrap();
        std::fs::write("uart_sender.pcf", &pcf).unwrap();
        eprintln!("{}", top.module);
    }

    fn test_uart_sender_at_speed(speed: u128) {
        let uart_sender = UartSender::new(9600 * speed /*12000000*/, 9600);
        let input = uart_sender
            .test_reset()
            .chain(uart_sender.test_transmission(0b01010011))
            .chain(uart_sender.test_wait_one_cycle());

        note_init_db();
        let results = simulate_with_clock(
            uart_sender,
            input,
            ClockDetails::new("clock", 1000 * 1000, 0, false),
        )
        .collect_vec();
        let mut vcd_file = std::fs::File::create(format!("uart_sender_{}.vcd", speed)).unwrap();
        note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();

        // Assert that we start high
        assert_eq!(results[1].0.rs232, true);

        let start_cycle = 2usize;
        let middle_start = start_cycle + (speed / 2) as usize;

        // Assert that the start byte is low
        assert_eq!(results[middle_start].0.rs232, false);

        // Assert that the data is correct
        assert_eq!(results[middle_start + (1 * speed as usize)].0.rs232, true);
        assert_eq!(results[middle_start + (2 * speed as usize)].0.rs232, true);
        assert_eq!(results[middle_start + (3 * speed as usize)].0.rs232, false);
        assert_eq!(results[middle_start + (4 * speed as usize)].0.rs232, false);
        assert_eq!(results[middle_start + (5 * speed as usize)].0.rs232, true);
        assert_eq!(results[middle_start + (6 * speed as usize)].0.rs232, false);
        assert_eq!(results[middle_start + (7 * speed as usize)].0.rs232, true);
        assert_eq!(results[middle_start + (8 * speed as usize)].0.rs232, false);

        // Assert that the stop bit is high
        assert_eq!(results[middle_start + (9 * speed as usize)].0.rs232, true);

        // Assert that ready goes high after the stop bit
        assert_eq!(
            results[start_cycle + (10 * speed as usize) - 1].0.ready,
            false
        );
        assert_eq!(results[start_cycle + (10 * speed as usize)].0.ready, true);
    }

    #[test]
    fn test_uart_sender_speed_1() {
        test_uart_sender_at_speed(1);
    }
    #[test]
    fn test_uart_sender_speed_2() {
        test_uart_sender_at_speed(2);
    }
    #[test]
    fn test_uart_sender_speed_3() {
        test_uart_sender_at_speed(3);
    }
    #[test]
    fn test_uart_sender_speed_4() {
        test_uart_sender_at_speed(4);
    }
}
