use crate::ppu::status_register::StatusRegister;

#[test]
fn test_status_register_new() {
    let status_register = StatusRegister::new();
    assert_eq!(status_register.bits(), 0b00000000);
}

#[test]
fn test_status_register_set_vblank_status() {
    let mut status_register = StatusRegister::new();
    status_register.set_vblank_status(true);
    assert_eq!(status_register.bits(), 0b10000000);

    status_register.set_vblank_status(false);
    assert_eq!(status_register.bits(), 0b00000000);
}

#[test]
fn test_status_register_set_sprite_zero_hit() {
    let mut status_register = StatusRegister::new();
    status_register.set_sprite_zero_hit(true);
    assert_eq!(status_register.bits(), 0b01000000);

    status_register.set_sprite_zero_hit(false);
    assert_eq!(status_register.bits(), 0b00000000);
}

#[test]
fn test_status_register_set_sprite_overflow() {
    let mut status_register = StatusRegister::new();
    status_register.set_sprite_overflow(true);
    assert_eq!(status_register.bits(), 0b00100000);

    status_register.set_sprite_overflow(false);
    assert_eq!(status_register.bits(), 0b00000000);
}

#[test]
fn test_status_register_reset_vblank_status() {
    let mut status_register = StatusRegister::new();
    status_register.set_vblank_status(true);
    status_register.reset_vblank_status();
    assert_eq!(status_register.bits(), 0b00000000);
}

#[test]
fn test_status_register_is_in_vblank() {
    let mut status_register = StatusRegister::new();
    status_register.set_vblank_status(true);
    assert_eq!(status_register.is_in_vblank(), true);

    status_register.set_vblank_status(false);
    assert_eq!(status_register.is_in_vblank(), false);
}

#[test]
fn test_status_register_snapshot() {
    let mut status_register = StatusRegister::new();
    status_register.set_vblank_status(true);
    status_register.snapshot();
    assert_eq!(status_register.bits(), 0b10000000);
}
