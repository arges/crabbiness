use crate::ppu::Ppu;
use crate::rom::Rom;
use log::debug;
use std::borrow::Borrow;

pub struct Bus {
    pub ram: [u8; 2048],
    rom: Rom,
    pub ppu: Ppu,
    cycle: usize,
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        let ppu = Ppu::new(rom.chr_rom.clone(), rom.mirroring());

        Bus {
            ram: [0; 2048],
            rom,
            ppu,
            cycle: 0,
        }
    }

    pub fn tick(&mut self, cycle: u8) {
        self.cycle += cycle as usize;
        self.ppu.tick(cycle * 3);
    }

    pub fn take_nmi(&mut self) -> bool {
        if self.ppu.has_nmi {
            self.ppu.has_nmi = false;
            return true;
        }
        false
    }

    pub fn read_u8(&mut self, address: u16) -> u8 {
        debug!("reading @ {:04x}", address);
        match address {
            0x0000..=0x1fff => self.ram[address as usize % 0x0800],
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                panic!("Attempt to read from write-only PPU address {:x}", address);
            }
            0x2002 => self.ppu.read_ppustatus(),
            0x2007 => self.ppu.read_data(),
            0x2008..=0x3fff => self.read_u8(address & 0x2007),
            0x8000..=0xffff => self.rom.read_byte(address),
            _ => panic!("invalid read address {:04X}", address),
        }
    }

    pub fn write_u8(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x1fff => self.ram[address as usize % 0x0800] = data,
            0x2000 => self.ppu.write_ppuctrl(data),

            0x2001 => {}
            0x2002 => panic!("attemped to write status reg"),
            0x2003 => {}
            0x2004 => {}
            0x2005 => {}

            0x2006 => self.ppu.write_ppuaddr(data),
            0x2007 => self.ppu.write_ppudata(data),
            0x2008..=0x3fff => {
                self.write_u8(address & 0x2007, data);
            }

            0x4000..=0x4013 | 0x4015 => {
                // TODO: implement APU
            }
            _ => panic!("invalid write address {:04X}", address),
        }
    }

    pub fn read_u16(&mut self, address: u16) -> u16 {
        (self.read_u8(address.wrapping_add(0)) as u16)
            | ((self.read_u8(address.wrapping_add(1)) as u16) << 8)
    }

    pub fn read_bytes(&mut self, address: u16, size: u8) -> Vec<u8> {
        let mut bytes = vec![0; size as usize];
        for i in 0..size {
            bytes[i as usize] = self.read_u8(address.wrapping_add(i as u16));
        }
        bytes
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn setup_bus(prg_rom: Vec<u8>) -> Bus {
        let rom = Rom::new_from_vec(prg_rom);
        Bus::new(rom)
    }

    #[test]
    fn test_read_u8() {
        let mut bus = setup_bus(vec![0xff]);
        assert_eq!(bus.read_u8(0x8000), 0xff);
    }

    #[test]
    fn test_read_u16() {
        let mut bus = setup_bus(vec![0xcd, 0xab]);
        assert_eq!(bus.read_u16(0x8000), 0xabcd);
    }
}
