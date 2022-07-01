#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

extern crate core;

use std::io::Read;
use std::{fs, thread};

use macroquad::prelude::*;

mod bus;
mod cpu;
mod ppu;
mod rom;

#[macroquad::main("crabbiness")]
async fn main() {
    // load rom from disk
    let mut file = fs::File::open("nestest.nes").unwrap();
    let mut data: Vec<u8> = Vec::new();
    file.read_to_end(&mut data).unwrap();
    let r = rom::Rom::new_from_ines(&data);

    // setup bus, cpu
    let bus = bus::Bus::new(r);
    let mut cpu = cpu::Cpu::new(bus);

    // run cpu
    cpu.reset();

    loop {
        cpu.step();
        clear_background(BLUE);
        draw_text(cpu.to_string().as_str(), 0.0, 20.0, 30.0, WHITE);
        next_frame().await
    }
}
