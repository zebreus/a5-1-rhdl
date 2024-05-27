#[cfg(test)]
mod tests {
    // tag::main[]
    use std::path::PathBuf;

    use rhdl::{
        bits::bits,
        synchronous::{Blinker, OneShot, Pulser, Strobe},
    };
    use rhdl_core::{compile_design, generate_verilog, DigitalFn, KernelFnKind, Synchronous};
    use rhdl_fpga::{make_constrained_verilog, Constraint, PinConstraint, Result};

    #[test]
    fn get_blinker_fpga() -> Result<()> {
        // tag::blinker[]
        let blinker = Blinker {
            pulser: Pulser::<26> {
                one_shot: OneShot::<26> {
                    duration: bits(10_000_000),
                },
                strobe: Strobe::<26> {
                    period: bits(50_000_000),
                },
            },
        };
        // end::blinker[]

        // tag::constraints[]
        // Make pin constraints for the outputs
        let mut constraints = (0..4)
            .map(|i| PinConstraint {
                kind: rhdl_fpga::PinConstraintKind::Output,
                index: i,
                constraint: Constraint::Location(
                    rhdl_fpga::bsp::alchitry::cu::LED_ARRAY_LOCATIONS[i],
                ),
            })
            .collect::<Vec<_>>();
        constraints.push(PinConstraint {
            kind: rhdl_fpga::PinConstraintKind::Input,
            index: 0,
            constraint: Constraint::Unused,
        });
        let top = make_constrained_verilog(
            blinker,
            constraints,
            Constraint::Location(rhdl_fpga::bsp::alchitry::cu::BASE_CLOCK_100MHZ_LOCATION),
        )?;
        let pcf = top.pcf()?;
        std::fs::write("blinker_fpga.v", &top.module)?;
        std::fs::write("blink.pcf", &pcf)?;
        eprintln!("{}", top.module);
        rhdl_fpga::bsp::alchitry::cu::synth_yosys_nextpnr_icepack(&top, &PathBuf::from("blink"))?;
        // end::constraints[]
        Ok(())
    }
    // end::main[]

    #[test]
    fn get_blinker_module() {
        // Blinker is unused, because compile_design only requires the kernel function
        let _blinker = Blinker {
            pulser: Pulser::<26> {
                one_shot: OneShot::<26> {
                    duration: bits(10_000_000),
                },
                strobe: Strobe::<26> {
                    period: bits(50_000_000),
                },
            },
        };

        let Some(KernelFnKind::Kernel(kernel)) = <Blinker as Synchronous>::Update::kernel_fn()
        else {
            panic!("No kernel function found");
        };
        let design = &compile_design(kernel).unwrap();
        let verilog = generate_verilog(design).unwrap();
        let module_code = format!("{}", verilog);
        std::fs::write("blinker_module.v", module_code).unwrap();
    }

    // #[test]
    // fn get_blinker_circuit() {
    //     let blinker = Blinker {
    //         pulser: Pulser::<26> {
    //             one_shot: OneShot::<26> {
    //                 duration: bits(10_000_000),
    //             },
    //             strobe: Strobe::<26> {
    //                 period: bits(50_000_000),
    //             },
    //         },
    //     };
    //
    //     let Some(KernelFnKind::Kernel(kernel)) = <Blinker as Synchronous>::Update::kernel_fn() else {
    //         panic!("No kernel function found");
    //     };
    //     let design = &compile_design(kernel).unwrap();
    //     let verilog = generate_verilog(design).unwrap();
    //     let module_code = format!("{}", verilog);
    //     std::fs::write("blinker_module.v", module_code).unwrap();
    // }
}
