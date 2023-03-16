use bitflags::bitflags;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    bus::cpu_bus::CPUBus,
    memory::{page_cross, Memory},
};

use self::{
    instruction::{AddressingMode, Instructions},
    unofficial_instruction::UnofficialInstructions,
};

#[cfg(test)]
pub mod mod_tests;

mod instruction;
#[cfg(test)]
mod instruction_tests;

mod unofficial_instruction;
#[cfg(test)]
mod unofficial_instruction_tests;

mod interrupt;

pub mod memory;
mod opcode;
pub mod trace;

#[cfg(test)]
mod trace_tests;

bitflags! {
    /// # Status Register (P) http://wiki.nesdev.com/w/index.php/Status_flags
    ///
    ///  7 6 5 4 3 2 1 0
    ///  N V _ B D I Z C
    ///  | |   | | | | +--- Carry Flag
    ///  | |   | | | +----- Zero Flag
    ///  | |   | | +------- Interrupt Disable
    ///  | |   | +--------- Decimal Mode (not used on NES)
    ///  | |   +----------- Break Command
    ///  | +--------------- Overflow Flag
    ///  +----------------- Negative Flag
    ///
    pub struct CPUFlags: u8 {
        const CARRY             = 0b00000001;
        const ZERO              = 0b00000010;
        const INTERRUPT_DISABLE = 0b00000100;
        const DECIMAL_MODE      = 0b00001000;
        const BREAK             = 0b00010000;
        const BREAK2            = 0b00100000;
        const OVERFLOW          = 0b01000000;
        const NEGATIV           = 0b10000000;
    }
}

const STACK: u16 = 0x0100;
const STACK_RESET: u8 = 0xfd;

/// CPU.
#[derive(Debug, Clone)]
pub struct CPU {
    register_a: u8,
    register_x: u8,
    register_y: u8,
    status: CPUFlags,
    pub(crate) program_counter: u16,
    stack_pointer: u8,
    memory: [u8; 0xFFFF],
    /// Remaining cycles count before moving to the next instruction.
    remaining_cycles: u8,
    bus: Option<Rc<RefCell<CPUBus>>>,
}

/// CPU Error.
#[derive(Debug)]
pub enum CPUError {
    UnknownOpCode(u8),
}

impl CPU {
    /// Creates a CPU.
    /// Used internal memory until bus is connected.
    /// Call connect_bus to connect to an external bus.
    pub fn new() -> Self {
        Self {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            stack_pointer: STACK_RESET,
            status: CPUFlags::from_bits_truncate(0b100100),
            program_counter: 0,
            memory: [0; 0xFFFF],
            remaining_cycles: 0,
            bus: None,
        }
    }

    /// Connects CPU to bus.
    pub fn connect_bus(&mut self, bus: &Rc<RefCell<CPUBus>>) {
        self.bus = Some(Rc::clone(bus));
    }

