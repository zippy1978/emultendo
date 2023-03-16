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
fn test_increment() {
    let mut reg = AddrRegister::new();
    reg.update(0x06);
    reg.update(0x00);
    reg.increment(0x01);
    assert_eq!(reg.get(), 0x0601); 
}
