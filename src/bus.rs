pub trait Bus {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);

    fn input(&self, port: u8) -> u8;
    fn output(&mut self, port: u8, value: u8);

    fn read16(&self, addr: u16) -> u16 {
        let lo = self.read(addr);
        let hi = self.read(addr.wrapping_add(1));
        ((hi as u16) << 8) | (lo as u16)
    }

    fn write16(&mut self, addr: u16, value: u16) {
        let lo = (value & 0x00FF) as u8;
        let hi = (value >> 8) as u8;
        self.write(addr, lo);
        self.write(addr.wrapping_add(1), hi);
    }
}