    /// Gets absolute address according to address + addressing mode.
    /// Returns absolute address and bool to detect page crossing
    pub fn get_absolute_address(&mut self, mode: &AddressingMode, addr: u16) -> (u16, bool) {
        match mode {
            AddressingMode::ZeroPage => (self.mem_read(addr) as u16, false),

            AddressingMode::Absolute => (self.mem_read_u16(addr), false),

            AddressingMode::ZeroPage_X => {
                let pos = self.mem_read(addr);
                let addr = pos.wrapping_add(self.register_x) as u16;
                (addr, false)
            }
            AddressingMode::ZeroPage_Y => {
                let pos = self.mem_read(addr);
                let addr = pos.wrapping_add(self.register_y) as u16;
                (addr, false)
            }

            AddressingMode::Absolute_X => {
                let base = self.mem_read_u16(addr);
                let addr = base.wrapping_add(self.register_x as u16);
                (addr, page_cross(base, addr))
            }
            AddressingMode::Absolute_Y => {
                let base = self.mem_read_u16(addr);
                let addr = base.wrapping_add(self.register_y as u16);
                (addr, page_cross(base, addr))
            }

            AddressingMode::Indirect_X => {
                let base = self.mem_read(addr);

                let ptr: u8 = (base as u8).wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                ((hi as u16) << 8 | (lo as u16), false)
            }
            AddressingMode::Indirect_Y => {
                let base = self.mem_read(addr);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                (deref, page_cross(deref, deref_base))
            }

            _ => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    /// Gets address according to addressing mode.
    /// Returns address and bool to detect page crossing
    fn get_operand_address(&mut self, mode: &AddressingMode) -> (u16, bool) {
        match mode {
            AddressingMode::Immediate => (self.program_counter, false),
            _ => self.get_absolute_address(mode, self.program_counter),
        }
    }

    /// Updates zero and neg flags in status.
    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.status.insert(CPUFlags::ZERO);
        } else {
            self.status.remove(CPUFlags::ZERO);
        }

        if result & 0b1000_0000 != 0 {
            self.status.insert(CPUFlags::NEGATIV);
        } else {
            self.status.remove(CPUFlags::NEGATIV);
        }
    }

    /// Compares address value with other value.
    fn compare(&mut self, mode: &AddressingMode, compare_with: u8) {
        let (addr, page_cross) = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        if data <= compare_with {
            self.status.insert(CPUFlags::CARRY);
        } else {
            self.status.remove(CPUFlags::CARRY);
        }

        self.update_zero_and_negative_flags(compare_with.wrapping_sub(data));

        if page_cross {
            self.remaining_cycles += 1;
        }
    }

    /// Handles branching.
    fn branch(&mut self, condition: bool) {
        if condition {
            self.remaining_cycles += 1;

            let jump: i8 = self.mem_read(self.program_counter) as i8;
            let jump_addr = self
                .program_counter
                .wrapping_add(1)
                .wrapping_add(jump as u16);

            if self.program_counter.wrapping_add(1) & 0xFF00 != jump_addr & 0xFF00 {
                self.remaining_cycles += 1;
            }

            self.program_counter = jump_addr;
        }
    }

    /// Substracts from register a.
    fn sub_from_register_a(&mut self, data: u8) {
        self.add_to_register_a(((data as i8).wrapping_neg().wrapping_sub(1)) as u8);
    }

    /// http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
    fn add_to_register_a(&mut self, data: u8) {
        let sum = self.register_a as u16
            + data as u16
            + (if self.status.contains(CPUFlags::CARRY) {
                1
            } else {
                0
            }) as u16;

        let carry = sum > 0xff;

        if carry {
            self.status.insert(CPUFlags::CARRY);
        } else {
            self.status.remove(CPUFlags::CARRY);
        }

        let result = sum as u8;

        if (data ^ result) & (result ^ self.register_a) & 0x80 != 0 {
            self.status.insert(CPUFlags::OVERFLOW);
        } else {
            self.status.remove(CPUFlags::OVERFLOW)
        }

        self.register_a = result;
        self.update_zero_and_negative_flags(self.register_a);
    }

