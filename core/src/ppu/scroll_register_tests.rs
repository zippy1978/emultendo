use crate::ppu::scroll_register::ScrollRegister;


#[test]
fn test_scroll_register_write() {
    let mut scroll_register = ScrollRegister::new();
    scroll_register.write(0x22);
    assert_eq!(scroll_register.scroll_x, 0x22);
    assert_eq!(scroll_register.scroll_y, 0);
    assert!(scroll_register.latch);

    scroll_register.write(0x33);
    assert_eq!(scroll_register.scroll_x, 0x22);
    assert_eq!(scroll_register.scroll_y, 0x33);
    assert!(!scroll_register.latch);

    scroll_register.write(0x44);
    assert_eq!(scroll_register.scroll_x, 0x44);
    assert_eq!(scroll_register.scroll_y, 0x33);
    assert!(scroll_register.latch);
}

#[test]
fn test_scroll_register_reset_latch() {
    let mut scroll_register = ScrollRegister::new();
    scroll_register.write(0x22);
    assert!(scroll_register.latch);

    scroll_register.reset_latch();
    assert!(!scroll_register.latch);
}
