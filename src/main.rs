use crate::cpu::Cycles;

mod bus;
mod cpu;
mod machine;

fn main() {
    let mut machine = machine::SimpleMachine::new();

    // 0000: JP 0003
    // 0003: NOP
    // 0004: NOP
    // 0005: HALT
    #[rustfmt::skip]
    let program: [u8; _] = [
        0xC3, 0x03, 0x00,
        0x00,
        0x00,
        0x76,
    ];

    machine.load(0x0000, &program);
    run_machine(&mut machine, 20);
}

fn run_machine(machine: &mut machine::SimpleMachine, max_cycles: Cycles) {
    let mut cycles: Cycles = 0;
    while cycles < max_cycles {
        cycles += machine.step();
        println!("{}", machine.cpu.to_string());
    }
}
