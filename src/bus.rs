use crate::rom::Rom;

pub struct Bus {
    ram: [u8; 2048],
    rom: Option<Rom>,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            ram: [0; 2048],
            rom: None,
        }
    }

    pub fn load(&mut self, rom: Rom) {
        self.rom = Some(rom);
    }

    pub fn read_u8(&self, address: u16) -> u8 {
        //println!("reading {:04x}", address);
        match address {
            0x0000..=0x1fff => self.ram[address as usize % 0x0800],
            0x2000..=0x401f => {
                println!("reading {:04x} not implemented", address);
                0
            }
            0x4020..=0xffff => match &self.rom {
                Some(r) => {
                    let output = r.read_byte(address);
                    // println!("got {:02x}", output);
                    output
                }
                None => 0,
            },
        }
    }

    pub fn write_u8(&mut self, address: u16, data: u8) {
        match address {
            0x0000..=0x1fff => {
                //println!("writing {:02x} into {:04x}", data, address);
                self.ram[address as usize % 0x0800] = data
            }
            _ => (),
        }
    }

    pub fn read_u16(&self, address: u16) -> u16 {
        ((self.read_u8(address.wrapping_add(1)) as u16) << 8) | (self.read_u8(address) as u16)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn setup_bus(prg_rom: Vec<u8>) -> Bus {
        let rom = Rom::new_from_vec(prg_rom);
        let mut bus = Bus::new();
        bus.load(rom);
        bus
    }

    #[test]
    fn test_read_u8() {
        let bus = setup_bus(vec![0xff]);
        assert_eq!(bus.read_u8(0x4020), 0xff);
    }

    #[test]
    fn test_read_u16() {
        let bus = setup_bus(vec![0xab, 0xcd]);
        assert_eq!(bus.read_u16(0x4020), 0xcdab);
    }
}
