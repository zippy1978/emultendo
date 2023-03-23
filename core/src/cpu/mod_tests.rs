use crate::cpu::Cpu;

#[test]
fn test_5_ops_working_together() {
    let mut cpu = Cpu::new();
    cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00])
        .unwrap();

    assert_eq!(cpu.register_x, 0xc1)
}
