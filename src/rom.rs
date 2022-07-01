const PRG_ROM_PAGE_SIZE: usize = 0x4000;
const PRG_RAM_PAGE_SIZE: usize = 0x2000;
const CHR_ROM_PAGE_SIZE: usize = 0x2000;
const CHR_RAM_PAGE_SIZE: usize = 0x2000;

#[derive(Debug)]
struct RomHeader {
    magic: bool,
    prg_rom_bytes: usize,
    chr_rom_bytes: usize,
    prg_ram_bytes: usize,
    chr_ram_bytes: usize,
    flags: u16,
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
            flags: data[6] as u16 | (data[7] as u16 >> 8),
        };

        Rom {
            header,
            prg_rom: data[16..(16 + prg_rom_bytes)].to_vec(),
            chr_rom: data[(16 + prg_rom_bytes)..(16 + prg_rom_bytes + chr_rom_bytes)].to_vec(),
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
                flags: 0,
            },
            prg_rom,
            chr_rom: vec![],
        }
    }
    pub fn read_byte(&self, address: u16) -> u8 {
        // This only implements mapper0
        // TODO: implement other mappers
        self.prg_rom[((address - 0x8000) % 0x4000) as usize]
    }
}
