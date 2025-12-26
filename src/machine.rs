use crate::bus::Bus;
use crate::cpu::{Cpu, Cycles};

pub struct SimpleMachine {
    pub cpu: Cpu,
    pub bus: SimpleBus,
}

pub struct SimpleBus {
    pub memory: [u8; 0x10000],
}

impl SimpleBus {
    pub fn new() -> Self {
        SimpleBus {
            memory: [0; 0x10000],
        }
    }
}

impl Bus for SimpleBus {
    fn read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn write(&mut self, addr: u16, value: u8) {
        self.memory[addr as usize] = value;
    }

    fn input(&self, _port: u8) -> u8 {
        // no operation
        0
    }

    fn output(&mut self, _port: u8, _value: u8) {
        // no operation
    }
}

impl SimpleMachine {
    pub fn new() -> Self {
        SimpleMachine {
            cpu: Cpu::new(),
            bus: SimpleBus::new(),
        }
    }

    pub fn step(&mut self) -> Cycles {
        self.cpu.step(&mut self.bus)
    }

    pub fn load(&mut self, addr: u16, data: &[u8]) {
        let start = addr as usize;
        let end = start + data.len();
        self.bus.memory[start..end].copy_from_slice(data);
    }
}
