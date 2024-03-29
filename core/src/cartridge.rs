use std::path::Path;

const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
const PRG_ROM_PAGE_SIZE: usize = 16384;
const CHR_ROM_PAGE_SIZE: usize = 8192;

/// Cartridge mirroring mode.
#[derive(Debug, PartialEq, Clone, Copy, Eq)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScreen,
}

/// Cartridge Error.
#[derive(Debug)]
pub enum CartridgeError {
    InvalidFormat(String),
    Io(String),
}

/// NES cartridge.
pub struct Cartridge {
    pub(crate) prg_rom: Vec<u8>,
    pub(crate) chr_rom: Vec<u8>,
    pub(crate) mapper: u8,
    pub(crate) screen_mirroring: Mirroring,
}

impl Cartridge {
    /// Creates a cartridge from raw bytes.
    /// Supports iNES format only.
    pub fn new(raw: &Vec<u8>) -> Result<Self, CartridgeError> {
        if &raw[0..4] != NES_TAG {
            return Err(CartridgeError::InvalidFormat(
                "File is not in iNES file format".to_string(),
            ));
        }

        let mapper = (raw[7] & 0b1111_0000) | (raw[6] >> 4);

        let ines_ver = (raw[7] >> 2) & 0b11;
        if ines_ver != 0 {
            return Err(CartridgeError::InvalidFormat(
                "NES2.0 format is not supported".to_string(),
            ));
        }

        let four_screen = raw[6] & 0b1000 != 0;
        let vertical_mirroring = raw[6] & 0b1 != 0;
        let screen_mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FourScreen,
            (false, true) => Mirroring::Vertical,
            (false, false) => Mirroring::Horizontal,
        };

        let prg_rom_size = raw[4] as usize * PRG_ROM_PAGE_SIZE;
        let chr_rom_size = raw[5] as usize * CHR_ROM_PAGE_SIZE;

        let skip_trainer = raw[6] & 0b100 != 0;

        let prg_rom_start = 16 + if skip_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        Ok(Self {
            prg_rom: raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec(),
            chr_rom: raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec(),
            mapper: mapper,
            screen_mirroring: screen_mirroring,
        })
    }

    pub fn chr_rom(&self) -> &Vec<u8> {
        &self.chr_rom
    }

    pub fn screen_mirroring(&self) -> &Mirroring {
        &self.screen_mirroring
    }

    /// Creates cartridge from file.
    pub fn from_file<P>(file: P) -> Result<Self, CartridgeError>
    where
        P: AsRef<Path>,
    {
        let bytes: Vec<u8> = std::fs::read(file).map_err(|e| CartridgeError::Io(e.to_string()))?;
        Self::new(&bytes)
    }
}
