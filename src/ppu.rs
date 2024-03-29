use bitflags::bitflags;
use log::debug;

pub struct Ppu {
    pub chr_rom: Vec<u8>,
    pub palette: [u8; 32],
    pub vram: [u8; 2048],
    pub oam: [u8; 256],
    pub oam_addr: u8,
    mirroring: bool, // false horizontal, true vertical
    pub ctrl_register: PpuCtrlRegister,
    mask_register: PpuMaskRegister,
    addr_register: PpuAddrRegister,
    status_register: PpuStatusRegister,
    pub scroll_register: PpuScrollRegister,
    buffer: u8,
    cycle: usize,
    scanline: u16,
    pub has_nmi: Option<bool>,
}

impl Ppu {
    /// implements the Picture Processing Unit
    ///
    /// This contains the registers, vram, chr_rom, pallets and oam data for graphics.
    pub fn new(chr_rom: Vec<u8>, mirroring: bool) -> Self {
        Self {
            chr_rom,
            palette: [0; 32],
            vram: [0; 2048],
            oam: [0; 256],
            oam_addr: 0,
            ctrl_register: PpuCtrlRegister::new(),
            mask_register: PpuMaskRegister::new(),
            addr_register: PpuAddrRegister::new(),
            status_register: PpuStatusRegister::new(),
            scroll_register: PpuScrollRegister::new(),
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
            self.set_sprite0_hit();
            self.cycle -= 341;

            self.scanline += 1;
            if self.scanline == 241 {
                self.status_register.set_vblank(true);
                self.status_register.clear_sprite0();
                if self.ctrl_register.nmi_starts_on_vblank_ok() {
                    self.has_nmi = Some(true);
                }
            }
            if self.scanline >= 262 {
                self.scanline = 0;
                self.has_nmi = None;
                self.status_register.clear_sprite0();
                self.status_register.set_vblank(false);
                return true;
            }
        }
        false
    }

    pub fn read_oamdata(&self) -> u8 {
        self.oam[self.oam_addr as usize] as u8
    }

    pub fn write_oamdata(&mut self, input: u8) {
        self.oam[self.oam_addr as usize] = input;
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    pub fn write_oamaddr(&mut self, addr: u8) {
        self.oam_addr = addr;
    }

    pub fn write_oamdata_dma(&mut self, data: &[u8; 256]) {
        for x in data {
            self.oam[self.oam_addr as usize] = *x;
            self.oam_addr = self.oam_addr.wrapping_add(1);
        }
    }

    pub fn write_scrolldata(&mut self, input: u8) {
        self.scroll_register.write(input);
    }

    pub fn write_ppumask(&mut self, input: u8) {
        self.mask_register.update(input);
    }

    pub fn write_ppuaddr(&mut self, input: u8) {
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
        let ret = self.status_register.read();
        self.scroll_register.reset();
        self.addr_register.reset();
        self.status_register.set_vblank(false);
        ret
    }

    fn increment_vram(&mut self) {
        self.addr_register.inc(self.ctrl_register.vram_inc());
    }

    /// check and update mask register show_sprites bit
    fn set_sprite0_hit(&mut self) {
        let (x, y) = (self.oam[3], self.oam[0]);
        self.status_register.set(
            PpuStatusRegister::SPRITE_0_HIT,
            y as u16 == self.scanline
                && x as usize <= self.cycle
                && self.mask_register.contains(PpuMaskRegister::SHOW_SPRITES),
        );
    }

    /// calculates the mirrored vram addressed based on mirror modes
    /// this supports a limited set of mirroring modes only horizontal and veritcal
    fn mirror_vram_addr(&mut self, addr: u16) -> u16 {
        let index = addr - 0x2000;
        let quadrant = index / 0x400;
        match (self.mirroring, quadrant) {
            (false, 1) => index - 0x400,
            (false, 2) => index - 0x400,
            (false, 3) => index - 0x800,
            (true, 2) => index - 0x800,
            (true, 3) => index - 0x800,
            _ => index,
        }
    }

    /// returns the regions of memory for rendering background
    pub fn get_background_addrs(&self) -> (&[u8], &[u8]) {
        match (self.mirroring, self.ctrl_register.base_addr()) {
            (false, 0x2000) | (false, 0x2400) | (true, 0x2000) | (true, 0x2800) => {
                (&self.vram[0..0x400], &self.vram[0x400..0x800])
            }
            (false, 0x2800) | (false, 0x2c00) | (true, 0x2400) | (true, 0x2c00) => {
                (&self.vram[0x400..0x800], &self.vram[0..0x400])
            }
            (_, _) => panic!(
                "not supported {} {}",
                self.mirroring,
                self.ctrl_register.base_addr()
            ),
        }
    }

    pub fn read_data(&mut self) -> u8 {
        let addr = self.addr_register.value;
        self.increment_vram();
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
    }

    pub fn inc(&mut self, input: u8) {
        self.value = self.value.wrapping_add(input as u16) & 0x3fff;
    }

    pub fn reset(&mut self) {
        self.latch = true;
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

    #[rstest]
    #[case(false, 0x2000, 0x0000)]
    #[case(false, 0x2800, 0x0400)]
    #[case(false, 0x2c00, 0x0400)]
    #[case(true, 0x2400, 0x0400)]
    #[case(true, 0x2c00, 0x0400)]
    fn test_mirror_vram_addr(#[case] mirroring: bool, #[case] input: u16, #[case] expected: u16) {
        let mut ppu = Ppu::new(vec![], mirroring);
        let output = ppu.mirror_vram_addr(input);
        assert_eq!(output, expected);
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
        match self.bits & 0b11 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2c00,
            _ => panic!("impossible"),
        }
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

    pub fn clear_sprite0(&mut self) {
        self.set(PpuStatusRegister::SPRITE_0_HIT, false);
    }

    pub fn read(&self) -> u8 {
        self.bits()
    }
}

bitflags! {
    pub struct PpuMaskRegister: u8 {
        const SHOW_SPRITES = 0b0001_0000;
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

pub struct PpuScrollRegister {
    pub x: u8,
    pub y: u8,
    latch: bool,
}

impl PpuScrollRegister {
    pub fn new() -> Self {
        PpuScrollRegister {
            x: 0,
            y: 0,
            latch: false,
        }
    }
    pub fn write(&mut self, data: u8) {
        if !self.latch {
            self.x = data;
        } else {
            self.y = data;
        }
        self.latch = !self.latch;
    }

    pub fn reset(&mut self) {
        self.latch = false;
    }
}
