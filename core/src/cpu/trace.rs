use std::collections::HashMap;

use crate::memory::Memory;

use super::{instruction::AddressingMode, CPU, opcode};


/// CPU tracing.
pub trait Trace {
    fn trace(&mut self) -> String;
}

impl Trace for CPU {

    /// Traces CPU state to String.
    fn trace(&mut self) -> String {
    
        let ref opscodes: HashMap<u8, &'static opcode::OpCode> = *opcode::OPCODES_MAP;

        let code = self.mem_read(self.program_counter);
        let ops = opscodes.get(&code).expect(format!("unknown opcode {:#06x}", code).as_str());

        let begin = self.program_counter;
        let mut hex_dump = vec![];
        hex_dump.push(code);

        let (mem_addr, stored_value) = match ops.mode {
            AddressingMode::Immediate | AddressingMode::NoneAddressing => (0, 0),
            _ => {
                let addr = self.get_absolute_address(&ops.mode, begin + 1).0;
                (addr, self.mem_read(addr))
            }
        };

        let tmp = match ops.len {
            1 => match ops.code {
                0x0a | 0x4a | 0x2a | 0x6a => format!("A "),
                _ => String::from(""),
            },
            2 => {
                let address: u8 = self.mem_read(begin + 1);
                hex_dump.push(address);

                match ops.mode {
                    AddressingMode::Immediate => format!("#${:02x}", address),
                    AddressingMode::ZeroPage => format!("${:02x} = {:02x}", mem_addr, stored_value),
                    AddressingMode::ZeroPage_X => format!(
                        "${:02x},X @ {:02x} = {:02x}",
                        address, mem_addr, stored_value
                    ),
                    AddressingMode::ZeroPage_Y => format!(
                        "${:02x},Y @ {:02x} = {:02x}",
                        address, mem_addr, stored_value
                    ),
                    AddressingMode::Indirect_X => format!(
                        "(${:02x},X) @ {:02x} = {:04x} = {:02x}",
                        address,
                        (address.wrapping_add(self.register_x)),
                        mem_addr,
                        stored_value
                    ),
                    AddressingMode::Indirect_Y => format!(
                        "(${:02x}),Y = {:04x} @ {:04x} = {:02x}",
                        address,
                        (mem_addr.wrapping_sub(self.register_y as u16)),
                        mem_addr,
                        stored_value
                    ),
                    AddressingMode::NoneAddressing => {
                        // assuming local jumps: BNE, BVS, etc....
                        let address: usize =
                            (begin as usize + 2).wrapping_add((address as i8) as usize);
                        format!("${:04x}", address)
                    }

                    _ => panic!(
                        "unexpected addressing mode {:?} has ops-len 2. code {:02x}",
                        ops.mode, ops.code
                    ),
                }
            }
            3 => {
                let address_lo = self.mem_read(begin + 1);
                let address_hi = self.mem_read(begin + 2);
                hex_dump.push(address_lo);
                hex_dump.push(address_hi);

                let address = self.mem_read_u16(begin + 1);

                match ops.mode {
                    AddressingMode::NoneAddressing => {
                        if ops.code == 0x6c {
                            //jmp indirect
                            let jmp_addr = if address & 0x00FF == 0x00FF {
                                let lo = self.mem_read(address);
                                let hi = self.mem_read(address & 0xFF00);
                                (hi as u16) << 8 | (lo as u16)
                            } else {
                                self.mem_read_u16(address)
                            };

                            format!("(${:04x}) = {:04x}", address, jmp_addr)
                        } else {
                            format!("${:04x}", address)
                        }
                    }
                    AddressingMode::Absolute => format!("${:04x} = {:02x}", mem_addr, stored_value),
                    AddressingMode::Absolute_X => format!(
                        "${:04x},X @ {:04x} = {:02x}",
                        address, mem_addr, stored_value
                    ),
                    AddressingMode::Absolute_Y => format!(
                        "${:04x},Y @ {:04x} = {:02x}",
                        address, mem_addr, stored_value
                    ),
                    _ => panic!(
                        "unexpected addressing mode {:?} has ops-len 3. code {:02x}",
                        ops.mode, ops.code
                    ),
                }
            }
            _ => String::from(""),
        };

        let hex_str = hex_dump
            .iter()
            .map(|z| format!("{:02x}", z))
            .collect::<Vec<String>>()
            .join(" ");
        let asm_str = format!("{:04x}  {:8} {: >4} {}", begin, hex_str, ops.mnemonic, tmp)
            .trim()
            .to_string();

        format!(
            "{:47} A:{:02x} X:{:02x} Y:{:02x} P:{:02x} SP:{:02x}",
            asm_str, self.register_a, self.register_x, self.register_y, self.status, self.stack_pointer,
        )
        .to_ascii_uppercase()
    }
}
