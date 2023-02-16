use crate::ppu::Ppu;
use bitflags::bitflags;
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

struct View {
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
    offset_x: isize,
    offset_y: isize,
}

impl View {
    fn new(x1: usize, y1: usize, x2: usize, y2: usize, offset_x: isize, offset_y: isize) -> Self {
        View {
            x1,
            y1,
            x2,
            y2,
            offset_x,
            offset_y,
        }
    }
}

/// returns the starting palette address given row and column
///
/// Given a 2x2 grid, select the 2 bits within the attribute that
/// apply to that particular tile. Those two bytes select the
/// palette table which are offset from 0x3f00 by one byte.
fn palette_start(row: usize, column: usize, attribute: u8) -> usize {
    let shift = (((row % 4) / 2) << 1) * 2 + ((column % 4) / 2) * 2;
    (1 + ((attribute >> shift) & 0b11) * 4) as usize
}

/// returns the palette indicies for a given tile
///
/// Given a tile row and column, select the attribute space for the
/// 4x4 tile region. Then select and return the palette table.
fn background_palette(ppu: &Ppu, nametable: &[u8], row: usize, column: usize) -> [u8; 4] {
    let index = (row / 4 * 8) + (column / 4);
    let attr = nametable[0x3c0 + index];
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
    nametable: &[u8],
    bank: u16,
    tile_num: u16,
    image: &mut Image,
    row: u32,
    column: u32,
    view: &View,
) {
    // Select the tile bits from memory
    let mem_start = (bank + tile_num * 16) as usize;
    let tile = &ppu.chr_rom[mem_start..(mem_start + 16)];
    let palette = background_palette(ppu, nametable, row as usize, column as usize);

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
            let pixel_x = (8 - x) + (column * 8) as usize;
            let pixel_y = y + (row * 8) as usize;
            if pixel_x >= view.x1 && pixel_x < view.x2 && pixel_y >= view.y1 && pixel_y < view.y2 {
                image.set_pixel(
                    (view.offset_x + pixel_x as isize) as u32,
                    (view.offset_y + pixel_y as isize) as u32,
                    color_u8!(r, g, b, 255),
                )
            }
        }
    }
}

/// draws the background layer
fn draw_background(ppu: &Ppu, nametable: &[u8], image: &mut Image, view: &View) {
    let bank = ppu.ctrl_register.bg_bank_addr();
    for i in 0..0x03c0 {
        let tile = nametable[i] as u16;
        let row = i / 32;
        let column = i % 32;
        draw_background_tile(
            ppu,
            nametable,
            bank,
            tile,
            image,
            row as u32,
            column as u32,
            view,
        );
    }
}

fn sprite_palette(ppu: &Ppu, index: u8) -> [u8; 4] {
    let start = 0x11 + (index * 4) as usize;
    [
        0,
        ppu.palette[start],
        ppu.palette[start + 1],
        ppu.palette[start + 2],
    ]
}

/// draw single sprite tile
///
/// similar to draw_background_tile, this function handles sprites with
/// transparency and flipping.
fn draw_sprite_tile(
    ppu: &Ppu,
    bank: u16,
    tile_num: u8,
    image: &mut Image,
    attr: u8,
    tile_x: u8,
    tile_y: u8,
) {
    // Select the tile bits from memory
    let mem_start = (bank + tile_num as u16 * 16) as usize;
    let tile = &ppu.chr_rom[mem_start..(mem_start + 16)];
    let palette = sprite_palette(ppu, attr & 0b11);

    // Iterate through the 8x8 tile and draw the pixels
    for y in 0..8 {
        let upper = tile[y];
        let lower = tile[y + 8];
        for x in 0..8 {
            let (r, g, b) = match (lower & (1 << x) > 0, upper & (1 << x) > 0) {
                (false, false) => continue,
                (false, true) => DEFAULT_PALETTE[palette[1] as usize],
                (true, false) => DEFAULT_PALETTE[palette[2] as usize],
                (true, true) => DEFAULT_PALETTE[palette[3] as usize],
            };
            let pixel_x = if (attr & 0x40) == 0x40 {
                tile_x.wrapping_add(x) as u8
            } else {
                tile_x.wrapping_add(7 - x)
            };
            let pixel_y = if (attr & 0x80) == 0x00 {
                tile_y.wrapping_add(y as u8) as u8
            } else {
                tile_y.wrapping_add(7 - y as u8)
            };
            image.set_pixel(pixel_x as u32, pixel_y as u32, color_u8!(r, g, b, 255))
        }
    }
}

/// draws the sprite layer
fn draw_sprites(ppu: &Ppu, image: &mut Image) {
    for i in (0..ppu.oam.len()).step_by(4) {
        let y = ppu.oam[i];
        let tile_num = ppu.oam[i + 1];
        let x = ppu.oam[i + 3];
        let attr = ppu.oam[i + 2];
        let bank = ppu.ctrl_register.sprite_bank_addr();

        draw_sprite_tile(ppu, bank, tile_num, image, attr, x, y);
    }
}

/// renders the entire frame
pub fn draw(ppu: &Ppu, image: &mut Image) {
    let (scroll_x, scroll_y) = (ppu.scroll_register.x, ppu.scroll_register.y);
    let (main_background, second_background) = ppu.get_background_addrs();

    draw_background(
        ppu,
        main_background,
        image,
        &View::new(
            scroll_x as usize,
            scroll_y as usize,
            256,
            240,
            -(scroll_x as isize),
            -(scroll_y as isize),
        ),
    );

    if scroll_x > 0 {
        draw_background(
            ppu,
            second_background,
            image,
            &View::new(0, 0, scroll_x as usize, 240, (255 - scroll_x) as isize, 0),
        );
    } else if scroll_y > 0 {
        draw_background(
            ppu,
            second_background,
            image,
            &View::new(0, 0, 256, scroll_y as usize, 0, (240 - scroll_y) as isize),
        );
    }

    draw_sprites(ppu, image);
}