    /// Pops stack (u8).
    fn stack_pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.mem_read((STACK as u16) + self.stack_pointer as u16)
    }

    /// Pushes to stack (u8).
    fn stack_push(&mut self, data: u8) {
        self.mem_write((STACK as u16) + self.stack_pointer as u16, data);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1)
    }

    /// Pushed to stack (u16).
    fn stack_push_u16(&mut self, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.stack_push(hi);
        self.stack_push(lo);
    }

    /// Pops stack (u16).
    fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop() as u16;
        let hi = self.stack_pop() as u16;

        hi << 8 | lo
    }

    /// Loads and runs program.
    pub fn load_and_run(&mut self, program: Vec<u8>) -> Result<(), CPUError> {
        self.load(program);
        self.reset();
        self.run()
    }

    /// Loads program into memory.
    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8600..(0x8600 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(0xFFFC, 0x8600);
    }

    /// Resets CPU.
    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.stack_pointer = STACK_RESET;
        self.status = CPUFlags::from_bits_truncate(0b100100);

        self.program_counter = self.mem_read_u16(0xFFFC);
        self.remaining_cycles = 0;
    }

    fn interrupt(&mut self, interrupt: interrupt::Interrupt) {
        self.stack_push_u16(self.program_counter);
        let mut flag = self.status.clone();
        flag.set(CPUFlags::BREAK, interrupt.b_flag_mask & 0b010000 == 1);
        flag.set(CPUFlags::BREAK2, interrupt.b_flag_mask & 0b100000 == 1);

        self.stack_push(flag.bits);
        self.status.insert(CPUFlags::INTERRUPT_DISABLE);

        self.remaining_cycles += interrupt.cpu_cycles;

        self.program_counter = self.mem_read_u16(interrupt.vector_addr);
    }

    /// Processes next cycle.
    /// Returns false if BRK is called.
    pub fn tick(&mut self) -> Result<bool, CPUError> {
        if self.remaining_cycles > 0 {
            self.remaining_cycles -= 1;
        } else {
            // Handle NMI interrupt
            if let Some(bus) = &self.bus {
                if bus.borrow_mut().poll_nmi_status() {
                    self.interrupt(interrupt::NMI);
                }
            }

            let ref opcodes: HashMap<u8, &'static opcode::OpCode> = *opcode::OPCODES_MAP;
            let code = self.mem_read(self.program_counter);

            let opcode = match opcodes.get(&code) {
                Some(c) => c,
                None => return Err(CPUError::UnknownOpCode(code)),
            };

            //println!("{:#06x} - {}", self.program_counter, opcode.mnemonic);

            self.remaining_cycles = opcode.cycles;

            self.program_counter += 1;
            let program_counter_state = self.program_counter;

            let mut brk = false;

            match code {
                /* LDA */
                0xa9 | 0xa5 | 0xb5 | 0xad | 0xbd | 0xb9 | 0xa1 | 0xb1 => self.lda(&opcode.mode),

                /* TAX */
                0xAA => self.tax(),

                /* INX */
                0xe8 => self.inx(),

                /* BRK */
                0x00 => return Ok(false),
                // Belows results in "attempt to write to cartridge PRG ROM space"
                /*0x00 => {
                    self.program_counter += 1;
                    if !self.status.contains(CPUFlags::INTERRUPT_DISABLE) {
                        self.interrupt(interrupt::BRK);
                    }
                }*/

                /* CLD */
                0xd8 => self.cld(),

                /* CLI */
                0x58 => self.cli(),

                /* CLV */
                0xb8 => self.clv(),

                /* CLC */
                0x18 => self.clc(),

                /* SEC */
                0x38 => self.sec(),

                /* SEI */
                0x78 => self.sei(),

                /* SED */
                0xf8 => self.sed(),

                /* PHA */
                0x48 => self.pha(),

                /* PLA */
                0x68 => self.pla(),

                /* PHP */
                0x08 => self.php(),

                /* PHP */
                0x28 => self.plp(),

                /* ADC */
                0x69 | 0x65 | 0x75 | 0x6d | 0x7d | 0x79 | 0x61 | 0x71 => self.adc(&opcode.mode),

                /* SBC */
                0xe9 | 0xe5 | 0xf5 | 0xed | 0xfd | 0xf9 | 0xe1 | 0xf1 => self.sbc(&opcode.mode),

                /* AND */
                0x29 | 0x25 | 0x35 | 0x2d | 0x3d | 0x39 | 0x21 | 0x31 => self.and(&opcode.mode),

                /* EOR */
                0x49 | 0x45 | 0x55 | 0x4d | 0x5d | 0x59 | 0x41 | 0x51 => self.eor(&opcode.mode),

                /* ORA */
                0x09 | 0x05 | 0x15 | 0x0d | 0x1d | 0x19 | 0x01 | 0x11 => self.ora(&opcode.mode),

                /* LSR */
                0x4a => self.lsr_accumulator(),

                /* LSR */
                0x46 | 0x56 | 0x4e | 0x5e => {
                    self.lsr(&opcode.mode);
                }

                /* ASL */
                0x0a => self.asl_accumulator(),

                /* ASL */
                0x06 | 0x16 | 0x0e | 0x1e => {
                    self.asl(&opcode.mode);
                }

                /*ROL*/ 0x2a => self.rol_accumulator(),

                /* ROL */
                0x26 | 0x36 | 0x2e | 0x3e => {
                    self.rol(&opcode.mode);
                }

                /* ROR */ 0x6a => self.ror_accumulator(),

                /* ROR */
                0x66 | 0x76 | 0x6e | 0x7e => {
                    self.ror(&opcode.mode);
                }

                /* INC */
                0xe6 | 0xf6 | 0xee | 0xfe => {
                    self.inc(&opcode.mode);
                }

                /* INY */
                0xc8 => self.iny(),

                /* DEC */
                0xc6 | 0xd6 | 0xce | 0xde => {
                    self.dec(&opcode.mode);
                }

                /* DEX */
                0xca => self.dex(),

                /* DEY */
                0x88 => self.dey(),

                /* CMP */
                0xc9 | 0xc5 | 0xd5 | 0xcd | 0xdd | 0xd9 | 0xc1 | 0xd1 => self.cmp(&opcode.mode),

                /* CPY */
                0xc0 | 0xc4 | 0xcc => self.cpy(&opcode.mode),

                /* CPX */
                0xe0 | 0xe4 | 0xec => self.cpx(&opcode.mode),

                /* JMP Absolute */
                0x4c => self.jmp_absolute(),

                /* JMP Indirect */
                0x6c => self.jmp_indirect(),

                /* JSR */
                0x20 => self.jsr(),

                /* RTS */
                0x60 => self.rts(),

                /* RTI */
                0x40 => self.rti(),

                /* BNE */
                0xd0 => self.bne(),

                /* BVS */
                0x70 => self.bvs(),

                /* BVC */
                0x50 => self.bvc(),

                /* BPL */
                0x10 => self.bpl(),

                /* BMI */
                0x30 => self.bmi(),

                /* BEQ */
                0xf0 => self.beq(),

                /* BCS */
                0xb0 => self.bcs(),

                /* BCC */
                0x90 => self.bcc(),

                /* BIT */
                0x24 | 0x2c => self.bit(&opcode.mode),

                /* STA */
                0x85 | 0x95 | 0x8d | 0x9d | 0x99 | 0x81 | 0x91 => self.sta(&opcode.mode),

                /* STX */
                0x86 | 0x96 | 0x8e => self.stx(&opcode.mode),

                /* STY */
                0x84 | 0x94 | 0x8c => self.sty(&opcode.mode),

                /* LDX */
                0xa2 | 0xa6 | 0xb6 | 0xae | 0xbe => self.ldx(&opcode.mode),

                /* LDY */
                0xa0 | 0xa4 | 0xb4 | 0xac | 0xbc => self.ldy(&opcode.mode),

                /* NOP */
                0xea => {
                    // Does nothing
                }

                /* TAY */
                0xa8 => self.tay(),

                /* TSX */
                0xba => self.tsx(),

                /* TXA */
                0x8a => self.txa(),

                /* TXS */
                0x9a => self.txs(),

                /* TYA */
                0x98 => self.tya(),

                /* *LAX */
                0xa7 | 0xb7 | 0xaf | 0xbf | 0xa3 | 0xb3 => self.lax(&opcode.mode),

                /* *SAX */
                0x87 | 0x97 | 0x8f | 0x83 => self.sax(&opcode.mode),

                /* *NOP */
                0x02 | 0x12 | 0x22 | 0x32 | 0x42 | 0x52 | 0x62 | 0x72 | 0x92 | 0xb2 | 0xd2
                | 0xf2 => { /* Nothing */ }

                /* *NOP */
                0x1a | 0x3a | 0x5a | 0x7a | 0xda | 0xfa => { /* Nothing */ }

                /* *NOP read */
                0x04 | 0x44 | 0x64 | 0x14 | 0x34 | 0x54 | 0x74 | 0xd4 | 0xf4 | 0x0c | 0x1c
                | 0x3c | 0x5c | 0x7c | 0xdc | 0xfc => {
                    let (addr, page_cross) = self.get_operand_address(&opcode.mode);
                    let data = self.mem_read(addr);
                    if page_cross {
                        self.remaining_cycles += 1;
                    }
                    /* Nothing */
                }

                /* *SBC */
                0xeb => self.unofficial_sbc(&opcode.mode),

                /* *DCP */
                0xc7 | 0xd7 | 0xCF | 0xdF | 0xdb | 0xd3 | 0xc3 => self.dcp(&opcode.mode),

                /* *ISB */
                0xe7 | 0xf7 | 0xef | 0xff | 0xfb | 0xe3 | 0xf3 => self.isb(&opcode.mode),

                /* *SLO */
                0x07 | 0x17 | 0x0F | 0x1f | 0x1b | 0x03 | 0x13 => self.slo(&opcode.mode),

                /* *RLA */
                0x27 | 0x37 | 0x2F | 0x3F | 0x3b | 0x33 | 0x23 => self.rla(&opcode.mode),

                /* *SRE */
                0x47 | 0x57 | 0x4F | 0x5f | 0x5b | 0x43 | 0x53 => self.sre(&opcode.mode),

                /* *RRA */
                0x67 | 0x77 | 0x6f | 0x7f | 0x7b | 0x63 | 0x73 => self.rra(&opcode.mode),

                /* *SKB */
                0x80 | 0x82 | 0x89 | 0xc2 | 0xe2 => {
                    /* 2 byte NOP (immediate ) */
                    // TODO might be worth doing the read (like NOP) ?
                }

                /* *AXS */
                0xcb => self.axs(&opcode.mode),

                /* *ARR */
                0x6b => self.arr(&opcode.mode),

                /* *ANC */
                0x0b | 0x2b => self.anc(&opcode.mode),

                /* *ALR */
                0x4b => self.alr(&opcode.mode),

                /* *LXA */
                0xab => self.lxa(&opcode.mode),

                /* *XAA */
                0x8b => self.xaa(&opcode.mode),

                /* *LAS */
                0xbb => self.las(&opcode.mode),

                /* *TAS */
                0x9b => self.tas(),

                /* *AHX  Indirect Y */
                0x93 => self.ahx_indirect_y(),

                /* *AHX  Absolute Y */
                0x9f => self.ahx_absolute_y(),

                /* *SHX */
                0x9e => self.shx(),

                /* *SHY */
                0x9c => self.shy(),
            };

            if program_counter_state == self.program_counter {
                self.program_counter += (opcode.len - 1) as u16;
            }
        }
        Ok(true)
    }

    /// Runs loaded program.
    pub fn run_with_callback<F>(&mut self, mut callback: F) -> Result<(), CPUError>
    where
        F: FnMut(&mut CPU),
    {
        let mut cont = true;
        while cont {
            // Callback is called only on instruction change
            // Not for every cycle
            if self.remaining_cycles == 0 {
                callback(self);
            }
            cont = self.tick()?;
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), CPUError> {
        self.run_with_callback(|_| {})
    }

    /// Indicates if CPU instruction changed since last tick.
    pub fn instruction_changed(&self) -> bool {
        self.remaining_cycles == 0
    }
}
