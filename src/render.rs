use crate::ppu::Ppu;
use macroquad::color::*;
use macroquad::prelude::*;

#[rustfmt::skip]
pub static DEFAULT_PALETTE: [(u8,u8,u8); 64] = [
   (0x80, 0x80, 0x80), (0x00, 0x3D, 0xA6), (0x00, 0x12, 0xB0), (0x44, 0x00, 0x96), (0xA1, 0x00, 0x5E),
   (0xC7, 0x00, 0x28), (0xBA, 0x06, 0x00), (0x8C, 0x17, 0x00), (0x5C, 0x2F, 0x00), (0x10, 0x45, 0x00),
   (0x05, 0x4A, 0x00), (0x00, 0x47, 0x2E), (0x00, 0x41, 0x66), (0x00, 0x00, 0x00), (0x05, 0x05, 0x05),
   (0x05, 0x05, 0x05), (0xC7, 0xC7, 0xC7), (0x00, 0x77, 0xFF), (0x21, 0x55, 0xFF), (0x82, 0x37, 0xFA),
   (0xEB, 0x2F, 0xB5), (0xFF, 0x29, 0x50), (0xFF, 0x22, 0x00), (0xD6, 0x32, 0x00), (0xC4, 0x62, 0x00),
   (0x35, 0x80, 0x00), (0x05, 0x8F, 0x00), (0x00, 0x8A, 0x55), (0x00, 0x99, 0xCC), (0x21, 0x21, 0x21),
   (0x09, 0x09, 0x09), (0x09, 0x09, 0x09), (0xFF, 0xFF, 0xFF), (0x0F, 0xD7, 0xFF), (0x69, 0xA2, 0xFF),
   (0xD4, 0x80, 0xFF), (0xFF, 0x45, 0xF3), (0xFF, 0x61, 0x8B), (0xFF, 0x88, 0x33), (0xFF, 0x9C, 0x12),
   (0xFA, 0xBC, 0x20), (0x9F, 0xE3, 0x0E), (0x2B, 0xF0, 0x35), (0x0C, 0xF0, 0xA4), (0x05, 0xFB, 0xFF),
   (0x5E, 0x5E, 0x5E), (0x0D, 0x0D, 0x0D), (0x0D, 0x0D, 0x0D), (0xFF, 0xFF, 0xFF), (0xA6, 0xFC, 0xFF),
   (0xB3, 0xEC, 0xFF), (0xDA, 0xAB, 0xEB), (0xFF, 0xA8, 0xF9), (0xFF, 0xAB, 0xB3), (0xFF, 0xD2, 0xB0),
   (0xFF, 0xEF, 0xA6), (0xFF, 0xF7, 0x9C), (0xD7, 0xE8, 0x95), (0xA6, 0xED, 0xAF), (0xA2, 0xF2, 0xDA),
   (0x99, 0xFF, 0xFC), (0xDD, 0xDD, 0xDD), (0x11, 0x11, 0x11), (0x11, 0x11, 0x11)
];

/// returns the starting palette address given row and column
///
/// Given a 2x2 grid, select the 2 bits within the attribute that
/// apply to that particular tile. Those two bytes select the
/// palette table which are offset from 0x3f00 by one byte.
fn palette_start(row: usize, column: usize, attribute: u8) -> usize {
    let shift = (((row % 4) / 2) << 1) * 2 + ((column % 4) / 2) * 2;
    (1 + ((attribute >> shift) & 0b11) * 4) as usize
}

// returns the palette indicies for a given tile
//
// Given a tile row and column, select the attribute space for the
// 4x4 tile region. Then select and return the palette table.
fn background_palette(ppu: &Ppu, row: usize, column: usize) -> [u8; 4] {
    let index = (row / 4 * 8) + (column / 4);
    let attr = ppu.vram[0x3c0 + index];
    let start = palette_start(row, column, attr);
    [
        ppu.palette[0],
        ppu.palette[start],
        ppu.palette[start + 1],
        ppu.palette[start + 2],
    ]
}

/// draw single background tile
///
/// given a row, column and tile number from vram, get the color indicies for
/// the tile and map them to the correct r,g,b values given the palette index.
/// Then actually draw them onto the image.
fn draw_background_tile(
    ppu: &Ppu,
    bank: u16,
    tile_num: u16,
    image: &mut Image,
    row: u32,
    column: u32,
) {
    // Select the tile bits from memory
    let mem_start = (bank + tile_num * 16) as usize;
    let tile = &ppu.chr_rom[mem_start..(mem_start + 16)];
    let palette = background_palette(ppu, row as usize, column as usize);

    // Iterate through the 8x8 tile and draw the pixels
    for y in 0..8 {
        let upper = tile[y];
        let lower = tile[y + 8];
        for x in 0..8 {
            let (r, g, b) = match (lower & (1 << x) > 0, upper & (1 << x) > 0) {
                (false, false) => DEFAULT_PALETTE[ppu.palette[0] as usize],
                (false, true) => DEFAULT_PALETTE[palette[1] as usize],
                (true, false) => DEFAULT_PALETTE[palette[2] as usize],
                (true, true) => DEFAULT_PALETTE[palette[3] as usize],
            };
            image.set_pixel(
                (8 - x) as u32 + (column * 8),
                y as u32 + (row * 8),
                color_u8!(r, g, b, 255),
            )
        }
    }
}

/// draws the background layer
fn draw_background(ppu: &Ppu, image: &mut Image) {
    let bank = ppu.ctrl_register.bg_bank_addr();
    for i in 0..0x03c0 {
        let tile = ppu.vram[i] as u16;
        let row = i / 32;
        let column = i % 32;
        draw_background_tile(ppu, bank, tile, image, row as u32, column as u32);
    }
}

/// draws the sprite layer
fn draw_sprites(ppu: &Ppu, image: &mut Image) {}

/// renders the entire frame
pub fn draw(ppu: &Ppu, image: &mut Image) {
    draw_background(ppu, image);
    //draw_sprites(ppu, image);
}
