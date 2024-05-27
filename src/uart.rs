mod uart_receiver;
// use rhdl::Digital;
// use rhdl_core::Synchronous;

// #[derive(Copy, Clone, PartialEq, Eq, Debug, Digital, Default)]
// struct UART {}

// impl Synchronous for UART {
//     type Input = (u8, u8);
//     type Output = u8;
//     type State = ();
//     type Update = uart_update;

//     const INITIAL_STATE: Self::State = ();
//     const UPDATE: fn(Self, Self::State, Self::Input) -> (Self::State, Self::Output) = adder_update;
// }

// #[kernel]
// fn uart_update(_params: UART, _state: (), input: (u8, u8)) -> ((), u8) {
//     let (data, _valid) = input;
//     (_state, data)
// }
