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
    pub iff: bool,
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
            iff: true,
        }
    }

    pub fn step(&mut self, bus: &mut dyn Bus) -> Cycles {
        if self.state == State::Halted {
            return 4;
        }

        let opcode = self.fetch_byte(bus);

        match opcode {
            // NOP
            0x00 | 0x08 | 0x10 | 0x18 | 0x20 | 0x28 | 0x30 | 0x38 => 4,

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

            // LD BC,nn
            0x01 => {
                let value = self.fetch_word(bus);
                self.set_bc(value);
                10
            }
            // LD DE,nn
            0x11 => {
                let value = self.fetch_word(bus);
                self.set_de(value);
                10
            }
            // LD HL,nn
            0x21 => {
                let value = self.fetch_word(bus);
                self.set_hl(value);
                10
            }
            // LD SP,nn
            0x31 => {
                let value = self.fetch_word(bus);
                self.sp = value;
                10
            }

            // LD SP,HL
            0xF9 => {
                self.sp = self.hl();
                6
            }
            // LD HL,(nn)
            0x2A => {
                let addr = self.fetch_word(bus);
                let value = bus.read_word(addr);
                self.set_hl(value);
                16
            }
            // LD (nn),HL
            0x22 => {
                let addr = self.fetch_word(bus);
                let value = self.hl();
                bus.write_word(addr, value);
                16
            }

            // EX (SP),HL
            0xE3 => {
                let hl_old = self.hl();
                let sp_old = bus.read_word(self.sp);
                self.set_hl(sp_old);
                bus.write_word(self.sp, hl_old);
                19
            }
            // EX DE,HL
            0xEB => {
                let de_old = self.de();
                let hl_old = self.hl();
                self.set_de(hl_old);
                self.set_hl(de_old);
                4
            }

            // PUSH BC
            0xC5 => {
                self.op_push(bus, self.bc());
                11
            }
            // PUSH DE
            0xD5 => {
                self.op_push(bus, self.de());
                11
            }
            // PUSH HL
            0xE5 => {
                self.op_push(bus, self.hl());
                11
            }
            // PUSH AF
            0xF5 => {
                self.op_push(bus, self.af());
                11
            }

            // POP BC
            0xC1 => {
                let value = self.op_pop(bus);
                self.set_bc(value);
                10
            }
            // POP DE
            0xD1 => {
                let value = self.op_pop(bus);
                self.set_de(value);
                10
            }
            // POP HL
            0xE1 => {
                let value = self.op_pop(bus);
                self.set_hl(value);
                10
            }
            // POP AF
            0xF1 => {
                let value = self.op_pop(bus);
                self.set_af(value);
                10
            }

            // ADD A,r
            0x80..=0x87 => {
                let src = opcode & 0x07;
                let value = self.reg(src, bus);
                self.op_add(value);
                if src == 0x06 { 7 } else { 4 }
            }
            // ADC A,r
            0x88..=0x8F => {
                let src = opcode & 0x07;
                let value = self.reg(src, bus);
                self.op_adc(value);
                if src == 0x06 { 7 } else { 4 }
            }
            // SUB A,r
            0x90..=0x97 => {
                let src = opcode & 0x07;
                let value = self.reg(src, bus);
                self.op_sub(value);
                if src == 0x06 { 7 } else { 4 }
            }
            // SBC A,r
            0x98..=0x9F => {
                let src = opcode & 0x07;
                let value = self.reg(src, bus);
                self.op_sbc(value);
                if src == 0x06 { 7 } else { 4 }
            }
            // AND A,r
            0xA0..=0xA7 => {
                let src = opcode & 0x07;
                let value = self.reg(src, bus);
                self.op_and(value);
                if src == 0x06 { 7 } else { 4 }
            }
            // OR A,r
            0xB0..=0xB7 => {
                let src = opcode & 0x07;
                let value = self.reg(src, bus);
                self.op_or(value);
                if src == 0x06 { 7 } else { 4 }
            }
            // XOR A,r
            0xA8..=0xAF => {
                let src = opcode & 0x07;
                let value = self.reg(src, bus);
                self.op_xor(value);
                if src == 0x06 { 7 } else { 4 }
            }
            // CP A,r
            0xB8..=0xBF => {
                let src = opcode & 0x07;
                let value = self.reg(src, bus);
                self.op_cp(value);
                if src == 0x06 { 7 } else { 4 }
            }

            // ADD A,n
            0xC6 => {
                let value = self.fetch_byte(bus);
                self.op_add(value);
                7
            }
            // ADC A,n
            0xCE => {
                let value = self.fetch_byte(bus);
                self.op_adc(value);
                7
            }
            // SUB A,n
            0xD6 => {
                let value = self.fetch_byte(bus);
                self.op_sub(value);
                7
            }
            // SBC A,n
            0xDE => {
                let value = self.fetch_byte(bus);
                self.op_sbc(value);
                7
            }
            // AND A,n
            0xE6 => {
                let value = self.fetch_byte(bus);
                self.op_and(value);
                7
            }
            // OR A,n
            0xF6 => {
                let value = self.fetch_byte(bus);
                self.op_or(value);
                7
            }
            // XOR A,n
            0xEE => {
                let value = self.fetch_byte(bus);
                self.op_xor(value);
                7
            }
            // CP A,n
            0xFE => {
                let value = self.fetch_byte(bus);
                self.op_cp(value);
                7
            }

            // INC r
            0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => {
                let dest = (opcode >> 3) & 0x07;
                let value = self.reg(dest, bus);
                let result = self.op_inc(value);
                self.set_reg(dest, result, bus);
                if dest == 0x06 { 10 } else { 5 }
            }
            // DEC r
            0x05 | 0x0D | 0x15 | 0x1D | 0x25 | 0x2D | 0x35 | 0x3D => {
                let dest = (opcode >> 3) & 0x07;
                let value = self.reg(dest, bus);
                let result = self.op_dec(value);
                self.set_reg(dest, result, bus);
                if dest == 0x06 { 10 } else { 5 }
            }

            // ADD HL,BC
            0x09 => {
                self.op_add16(self.bc());
                11
            }
            // ADD HL,DE
            0x19 => {
                self.op_add16(self.de());
                11
            }
            // ADD HL,HL
            0x29 => {
                self.op_add16(self.hl());
                11
            }
            // ADD HL,SP
            0x39 => {
                self.op_add16(self.sp);
                11
            }

            // INC BC
            0x03 => {
                let value = self.bc().wrapping_add(1);
                self.set_bc(value);
                6
            }
            // INC DE
            0x13 => {
                let value = self.de().wrapping_add(1);
                self.set_de(value);
                6
            }
            // INC HL
            0x23 => {
                let value = self.hl().wrapping_add(1);
                self.set_hl(value);
                6
            }
            // INC SP
            0x33 => {
                self.sp = self.sp.wrapping_add(1);
                6
            }
            // DEC BC
            0x0B => {
                let value = self.bc().wrapping_sub(1);
                self.set_bc(value);
                6
            }
            // DEC DE
            0x1B => {
                let value = self.de().wrapping_sub(1);
                self.set_de(value);
                6
            }
            // DEC HL
            0x2B => {
                let value = self.hl().wrapping_sub(1);
                self.set_hl(value);
                6
            }
            // DEC SP
            0x3B => {
                self.sp = self.sp.wrapping_sub(1);
                6
            }

            // RLCA
            0x07 => {
                let msb = self.a & 0x80;
                self.a = self.a.rotate_left(1);
                self.flags.carry = msb != 0;
                4
            }
            // RRCA
            0x0F => {
                let lsb = self.a & 0x01;
                self.a = self.a.rotate_right(1);
                self.flags.carry = lsb != 0;
                4
            }
            // RLA
            0x17 => {
                let msb = self.a & 0x80;
                self.a = (self.a << 1) | if self.flags.carry { 0x01 } else { 0 };
                self.flags.carry = msb != 0;
                4
            }
            // RRA
            0x1F => {
                let lsb = self.a & 0x01;
                self.a = (self.a >> 1) | if self.flags.carry { 0x80 } else { 0 };
                self.flags.carry = lsb != 0;
                4
            }

            // DAA
            0x27 => {
                // TODO: implement DAA
                4
            }
            // CPL
            0x2F => {
                self.a = !self.a;
                self.flags.aux_carry = true;
                4
            }
            // SCF
            0x37 => {
                self.flags.carry = true;
                4
            }
            // CCF
            0x3F => {
                self.flags.carry = !self.flags.carry;
                4
            }

            // JP addr
            0xC3 | 0xCB => {
                self.op_jp(bus, true);
                10
            }
            // JP cc,addr
            0xC2 | 0xCA | 0xD2 | 0xDA | 0xE2 | 0xEA | 0xF2 | 0xFA => {
                let cc = (opcode >> 3) & 0x07;
                let condition = self.condition(cc);
                self.op_jp(bus, condition);
                10
            }

            // RET
            0xC9 | 0xD9 => {
                self.op_ret(bus, true);
                10
            }
            // RET cc
            0xC0 | 0xC8 | 0xD0 | 0xD8 | 0xE0 | 0xE8 | 0xF0 | 0xF8 => {
                let cc = (opcode >> 3) & 0x07;
                let condition = self.condition(cc);
                self.op_ret(bus, condition);
                if condition { 11 } else { 5 }
            }

            // CALL addr
            0xCD | 0xDD | 0xED | 0xFD => {
                self.op_call(bus, true);
                17
            }
            // CALL cc,addr
            0xC4 | 0xCC | 0xD4 | 0xDC | 0xE4 | 0xEC | 0xF4 | 0xFC => {
                let cc = (opcode >> 3) & 0x07;
                let condition = self.condition(cc);
                self.op_call(bus, condition);
                if condition { 17 } else { 11 }
            }

            // RST p
            0xC7 | 0xCF | 0xD7 | 0xDF | 0xE7 | 0xEF | 0xF7 | 0xFF => {
                let addr = (opcode & 0b00_111_000) as u16;
                self.op_push(bus, self.pc);
                self.pc = addr;
                11
            }
            // JP (HL)
            0xE9 => {
                self.pc = self.hl();
                5
            }

            // EI
            0xFB => {
                self.iff = true;
                4
            }
            // DI
            0xF3 => {
                self.iff = false;
                4
            }

            // IN A,(n)
            0xDB => {
                let port = self.fetch_byte(bus);
                self.a = bus.input(port);
                10
            }
            // OUT (n),A
            0xD3 => {
                let port = self.fetch_byte(bus);
                bus.output(port, self.a);
                10
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

    fn op_add(&mut self, value: u8) {
        let r = self.a as u16 + value as u16;
        self.a = r as u8;

        self.set_zsp(self.a);
        self.flags.carry = r & 0x100 != 0;
        self.flags.aux_carry = ((self.a & 0x0F) + (value & 0x0F)) & 0x10 != 0;
    }

    fn op_adc(&mut self, value: u8) {
        let carry = if self.flags.carry { 1 } else { 0 };
        let r = self.a as u16 + value as u16 + carry as u16;
        self.a = r as u8;

        self.set_zsp(self.a);
        self.flags.carry = r & 0x100 != 0;
        self.flags.aux_carry = ((self.a & 0x0F) + (value & 0x0F) + carry) & 0x10 != 0;
    }

    fn op_sub(&mut self, value: u8) {
        let r = self.a as i16 - value as i16;
        self.a = r as u8;

        self.set_zsp(self.a);
        self.flags.carry = r < 0;
        self.flags.aux_carry = (self.a & 0x0F) < (value & 0x0F);
    }

    fn op_sbc(&mut self, value: u8) {
        let carry = if self.flags.carry { 1 } else { 0 };
        let r = self.a as i16 - value as i16 - carry as i16;
        self.a = r as u8;

        self.set_zsp(self.a);
        self.flags.carry = r < 0;
        self.flags.aux_carry = (self.a & 0x0F) < (value & 0x0F) + carry;
    }

    fn op_and(&mut self, value: u8) {
        self.a &= value;

        self.set_zsp(self.a);
        self.flags.carry = false;
        self.flags.aux_carry = true;
    }

    fn op_or(&mut self, value: u8) {
        self.a |= value;

        self.set_zsp(self.a);
        self.flags.carry = false;
        self.flags.aux_carry = false;
    }

    fn op_xor(&mut self, value: u8) {
        self.a ^= value;

        self.set_zsp(self.a);
        self.flags.carry = false;
        self.flags.aux_carry = false;
    }

    fn op_cp(&mut self, value: u8) {
        let r = self.a as i16 - value as i16;

        self.set_zsp(r as u8);
        self.flags.carry = r < 0;
        self.flags.aux_carry = (self.a & 0x0F) < (value & 0x0F);
    }

    fn op_inc(&mut self, value: u8) -> u8 {
        let r = value.wrapping_add(1);
        self.set_zsp(r);
        self.flags.aux_carry = (value & 0x0F) + 1 > 0x0F;
        r
    }

    fn op_dec(&mut self, value: u8) -> u8 {
        let r = value.wrapping_sub(1);
        self.set_zsp(r);
        self.flags.aux_carry = (value & 0x0F) == 0x00;
        r
    }

    fn op_add16(&mut self, value: u16) {
        let (r, carry) = self.hl().carrying_add(value, false);
        self.set_hl(r);
        self.flags.carry = carry;
    }

    fn op_jp(&mut self, bus: &dyn Bus, condition: bool) {
        let addr = self.fetch_word(bus);
        if condition {
            self.pc = addr;
        }
    }

    fn op_call(&mut self, bus: &mut dyn Bus, condition: bool) {
        let addr = self.fetch_word(bus);
        if condition {
            self.sp = self.sp.wrapping_sub(2);
            bus.write_word(self.sp, self.pc);
            self.pc = addr;
        }
    }

    fn op_ret(&mut self, bus: &dyn Bus, condition: bool) {
        if condition {
            let addr = bus.read_word(self.sp);
            self.sp = self.sp.wrapping_add(2);
            self.pc = addr;
        }
    }

    fn op_push(&mut self, bus: &mut dyn Bus, value: u16) {
        self.sp = self.sp.wrapping_sub(2);
        bus.write_word(self.sp, value);
    }

    fn op_pop(&mut self, bus: &dyn Bus) -> u16 {
        let value = bus.read_word(self.sp);
        self.sp = self.sp.wrapping_add(2);
        value
    }

    fn set_zsp(&mut self, value: u8) {
        self.flags.zero = value == 0;
        self.flags.sign = (value & 0x80) != 0;
        self.flags.parity = value.count_ones() % 2 == 0;
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

    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = (value & 0xFF) as u8;
    }

    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = (value & 0xFF) as u8;
    }

    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = (value & 0xFF) as u8;
    }

    pub fn af(&self) -> u16 {
        let f = 0x02
            | (if self.flags.zero { 0x40 } else { 0 })
            | (if self.flags.sign { 0x80 } else { 0 })
            | (if self.flags.parity { 0x04 } else { 0 })
            | (if self.flags.aux_carry { 0x10 } else { 0 })
            | (if self.flags.carry { 0x01 } else { 0 });
        ((self.a as u16) << 8) | (f as u16)
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        let f = (value & 0xFF) as u8;
        self.flags.zero = (f & 0x40) != 0;
        self.flags.sign = (f & 0x80) != 0;
        self.flags.parity = (f & 0x04) != 0;
        self.flags.aux_carry = (f & 0x10) != 0;
        self.flags.carry = (f & 0x01) != 0;
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

    fn condition(&self, code: u8) -> bool {
        match code {
            0 => !self.flags.zero,   // NZ
            1 => self.flags.zero,    // Z
            2 => !self.flags.carry,  // NC
            3 => self.flags.carry,   // C
            4 => !self.flags.parity, // PO
            5 => self.flags.parity,  // PE
            6 => !self.flags.sign,   // P
            7 => self.flags.sign,    // M
            _ => unreachable!(),
        }
    }
}
