use crate::controller::Controller;
use crate::ppu::Ppu;
use crate::rom::Rom;
use log::debug;
use std::borrow::Borrow;

pub struct Bus {
    pub ram: [u8; 2048],
    rom: Rom,
    pub ppu: Ppu,
    pub controller: Controller,
    cycle: usize,
}

impl Bus {
    /// Implements the 'bus' connecting PPU, CPU, RAM and controllers
    ///
    /// Normally each component in the NES would be independent operating on clocks and various
    /// signals passed via a 'bus'. In this implementation functions act more syncronously and
    /// timings are guided by cycles and ticks.
    pub fn new(rom: Rom) -> Self {
        let ppu = Ppu::new(rom.chr_rom.clone(), rom.mirroring());

        Bus {
            ram: [0; 2048],
            rom,
            ppu,
            cycle: 0,
            controller: Controller::new(),
        }
    }

    pub fn tick(&mut self, cycle: u8) -> bool {
        self.cycle += cycle as usize;
        let before = self.ppu.has_nmi.is_some();
        self.ppu.tick(cycle * 3);
        !before && self.ppu.has_nmi.is_some()
    }

    /// actively poll for new keys and update internal data
    pub fn read_keys(&mut self) {
        self.controller.read_keys()
    }

    pub fn take_nmi(&mut self) -> bool {
        self.ppu.has_nmi.take() == Some(true)
    }

    /// reads a byte matches the address to the correct component on the bus
    pub fn read_u8(&mut self, address: u16) -> u8 {
        debug!("reading @ {:04x}", address);
        match address {
            0x0000..=0x1fff => self.ram[address as usize % 0x0800],
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                panic!("attempt to read from write-only PPU address {:x}", address);
            }
            0x2002 => self.ppu.read_ppustatus(),
            0x2004 => self.ppu.read_oamdata(),
            0x2007 => self.ppu.read_data(),
            0x4016 => self.controller.read(),
            0x4017 => 0, // TODO player 2
            0x2008..=0x3fff => self.read_u8(address & 0x2007),
            0x8000..=0xffff => self.rom.read_byte(address),
            _ => panic!("invalid read address {:04X}", address),
        }
    }

    /// write a byte matches the address to the correct component on the bus
    pub fn write_u8(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x1fff => self.ram[address as usize % 0x0800] = data,
            0x2000 => self.ppu.write_ppuctrl(data),

            0x2001 => self.ppu.write_ppumask(data),
            0x2002 => panic!("attemped to write status reg"),
            0x2003 => self.ppu.write_oamaddr(data),
            0x2004 => self.ppu.write_oamdata(data),
            0x2005 => self.ppu.write_scrolldata(data),
            0x2006 => self.ppu.write_ppuaddr(data),
            0x2007 => self.ppu.write_ppudata(data),
            0x2008..=0x3fff => {
                self.write_u8(address & 0x2007, data);
            }
            0x4000..=0x4013 => {} // TODO implement APU
            0x4014 => {
                let mut buffer: [u8; 256] = [0; 256];
                let start = (data as u16) << 8;
                for i in 0..256u16 {
                    buffer[i as usize] = self.read_u8(start + i);
                }
                self.ppu.write_oamdata_dma(&buffer);
            }
            0x4015 => {}
            0x4016 => self.controller.write(data),
            0x4017 => {} // TODO player 2
            _ => panic!("invalid write address {:04X}", address),
        }
    }

    /// reads 16 bits by calling read_u8 twice
    pub fn read_u16(&mut self, address: u16) -> u16 {
        (self.read_u8(address.wrapping_add(0)) as u16)
            | ((self.read_u8(address.wrapping_add(1)) as u16) << 8)
    }

    /// reads an abitrary number of bytes by calling read_u8 in a loop
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
