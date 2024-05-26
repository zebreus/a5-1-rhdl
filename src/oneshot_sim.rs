//! Try to build a minimal example for the JKFF module

#[cfg(test)]
mod test {
    use rhdl::{
        bits::bits,
        synchronous::{simulate, OneShot},
    };
    use rhdl_core::{note_init_db, note_take};

    #[test]
    fn test_start_pulse_simulation() {
        let inputs = vec![false, true, false, false, false, false, false, false].into_iter();
        let dut = OneShot::<26> { duration: bits(3) };
        note_init_db();
        simulate(dut, inputs).count();
        let mut vcd_file = std::fs::File::create("oneshot.vcd").unwrap();
        note_take().unwrap().dump_vcd(&[], &mut vcd_file).unwrap();
    }
}
