use bitflags::bitflags;
use log::debug;

pub struct Ppu {
    pub chr_rom: Vec<u8>,
    pub palette: [u8; 32],
    pub vram: [u8; 2048],
    pub oam: [u8; 256],
    mirroring: bool,
    pub ctrl_register: PpuCtrlRegister,
    mask_register: PpuMaskRegister,
    addr_register: PpuAddrRegister,
    status_register: PpuStatusRegister,
    buffer: u8,
    cycle: usize,
    scanline: u16,
    pub has_nmi: Option<bool>,
}

impl Ppu {
    pub fn new(chr_rom: Vec<u8>, mirroring: bool) -> Self {
        Self {
            chr_rom,
            palette: [0; 32],
            vram: [0; 2048],
            oam: [0; 256],
            ctrl_register: PpuCtrlRegister::new(),
            mask_register: PpuMaskRegister::new(),
            addr_register: PpuAddrRegister::new(),
            status_register: PpuStatusRegister::new(),
            mirroring,
            buffer: 0,
            cycle: 0,
            scanline: 0,
            has_nmi: None,
        }
    }

    pub fn tick(&mut self, cycle: u8) -> bool {
        self.cycle += cycle as usize;
        debug!("ppu cycle {} scanline {}", self.cycle, self.scanline);
        if self.cycle >= 341 {
            self.cycle -= 341;
            self.scanline += 1;
            if self.scanline == 241 {
                self.status_register.set_vblank(true);
                if self.ctrl_register.nmi_starts_on_vblank_ok() {
                    self.has_nmi = Some(true);
                }
            }
            if self.scanline >= 262 {
                self.scanline = 0;
                self.has_nmi = None;
                self.status_register.set_vblank(false);
                return true;
            }
        }
        false
    }

    pub fn write_ppumask(&mut self, input: u8) {
        debug!("write_ppumask {:02X}", input);
        self.mask_register.update(input);
    }

    pub fn write_ppuaddr(&mut self, input: u8) {
        debug!("write_ppuaddr {:02X}", input);
        self.addr_register.update(input);
    }

    pub fn write_ppudata(&mut self, input: u8) {
        let addr = self.addr_register.value;
        match addr {
            0..=0x1fff => panic!("cannot write into chr_rom"),
            0x2000..=0x2fff => {
                self.vram[self.mirror_vram_addr(addr) as usize] = input;
            }
            0x3000..=0x3eff => panic!("not expecting this to be used"),
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => {
                self.palette[(addr - 0x3f00 - 0x10) as usize] = input
            }
            0x3f00..=0x3fff => self.palette[(addr - 0x3f00) as usize] = input,
            _ => panic!("unexpected ppudata write to {:02X}", addr),
        }
        self.increment_vram();
    }

    pub fn write_ppuctrl(&mut self, input: u8) {
        let before = self.ctrl_register.nmi_starts_on_vblank_ok();
        self.ctrl_register.update(input);
        if !before
            && self.ctrl_register.nmi_starts_on_vblank_ok()
            && self.status_register.is_vblank()
        {
            self.has_nmi = Some(true);
        }
    }

    pub fn read_ppustatus(&mut self) -> u8 {
        self.status_register.read()
    }

    fn increment_vram(&mut self) {
        self.addr_register.inc(self.ctrl_register.vram_inc());
    }

    fn mirror_vram_addr(&mut self, addr: u16) -> u16 {
        let index = addr - 0x2000;
        let quadrant = index / 0x400;
        match (quadrant, !&self.mirroring) {
            (1, false) => index - 0x400,
            (1, true) => index - 0x800,
            (2, false) => index - 0x400,
            (2, true) => index - 0x800,
            (3, false) => index - 0x800,
            (3, true) => index - 0x800,
            _ => index,
        }
    }

    pub fn read_data(&mut self) -> u8 {
        self.increment_vram();
        let addr = self.addr_register.value;
        debug!("read_data addr {:04X}", addr);
        match addr {
            0..=0x1fff => {
                let result = self.buffer;
                self.buffer = self.chr_rom[addr as usize];
                result
            }
            0x2000..=0x2fff => {
                let result = self.buffer;
                self.buffer = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            }
            0x3000..=0x3eff => panic!("not expecting this to be used"),
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => self.palette[(addr - 0x3f00 - 0x10) as usize],
            0x3f00..=0x3fff => self.palette[(addr - 0x3f00) as usize],
            _ => panic!("unexpected access"),
        }
    }
}

