#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

extern crate core;

use std::env;
use std::io::Read;
use std::{fs, thread};

use macroquad::prelude::*;

mod bus;
mod controller;
mod cpu;
mod ppu;
mod render;
mod rom;

#[macroquad::main("crabbiness")]
async fn main() {
    // setup logger
    env_logger::init();

    // parse command line args
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("usage: <nes file>");
        return;
    }
    let filename = &args[1];

    // load rom from disk
    let mut file = fs::File::open(filename).unwrap();
    let mut data: Vec<u8> = Vec::new();
    file.read_to_end(&mut data).unwrap();
    let r = rom::Rom::new_from_ines(&data);

    // setup bus, cpu
    let bus = bus::Bus::new(r);
    let mut cpu = cpu::Cpu::new(bus);

    // setup graphics
    let mut image = Image::gen_image_color(320 as u16, 320 as u16, BLACK);

    let mut counter: u32 = 0;

    // run cpu
    cpu.reset();
    clear_background(BLUE);
    loop {
        let cycles = if cpu.bus.take_nmi() {
            cpu.nmi();
            2
        } else {
            cpu.step()
        };
        let render = cpu.bus.tick(cycles);

        // get controller input from keyboard
        cpu.bus.read_keys();
        counter = counter.wrapping_add(1);

        if render {
            render::draw(&cpu.bus.ppu, &mut image);
            let tex_params = DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                source: None,
                rotation: 0.0,
                flip_x: false,
                flip_y: false,
                pivot: None,
            };
            draw_texture_ex(Texture2D::from_image(&image), 0.0, 0.0, WHITE, tex_params);
            draw_text(
                cpu.to_string().as_str(),
                0.0,
                screen_height() - 20.0,
                30.0,
                WHITE,
            );
            draw_text(
                cpu.bus.controller.to_string().as_str(),
                0.0,
                screen_height() - 70.0,
                30.0,
                GRAY,
            );

            draw_text(
                format!("{} {}", counter, get_fps()).trim(),
                screen_width() - 150.0,
                screen_height() - 20.0,
                30.0,
                GREEN,
            );

            next_frame().await
        }
    }
}
