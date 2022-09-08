use macroquad::prelude::*;

#[rustfmt::skip]
pub static DEFAULT_PALLETE: [(u8,u8,u8); 64] = [
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

fn draw_tile(
    chr_rom: &[u8],
    bank: usize,
    tile_num: usize,
    image: &mut Image,
    offset_x: u32,
    offset_y: u32,
) {
    // Select the tile bits from memory
    let bank = if bank == 1 { 0x1000 } else { 0x0000 };
    let mem_start = bank + tile_num * 16;
    let tile = &chr_rom[mem_start..(mem_start + 16)];

    // Iterate through the 8x8 tile and draw the pixels
    for y in 0..8 {
        let upper = tile[y];
        let lower = tile[y + 8];
        for x in 0..8 {
            let rgb = match (upper & (1 << x) > 0, lower & (1 << x) > 0) {
                (false, false) => WHITE,
                (false, true) => BLACK,
                (true, false) => RED,
                (true, true) => BLUE,
            };
            image.set_pixel((8 - x) as u32 + offset_x, y as u32 + offset_y, rgb)
        }
    }
}

pub fn draw_background(chr_rom: &[u8], vram: &[u8; 2048], bank: usize, image: &mut Image) {
    for i in 0..0x03c0 {
        // just for now, lets use the first nametable
        let tile = vram[i] as usize;
        let tile_x = (i % 32) * 8;
        let tile_y = (i / 32) * 8;
        draw_tile(chr_rom, bank, tile, image, tile_x as u32, tile_y as u32);
    }
}
