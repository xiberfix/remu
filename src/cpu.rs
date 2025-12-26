use crate::bus::Bus;

pub type Cycles = u64;

#[derive(Default)]
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
}

#[derive(Default)]
pub struct Flags {
    zero: bool,
    sign: bool,
    parity: bool,
    aux_carry: bool,
    carry: bool,
}

impl Cpu {
    pub fn step(&mut self, _bus: &mut dyn Bus) -> Cycles {
        1
    }

    pub fn to_string(&self) -> String {
        format!(
            "PC={:04X} SP={:04X} A={:02X} BC={:02X}{:02X} DE={:02X}{:02X} HL={:02X}{:02X} F=[Z:{} S:{} P:{} AC:{} C:{}]",
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
        )
    }
}
