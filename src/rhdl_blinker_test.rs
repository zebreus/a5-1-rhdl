#[test]
fn get_blinker_synth() -> Result<()> {
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
    // Make pin constraints for the outputs
    let mut constraints = (0..4)
        .map(|i| PinConstraint {
            kind: rhdl_fpga::PinConstraintKind::Output,
            index: i,
            constraint: Constraint::Location(rhdl_fpga::bsp::alchitry::cu::LED_ARRAY_LOCATIONS[i]),
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
    std::fs::write("blink.v", &top.module)?;
    std::fs::write("blink.pcf", &pcf)?;
    eprintln!("{}", top.module);
    rhdl_fpga::bsp::alchitry::cu::synth_yosys_nextpnr_icepack(&top, &PathBuf::from("blink"))?;
    Ok(())
}