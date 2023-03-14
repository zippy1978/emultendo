use crate::{
    cpu::{instruction_tests::run_code, CPUFlags, CPU},
    memory::Memory,
};

#[test]
fn test_0xaf_lax() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x10, 0x02);
    run_code(&mut cpu, vec![0xaf, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0x02);
    assert_eq!(cpu.register_x, 0x02);
}

#[test]
fn test_0x8f_sax() {
    let mut cpu = CPU::new();
    cpu.register_a = 0b1;
    cpu.register_x = 0b1;
    run_code(&mut cpu, vec![0x8f, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.mem_read(0x10), 0b1);
}

#[test]
fn test_unofficial_sbc() {
    let mut cpu = CPU::new();
    cpu.status.insert(CPUFlags::CARRY);
    cpu.register_a = 0x05;
    run_code(&mut cpu, vec![0xeb, 0x02, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0x03);
}

#[test]
fn test_0xcf_dcp() {
    let mut cpu = CPU::new();
    cpu.mem_write(0x10, 0x02);
    run_code(&mut cpu, vec![0xcf, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.mem_read(0x10), 0x01);
}

#[test]
fn test_0xef_isb() {
    let mut cpu = CPU::new();
    cpu.status.insert(CPUFlags::CARRY);
    cpu.register_a = 0x06;
    cpu.mem_write(0x10, 0x02);
    run_code(&mut cpu, vec![0xef, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.mem_read(0x10), 0x03);
    assert_eq!(cpu.register_a, 0x03);
}

#[test]
fn test_0x0f_slo() {
    let mut cpu = CPU::new();
    cpu.register_a = 0b1;
    cpu.mem_write(0x10, 0b1);
    run_code(&mut cpu, vec![0x0f, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0b11);
}

#[test]
fn test_0x2f_rla() {
    let mut cpu = CPU::new();
    cpu.register_a = 0b10;
    cpu.mem_write(0x10, 0b1);
    run_code(&mut cpu, vec![0x2f, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0b10);
}

#[test]
fn test_0x4f_sre() {
    let mut cpu = CPU::new();
    cpu.register_a = 0b10;
    cpu.mem_write(0x10, 0b10);
    run_code(&mut cpu, vec![0x4f, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0b11);
}

#[test]
fn test_0x6f_rra() {
    let mut cpu = CPU::new();
    cpu.register_a = 0b10;
    cpu.mem_write(0x10, 0b10);
    run_code(&mut cpu, vec![0x6f, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0b11);
}

#[test]
fn test_axs() {
    let mut cpu = CPU::new();
    cpu.register_a = 0x01;
    cpu.register_x = 0x01;
    run_code(&mut cpu, vec![0xcb, 0x01, 0x00]).unwrap();
    assert_eq!(cpu.register_x, 0x0);
}

#[test]
fn test_arr() {
    let mut cpu = CPU::new();
    cpu.register_a = 0b1;
    run_code(&mut cpu, vec![0x6b, 0b1, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0b0);
    assert!(!cpu.status.contains(CPUFlags::CARRY));
    assert!(!cpu.status.contains(CPUFlags::OVERFLOW));
}

#[test]
fn test_0x0b_anc() {
    let mut cpu = CPU::new();
    cpu.register_a = 0b1;
    run_code(&mut cpu, vec![0x0b, 0b1, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0b1);
    assert!(!cpu.status.contains(CPUFlags::CARRY));
}

#[test]
fn test_alr() {
    let mut cpu = CPU::new();
    cpu.register_a = 0b1;
    run_code(&mut cpu, vec![0x4b, 0b1, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0b0);
    assert!(cpu.status.contains(CPUFlags::CARRY));
}

#[test]
fn test_lxa() {
    let mut cpu = CPU::new();
    cpu.register_a = 0b1;
    run_code(&mut cpu, vec![0xab, 0b1, 0x00]).unwrap();
    assert_eq!(cpu.register_x, 0b1);
}

#[test]
fn test_xaa() {
    let mut cpu = CPU::new();
    cpu.register_x = 0b1;
    run_code(&mut cpu, vec![0x8b, 0b1, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0b1);
}

#[test]
fn test_las() {
    let mut cpu = CPU::new();
    cpu.stack_pointer = 0b1;
    cpu.mem_write(0x10, 0b1);
    run_code(&mut cpu, vec![0xbb, 0x10, 0x00]).unwrap();
    assert_eq!(cpu.register_a, 0b1);
    assert_eq!(cpu.register_x, 0b1);
    assert_eq!(cpu.stack_pointer, 0b1);
}

#[test]
fn test_tas() {
    let mut cpu = CPU::new();
    cpu.register_a = 0b1;
    cpu.register_x = 0b1;
    run_code(&mut cpu, vec![0x9b, 0x00]).unwrap();
    assert_eq!(cpu.stack_pointer, 0b1);
}

