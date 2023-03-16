use super::control_register::ControlRegister;

#[test]
fn test_update() {
    let mut reg = ControlRegister::new();
    reg.update(0b11111111);
    assert_eq!(reg.bits(), 0b11111111);
}