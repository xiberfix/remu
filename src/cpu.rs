use crate::bus::Bus;

pub type Cycles = u64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    Running,
    Halted,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Flags {
    zero: bool,
    sign: bool,
    parity: bool,
    aux_carry: bool,
    carry: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cpu {
    pub a: u8,
    pub flags: Flags,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
    pub state: State,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            a: 0,
            flags: Flags {
                zero: false,
                sign: false,
                parity: false,
                aux_carry: false,
                carry: false,
            },
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
            state: State::Running,
        }
    }

    pub fn step(&mut self, bus: &mut dyn Bus) -> Cycles {
        if self.state == State::Halted {
            return 4;
        }

        let opcode = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);

        match opcode {
            // NOP
            0x00 => 4,
            // HALT
            0x76 => {
                self.state = State::Halted;
                4
            }
            // JP addr
            0xC3 => {
                let addr = bus.read_word(self.pc);
                self.pc = addr;
                10
            }
            _ => {
                panic!(
                    "unimplemented opcode: {:02X} at PC={:04X}",
                    opcode,
                    self.pc.wrapping_sub(1)
                );
            }
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "PC={:04X} SP={:04X} A={:02X} BC={:02X}{:02X} DE={:02X}{:02X} HL={:02X}{:02X} F=[Z:{} S:{} P:{} AC:{} C:{}] ({:?})",
            self.pc,
            self.sp,
            self.a,
            self.b,
            self.c,
            self.d,
            self.e,
            self.h,
            self.l,
            self.flags.zero as u8,
            self.flags.sign as u8,
            self.flags.parity as u8,
            self.flags.aux_carry as u8,
            self.flags.carry as u8,
            self.state,
        )
    }
}
