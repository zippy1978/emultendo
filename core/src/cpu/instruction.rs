use crate::memory::Memory;

use super::{CPUFlags, CPU};

/// Instructions addressing mode
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPage_X,
    ZeroPage_Y,
    Absolute,
    Absolute_X,
    Absolute_Y,
    Indirect_X,
    Indirect_Y,
    NoneAddressing,
}

/// NES CPU instructions.
/// Reference: https://www.nesdev.org/obelisk-6502-guide/reference.htm.
pub(crate) trait Instructions {
    fn lda(&mut self, mode: &AddressingMode);
    fn sta(&mut self, mode: &AddressingMode);
    fn tax(&mut self);
    fn inx(&mut self);
    fn cld(&mut self);
    fn cli(&mut self);
    fn clv(&mut self);
    fn clc(&mut self);
    fn sec(&mut self);
    fn sei(&mut self);
    fn sed(&mut self);
    fn pha(&mut self);
    fn pla(&mut self);
    fn php(&mut self);
    fn plp(&mut self);
    fn adc(&mut self, mode: &AddressingMode);
    fn sbc(&mut self, mode: &AddressingMode);
    fn and(&mut self, mode: &AddressingMode);
    fn eor(&mut self, mode: &AddressingMode);
    fn ora(&mut self, mode: &AddressingMode);
    fn lsr_accumulator(&mut self);
    fn lsr(&mut self, mode: &AddressingMode) -> u8;
    fn asl_accumulator(&mut self);
    fn asl(&mut self, mode: &AddressingMode) -> u8;
    fn rol_accumulator(&mut self);
    fn rol(&mut self, mode: &AddressingMode) -> u8;
    fn ror_accumulator(&mut self);
    fn ror(&mut self, mode: &AddressingMode) -> u8;
    fn inc(&mut self, mode: &AddressingMode) -> u8;
    fn iny(&mut self);
    fn dec(&mut self, mode: &AddressingMode) -> u8;
    fn dex(&mut self);
    fn dey(&mut self);
    fn cmp(&mut self, mode: &AddressingMode);
    fn cpy(&mut self, mode: &AddressingMode);
    fn cpx(&mut self, mode: &AddressingMode);
    fn jmp_indirect(&mut self);
    fn jmp_absolute(&mut self);
    fn jsr(&mut self);
    fn rts(&mut self);
    fn rti(&mut self);
    fn bne(&mut self);
    fn bvs(&mut self);
    fn bvc(&mut self);
    fn bpl(&mut self);
    fn bmi(&mut self);
    fn beq(&mut self);
    fn bcs(&mut self);
    fn bcc(&mut self);
    fn bit(&mut self, mode: &AddressingMode);
    fn stx(&mut self, mode: &AddressingMode);
    fn sty(&mut self, mode: &AddressingMode);
    fn ldx(&mut self, mode: &AddressingMode);
    fn ldy(&mut self, mode: &AddressingMode);
    fn tay(&mut self);
    fn tsx(&mut self);
    fn txa(&mut self);
    fn txs(&mut self);
    fn tya(&mut self);
}

impl Instructions for CPU {
    fn lda(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(&mode);
        let value = self.mem_read(addr);

        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
        if page_cross {
            self.remaining_cycles += 1;
        }
    }

