use bitflags::bitflags;
use log::debug;

const PRG_ROM_PAGE_SIZE: usize = 0x4000;
const PRG_RAM_PAGE_SIZE: usize = 0x2000;
const CHR_ROM_PAGE_SIZE: usize = 0x2000;
const CHR_RAM_PAGE_SIZE: usize = 0x2000;

bitflags! {
    pub struct RomFlags: u8 {
        const MIRRORING = 0b0000_0001;
        const TRAINER = 0b0000_0100;
    }
}

#[derive(Debug)]
struct RomHeader {
    magic: bool,
    prg_rom_bytes: usize,
    chr_rom_bytes: usize,
    prg_ram_bytes: usize,
    chr_ram_bytes: usize,
    flags: RomFlags,
}

#[derive(Debug)]
pub struct Rom {
    header: RomHeader,
    prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
}

impl Rom {
    pub fn new_from_ines(data: &[u8]) -> Self {
        let prg_rom_bytes = data[4] as usize * PRG_ROM_PAGE_SIZE;
        let chr_rom_bytes = data[5] as usize * CHR_ROM_PAGE_SIZE;
        let header = RomHeader {
            magic: data[0..4] == [0x4e, 0x45, 0x53, 0x1a],
            prg_rom_bytes,
            chr_rom_bytes,
            prg_ram_bytes: if data[8] == 0 {
                PRG_RAM_PAGE_SIZE
            } else {
                data[8] as usize * PRG_RAM_PAGE_SIZE
            },
            chr_ram_bytes: if data[5] == 0 { CHR_RAM_PAGE_SIZE } else { 0 },
            flags: RomFlags { bits: data[6] },
        };

        let prg_rom_offset = if header.flags.contains(RomFlags::TRAINER) {
            528
        } else {
            16
        };
        let chr_rom_offset = prg_rom_offset + prg_rom_bytes;

        Rom {
            header,
            prg_rom: data[prg_rom_offset..(prg_rom_offset + prg_rom_bytes)].to_vec(),
            chr_rom: data[chr_rom_offset..(chr_rom_offset + chr_rom_bytes)].to_vec(),
        }
    }

    pub fn new_from_vec(prg_rom: Vec<u8>) -> Self {
        Rom {
            header: RomHeader {
                magic: false,
                prg_rom_bytes: 0,
                chr_rom_bytes: 0,
                prg_ram_bytes: 0,
                chr_ram_bytes: 0,
                flags: RomFlags { bits: 0 },
            },
            prg_rom,
            chr_rom: vec![],
        }
    }
    pub fn read_byte(&self, address: u16) -> u8 {
        // This only implements mapper0
        // TODO: implement other mappers
        let mut addr = address - 0x8000;
        if self.prg_rom.len() <= 0x4000 && addr >= 0x4000 {
            addr %= 0x4000;
        }
        self.prg_rom[addr as usize]
    }

    pub fn mirroring(&self) -> bool {
        self.header.flags.contains(RomFlags::MIRRORING)
    }
}
