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

        let opcode = self.fetch_byte(bus);

        match opcode {
            // NOP
            0x00 => 4,

            // HALT
            0x76 => {
                self.state = State::Halted;
                4
            }

            // LD r,r'
            0x40..=0x7F => {
                let src = opcode & 0x07;
                let dest = (opcode >> 3) & 0x07;

                let value = self.reg(src, bus);
                self.set_reg(dest, value, bus);

                if src == 0x06 || dest == 0x06 { 7 } else { 5 }
            }

            // LD r,n
            0x06 | 0x0E | 0x16 | 0x1E | 0x26 | 0x2E | 0x36 | 0x3E => {
                let dest = (opcode >> 3) & 0x07;

                let value = self.fetch_byte(bus);
                self.set_reg(dest, value, bus);

                if dest == 0x06 { 10 } else { 7 }
            }

            // LD A,(BC)
            0x0A => {
                self.a = bus.read(self.bc());
                7
            }
            // LD A,(DE)
            0x1A => {
                self.a = bus.read(self.de());
                7
            }
            // LD A,(nn)
            0x3A => {
                self.a = bus.read(self.fetch_word(bus));
                13
            }

            // LD (BC),A
            0x02 => {
                bus.write(self.bc(), self.a);
                7
            }
            // LD (DE),A
            0x12 => {
                bus.write(self.de(), self.a);
                7
            }
            // LD (nn),A
            0x32 => {
                bus.write(self.fetch_word(bus), self.a);
                13
            }

            // JP addr
            0xC3 => {
                self.pc = self.fetch_word(bus);
                10
            }
            // RET
            0xC9 => {
                let addr = bus.read_word(self.sp);
                self.sp = self.sp.wrapping_add(2);
                self.pc = addr;
                10
            }
            // CALL addr
            0xCD => {
                let addr = self.fetch_word(bus);
                self.sp = self.sp.wrapping_sub(2);
                bus.write_word(self.sp, self.sp);
                self.pc = addr;
                17
            }

            _ => {
                panic!(
                    "unimplemented opcode: {:02X} at PC={:04X}",
                    opcode,
                    self.pc.wrapping_sub(1),
                );
            }
        }
    }

    fn fetch_byte(&mut self, bus: &dyn Bus) -> u8 {
        let byte = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    fn fetch_word(&mut self, bus: &dyn Bus) -> u16 {
        let word = bus.read_word(self.pc);
        self.pc = self.pc.wrapping_add(2);
        word
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

    fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }

    fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0xFF) as u8;
    }

    fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0xFF) as u8;
    }

    fn reg(&self, code: u8, bus: &dyn Bus) -> u8 {
        match code {
            0 => self.b,
            1 => self.c,
            2 => self.d,
            3 => self.e,
            4 => self.h,
            5 => self.l,
            6 => bus.read(self.hl()),
            7 => self.a,
            _ => unreachable!(),
        }
    }

    fn set_reg(&mut self, code: u8, value: u8, bus: &mut dyn Bus) {
        match code {
            0 => self.b = value,
            1 => self.c = value,
            2 => self.d = value,
            3 => self.e = value,
            4 => self.h = value,
            5 => self.l = value,
            6 => bus.write(self.hl(), value),
            7 => self.a = value,
            _ => unreachable!(),
        }
    }
}
