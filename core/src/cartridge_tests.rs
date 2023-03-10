use std::path::PathBuf;

use crate::cartridge::Cartridge;

#[test]
fn test_from_file() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("res/test.nes");
    let cartridge = Cartridge::from_file(path).unwrap();
    assert!(cartridge.prg_rom.contains(&32));
    assert_eq!(cartridge.mapper, 0);
    assert!(matches!(cartridge.screen_mirroring, crate::cartridge::Mirroring::Vertical));
    // CHR_ROM not tested as test cartridge does not have gfx
}