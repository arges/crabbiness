use crate::ppu::Ppu;
use crate::rom::Rom;
use log::debug;
use std::borrow::Borrow;

pub struct Bus {
    pub ram: [u8; 2048],
    rom: Rom,
    pub ppu: Ppu,
}

impl Bus {
    pub fn new(rom: Rom) -> Self {
        let ppu = Ppu::new(rom.chr_rom.clone());

        Bus {
            ram: [0; 2048],
            rom,
            ppu,
        }
    }

    pub fn read_u8(&mut self, address: u16) -> u8 {
        //println!("reading {:04x}", address);
        debug!("reading @ {:04x}", address);
        match address {
            0x0000..=0x1fff => self.ram[address as usize % 0x0800],
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                panic!("Attempt to read from write-only PPU address {:x}", address);
            }
            0x2007 => self.ppu.read_data(),
            0x2008..=0x7fff => self.read_u8(address & 0x2007),
            0x8000..=0xffff => self.rom.read_byte(address),
            _ => 0,
        }
    }

    pub fn write_u8(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x1fff => self.ram[address as usize % 0x0800] = data,
            0x2000 => self.ppu.write_ppuctrl(data),
            0x2006 => self.ppu.write_ppuaddr(data),
            0x2007 => self.ppu.write_ppudata(data),
            0x2008..=0x7fff => {
                self.write_u8(address & 0x2007, data);
            }
            _ => (),
        }
    }

    pub fn read_u16(&mut self, address: u16) -> u16 {
        //FIXME: is this correct
        ((self.read_u8(address.wrapping_add(0)) as u16) << 8)
            | (self.read_u8(address.wrapping_add(1)) as u16)
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
        let bus = Bus::new(rom);
        bus
    }

    #[test]
    fn test_read_u8() {
        let mut bus = setup_bus(vec![0xff]);
        assert_eq!(bus.read_u8(0x8000), 0xff);
    }

    #[test]
    fn test_read_u16() {
        let mut bus = setup_bus(vec![0xab, 0xcd]);
        assert_eq!(bus.read_u16(0x8000), 0xabcd);
    }
}