    fn sta(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn cld(&mut self) {
        self.status.remove(CPUFlags::DECIMAL_MODE);
    }

    fn cli(&mut self) {
        self.status.remove(CPUFlags::INTERRUPT_DISABLE);
    }

    fn clv(&mut self) {
        self.status.remove(CPUFlags::OVERFLOW);
    }

    fn clc(&mut self) {
        self.status.remove(CPUFlags::CARRY);
    }

    fn sec(&mut self) {
        self.status.insert(CPUFlags::CARRY);
    }

    fn sei(&mut self) {
        self.status.insert(CPUFlags::INTERRUPT_DISABLE);
    }

    fn sed(&mut self) {
        self.status.insert(CPUFlags::DECIMAL_MODE);
    }

    fn pha(&mut self) {
        self.stack_push(self.register_a);
    }

    fn pla(&mut self) {
        self.register_a = self.stack_pop();
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn php(&mut self) {
        let mut flags = self.status.clone();
        flags.insert(CPUFlags::BREAK);
        flags.insert(CPUFlags::BREAK2);
        self.stack_push(flags.bits());
    }

    fn plp(&mut self) {
        self.status.bits = self.stack_pop();
        self.status.remove(CPUFlags::BREAK);
        self.status.insert(CPUFlags::BREAK2);
    }

    fn adc(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.add_to_register_a(value);
        if page_cross {
            self.remaining_cycles += 1;
        }
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(&mode);
        let data = self.mem_read(addr);
        self.add_to_register_a(((data as i8).wrapping_neg().wrapping_sub(1)) as u8);
        if page_cross {
            self.remaining_cycles += 1;
        }
    }
    fn and(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.register_a = data & self.register_a;
        self.update_zero_and_negative_flags(self.register_a);
        if page_cross {
            self.remaining_cycles += 1;
        }
    }

    fn eor(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.register_a = data ^ self.register_a;
        self.update_zero_and_negative_flags(self.register_a);
        if page_cross {
            self.remaining_cycles += 1;
        }
    }

    fn ora(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.register_a = data | self.register_a;
        self.update_zero_and_negative_flags(self.register_a);
        if page_cross {
            self.remaining_cycles += 1;
        }
    }

    fn lsr_accumulator(&mut self) {
        let mut data = self.register_a;
        if data & 1 == 1 {
            self.status.insert(CPUFlags::CARRY);
        } else {
            self.status.remove(CPUFlags::CARRY);
        }
        data = data >> 1;
        self.register_a = data;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn lsr(&mut self, mode: &AddressingMode) -> u8 {
        let (addr, _) = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        if data & 1 == 1 {
            self.status.insert(CPUFlags::CARRY);
        } else {
            self.status.remove(CPUFlags::CARRY);
        }
        data = data >> 1;
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
        data
    }

    fn asl_accumulator(&mut self) {
        let mut data = self.register_a;
        if data >> 7 == 1 {
            self.status.insert(CPUFlags::CARRY);
        } else {
            self.status.remove(CPUFlags::CARRY);
        }
        data = data << 1;
        self.register_a = data;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn asl(&mut self, mode: &AddressingMode) -> u8 {
        let (addr, _) = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        if data >> 7 == 1 {
            self.status.insert(CPUFlags::CARRY);
        } else {
            self.status.remove(CPUFlags::CARRY);
        }
        data = data << 1;
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
        data
    }

    fn rol_accumulator(&mut self) {
        let mut data = self.register_a;
        let old_carry = self.status.contains(CPUFlags::CARRY);

        if data >> 7 == 1 {
            self.status.insert(CPUFlags::CARRY);
        } else {
            self.status.remove(CPUFlags::CARRY);
        }
        data = data << 1;
        if old_carry {
            data = data | 1;
        }
        self.register_a = data;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn rol(&mut self, mode: &AddressingMode) -> u8 {
        let (addr, _) = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        let old_carry = self.status.contains(CPUFlags::CARRY);

        if data >> 7 == 1 {
            self.status.insert(CPUFlags::CARRY);
        } else {
            self.status.remove(CPUFlags::CARRY);
        }
        data = data << 1;
        if old_carry {
            data = data | 1;
        }
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
        data
    }

    fn ror_accumulator(&mut self) {
        let mut data = self.register_a;
        let old_carry = self.status.contains(CPUFlags::CARRY);

        if data & 1 == 1 {
            self.status.insert(CPUFlags::CARRY);
        } else {
            self.status.remove(CPUFlags::CARRY);
        }
        data = data >> 1;
        if old_carry {
            data = data | 0b10000000;
        }
        self.register_a = data;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn ror(&mut self, mode: &AddressingMode) -> u8 {
        let (addr, _) = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        let old_carry = self.status.contains(CPUFlags::CARRY);

        if data & 1 == 1 {
            self.status.insert(CPUFlags::CARRY);
        } else {
            self.status.remove(CPUFlags::CARRY);
        }
        data = data >> 1;
        if old_carry {
            data = data | 0b10000000;
        }
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
        data
    }

    fn inc(&mut self, mode: &AddressingMode) -> u8 {
        let (addr, _) = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        data = data.wrapping_add(1);
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
        data
    }

    fn iny(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn dec(&mut self, mode: &AddressingMode) -> u8 {
        let (addr, _) = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        data = data.wrapping_sub(1);
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
        data
    }

    fn dex(&mut self) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn dey(&mut self) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn cmp(&mut self, mode: &AddressingMode) {
        self.compare(mode, self.register_a);
    }
    fn cpy(&mut self, mode: &AddressingMode) {
        self.compare(mode, self.register_y);
    }
    fn cpx(&mut self, mode: &AddressingMode) {
        self.compare(mode, self.register_x);
    }

    fn jmp_indirect(&mut self) {
        let mem_address = self.mem_read_u16(self.program_counter);
        // 6502 bug mode with with page boundary:
        // If address $3000 contains $40, $30FF contains $80, and $3100 contains $50,
        // the result of JMP ($30FF) will be a transfer of control to $4080 rather than $5080 as you intended
        // i.e. the 6502 took the low byte of the address from $30FF and the high byte from $3000
        let indirect_ref = if mem_address & 0x00FF == 0x00FF {
            let lo = self.mem_read(mem_address);
            let hi = self.mem_read(mem_address & 0xFF00);
            (hi as u16) << 8 | (lo as u16)
        } else {
            self.mem_read_u16(mem_address)
        };

        self.program_counter = indirect_ref;
    }

    fn jmp_absolute(&mut self) {
        let mem_address = self.mem_read_u16(self.program_counter);
        self.program_counter = mem_address;
    }

    fn jsr(&mut self) {
        self.stack_push_u16(self.program_counter + 2 - 1);
        let target_address = self.mem_read_u16(self.program_counter);
        self.program_counter = target_address
    }

    fn rts(&mut self) {
        self.program_counter = self.stack_pop_u16() + 1;
    }

    fn rti(&mut self) {
        self.status.bits = self.stack_pop();
        self.status.remove(CPUFlags::BREAK);
        self.status.insert(CPUFlags::BREAK2);

        self.program_counter = self.stack_pop_u16();
    }

    fn bne(&mut self) {
        self.branch(!self.status.contains(CPUFlags::ZERO));
    }

    fn bvs(&mut self) {
        self.branch(self.status.contains(CPUFlags::OVERFLOW));
    }

    fn bvc(&mut self) {
        self.branch(!self.status.contains(CPUFlags::OVERFLOW));
    }

    fn bpl(&mut self) {
        self.branch(!self.status.contains(CPUFlags::NEGATIV));
    }

    fn bmi(&mut self) {
        self.branch(self.status.contains(CPUFlags::NEGATIV));
    }

    fn beq(&mut self) {
        self.branch(self.status.contains(CPUFlags::ZERO));
    }

    fn bcs(&mut self) {
        self.branch(self.status.contains(CPUFlags::CARRY));
    }

    fn bcc(&mut self) {
        self.branch(!self.status.contains(CPUFlags::CARRY));
    }

    fn bit(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        let and = self.register_a & data;
        if and == 0 {
            self.status.insert(CPUFlags::ZERO);
        } else {
            self.status.remove(CPUFlags::ZERO);
        }

        self.status.set(CPUFlags::NEGATIV, data & 0b10000000 > 0);
        self.status.set(CPUFlags::OVERFLOW, data & 0b01000000 > 0);
    }

    fn stx(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x);
    }

    fn sty(&mut self, mode: &AddressingMode) {
        let (addr, _) = self.get_operand_address(mode);
        self.mem_write(addr, self.register_y);
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.register_x = data;
        self.update_zero_and_negative_flags(self.register_x);
        if page_cross {
            self.remaining_cycles += 1;
        }
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.register_y = data;
        self.update_zero_and_negative_flags(self.register_y);
        if page_cross {
            self.remaining_cycles += 1;
        }
    }

    fn tay(&mut self) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }

    fn tsx(&mut self) {
        self.register_x = self.stack_pointer;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn txa(&mut self) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn txs(&mut self) {
        self.stack_pointer = self.register_x;
    }

    fn tya(&mut self) {
        self.register_a = self.register_y;
        self.update_zero_and_negative_flags(self.register_a);
    }
}
