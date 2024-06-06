mod uart_receiver;
mod uart_sender;

use rhdl::{kernel, Bits, Digital};
use rhdl_core::{note, note_pop_path, note_push_path, Synchronous};
use uart_receiver::{uart_receiver_update, UartReceiver, UartReceiverInput, UartReceiverState};
use uart_sender::{uart_sender_update, UartSender, UartSenderInput, UartSenderState};

/// Combines a UartReceiver and a UartSender into a single Uart component.
///
/// Not really sure, whether this is a good idea. But the example in the exercises also did it this way.
/// At least it is an exercise for combining multiple components into a single one.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct Uart {
    receiver: UartReceiver,
    sender: UartSender,
}

impl Uart {
    #[allow(dead_code)]
    /// Create a new Uart with a given clock speed and bit rate.
    pub fn new(clock_speed: u128, bit_rate: u128) -> Self {
        Uart {
            receiver: UartReceiver::new(clock_speed, bit_rate),
            sender: UartSender::new(clock_speed, bit_rate),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct UartInput {
    /// Reset signal. Pull high to reset the state machine.
    reset: bool,
    /// Data input line
    rx: bool,
    /// Data to transmit
    data: Bits<8>,
    /// Pulse high to transmit data
    start: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct UartOutput {
    /// Current output data
    received_data: Bits<8>,
    /// Set to true when new data was received transmitted
    valid: bool,
    /// Set to true when new data can be transmitted
    ready: bool,
    /// Data output line
    tx: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
pub struct UartState {
    receiver: UartReceiverState,
    sender: UartSenderState,
}

impl UartState {
    const fn default() -> Self {
        UartState {
            receiver: UartReceiverState::default(),
            sender: UartSenderState::default(),
        }
    }
}

impl Synchronous for Uart {
    type Input = UartInput;
    type Output = UartOutput;
    type State = UartState;
    type Update = uart_update;

    const INITIAL_STATE: Self::State = UartState::default();
    const UPDATE: fn(Self, Self::State, Self::Input) -> (Self::State, Self::Output) = uart_update;
}

#[kernel]
pub fn uart_update(params: Uart, state: UartState, input: UartInput) -> (UartState, UartOutput) {
    note("input", input);
    // TODO: Allow note_push_path in rhdl-macro-core
    // For now this is only a uncommitted change
    note_push_path("receiver");
    let (receiver_state, receiver_output) = uart_receiver_update(
        params.receiver,
        state.receiver,
        UartReceiverInput {
            reset: input.reset,
            rs232: input.rx,
        },
    );
    note_pop_path();
    note_push_path("sender");
    let (sender_state, sender_output) = uart_sender_update(
        params.sender,
        state.sender,
        UartSenderInput {
            reset: input.reset,
            data: input.data,
            ready: input.start,
        },
    );
    note_pop_path();

    let next_state: UartState = UartState {
        receiver: receiver_state,
        sender: sender_state,
    };
    let output = UartOutput {
        received_data: receiver_output.data,
        valid: receiver_output.valid,
        ready: sender_output.ready,
        tx: sender_output.rs232,
    };

    note("next_state", next_state);
    note("output", output);

    (next_state, output)
}

#[cfg(test)]
mod test {
    use crate::uart::Uart;

    use super::{UartInput, UartOutput};
    use rhdl::synchronous::{simulate_first_cycle, simulate_one_cycle};
    use rhdl_bits::bits;
    use rhdl_core::ClockDetails;
    use rhdl_core::{note_init_db, note_take};
    use rhdl_fpga::{make_constrained_verilog, Constraint};

    impl UartInput {
        fn new() -> Self {
            UartInput {
                reset: true,
                rx: true,
                data: Default::default(),
                start: false,
            }
        }

        fn reset() -> Self {
            UartInput {
                reset: true,
                rx: true,
                data: Default::default(),
                start: false,
            }
        }

        fn loopback(&self, previous_output: &UartOutput) -> Self {
            UartInput {
                reset: self.reset,
                rx: previous_output.tx,
                data: self.data,
                start: self.start,
            }
        }
    }

    // impl Uart {}

    #[test]
    fn synthesize_for_fpga() {
        let blinker = Uart::new(19200 /*12000000*/, 9600);
        let constraints = Vec::new();
        let top = make_constrained_verilog(
            blinker,
            constraints,
            Constraint::Location(rhdl_fpga::bsp::alchitry::cu::BASE_CLOCK_100MHZ_LOCATION),
        )
        .unwrap();
        let pcf = top.pcf().unwrap();
        std::fs::write("uart.v", &top.module).unwrap();
        std::fs::write("uart.pcf", &pcf).unwrap();
        eprintln!("{}", top.module);
    }

    fn test_uart_at_speed(speed: u128) {
        let uart = Uart::new(9600 * speed /*12000000*/, 9600);

        note_init_db();
        let clock = ClockDetails::new("clock", 1000 * 1000, 0, false);
        let mut outputs = Vec::new();
        let (mut state, mut output, mut time) =
            simulate_first_cycle(uart, UartInput::reset(), &clock);
        outputs.push((output, time));
        (state, output, time) = simulate_one_cycle(
            uart,
            UartInput {
                reset: false,
                rx: true,
                data: bits::<8>(0b010100011),
                start: true,
            },
            state,
            time,
            &clock,
        );

        for _ in 0..(20 * speed) {
            (state, output, time) = simulate_one_cycle(
                uart,
                UartInput::new().loopback(&output),
                state,
                time,
                &clock,
            );
            outputs.push((output, time));
        }

        let mut vcd_file = std::fs::File::create(format!("uart_{}.vcd", speed)).unwrap();
        note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();

        // Assert that the reflected data is read correctly
        let output_with_valid_input = outputs.iter().find(|(output, _)| output.valid).unwrap();
        assert!(output_with_valid_input.0.received_data == bits::<8>(0b010100011));
    }

    #[test]
    fn test_uart_sender_speed_1() {
        test_uart_at_speed(1);
    }
    #[test]
    fn test_uart_sender_speed_2() {
        test_uart_at_speed(2);
    }
    #[test]
    fn test_uart_sender_speed_3() {
        test_uart_at_speed(3);
    }
    #[test]
    fn test_uart_sender_speed_4() {
        test_uart_at_speed(4);
    }
}
