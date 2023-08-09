use crate::memory::Memory;

use super::{
    instruction::{AddressingMode, Instructions},
    CpuFlags, Cpu,
};

/// NES CPU unofficial instructions.
/// Reference: https://www.nesdev.org/undocumented_opcodes.txt.
pub(crate) trait UnofficialInstructions {
    fn lax(&mut self, mode: &AddressingMode);
    fn sax(&mut self, mode: &AddressingMode);
    fn unofficial_sbc(&mut self, mode: &AddressingMode);
    fn dcp(&mut self, mode: &AddressingMode);
    fn isb(&mut self, mode: &AddressingMode);
    fn slo(&mut self, mode: &AddressingMode);
    fn rla(&mut self, mode: &AddressingMode);
    fn sre(&mut self, mode: &AddressingMode);
    fn rra(&mut self, mode: &AddressingMode);
    fn axs(&mut self, mode: &AddressingMode);
    fn arr(&mut self, mode: &AddressingMode);
    fn anc(&mut self, mode: &AddressingMode);
    fn alr(&mut self, mode: &AddressingMode);
    fn lxa(&mut self, mode: &AddressingMode);
    fn xaa(&mut self, mode: &AddressingMode);
    fn las(&mut self, mode: &AddressingMode);
    fn tas(&mut self);
    fn ahx_indirect_y(&mut self);
    fn ahx_absolute_y(&mut self);
    fn shx(&mut self);
    fn shy(&mut self);
}

impl UnofficialInstructions for Cpu {
    fn lax(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.register_a = data;
        self.update_zero_and_negative_flags(self.register_a);
        self.register_x = self.register_a;
    }

    fn sax(&mut self, mode: &AddressingMode) {
        let data = self.register_a & self.register_x;
        let (addr, _) = self.get_operand_address(mode);
        self.mem_write(addr, data);
    }

    fn unofficial_sbc(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.sub_from_register_a(data);
    }

    fn dcp(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        data = data.wrapping_sub(1);
        self.mem_write(addr, data);
        if data <= self.register_a {
            self.status.insert(CpuFlags::CARRY);
        }

        self.update_zero_and_negative_flags(self.register_a.wrapping_sub(data));
    }

    fn isb(&mut self, mode: &AddressingMode) {
        let data = self.inc(mode);
        self.sub_from_register_a(data);
    }

    fn slo(&mut self, mode: &AddressingMode) {
        let data = self.asl(mode);
        self.register_a = data | self.register_a;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn rla(&mut self, mode: &AddressingMode) {
        let data = self.rol(mode);
        self.register_a = data & self.register_a;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn sre(&mut self, mode: &AddressingMode) {
        let data = self.lsr(mode);
        self.register_a = data ^ self.register_a;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn rra(&mut self, mode: &AddressingMode) {
        let data = self.ror(mode);
        self.add_to_register_a(data);
    }

    fn axs(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        let x_and_a = self.register_x & self.register_a;
        let result = x_and_a.wrapping_sub(data);

        if data <= x_and_a {
            self.status.insert(CpuFlags::CARRY);
        }
        self.update_zero_and_negative_flags(result);

        self.register_x = result;
    }
    fn arr(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.register_a = data & self.register_a;
        self.update_zero_and_negative_flags(self.register_a);
        self.ror_accumulator();

        let result = self.register_a;
        let bit_5 = (result >> 5) & 1;
        let bit_6 = (result >> 6) & 1;

        if bit_6 == 1 {
            self.status.insert(CpuFlags::CARRY)
        } else {
            self.status.remove(CpuFlags::CARRY)
        }

        if bit_5 ^ bit_6 == 1 {
            self.status.insert(CpuFlags::OVERFLOW);
        } else {
            self.status.remove(CpuFlags::OVERFLOW);
        }

        self.update_zero_and_negative_flags(result);
    }

    fn anc(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.register_a = data & self.register_a;
        self.update_zero_and_negative_flags(self.register_a);
        if self.status.contains(CpuFlags::NEGATIV) {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }
    }

    fn alr(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.register_a = data & self.register_a;
        self.update_zero_and_negative_flags(self.register_a);
        self.lsr_accumulator();
    }

    fn lxa(&mut self, mode: &AddressingMode) {
        self.lda(mode);
        self.tax();
    }

    fn xaa(&mut self, mode: &AddressingMode) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
        let (addr, _) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.register_a = data & self.register_a;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn las(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        data = data & self.stack_pointer;
        self.register_a = data;
        self.register_x = data;
        self.stack_pointer = data;
        self.update_zero_and_negative_flags(data);
    }

    fn tas(&mut self) {
        let data = self.register_a & self.register_x;
        self.stack_pointer = data;
        let mem_address = self.mem_read_u16(self.program_counter) + self.register_y as u16;
        let data = ((mem_address >> 8) as u8 + 1) & self.stack_pointer;
        self.mem_write(mem_address, data);
    }

    fn ahx_indirect_y(&mut self) {
        let pos: u8 = self.mem_read(self.program_counter);
        let mem_address = self.mem_read_u16(pos as u16) + self.register_y as u16;
        let data = self.register_a & self.register_x & (mem_address >> 8) as u8;
        self.mem_write(mem_address, data);
    }

    fn ahx_absolute_y(&mut self) {
        let mem_address = self.mem_read_u16(self.program_counter) + self.register_y as u16;
        let data = self.register_a & self.register_x & (mem_address >> 8) as u8;
        self.mem_write(mem_address, data);
    }

    fn shx(&mut self) {
        let mem_address = self.mem_read_u16(self.program_counter) + self.register_y as u16;
        let data = self.register_x & ((mem_address >> 8) as u8 + 1);
        self.mem_write(mem_address, data);
    }

    fn shy(&mut self) {
        let mem_address = self.mem_read_u16(self.program_counter) + self.register_x as u16;
        let data = self.register_y & ((mem_address >> 8) as u8 + 1);
        self.mem_write(mem_address, data);
    }
}