pub struct PpuAddrRegister {
    pub value: u16,
    latch: bool,
}

impl PpuAddrRegister {
    pub fn new() -> Self {
        Self {
            value: 0,
            latch: true,
        }
    }

    pub fn update(&mut self, data: u8) {
        let b = self.value.to_le_bytes();
        if self.latch {
            self.value = u16::from_le_bytes([b[0], data]);
        } else {
            self.value = u16::from_le_bytes([data, b[1]]);
        }
        self.latch = !self.latch;
        debug!("update ppuaddr {:04X}", self.value);
    }

    pub fn inc(&mut self, input: u8) {
        self.value = self.value.wrapping_add(input as u16);
    }

    pub fn reset(&mut self) {
        self.latch = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rom::Rom;
    use rstest::rstest;

    #[test]
    fn test_ppu_addr_reg() {
        let mut addr_reg = PpuAddrRegister::new();
        assert_eq!(addr_reg.value, 0);
        addr_reg.update(0x06);
        addr_reg.update(0x50);
        assert_eq!(addr_reg.value, 0x0650);
        addr_reg.inc(0xa);
        assert_eq!(addr_reg.value, 0x065a);
    }
}

bitflags! {
    pub struct PpuCtrlRegister: u8 {
        const BASE_NAMETABLE_ADDR_LOW = 0b0000_0001;
        const BASE_NAMETABLE_ADDR_HIGH = 0b0000_0010;
        const VRAM_INC_PER_CPU = 0b0000_0100;
        const SPRITE_PATTERN_TABLE_ADDR = 0b0000_1000;
        const BG_PATTERN_TABLE_ADDR = 0b0001_0000;
        const SPRITE_SIZE = 0b0010_0000;
        const PPU_SELECT = 0b0100_0000;
        const NMISTARTS_ON_VBI = 0b1000_0000;
    }
}

impl PpuCtrlRegister {
    pub fn new() -> Self {
        PpuCtrlRegister::empty()
    }

    pub fn vram_inc(&self) -> u8 {
        if self.contains(PpuCtrlRegister::VRAM_INC_PER_CPU) {
            32
        } else {
            1
        }
    }

    pub fn base_addr(&self) -> u16 {
        0x2000 | (((self.bits() & 0b11) as u16) << 12)
    }

    pub fn bg_bank_addr(&self) -> u16 {
        if self.contains(PpuCtrlRegister::BG_PATTERN_TABLE_ADDR) {
            0x1000
        } else {
            0x0000
        }
    }

    pub fn sprite_bank_addr(&self) -> u16 {
        if self.contains(PpuCtrlRegister::SPRITE_PATTERN_TABLE_ADDR) {
            0x1000
        } else {
            0x0000
        }
    }

    pub fn nmi_starts_on_vblank_ok(&mut self) -> bool {
        self.contains(PpuCtrlRegister::NMISTARTS_ON_VBI)
    }

    pub fn update(&mut self, input: u8) {
        self.bits = input;
    }
}

bitflags! {
    pub struct PpuStatusRegister: u8 {
        const SPRITE_OVERFLOW = 0b0010_0000;
        const SPRITE_0_HIT = 0b0100_0000;
        const VBLANK_START = 0b1000_0000;
    }
}

impl PpuStatusRegister {
    pub fn new() -> Self {
        PpuStatusRegister::empty()
    }

    pub fn set_vblank(&mut self, status: bool) {
        self.set(PpuStatusRegister::VBLANK_START, status);
    }

    pub fn is_vblank(self) -> bool {
        self.contains(PpuStatusRegister::VBLANK_START)
    }

    pub fn read(&self) -> u8 {
        self.bits()
    }
}

bitflags! {
    pub struct PpuMaskRegister: u8 {
        const EMPHASIZE_RED = 0b0010_0000;
        const EMPHASIZE_GREEN = 0b0100_0000;
        const EMPHASIZE_BLUE = 0b1000_0000;
    }
}

impl PpuMaskRegister {
    pub fn new() -> Self {
        PpuMaskRegister::empty()
    }

    pub fn read(&self) -> u8 {
        self.bits()
    }

    pub fn update(&mut self, data: u8) {
        self.bits = data;
    }
}
