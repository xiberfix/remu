use crate::bus::Bus;
use crate::cpu::Cycles;

mod bus;
mod cpu;
mod machine;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: {} <test>", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];
    println!("test: {}", path);
    let cycles = run_test(path);
    println!("\ncycles: {}", cycles);
}

fn run_test(path: &str) -> Cycles {
    let mut machine = machine::SimpleMachine::new();
    let mut cycles: Cycles = 0;

    let program = std::fs::read(path).expect("failed to read test file");
    machine.load(0x0100, &program);
    machine.cpu.pc = 0x0100;

    // CP/M hook
    machine.bus.write(0x0005, 0xC9); // RET
    machine.bus.write(0x0000, 0x76); // HALT

    loop {
        if machine.cpu.state == cpu::State::Halted {
            break;
        }

        // CP/M hook
        if machine.cpu.pc == 0x0005 {
            process_cpm_call(&mut machine);
        }

        cycles += machine.step();
    }

    cycles
}

fn process_cpm_call(machine: &mut machine::SimpleMachine) {
    match machine.cpu.c {
        0x00 => {
            // system reset
            machine.cpu.pc = 0x0000;
        }
        0x02 => {
            // character output
            let char = machine.cpu.e;
            print!("{}", char as char);
        }
        0x09 => {
            // string output
            let mut addr = machine.cpu.de();
            loop {
                let char = machine.bus.read(addr);
                if char == b'$' {
                    break;
                }
                print!("{}", char as char);
                addr = addr.wrapping_add(1);
            }
        }
        _ => {
            println!("unsupported {:02X} CP/M call", machine.cpu.c);
        }
    }
}
