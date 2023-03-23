use crate::{cpu::{Memory, Cpu, CpuFlags}};

use super::CpuError;

pub(crate) fn run_code(cpu: &mut Cpu, code: Vec<u8>) -> Result<(), CpuError>{
    cpu.load(code);
    cpu.program_counter = cpu.mem_read_u16(0xFFFC);
    cpu.run()
}

#[test]
fn test_0xa9_lda_immediate_load_data() {
    let mut cpu = Cpu::new();
    run_code(&mut cpu,vec![0xa9, 0x05, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 5);
    assert!(cpu.status.bits() & 0b0000_0010 == 0);
    assert!(cpu.status.bits() & 0b1000_0000 == 0);
}

#[test]
fn test_0xa9_lda_zero_flag() {
    let mut cpu = Cpu::new();
    run_code(&mut cpu,vec![0xa9, 0x00, 0x00]).unwrap();
    assert!(cpu.status.bits() & 0b0000_0010 == 0b10);
}

#[test]
fn test_0xaa_tax() {
    let mut cpu = Cpu::new();
    run_code(&mut cpu,vec![0xa9, 0x0A, 0xaa, 0x00]).unwrap();

    assert_eq!(cpu.register_x, 10)
}

#[test]
fn test_inx_overflow() {
    let mut cpu = Cpu::new();
    run_code(&mut cpu,vec![0xa9, 0xff, 0xaa, 0xe8, 0xe8, 0x00]).unwrap();

    assert_eq!(cpu.register_x, 1)
}

#[test]
fn test_lda_from_memory() {
    let mut cpu = Cpu::new();
    cpu.mem_write(0x10, 0x55);

    run_code(&mut cpu,vec![0xa5, 0x10, 0x00]).unwrap();

    assert_eq!(cpu.register_a, 0x55);
}

#[test]
fn test_cld() {
    let mut cpu = Cpu::new();
    run_code(&mut cpu,vec![0xd8, 0x00]).unwrap();
    assert!(cpu.status.bits() & 0b0000_1000 == 0);
}

#[test]
fn test_cli() {
    let mut cpu = Cpu::new();
    run_code(&mut cpu,vec![0x58, 0x00]).unwrap();
    assert!(cpu.status.bits() & 0b0000_0100 == 0);
}

#[test]
fn test_clv() {
    let mut cpu = Cpu::new();
    run_code(&mut cpu,vec![0xb8, 0x00]).unwrap();
    assert!(cpu.status.bits() & 0b0100_0000 == 0);
}

#[test]
fn test_clc() {
    let mut cpu = Cpu::new();
    run_code(&mut cpu,vec![0x18, 0x00]).unwrap();
    assert!(cpu.status.bits() & 0b0000_0001 == 0);
}

#[test]
fn test_sec() {
    let mut cpu = Cpu::new();
    run_code(&mut cpu,vec![0x38, 0x00]).unwrap();
    assert!(cpu.status.bits() & 0b0000_0001 == 0b1);
}

#[test]
fn test_sei() {
    let mut cpu = Cpu::new();
    run_code(&mut cpu,vec![0x78, 0x00]).unwrap();
    assert!(cpu.status.bits() & 0b0000_0100 == 0b100);
}

#[test]
fn test_sed() {
    let mut cpu = Cpu::new();
    run_code(&mut cpu,vec![0xf8, 0x00]).unwrap();
    assert!(cpu.status.bits() & 0b0000_1000 == 0b1000);
}

#[test]
fn test_pha() {
    let mut cpu = Cpu::new();
    run_code(&mut cpu,vec![0xa9, 0x05, 0x48, 0x00]).unwrap();
    assert_eq!(cpu.stack_pop(), 0x05);
}

#[test]
fn test_pla() {
    let mut cpu = Cpu::new();
    // Push 5 to a, push a to stack, push 0 to a, then pop stack to a
    run_code(&mut cpu,vec![0xa9, 0x05, 0x48, 0xa9, 0x00, 0x68, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0x05);
}

#[test]
fn test_php() {
    let mut cpu = Cpu::new();
    run_code(&mut cpu,vec![0x08, 0x00]).unwrap();
    assert!(cpu.stack_pop() & 0b0011_0000 == 0b110000);
}

#[test]
fn test_plp() {
    let mut cpu = Cpu::new();
    run_code(&mut cpu,vec![0x08, 0x28, 0x00]).unwrap();
    assert!(cpu.stack_pop() & 0b0000_0000 == 0);
}

#[test]
fn test_0x6d_adc() {
    let mut cpu = Cpu::new();
    cpu.mem_write(0x10, 0x02);
    // Push 5 to a, add mem 0x10 to a
    run_code(&mut cpu,vec![0xa9, 0x05, 0x6d, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0x07);
}

#[test]
fn test_0xed_sbc() {
    let mut cpu = Cpu::new();
    cpu.mem_write(0x10, 0x02);
    // Push 5 to a, sub mem 0x10 from a
    run_code(&mut cpu,vec![0xa9, 0x05, 0xed, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0x02);
}

#[test]
fn test_0x2d_and() {
    let mut cpu = Cpu::new();
    cpu.mem_write(0x10, 0x02);
    // Push 5 to a, and mem 0x10 to a
    run_code(&mut cpu,vec![0xa9, 0x05, 0x2d, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0x05 & 0x02);
}

#[test]
fn test_0x4d_eor() {
    let mut cpu = Cpu::new();
    cpu.mem_write(0x10, 0x02);
    // Push 5 to a, eor mem 0x10 to a
    run_code(&mut cpu,vec![0xa9, 0x05, 0x4d, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0x05 ^ 0x02);
}

#[test]
fn test_0x0d_ora() {
    let mut cpu = Cpu::new();
    cpu.mem_write(0x10, 0x02);
    // Push 5 to a, ora mem 0x10 to a
    run_code(&mut cpu,vec![0xa9, 0x05, 0x0d, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0x05 | 0x02);
}

#[test]
fn test_lsr_accumulator() {
    let mut cpu = Cpu::new();
    run_code(&mut cpu,vec![0xa9, 0b0000_00010, 0x4a, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0b0000_00001);
}

#[test]
fn test_0x4e_lsr() {
    let mut cpu = Cpu::new();
    cpu.mem_write(0x10, 0b0000_00010);
    run_code(&mut cpu,vec![0x4e, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.mem_read(0x10), 0b0000_00001);
}

#[test]
fn test_asl_accumulator() {
    let mut cpu = Cpu::new();
    run_code(&mut cpu,vec![0xa9, 0b0000_00001, 0x0a, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0b0000_00010);
}

#[test]
fn test_0x0e_lsr() {
    let mut cpu = Cpu::new();
    cpu.mem_write(0x10, 0b0000_00001);
    run_code(&mut cpu,vec![0x0e, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.mem_read(0x10), 0b0000_00010);
}

#[test]
fn test_rol_accumulator() {
    let mut cpu = Cpu::new();
    cpu.status.insert(CpuFlags::CARRY);
    run_code(&mut cpu,vec![0xa9, 0b0100_00001, 0x2a, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0b0000_00011);
    assert!(cpu.status.contains(CpuFlags::CARRY));
}

#[test]
fn test_0x2e_rol() {
    let mut cpu = Cpu::new();
    cpu.status.insert(CpuFlags::CARRY);
    cpu.mem_write(0x10, 0b0100_00001);
    run_code(&mut cpu,vec![0x2e, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.mem_read(0x10), 0b0000_00011);
    assert!(cpu.status.contains(CpuFlags::CARRY));
}

#[test]
fn test_ror_accumulator() {
    let mut cpu = Cpu::new();
    cpu.status.insert(CpuFlags::CARRY);
    run_code(&mut cpu,vec![0xa9, 0b0000_00010, 0x6a, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0b0100_00001);
    assert!(!cpu.status.contains(CpuFlags::CARRY));
}

#[test]
fn test_0x6e_ror() {
    let mut cpu = Cpu::new();
    cpu.status.insert(CpuFlags::CARRY);
    cpu.mem_write(0x10, 0b0000_00010);
    run_code(&mut cpu,vec![0x6e, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.mem_read(0x10), 0b0100_00001);
    assert!(!cpu.status.contains(CpuFlags::CARRY));
}

#[test]
fn test_0xee_inc() {
    let mut cpu = Cpu::new();
    cpu.mem_write(0x10, 0x01);
    run_code(&mut cpu,vec![0xee, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.mem_read(0x10), 0x02);
}

#[test]
fn test_iny() {
    let mut cpu = Cpu::new();
    cpu.register_y = 0x01;
    run_code(&mut cpu,vec![0xc8, 0x00]).unwrap();
    assert_eq!(cpu.register_y, 0x02);
}

#[test]
fn test_0xce_dec() {
    let mut cpu = Cpu::new();
    cpu.mem_write(0x10, 0x01);
    run_code(&mut cpu,vec![0xce, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.mem_read(0x10), 0x0);
}

#[test]
fn test_dex() {
    let mut cpu = Cpu::new();
    cpu.register_x = 0x01;
    run_code(&mut cpu,vec![0xca, 0x00]).unwrap();
    assert_eq!(cpu.register_x, 0x0);
}

#[test]
fn test_dey() {
    let mut cpu = Cpu::new();
    cpu.register_y = 0x01;
    run_code(&mut cpu,vec![0x88, 0x00]).unwrap();
    assert_eq!(cpu.register_y, 0x0);
}

#[test]
fn test_0xcd_cmp() {
    let mut cpu = Cpu::new();
    // A = M
    cpu.register_a = 0x01;
    cpu.mem_write(0x10, 0x01);
    run_code(&mut cpu,vec![0xcd, 0x10, 0x00]).unwrap();
    assert!(cpu.status.contains(CpuFlags::CARRY));
    assert!(cpu.status.contains(CpuFlags::ZERO));
    // A < M
    cpu.register_a = 0x01;
    cpu.mem_write(0x10, 0x05);
    run_code(&mut cpu,vec![0xcd, 0x10, 0x00]).unwrap();
    assert!(!cpu.status.contains(CpuFlags::CARRY));
    assert!(!cpu.status.contains(CpuFlags::ZERO));
    // A > M
    cpu.register_a = 0x05;
    cpu.mem_write(0x10, 0x01);
    run_code(&mut cpu,vec![0xcd, 0x10, 0x00]).unwrap();
    assert!(cpu.status.contains(CpuFlags::CARRY));
    assert!(!cpu.status.contains(CpuFlags::ZERO));
}

#[test]
fn test_0xcc_cpy() {
    let mut cpu = Cpu::new();
    // X = M
    cpu.register_y = 0x01;
    cpu.mem_write(0x10, 0x01);
    run_code(&mut cpu,vec![0xcc, 0x10, 0x00]).unwrap();
    assert!(cpu.status.contains(CpuFlags::CARRY));
    assert!(cpu.status.contains(CpuFlags::ZERO));
    // X < M
    cpu.register_y = 0x01;
    cpu.mem_write(0x10, 0x05);
    run_code(&mut cpu,vec![0xcc, 0x10, 0x00]).unwrap();
    assert!(!cpu.status.contains(CpuFlags::CARRY));
    assert!(!cpu.status.contains(CpuFlags::ZERO));
    // X > M
    cpu.register_y = 0x05;
    cpu.mem_write(0x10, 0x01);
    run_code(&mut cpu,vec![0xcc, 0x10, 0x00]).unwrap();
    assert!(cpu.status.contains(CpuFlags::CARRY));
    assert!(!cpu.status.contains(CpuFlags::ZERO));
}

#[test]
fn test_0xec_cpx() {
    let mut cpu = Cpu::new();
    // X = M
    cpu.register_x = 0x01;
    cpu.mem_write(0x10, 0x01);
    run_code(&mut cpu,vec![0xec, 0x10, 0x00]).unwrap();
    assert!(cpu.status.contains(CpuFlags::CARRY));
    assert!(cpu.status.contains(CpuFlags::ZERO));
    // X < M
    cpu.register_x = 0x01;
    cpu.mem_write(0x10, 0x05);
    run_code(&mut cpu,vec![0xec, 0x10, 0x00]).unwrap();
    assert!(!cpu.status.contains(CpuFlags::CARRY));
    assert!(!cpu.status.contains(CpuFlags::ZERO));
    // X > M
    cpu.register_x = 0x05;
    cpu.mem_write(0x10, 0x01);
    run_code(&mut cpu,vec![0xec, 0x10, 0x00]).unwrap();
    assert!(cpu.status.contains(CpuFlags::CARRY));
    assert!(!cpu.status.contains(CpuFlags::ZERO));
}

#[test]
fn test_jmp_indirect() {
    let mut cpu = Cpu::new();
    run_code(&mut cpu,vec![0x6c, 0x10]).unwrap();
    assert_eq!(cpu.program_counter, 0x01);
}

#[test]
fn test_jmp_absolute() {
    let mut cpu = Cpu::new();
    run_code(&mut cpu,vec![0x4c, 0x10]).unwrap();
    assert_eq!(cpu.program_counter, 0x11);
}

#[test]
fn test_rts() {
    let mut cpu = Cpu::new();
    cpu.stack_push_u16(0x05);
    run_code(&mut cpu,vec![0x60]).unwrap();
    assert_eq!(cpu.program_counter, 0x07);
}

#[test]
fn test_rti() {
    let mut cpu = Cpu::new();
    cpu.stack_push(0b1111_1111);
    run_code(&mut cpu,vec![0x40]).unwrap();
    cpu.status.bits();
    assert!(cpu.status.bits() & 0b0001_0000 == 0)
}

#[test]
fn test_bne() {
    let mut cpu = Cpu::new();
    cpu.status.remove(CpuFlags::ZERO);
    // Push 5 in a if branching is ok
    run_code(&mut cpu,vec![0xd0, 0x01, 0x00, 0xa9, 0x05]).unwrap();
    assert_eq!(cpu.register_a, 0x05);
}

#[test]
fn test_bvs() {
    let mut cpu = Cpu::new();
    cpu.status.insert(CpuFlags::OVERFLOW);
    // Push 5 in a if branching is ok
    run_code(&mut cpu,vec![0x70, 0x01, 0x00, 0xa9, 0x05]).unwrap();
    assert_eq!(cpu.register_a, 0x05);
}

#[test]
fn test_bvc() {
    let mut cpu = Cpu::new();
    cpu.status.remove(CpuFlags::OVERFLOW);
    // Push 5 in a if branching is ok
    run_code(&mut cpu,vec![0x50, 0x01, 0x00, 0xa9, 0x05]).unwrap();
    assert_eq!(cpu.register_a, 0x05);
}

#[test]
fn test_bpl() {
    let mut cpu = Cpu::new();
    cpu.status.remove(CpuFlags::NEGATIV);
    // Push 5 in a if branching is ok
    run_code(&mut cpu,vec![0x10, 0x01, 0x00, 0xa9, 0x05]).unwrap();
    assert_eq!(cpu.register_a, 0x05);
}

#[test]
fn test_bmi() {
    let mut cpu = Cpu::new();
    cpu.status.insert(CpuFlags::NEGATIV);
    // Push 5 in a if branching is ok
    run_code(&mut cpu,vec![0x30, 0x01, 0x00, 0xa9, 0x05]).unwrap();
    assert_eq!(cpu.register_a, 0x05);
}

#[test]
fn test_beq() {
    let mut cpu = Cpu::new();
    cpu.status.insert(CpuFlags::ZERO);
    // Push 5 in a if branching is ok
    run_code(&mut cpu,vec![0xf0, 0x01, 0x00, 0xa9, 0x05]).unwrap();
    assert_eq!(cpu.register_a, 0x05);
}

#[test]
fn test_bcs() {
    let mut cpu = Cpu::new();
    cpu.status.insert(CpuFlags::CARRY);
    // Push 5 in a if branching is ok
    run_code(&mut cpu,vec![0xb0, 0x01, 0x00, 0xa9, 0x05]).unwrap();
    assert_eq!(cpu.register_a, 0x05);
}

#[test]
fn test_bcc() {
    let mut cpu = Cpu::new();
    cpu.status.remove(CpuFlags::CARRY);
    // Push 5 in a if branching is ok
    run_code(&mut cpu,vec![0x90, 0x01, 0x00, 0xa9, 0x05]).unwrap();
    assert_eq!(cpu.register_a, 0x05);
}

#[test]
fn test_0x2c_bit() {
    let mut cpu = Cpu::new();
    cpu.register_a = 0x05;
    run_code(&mut cpu,vec![0x2c, 0x05, 0x00]).unwrap();
    assert!(cpu.status.contains(CpuFlags::ZERO));
}

#[test]
fn test_0x8d_sta() {
    let mut cpu = Cpu::new();
    cpu.register_a = 0x05;
    run_code(&mut cpu,vec![0x8d, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.mem_read(0x10), 0x05);
}

#[test]
fn test_0x8e_stx() {
    let mut cpu = Cpu::new();
    cpu.register_x = 0x05;
    run_code(&mut cpu,vec![0x8e, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.mem_read(0x10), 0x05);
}

#[test]
fn test_0x8c_sty() {
    let mut cpu = Cpu::new();
    cpu.register_y = 0x05;
    run_code(&mut cpu,vec![0x8c, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.mem_read(0x10), 0x05);
}

#[test]
fn test_0xae_ldx() {
    let mut cpu = Cpu::new();
    cpu.mem_write(0x10, 0x05);
    run_code(&mut cpu,vec![0xae, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.register_x, 0x05);
}

#[test]
fn test_0xac_ldy() {
    let mut cpu = Cpu::new();
    cpu.mem_write(0x10, 0x05);
    run_code(&mut cpu,vec![0xac, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.register_y, 0x05);
}

#[test]
fn test_tay() {
    let mut cpu = Cpu::new();
    cpu.register_a = 0x05;
    run_code(&mut cpu,vec![0xa8, 0x00]).unwrap();
    assert_eq!(cpu.register_y, 0x05);
}

#[test]
fn test_tsx() {
    let mut cpu = Cpu::new();
    cpu.stack_pointer = 0x05;
    run_code(&mut cpu,vec![0xba, 0x00]).unwrap();
    assert_eq!(cpu.register_x, 0x05);
}

#[test]
fn test_txa() {
    let mut cpu = Cpu::new();
    cpu.register_x = 0x05;
    run_code(&mut cpu,vec![0x8a, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0x05);
}

#[test]
fn test_txs() {
    let mut cpu = Cpu::new();
    cpu.register_x = 0x05;
    run_code(&mut cpu,vec![0x9a, 0x00]).unwrap();
    assert_eq!(cpu.stack_pointer, 0x05);
}

#[test]
fn test_tya() {
    let mut cpu = Cpu::new();
    cpu.register_y = 0x05;
    run_code(&mut cpu, vec![0x98, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0x05);
}