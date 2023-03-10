use crate::memory::Memory;

use super::CPU;

impl Memory for CPU {
    fn mem_read(&self, addr: u16) -> u8 {
        if let Some(bus) = &self.bus {
            bus.borrow().mem_read(addr)
        } else {
            self.memory[addr as usize]
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        if let Some(bus) = &self.bus {
            bus.borrow_mut().mem_write(addr, data);
        } else {
            self.memory[addr as usize] = data;
        }
    }
    fn mem_read_u16(&self, pos: u16) -> u16 {
        if let Some(bus) = &self.bus {
            bus.borrow().mem_read_u16(pos)
        } else {
            let lo = self.mem_read(pos) as u16;
            let hi = self.mem_read(pos + 1) as u16;
            (hi << 8) | (lo as u16)
        }
    }

    fn mem_write_u16(&mut self, pos: u16, data: u16) {
        if let Some(bus) = &self.bus {
            bus.borrow_mut().mem_write_u16(pos, data);
        } else {
            let hi = (data >> 8) as u8;
            let lo = (data & 0xff) as u8;
            self.mem_write(pos, lo);
            self.mem_write(pos + 1, hi);
        }
    }
}
