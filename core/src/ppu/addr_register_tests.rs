use super::addr_register::AddrRegister;

#[test]
fn test_update() {
    let mut reg = AddrRegister::new();
    reg.update(0x06);
    reg.update(0x00);
    assert_eq!(reg.get(), 0x0600);
}

#[test]
fn test_update_mirror() {
    let mut reg = AddrRegister::new();
    reg.update(0x4f);
    reg.update(0xff);
    assert_eq!(reg.get(), 0x4fff - 0x3fff - 1);
}

#[test]
fn test_increment_within_bounds() {
    let mut reg = AddrRegister::new();
    reg.set(0x1234);
    reg.increment(0x01);
    assert_eq!(reg.get(), 0x1235);
}

#[test]
fn test_increment_wraps_around() {
    let mut reg = AddrRegister::new();
    reg.set(0x3fff);
    reg.increment(0x01);
    assert_eq!(reg.get(), 0x0000);
}

#[test]
fn test_increment_overflow() {
    let mut reg = AddrRegister::new();
    reg.set(0x3fff);
    reg.increment(0xff);
    assert_eq!(reg.get(), 0x00fe);
}